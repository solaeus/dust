//! Command line interface for the dust programming language.
mod cli;

use ariadne::sources;
use clap::Parser;
use cli::run_shell;
use colored::Colorize;
use log::Level;

use std::{
    fs::read_to_string,
    io::{stderr, Write},
    sync::Arc,
    vec,
};

use dust_lang::{context::Context, Interpreter};

/// Command-line arguments to be parsed.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Dust source code to evaluate.
    #[arg(short, long)]
    command: Option<String>,

    // Display lexer tokens of the input source.
    #[arg(short, long)]
    lex: bool,

    // Display abstract tree of the input source.
    #[arg(short, long)]
    parse: bool,

    #[arg(long)]
    compile: bool,

    /// Location of the file to run.
    path: Option<String>,
}

fn main() {
    env_logger::Builder::from_env("DUST_LOG")
        .format(|buffer, record| {
            let args = record.args();
            let log_level = match record.level() {
                Level::Trace => "TRACE".cyan().bold(),
                Level::Warn => "WARN".yellow().bold(),
                Level::Debug => "DEBUG".green().bold(),
                Level::Error => "ERROR".red().bold(),
                Level::Info => "INFO".white().bold(),
            };

            writeln!(buffer, "[{}] {}", log_level, args)
        })
        .init();

    let args = Args::parse();
    let context = Context::new(None);
    let interpreter = Interpreter::new(context.clone());

    let (source_id, source): (Arc<str>, Arc<str>) = if let Some(path) = args.path {
        let source = read_to_string(&path).unwrap();

        (Arc::from(path.as_str()), Arc::from(source))
    } else if let Some(command) = args.command {
        (Arc::from("command"), Arc::from(command))
    } else {
        match run_shell(context) {
            Ok(_) => {}
            Err(error) => eprintln!("{error}"),
        }

        return;
    };

    if args.lex {
        match interpreter.lex(source_id, source.as_ref()) {
            Ok(tokens) => println!("{tokens:?}"),
            Err(error) => {
                for report in error.build_reports() {
                    report
                        .write_for_stdout(
                            sources::<Arc<str>, Arc<str>, vec::IntoIter<(Arc<str>, Arc<str>)>>(
                                interpreter.sources(),
                            ),
                            stderr(),
                        )
                        .unwrap();
                }
            }
        }

        return;
    }

    if args.parse {
        match interpreter.parse(source_id, source.as_ref()) {
            Ok(abstract_tree) => println!("{abstract_tree:?}"),
            Err(error) => {
                for report in error.build_reports() {
                    report
                        .write_for_stdout(sources(interpreter.sources()), stderr())
                        .unwrap();
                }
            }
        }

        return;
    }

    if args.compile {
        match interpreter.parse(source_id, source.as_ref()) {
            Ok(abstract_tree) => {
                let ron = ron::to_string(&abstract_tree).unwrap();

                println!("{ron}")
            }
            Err(error) => {
                for report in error.build_reports() {
                    report
                        .write_for_stdout(sources(interpreter.sources()), stderr())
                        .unwrap();
                }
            }
        }

        return;
    }

    let run_result = interpreter.run(source_id.clone(), source);

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
