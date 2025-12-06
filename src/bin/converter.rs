mod common;

use clap::Parser;
use parser;
use std::fs::File;
use std::io;

fn main() {
    let args = common::args::Convert::parse();

    let input: Box<dyn io::Read> = match &args.input {
        Some(input) => Box::new(File::open(input).unwrap()),
        None => Box::new(io::stdin()),
    };
    let output: Box<dyn io::Write> = match &args.output {
        Some(output) => Box::new(File::create(output).unwrap()),
        None => Box::new(io::stdout()),
    };

    match parser::read(input, args.input_format.into()) {
        Ok(tx) => match parser::write(output, args.output_format.into(), tx) {
            Ok(_) => println!("Success"),
            Err(e) => eprintln!("parse file '{}': {}", args.output.unwrap(), e),
        },
        Err(e) => eprintln!("parse file '{}': {}", args.input.unwrap(),e),
    };
}
