extern crate close_enough;
extern crate clap;

use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{PathBuf};
use std::process;
use clap::{App, Arg, AppSettings, SubCommand};

const CE_SCRIPT_SOURCE: &'static str = include_str!("scripts/ce.sh");

fn cle_app<'a, 'b>() -> App<'a, 'b>
{
    App::new("cle")
        .author("Pirh, pirh.badger@gmail.com")
        .version("0.1.1")
        .about("Fuzzy-search the input and return the closest match")
        .settings(&[AppSettings::SubcommandsNegateReqs, AppSettings::DisableHelpSubcommand])
        .subcommand(SubCommand::with_name("-gen-script")
            .about("Generate useful companion scripts")
            .subcommand(SubCommand::with_name("ce")
                .about("Generate 'ce' command for fuzzy directory changing")
            )
        )
        .subcommand(SubCommand::with_name("-ce")
            .about("Fuzzy-searching cd command")
            .usage("ce <dirs>...")
            .arg(
                Arg::with_name("dirs")
                .help("Sequence of (fuzzy) directory names to cd through")
                .multiple(true)
                .required(true)
            )
        )
        .arg(
            Arg::with_name("query")
            .help("The string to search for")
            .required(true)
        )
        .arg(
            Arg::with_name("inputs")
            .long("--inputs")
            .short("-i")
            .help("Lines of input to search")
            .takes_value(true)
            .multiple(true)
        )
        .after_help(
r#"Fuzzy-search a list of inputs with one or more query strings.
The closest match to each query string is returned on its own line.
If no inputs are provided, inputs are read from stdin."#)
}

fn main()
{
    let args = cle_app().get_matches();
    match args.subcommand()
    {
        ("-gen-script", Some(args)) =>
        {
            match args.subcommand_name()
            {
                Some("ce") => output_success(CE_SCRIPT_SOURCE),
                _ => unreachable!()
            }
        },

        ("-ce", Some(args)) =>
        {
            let queries = args.values_of("dirs").unwrap();
            let starting_dir = env::current_dir().expect("cle: error: failed to identify current directory");
            let mut working_dir = PathBuf::new();
            working_dir.push(starting_dir);
            for query in queries
            {
                let dir_contents = fs::read_dir(&working_dir).unwrap();
                let inputs = dir_contents.map(|e|
                    e.unwrap()).filter_map(|entry|
                        if entry.file_type().unwrap().is_dir()
                        {
                            entry.file_name().into_string().ok()
                        }
                        else
                        {
                            None
                        });
                let result = close_enough::closest_enough(inputs, query);
                match result
                {
                    Some(dir) => working_dir.push(dir),
                    None => output_failure(format!("ce: No directory name matching '{}': Reached '{}'\n", query, working_dir.display()))
                }
            }
            output_success(working_dir.as_path().to_str().unwrap());
        },

        _ =>
        {
            let query = args.value_of("query").unwrap();
            let inputs = args.values_of("inputs");
            let mut stdin = String::new();
            let result: Option<&str> = match inputs
            {
                Some(inputs) => close_enough::closest_enough(inputs, query),
                None =>
                {
                    io::stdin().read_to_string(&mut stdin).expect("cle: error: Failed to read from stdin");
                    close_enough::closest_enough(stdin.lines(), query)
                }
            };
            match result
            {
                Some(matching) => output_success(matching),
                None => exit_with_failure()
            }
        }
    }
    unreachable!();
}

fn output_success<T>(output: T)
    where T: AsRef<str>
{
    io::stdout().write_all(output.as_ref().as_bytes()).expect("cle: error: Failed to write to stdout");
    process::exit(0);
}

fn output_failure<T>(message: T)
    where T: AsRef<str>
{
    io::stderr().write_all(message.as_ref().as_bytes()).expect("cle: error: Failed to write to stderr");
    process::exit(1);
}

fn exit_with_failure()
{
    process::exit(1);
}
