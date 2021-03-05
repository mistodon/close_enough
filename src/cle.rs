use std::path::{Path, PathBuf};

use structopt::{clap::AppSettings, StructOpt};

/// Fuzzy-search the input and return the closest match
#[derive(StructOpt)]
#[structopt(
    name = "cle",
    author = "Vi, wishing.engine@gmail.com",
    settings = &[
        AppSettings::SubcommandsNegateReqs,
        AppSettings::DisableHelpSubcommand,
        AppSettings::VersionlessSubcommands,
    ],
    after_help = r#"Fuzzy-search a list of inputs with a query string.
The closest match is written to stdout.
If no inputs are provided, inputs are read from stdin."#,
)]
pub struct CleCmd {
    #[structopt(subcommand)]
    sub: Option<CleSubCmd>,

    /// The string to search for
    #[structopt(required_unless("sub"))]
    query: Option<String>,

    /// Lines of input to search
    #[structopt(long, short, required = true)]
    inputs: Vec<String>,
}

#[derive(StructOpt)]
pub enum CleSubCmd {
    /// Generate shell script for the `ce` command
    #[structopt(name = "-ce-script")]
    CeScript {
        #[structopt(required = true, possible_values = &["bash"])]
        shell: String,

        #[structopt(long)]
        with_hop: bool,
    },

    /// Generate shell script for the `hop` command
    #[structopt(name = "-hop-script")]
    HopScript {
        #[structopt(required = true, possible_values = &["bash"])]
        shell: String,
    },

    /// Fuzzy-searching cd command
    #[structopt(name = "-ce", usage = "ce <dirs>...")]
    Ce {
        /// Sequence of (fuzzy) directory names to cd through
        #[structopt(required = true)]
        dirs: Vec<String>,
    },

    #[structopt(name = "-hop", usage = "hop <query>")]
    Hop {
        #[structopt(subcommand)]
        sub: HopSubCmd,
    },
}

#[derive(StructOpt)]
#[structopt(
    settings = &[
        AppSettings::SubcommandsNegateReqs,
        AppSettings::DisableHelpSubcommand,
        AppSettings::VersionlessSubcommands,
    ],
)]
pub enum HopSubCmd {
    /// Change to a recently used directory that fuzzy-matches a query
    To {
        /// The string to search working directory history for
        #[structopt(required = true)]
        query: String,
    },

    /// Log a directory as being recently used
    Log {
        /// The directory to increment the recently-used count for
        #[structopt(required = true)]
        dir: String,
    },

    /// Forget how many times this directory has been used
    Forget {
        /// The directory to forget about
        #[structopt(required = true)]
        dir: String,
    },

    /// List directories in the recently-used list
    List,
}

