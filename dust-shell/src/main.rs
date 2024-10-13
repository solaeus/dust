use std::{fs::read_to_string, io::Write};

use clap::Parser;
use colored::Colorize;
use dust_lang::{parse, run, Formatter};
use log::Level;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    command: Option<String>,

    #[arg(short, long)]
    format: bool,

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

    let source = if let Some(path) = &args.path {
        &read_to_string(path).expect("Failed to read file")
    } else if let Some(command) = &args.command {
        command
    } else {
        eprintln!("No input provided");
        return;
    };

    if args.parse {
        parse_source(source, args.styled);
    }

    if args.format {
        format_source(source);
    }

    if !args.format && !args.parse {
        run_source(source);
    }
}

fn format_source(source: &str) {
    println!("{}", Formatter::new(source).format())
}

fn parse_source(source: &str, styled: bool) {
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

fn run_source(source: &str) {
    match run(source) {
        Ok(Some(value)) => println!("{}", value),
        Ok(_) => {}
        Err(error) => {
            eprintln!("{}", error.report());
        }
    }
}
