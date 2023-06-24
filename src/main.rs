extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::env;
use std::process::ExitCode;

mod ast;
mod error;
mod gencode;
mod parser;
mod regalloc;
mod symtable;
mod three_ac;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("main: expected two arguments");
        return ExitCode::FAILURE;
    }

    let (ast, symtable) = match parser::parse_file(&args[1]) {
        Ok(res) => res,
        Err(error) => {
            eprintln!("{error}");
            return match error {
                error::Error::Type => ExitCode::from(7),
                _ => ExitCode::FAILURE,
            };
        }
    };

    match gencode::generate_code(ast, symtable, args[2].parse::<u32>().unwrap()) {
        Ok(s) => {
            print!("{s}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            match error {
                error::Error::Type => ExitCode::from(7),
                _ => ExitCode::FAILURE,
            }
        }
    }
}
