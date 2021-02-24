use structopt::{clap::AppSettings, StructOpt};

/// Fuzzy-search the input and return the closest match
#[derive(StructOpt)]
#[structopt(
    name = "cle",
    author = "Vi, ***redacted.email@redacted.nope***",
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
    },

    /// Fuzzy-searching cd command
    #[structopt(name = "-ce", usage = "ce <dirs>...")]
    Ce {
        /// Sequence of (fuzzy) directory names to cd through
        #[structopt(required = true)]
        dirs: Vec<String>,
    },
}

fn main() {
    let args = CleCmd::from_args();
    match args.sub {
        Some(CleSubCmd::CeScript { shell }) => {
            // TODO: Support other shells?
            assert_eq!(shell, "bash");
            output_success(include_str!("scripts/ce.sh"));
        }
        Some(CleSubCmd::Ce { dirs }) => {
            use std::path::{Path, PathBuf};

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
