use clap::App;

fn cle_app<'a, 'b>() -> App<'a, 'b> {
    use clap::{AppSettings, Arg, SubCommand};

    App::new("cle")
        .author("Pirh, ***redacted.email@redacted.nope***")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Fuzzy-search the input and return the closest match")
        .settings(&[
            AppSettings::SubcommandsNegateReqs,
            AppSettings::DisableHelpSubcommand,
            AppSettings::VersionlessSubcommands,
        ])
        .subcommand(
            SubCommand::with_name("-ce-script")
                .about("Generate shell script for the `ce` command")
                .arg(
                    Arg::with_name("shell")
                        .required(true)
                        .takes_value(true)
                        .possible_values(&["bash"]),
                ),
        )
        .subcommand(
            SubCommand::with_name("-ce")
                .about("Fuzzy-searching cd command")
                .usage("ce <dirs>...")
                .arg(
                    Arg::with_name("dirs")
                        .help("Sequence of (fuzzy) directory names to cd through")
                        .multiple(true)
                        .required(true),
                ),
        )
        .arg(
            Arg::with_name("query")
                .help("The string to search for")
                .required(true),
        )
        .arg(
            Arg::with_name("inputs")
                .long("--inputs")
                .short("-i")
                .help("Lines of input to search")
                .takes_value(true)
                .multiple(true),
        )
        .after_help(
            r#"Fuzzy-search a list of inputs with a query string.
The closest match is written to stdout.
If no inputs are provided, inputs are read from stdin."#,
        )
}

fn main() {
    let args = cle_app().get_matches();
    match args.subcommand() {
        ("-ce-script", Some(args)) => match args.value_of("shell").unwrap() {
            "bash" => output_success(include_str!("scripts/ce.sh")),
            _ => unreachable!(),
        },

        ("-ce", Some(args)) => {
            use std::path::PathBuf;

            let queries = args.values_of("dirs").unwrap();
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
                    let dir_contents = std::fs::read_dir(&working_dir).unwrap();

                    let inputs = dir_contents.map(|e| e.unwrap()).filter_map(|entry| {
                        let metadata = std::fs::metadata(entry.path()).unwrap();
                        if metadata.is_dir() {
                            entry.file_name().into_string().ok()
                        } else {
                            None
                        }
                    });

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
            output_success(working_dir.as_path().to_str().unwrap());
        }

        _ => {
            use std::io::Read;

            let query = args.value_of("query").unwrap();
            let inputs = args.values_of("inputs");
            let mut stdin = String::new();

            let result: Option<&str> = match inputs {
                Some(inputs) => close_enough::close_enough(inputs, query),
                None => {
                    std::io::stdin()
                        .read_to_string(&mut stdin)
                        .expect("cle: error: Failed to read from stdin");
                    close_enough::close_enough(stdin.lines(), query)
                }
            };

            match result {
                Some(matching) => output_success(matching),
                None => exit_with_failure(),
            }
        }
    }
    unreachable!();
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
