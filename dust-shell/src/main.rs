use std::fs::read_to_string;

use clap::Parser;
use dust_lang::{parse, run};
use env_logger::WriteStyle;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    command: Option<String>,

    #[arg(short, long)]
    parse: bool,

    path: Option<String>,
}

fn main() {
    env_logger::builder()
        .parse_env("DUST_LOG")
        .format_timestamp_secs()
        .write_style(WriteStyle::Always)
        .init();

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
    match parse(source) {
        Ok(chunk) => println!("{}", chunk.disassemble("Dust CLI Input")),
        Err(error) => {
            eprintln!("{:?}", error);
        }
    }
}

fn run_and_display_errors(source: &str) {
    match run(source) {
        Ok(Some(value)) => println!("{}", value),
        Ok(_) => {}
        Err(error) => {
            eprintln!("{}", error.report());
        }
    }
}
