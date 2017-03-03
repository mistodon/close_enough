extern crate close_enough;
extern crate clap;

use std::io::{self, Read, Write};
use clap::{App, Arg};


fn main()
{
    let args = App::new("ce")
        .author("Pirh, ***redacted.email@redacted.nope***")
        .version("0.1.0")
        .about("Fuzzy-search the input and return the closest match")
        .arg(
            Arg::with_name("query")
            .help("the string to search for")
            .required(true)
        )
        .arg(
            Arg::with_name("inputs")
            .help("lines of input to search")
            .multiple(true)
        )
        .after_help("Longer explaination to appear after the options when \
                     displaying the help information from --help or -h")
        .get_matches();

    let query = args.value_of("query").expect("Expected query argument");
    let maybe_inputs = args.values_of("inputs");

    let result = match maybe_inputs
    {
        Some(inputs) => ce_over_inputs(inputs.map(|s| s.as_ref()).collect(), query),
        None => ce_over_stdin(query)
    };

    let output = result.expect("ce: error: could not match any inputs");
    io::stdout().write(output.as_bytes()).expect("ce: error: failed to write result");
}

fn ce_over_inputs(inputs: Vec<&str>, query: &str) -> Option<String>
{
    close_enough::closest_enough(&inputs, query).map(|s| s.to_owned())
}

fn ce_over_stdin(query: &str) -> Option<String>
{
    let mut stdin = io::stdin();
    let input = {
        let mut s = String::new();
        stdin.read_to_string(&mut s).expect("ce: error: failed to read from stdin");
        s
    };

    let lines: Vec<&str> = input.lines().collect();

    close_enough::closest_enough(&lines, query).map(|s| s.to_owned())
}