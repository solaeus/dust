//! Command line interface for the dust programming language.
mod cli;
mod error;

use ariadne::sources;
use clap::Parser;
use cli::run_shell;
use colored::Colorize;
use error::Error;

use std::{
    fs::read_to_string,
    io::{stderr, Write},
    rc::Rc,
};

use dust_lang::{context::Context, interpret};

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
    let (source, source_id) = if let Some(path) = args.path {
        (read_to_string(&path).unwrap(), Rc::new(path))
    } else if let Some(command) = args.command {
        (command, Rc::new("input".to_string()))
    } else {
        match run_shell(context) {
            Ok(_) => {}
            Err(error) => eprintln!("{error}"),
        }

        return;
    };

    let eval_result = interpret(&source);

    match eval_result {
        Ok(value) => {
            if let Some(value) = value {
                println!("{value}")
            }
        }
        Err(errors) => {
            let reports = Error::Dust { errors }
                .build_reports(source_id.clone())
                .unwrap();

            for report in reports {
                report
                    .write_for_stdout(
                        sources([
                            (source_id.clone(), source.as_str()),
                            (
                                Rc::new("std/io.ds".to_string()),
                                include_str!("../../std/io.ds"),
                            ),
                            (
                                Rc::new("std/thread.ds".to_string()),
                                include_str!("../../std/thread.ds"),
                            ),
                        ]),
                        stderr(),
                    )
                    .unwrap();
            }
        }
    }
}
