use std::{fs::read_to_string, io::Write};

use clap::Parser;
use colored::Colorize;
use dust_lang::{parse, run};
use log::Level;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    command: Option<String>,

    #[arg(short, long)]
    parse: bool,

    #[arg(short, long)]
    styled: bool,

    path: Option<String>,
}

fn main() {
    env_logger::builder()
        .parse_env("DUST_LOG")
        .format(|buf, record| {
            let level_display = match record.level() {
                Level::Info => "INFO".bold().white(),
                Level::Debug => "DEBUG".bold().blue(),
                Level::Warn => "WARN".bold().yellow(),
                Level::Error => "ERROR".bold().red(),
                Level::Trace => "TRACE".bold().purple(),
            };
            let module = record
                .module_path()
                .map(|path| path.split("::").last().unwrap_or(path))
                .unwrap_or("unknown")
                .dimmed();
            let display = format!("{level_display:5} {module:^6} {args}", args = record.args());

            writeln!(buf, "{display}")
        })
        .init();

    let args = Cli::parse();

    if let Some(command) = &args.command {
        if args.parse {
            parse_and_display(command, args.styled);
        } else {
            run_and_display(command);
        }
    } else if let Some(path) = &args.path {
        let source = read_to_string(path).expect("Failed to read file");

        if args.parse {
            parse_and_display(&source, args.styled);
        } else {
            run_and_display(&source);
        }
    }
}

fn parse_and_display(source: &str, styled: bool) {
    match parse(source) {
        Ok(chunk) => println!(
            "{}",
            chunk
                .disassembler("Dust CLI Input")
                .source(source)
                .styled(styled)
                .disassemble()
        ),
        Err(error) => {
            eprintln!("{}", error.report());
        }
    }
}

fn run_and_display(source: &str) {
    match run(source) {
        Ok(Some(value)) => println!("{}", value),
        Ok(_) => {}
        Err(error) => {
            eprintln!("{}", error.report());
        }
    }
}
