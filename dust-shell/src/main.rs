use std::{collections::HashMap, fs::read_to_string};

use clap::Parser;
use dust_lang::run;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    command: Option<String>,

    path: Option<String>,
}

fn main() {
    let args = Cli::parse();
    let mut variables = HashMap::new();

    let result = if let Some(command) = &args.command {
        run(command, &mut variables)
    } else if let Some(path) = &args.path {
        let content = read_to_string(path).unwrap();

        run(&content, &mut variables)
    } else {
        panic!("No command or path provided");
    };

    match result {
        Ok(return_value) => {
            if let Some(value) = return_value {
                println!("{}", value);
            }
        }
        Err(error) => eprintln!("{}", error),
    }
}
