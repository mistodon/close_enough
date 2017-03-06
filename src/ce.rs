extern crate close_enough;
extern crate clap;

use std::borrow::Cow;
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use clap::{App, Arg};


fn main()
{
    let args = App::new("ce")
        .author("Pirh, pirh.badger@gmail.com")
        .version("0.1.0")
        .about("Fuzzy-search the input and return the closest match")
        .arg(
            Arg::with_name("query")
            .help("The string or strings to search for;\nIf multiple strings are given, the closest match of each is returned")
            .multiple(true)
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
        .arg(
            Arg::with_name("cwd")
            .long("--cwd")
            .help("Use current working directory contents as inputs")
            .conflicts_with("inputs")
        )
        .arg(
            Arg::with_name("sep")
            .long("--sep")
            .help("The seperator to join the results with;\nDefaults to newline")
            .takes_value(true)
            .default_value("\n")
        )
        .after_help(
r#"Fuzzy-search a list of inputs with one or more query strings.
The closest match to each query string is returned on its own line.
If no inputs are provided, inputs are read from stdin."#)
        .get_matches();

    let queries = args.values_of("query").expect("Expected query argument");

    let input_lines = fetch_input_lines(args.values_of("inputs"), args.is_present("cwd"));
    let inputs: Vec<&str> = input_lines.iter().map(|s| s.as_ref()).collect();
    let separator = args.value_of("sep").expect("ce: error: could not find separator");

    let output: Vec<&str> = queries.map(
        |q| close_enough::closest_enough(&inputs, q).expect("ce: error: query failed to match any inputs")
    ).collect();

    let output = &output.join(separator);

    io::stdout().write(&output.as_bytes()).expect("ce: error: failed to write results");
}

fn fetch_input_lines<'a, I>(input_args: Option<I>, using_cwd: bool) -> Vec<Cow<'a, str>>
    where I: Iterator<Item=&'a str>
{
    match (input_args, using_cwd)
    {
        (Some(inputs), _) => inputs.map(|s| Cow::Borrowed(s)).collect(),
        (None, true) => list_cwd(),
        (None, false) => read_stdin()
    }
}

fn list_cwd<'a>() -> Vec<Cow<'a, str>>
{
    let here = env::current_dir().expect("ce: error: failed to identify current directory");
    let contents = fs::read_dir(&here).expect("ce: error: failed to read current directory");

    contents.map(|entry| Cow::Owned(
        entry.expect("ce: error: failed to read directory entry")
        .file_name()
        .into_string()
        .expect("ce: error: failed to read directory entry"))
    ).collect()
}

fn read_stdin<'a>() -> Vec<Cow<'a, str>>
{
    let mut s = String::new();
    io::stdin().read_to_string(&mut s).expect("ce: error: failed to read from stdin");

    s.lines().map(|s| Cow::Owned(s.to_owned())).collect()
}