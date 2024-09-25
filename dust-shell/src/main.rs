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
            let level = match record.level() {
                Level::Info => "INFO".white().bold(),
                Level::Debug => "DEBUG".blue().bold(),
                Level::Warn => "WARN".yellow().bold(),
                Level::Error => "ERROR".red().bold(),
                Level::Trace => "TRACE".purple().bold(),
            }
            .bold();
            let level_display = format!("{level:<5}");
            let module = record
                .module_path()
                .map(|path| path.split("::").last().unwrap_or(path))
                .unwrap_or("unknown")
                .dimmed();

            writeln!(buf, "{level_display:^10} {module:^6} {}", record.args())
        })
        .init();

    let args = Cli::parse();

    if let Some(command) = &args.command {
        if args.parse {
            parse_and_display_errors(command, args.styled);
        } else {
            run_and_display_errors(command);
        }
    } else if let Some(path) = &args.path {
        let source = read_to_string(path).expect("Failed to read file");

        if args.parse {
            parse_and_display_errors(&source, args.styled);
        } else {
            run_and_display_errors(&source);
        }
    }
}

fn parse_and_display_errors(source: &str, pretty_print: bool) {
    match parse(source) {
        Ok(chunk) => println!(
            "{}",
            chunk
                .disassembler("Dust CLI Input")
                .styled(pretty_print)
                .disassemble()
        ),
        Err(error) => {
            eprintln!("{}", error.report());
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
