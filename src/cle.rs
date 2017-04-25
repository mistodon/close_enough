// extern crate close_enough;
// extern crate clap;
//
// use std::borrow::Cow;
// use std::env;
// use std::fs;
// use std::io::{self, Read, Write};
// use std::path::{Path, PathBuf};
// use clap::{App, Arg, AppSettings, SubCommand, Values};
//
//
// fn cle_app<'a, 'b>() -> App<'a, 'b>
// {
//     App::new("cle")
//         .author("Pirh, pirh.badger@gmail.com")
//         .version("0.1.0")
//         .about("Fuzzy-search the input and return the closest match")
//         .settings(&[AppSettings::SubcommandsNegateReqs, AppSettings::DisableHelpSubcommand])
//         .subcommand(SubCommand::with_name("-gen-script")
//             .about("Generate useful companion scripts")
//             .arg(
//                 Arg::with_name("script")
//                 .help("The script to generate")
//                 .required(true)
//             )
//         )
//         .subcommand(SubCommand::with_name("-ce")
//             .about("Fuzzy-searching cd command")
//             .usage("ce <dirs>...")
//             .arg(
//                 Arg::with_name("dirs")
//                 .help("Sequence of (fuzzy) directory names to cd through")
//                 .multiple(true)
//                 .required(true)
//             )
//         )
//         .arg(
//             Arg::with_name("query")
//             .help("The string or strings to search for;\nIf multiple strings are given, the closest match of each is returned")
//             .multiple(true)
//             .required(true)
//         )
//         .arg(
//             Arg::with_name("inputs")
//             .long("--inputs")
//             .short("-i")
//             .help("Lines of input to search")
//             .takes_value(true)
//             .multiple(true)
//         )
//         .arg(
//             Arg::with_name("sep")
//             .long("--sep")
//             .help("The seperator to join the results with;\nDefaults to newline")
//             .takes_value(true)
//             .default_value("\n")
//         )
//         .arg(
//             Arg::with_name("cwd")
//             .long("--cwd")
//             .help("Use current working directory contents as inputs")
//             .conflicts_with("inputs")
//         )
//         .arg(
//             Arg::with_name("files_only")
//             .short("-f")
//             .help("Used with --cwd: only allow files in results")
//             .requires("cwd")
//             .conflicts_with("dirs_only")
//         )
//         .arg(
//             Arg::with_name("dirs_only")
//             .short("-d")
//             .help("Used with --cwd: only allow directories in results")
//             .requires("cwd")
//             .conflicts_with("files_only")
//         )
//         .arg(
//             Arg::with_name("recursive")
//             .short("-r")
//             .help("Used with --cwd: query recursively through directories with each query string in sequence")
//             .requires("cwd")
//         )
//         .after_help(
// r#"Fuzzy-search a list of inputs with one or more query strings.
// The closest match to each query string is returned on its own line.
// If no inputs are provided, inputs are read from stdin."#)
// }
//
// #[derive(Copy, Clone)]
// enum CwdSearchStrategy
// {
//     FilesOnly,
//     DirectoriesOnly,
//     Anything
// }
//
// impl CwdSearchStrategy
// {
//     fn create(cwd: bool, files_only: bool, dirs_only: bool) -> Option<CwdSearchStrategy>
//     {
//         match (cwd, files_only, dirs_only)
//         {
//             (true, true, _) => Some(CwdSearchStrategy::FilesOnly),
//             (true, _, true) => Some(CwdSearchStrategy::DirectoriesOnly),
//             (true, _, _) => Some(CwdSearchStrategy::Anything),
//             _ => None
//         }
//     }
// }
//
// struct CleError
// {
//     pub message: String
// }
// impl CleError
// {
//     pub fn new(message: String) -> Self
//     {
//         CleError { message: message }
//     }
// }
//
// type CleResult<'a> = Result<Cow<'a, str>, CleError>;
//
//
// fn main()
// {
//     let args = cle_app().get_matches();
//
//     let (exe_name, result) = match args.subcommand()
//     {
//         ("-gen-script", Some(gen_args)) =>
//         {
//             ("cle", generate_script(gen_args.value_of("script").expect("cle: error: No script found to generate")))
//         },
//
//         ("-ce", Some(ce_args)) =>
//         {
//             let queries: Vec<&str> = ce_args.values_of("dirs").expect("cle: error: expected dirs argument").collect();
//             let inputs: Option<Values> = None;
//             let separator = "/";
//             let cwd_search_strategy = Some(CwdSearchStrategy::DirectoriesOnly);
//             let recursive = true;
//             ("ce", cle(queries, inputs, separator, cwd_search_strategy, recursive))
//         },
//
//         _ =>
//         {
//             let queries: Vec<&str> = args.values_of("query").expect("cle: error: expected query argument").collect();
//             let inputs = args.values_of("inputs");
//             let separator = args.value_of("sep").expect("cle: error: could not find separator");
//             let cwd_search_strategy = CwdSearchStrategy::create(args.is_present("cwd"), args.is_present("files_only"), args.is_present("dirs_only"));
//             let recursive = args.is_present("recursive");
//             ("cle", cle(queries, inputs, separator, cwd_search_strategy, recursive))
//         }
//     };
//     match result
//     {
//         Ok(output) => io::stdout().write_all(output.as_bytes()).expect("cle: error: Failed to write to stdout"),
//         Err(error) => io::stderr().write_all(format!("{}: {}\n", exe_name, error.message).as_bytes()).expect("cle: error: Failed to write to stderr")
//     };
// }
//
// fn cle<'a, I>(queries: Vec<&str>, inputs: Option<I>, separator: &str, cwd_search_strategy: Option<CwdSearchStrategy>, recursive: bool) -> CleResult<'a>
//     where I: Iterator<Item=&'a str>
// {
//     let query_count = queries.len();
//     let input_lines = fetch_input_lines(inputs, cwd_search_strategy);
//
//     input_lines.get(0).expect("cle: error: no valid inputs");
//
//     let inputs: Vec<&str> = input_lines.iter().map(|s| s.as_ref()).collect();
//
//     let mut outputs: Vec<&str> = Vec::new();
//
//     for (i, query) in queries.iter().enumerate()
//     {
//         if recursive
//         {
//             let last_query = i == query_count-1;
//             let strategy = if last_query { cwd_search_strategy.unwrap_or(CwdSearchStrategy::Anything) } else { CwdSearchStrategy::DirectoriesOnly };
//             let mut working_path = PathBuf::new();
//             working_path.push(".");
//             for o in &outputs
//             {
//                 working_path.push(o);
//             }
//             let working_inputs = list_directory(working_path, strategy);
//             let inputs: Vec<&str> = working_inputs.iter().map(|s| s.as_ref()).collect();
//             // outputs.push(close_enough::closest_enough(&inputs, query).expect("cle: error: query failed to match any inputs").to_owned())
//         }
//         else
//         {
//             // outputs.push(close_enough::closest_enough(&inputs, query).expect("cle: error: query failed to match any inputs").to_owned())
//         }
//     }
//
//     let output = &outputs.join(separator);
//
//     Ok(Cow::Owned(output.to_owned()))
// }
//
//
// fn fetch_input_lines<'a, I>(input_args: Option<I>, cwd_search_strategy: Option<CwdSearchStrategy>) -> Vec<Cow<'a, str>>
//     where I: Iterator<Item=&'a str>
// {
//     match (input_args, cwd_search_strategy)
//     {
//         (Some(inputs), _) => inputs.map(|s| Cow::Borrowed(s)).collect(),
//         (None, None) => read_stdin(),
//         (None, Some(strategy)) => list_directory(env::current_dir().expect("cle: error: failed to identify current directory"), strategy)
//     }
// }
//
// fn list_directory<'a, P: AsRef<Path> + std::fmt::Debug>(dir: P, strategy: CwdSearchStrategy) -> Vec<Cow<'a, str>>
// {
//     let contents = fs::read_dir(&dir).expect(&format!("cle: error: failed to read contents of '{:?}'", dir));
//
//     contents.filter_map(move |entry|
//         {
//             let entry = entry.expect("cle: error: failed to read directory entry");
//             let entry = match strategy
//             {
//                 CwdSearchStrategy::FilesOnly => if entry.file_type().expect("cle: error: failed to read file type").is_file() { Some(entry) } else { None },
//                 CwdSearchStrategy::DirectoriesOnly => if entry.file_type().expect("cle: error: failed to read file type").is_dir() { Some(entry) } else { None },
//                 _ => Some(entry)
//             };
//             entry.map(|entry| Cow::Owned(entry.file_name().into_string().expect("cle: error: failed to read directory entry")))
//         }
//     ).collect()
// }
//
// fn read_stdin<'a>() -> Vec<Cow<'a, str>>
// {
//     let mut s = String::new();
//     io::stdin().read_to_string(&mut s).expect("cle: error: failed to read from stdin");
//
//     s.lines().map(|s| Cow::Owned(s.to_owned())).collect()
// }
//
//
// const CE_SCRIPT_SOURCE: &'static str = include_str!("scripts/ce.sh");
//
//
// fn generate_script(script_name: &str) -> CleResult
// {
//     match script_name
//     {
//         "ce" => Ok(Cow::Borrowed(CE_SCRIPT_SOURCE)),
//         _ => Err(CleError::new(format!("Expected script name: No script available named '{}'", script_name)))
//     }
// }

fn main()
{
    
}