fn main() {
    let args = CleCmd::from_args();
    match args.sub {
        // TODO: Support other shells?
        Some(CleSubCmd::CeScript { shell, with_hop }) => {
            assert_eq!(shell, "bash");
            let script = match with_hop {
                false => include_str!("scripts/ce.sh"),
                true => include_str!("scripts/ce_with_hop.sh"),
            };
            output_success(script);
        }
        Some(CleSubCmd::HopScript { shell }) => {
            assert_eq!(shell, "bash");
            output_success(include_str!("scripts/hop.sh"));
        }
        Some(CleSubCmd::Ce { dirs }) => {
            let queries = dirs;
            let starting_dir =
                std::env::current_dir().expect("cle: error: failed to identify current directory");
            let mut working_dir = PathBuf::new();
            working_dir.push(starting_dir);

            for query in queries {
                if query.starts_with('/') {
                    working_dir = PathBuf::new();
                    working_dir.push(query);
                    continue;
                }

                let query = query.trim_end_matches('/');
                let reverse_searching = query.starts_with("..");
                let nested_searching = query.starts_with('%');

                if reverse_searching {
                    let (_, query) = query.split_at(2);

                    match query {
                        "" => {
                            working_dir.pop();
                        }
                        query => {
                            if let Ok(popcount) = query.parse::<u64>() {
                                for _ in 0..popcount {
                                    working_dir.pop();
                                }
                            } else {
                                let target = {
                                    let path_components =
                                        working_dir.iter().filter_map(|path| path.to_str());
                                    let target = close_enough::close_enough(path_components, query);
                                    target.map(|t| t.to_owned())
                                };

                                match target {
                                    Some(ref dir) => {
                                        let working_dir = &mut working_dir;
                                        while !working_dir.ends_with(dir) {
                                            working_dir.pop();
                                        }
                                    }
                                    None => output_failure(format!(
                                        "ce: No directory name matching '{}': Reached '{}'",
                                        query,
                                        working_dir.display()
                                    )),
                                }
                            }
                        }
                    }
                } else {
                    // TODO: use ignore
                    fn fetch_dirs(working_dir: &Path) -> impl Iterator<Item = String> {
                        let dir_contents = std::fs::read_dir(working_dir).unwrap();

                        dir_contents.map(|e| e.unwrap()).filter_map(|entry| {
                            let metadata = std::fs::metadata(entry.path()).unwrap();
                            if metadata.is_dir() {
                                entry.file_name().into_string().ok()
                            } else {
                                None
                            }
                        })
                    }

                    if nested_searching {
                        let (_, query) = query.split_at(1);
                        let mut working = vec![working_dir.clone()];
                        let mut success = false;
                        while let Some(mut path) = working.pop() {
                            let inputs = fetch_dirs(&path).collect::<Vec<_>>();
                            let result = close_enough::close_enough(inputs.iter(), query);

                            match result {
                                Some(dir) => {
                                    path.push(dir);
                                    working_dir = path;
                                    success = true;
                                    break;
                                }
                                None => {
                                    for dir in inputs {
                                        let mut nextpath = path.clone();
                                        nextpath.push(dir);
                                        working.push(nextpath);
                                    }
                                }
                            }
                        }

                        if !success {
                            output_failure(format!(
                                "ce: No directory name matching in tree '{}': Reached '{}'",
                                query,
                                working_dir.display()
                            ))
                        }
                    } else {
                        let inputs = fetch_dirs(&working_dir);
                        let result = close_enough::close_enough(inputs, query);

                        match result {
                            Some(dir) => working_dir.push(dir),
                            None => output_failure(format!(
                                "ce: No directory name matching '{}': Reached '{}'",
                                query,
                                working_dir.display()
                            )),
                        }
                    }
                }
            }
            output_success(working_dir.as_path().to_str().unwrap());
        }
        Some(CleSubCmd::Hop { sub }) => {
            let hopfile_path = std::env::var("HOPFILE_PATH")
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    let mut path = home::home_dir().expect("Failed to find home directory");
                    path.push(".hopfile");
                    path
                });

            // ensure
            {
                if !hopfile_path.exists() {
                    std::fs::write(&hopfile_path, []).unwrap();
                }
            }

            let history_entries = std::fs::read_to_string(&hopfile_path).unwrap();
            let mut history_entries = history_entries.lines().collect::<Vec<_>>();

            match sub {
                HopSubCmd::Log { dir } => {
                    if !dir.trim().is_empty() {
                        let s = format!("{}\n", dir);
                        history_entries.push(&s);
                        history_entries.sort_unstable();
                        history_entries.dedup();
                        std::fs::write(&hopfile_path, history_entries.join("\n")).unwrap();
                    }
                }
                HopSubCmd::Forget { dir } => {
                    history_entries.retain(|line| line != &dir);
                    std::fs::write(&hopfile_path, history_entries.join("\n")).unwrap();
                }
                HopSubCmd::To { query } => {
                    let inputs = history_entries.iter().filter_map(|line| {
                        <_ as AsRef<Path>>::as_ref(line)
                            .file_name()
                            .and_then(std::ffi::OsStr::to_str)
                    });

                    let result = close_enough::close_enough(inputs, &query);

                    match result {
                        Some(matching) => {
                            // TODO: Kind of lame doing this again
                            let full_matching_path = history_entries
                                .iter()
                                .find(|line| {
                                    <_ as AsRef<Path>>::as_ref(line)
                                        .file_name()
                                        .and_then(std::ffi::OsStr::to_str)
                                        == Some(matching)
                                })
                                .unwrap();
                            output_success(full_matching_path);
                        }
                        None => exit_with_failure(),
                    }
                }
                HopSubCmd::List => {
                    println!("{}", history_entries.join("\n"));
                }
            }
        }
        None => {
            use std::io::Read;

            let query = &args.query.unwrap();
            let inputs = &args.inputs;
            let mut stdin = String::new();

            let result: Option<String> = match inputs.is_empty() {
                false => close_enough::close_enough(inputs, query).cloned(),
                true => {
                    std::io::stdin()
                        .read_to_string(&mut stdin)
                        .expect("cle: error: Failed to read from stdin");
                    close_enough::close_enough(stdin.lines(), query).map(str::to_owned)
                }
            };

            match result {
                Some(matching) => output_success(matching),
                None => exit_with_failure(),
            }
        }
    }
}

fn output_success<T>(output: T)
where
    T: AsRef<str>,
{
    print!("{}", output.as_ref());
    std::process::exit(0);
}

fn output_failure<T>(message: T)
where
    T: AsRef<str>,
{
    eprintln!("{}", message.as_ref());
    std::process::exit(1);
}

fn exit_with_failure() {
    std::process::exit(1);
}
