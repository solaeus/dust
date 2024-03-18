//! Command line interface for the dust programming language.

use ariadne::{sources, Color, Label, Report, ReportKind, Source};
use chumsky::span::SimpleSpan;
use clap::Parser;
use colored::Colorize;

use std::{fs::read_to_string, io::Write, ops::Range};

use dust_lang::{context::Context, error::Error, Interpreter};

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
            if let Some(value) = value {
                println!("{value}")
            }
        }
        Err(errors) => {
            for error in errors {
                let mut report_builder = match &error {
                    Error::Parse { expected, span } => {
                        let message = if expected.is_empty() {
                            "Invalid token.".to_string()
                        } else {
                            format!("Expected {expected}.")
                        };

                        Report::build(
                            ReportKind::Custom("Parsing Error", Color::White),
                            "input",
                            span.1,
                        )
                        .with_label(
                            Label::new(("input", span.0..span.1))
                                .with_message(message)
                                .with_color(Color::Red),
                        )
                    }
                    Error::Lex { expected, span } => {
                        let message = if expected.is_empty() {
                            "Invalid token.".to_string()
                        } else {
                            format!("Expected {expected}.")
                        };

                        Report::build(
                            ReportKind::Custom("Dust Error", Color::White),
                            "input",
                            span.1,
                        )
                        .with_label(
                            Label::new(("input", span.0..span.1))
                                .with_message(message)
                                .with_color(Color::Red),
                        )
                    }
                    Error::Runtime { error, position } => Report::build(
                        ReportKind::Custom("Dust Error", Color::White),
                        "input",
                        position.1,
                    ),
                    Error::Validation { error, position } => Report::build(
                        ReportKind::Custom("Dust Error", Color::White),
                        "input",
                        position.1,
                    ),
                };

                report_builder = error.build_report(report_builder);

                report_builder
                    .finish()
                    .eprint(sources([("input", &source)]))
                    .unwrap()
            }
        }
    }
}
