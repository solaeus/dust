//! Command line interface for the dust programming language.

use ariadne::Source;
use clap::Parser;
use colored::Colorize;

use std::{fs::read_to_string, io::Write};

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
        String::with_capacity(0)
    };

    let mut interpreter = Interpreter::new(context);

    let eval_result = interpreter.run(&source);

    match eval_result {
        Ok(value) => {
            if !value.is_none() {
                println!("{value}")
            }
        }
        Err(errors) => {
            for error in errors {
                error.report(&source).eprint(Source::from(&source)).unwrap();
            }
        }
    }
}
