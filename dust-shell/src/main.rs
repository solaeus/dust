//! Command line interface for the dust programming language.
mod cli;

use clap::Parser;
use cli::run_shell;
use colored::Colorize;

use std::{
    fs::read_to_string,
    io::{stderr, Write},
};

use dust_lang::{context::Context, Interpreter};

/// Command-line arguments to be parsed.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Dust source code to evaluate.
    #[arg(short, long)]
    command: Option<String>,

    /// Location of the file to run.
    path: Option<String>,
}

fn main() {
    env_logger::Builder::from_env("DUST_LOG")
        .format(|buffer, record| {
            let args = record.args();
            let log_level = record.level().to_string().bold();
            let timestamp = buffer.timestamp_seconds().to_string().dimmed();

            writeln!(buffer, "[{log_level} {timestamp}] {args}")
        })
        .init();

    let args = Args::parse();
    let context = Context::new();

    let source = if let Some(path) = &args.path {
        read_to_string(path).unwrap()
    } else if let Some(command) = args.command {
        command
    } else {
        return run_shell(context);
    };

    let mut interpreter = Interpreter::new(context);

    let eval_result = interpreter.run(&source);

    match eval_result {
        Ok(value) => {
            if let Some(value) = value {
                println!("{value}")
            }
        }
        Err(errors) => {
            for error in errors {
                let report = error.build_report(&source);

                stderr().write_all(&report).unwrap();
            }
        }
    }
}
