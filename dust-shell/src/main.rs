use std::fs::read_to_string;

use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    command: Option<String>,

    #[arg(short, long)]
    parse: bool,

    path: Option<String>,
}

fn main() {
    env_logger::init();

    let args = Cli::parse();

    if let Some(command) = &args.command {
        if args.parse {
            parse_and_display_errors(command);
        } else {
            run_and_display_errors(command);
        }
    } else if let Some(path) = &args.path {
        let source = read_to_string(path).expect("Failed to read file");

        if args.parse {
            parse_and_display_errors(&source);
        } else {
            run_and_display_errors(&source);
        }
    }
}

fn parse_and_display_errors(source: &str) {
    todo!()
}

fn run_and_display_errors(source: &str) {
    todo!()
}
