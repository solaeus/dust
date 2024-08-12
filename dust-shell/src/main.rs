use std::fs::read_to_string;

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

    if let Some(command) = &args.command {
        run_and_display_errors(command);
    } else if let Some(path) = &args.path {
        let source = read_to_string(path).expect("Failed to read file");

        run_and_display_errors(&source)
    } else {
        panic!("No command or path provided");
    };
}

fn run_and_display_errors(source: &str) {
    match run(source) {
        Ok(return_value) => {
            if let Some(value) = return_value {
                println!("{}", value);
            }
        }
        Err(error) => eprintln!("{}", error.report()),
    }
}
