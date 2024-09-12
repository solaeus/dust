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

    path: Option<String>,
}

fn main() {
    env_logger::builder()
        .parse_env("DUST_LOG")
        .format(|buf, record| {
            let level = match record.level() {
                Level::Error => "ERROR".red(),
                Level::Warn => "WARN".yellow(),
                Level::Info => "INFO".white(),
                Level::Debug => "DEBUG".blue(),
                Level::Trace => "TRACE".purple(),
            }
            .bold();
            let level_display = format!("[{level:^5}]").white().on_black();
            let module = record
                .module_path()
                .map(|path| path.split("::").last().unwrap_or("unknown"))
                .unwrap_or("unknown")
                .dimmed();

            writeln!(buf, "{level_display} {module:^6} {}", record.args())
        })
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
        Ok(chunk) => println!(
            "{}",
            chunk
                .disassembler("Dust CLI Input")
                .styled()
                .width(80)
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
