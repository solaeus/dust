//! Command line interface for the dust programming language.
mod cli;

use ariadne::sources;
use clap::Parser;
use cli::run_shell;
use colored::Colorize;

use std::{
    fs::read_to_string,
    io::{stderr, Write},
    rc::Rc,
};

use dust_lang::{context::Context, Interpreter};

/// Command-line arguments to be parsed.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Dust source code to evaluate.
    #[arg(short, long)]
    command: Option<String>,

    #[arg(long)]
    no_std: bool,

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
    let mut interpreter = Interpreter::new(context.clone());

    interpreter.load_std().unwrap();

    let (source_id, source) = if let Some(path) = args.path {
        let source = read_to_string(&path).unwrap();

        (Rc::from(path), source)
    } else if let Some(command) = args.command {
        (Rc::from("input"), command)
    } else {
        match run_shell(context) {
            Ok(_) => {}
            Err(error) => eprintln!("{error}"),
        }

        return;
    };

    let run_result = interpreter.run(source_id, Rc::from(source));

    match run_result {
        Ok(value) => {
            if let Some(value) = value {
                println!("{value}")
            }
        }
        Err(error) => {
            for report in error.build_reports() {
                report
                    .write_for_stdout(sources(interpreter.sources()), stderr())
                    .unwrap();
            }
        }
    }
}
