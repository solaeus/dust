use std::fs::read_to_string;

use clap::Parser;
use dust_lang::{run, Context};

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    command: Option<String>,

    path: Option<String>,
}

fn main() {
    let args = Cli::parse();
    let mut context = Context::new();

    if let Some(command) = &args.command {
        run_and_display_errors(command, &mut context);
    } else if let Some(path) = &args.path {
        let source = read_to_string(path).expect("Failed to read file");

        run_and_display_errors(&source, &mut context)
    } else {
        panic!("No command or path provided");
    };
}

fn run_and_display_errors(source: &str, variables: &mut Context) {
    match run(source, variables) {
        Ok(return_value) => {
            if let Some(value) = return_value {
                println!("{}", value);
            }
        }
        Err(error) => eprintln!("{}", error.report()),
    }
}
