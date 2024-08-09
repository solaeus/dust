use std::{collections::HashMap, fs::read_to_string};

use clap::Parser;
use dust_lang::{run, DustError, Identifier, Value};

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    command: Option<String>,

    path: Option<String>,
}

fn main() {
    let args = Cli::parse();
    let mut variables = HashMap::new();

    if let Some(command) = &args.command {
        run_and_display_errors(command, &mut variables);
    } else if let Some(path) = &args.path {
        let source = read_to_string(path).expect("Failed to read file");

        run_and_display_errors(&source, &mut variables)
    } else {
        panic!("No command or path provided");
    };
}

fn run_and_display_errors(source: &str, variables: &mut HashMap<Identifier, Value>) {
    match run(source, variables) {
        Ok(return_value) => {
            if let Some(value) = return_value {
                println!("{}", value);
            }
        }
        Err(error) => eprintln!("{}", DustError::new(error, source).report()),
    }
}
