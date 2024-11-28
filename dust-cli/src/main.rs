use std::fs::read_to_string;
use std::io::Write;

use clap::Parser;
use colored::Colorize;
use dust_lang::{compile, run};
use log::{Level, LevelFilter};

#[derive(Parser)]
struct Cli {
    /// Source code sent via command line
    #[arg(short, long)]
    command: Option<String>,

    /// Whether to output the disassembled chunk
    #[arg(short, long)]
    parse: bool,

    /// Whether to style the disassembled chunk
    #[arg(long)]
    style_disassembly: Option<bool>,

    /// Log level
    #[arg(short, long)]
    log: Option<LevelFilter>,

    /// Path to a source code file
    path: Option<String>,
}

fn main() {
    let args = Cli::parse();
    let mut logger = env_logger::builder();

    logger.format(|buf, record| {
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
    });

    if let Some(level) = args.log {
        logger.filter_level(level).init();
    } else {
        logger.parse_env("DUST_LOG").init();
    }

    let source = if let Some(path) = &args.path {
        &read_to_string(path).expect("Failed to read file")
    } else if let Some(command) = &args.command {
        command
    } else {
        eprintln!("No input provided");

        return;
    };

    if args.parse {
        let style = args.style_disassembly.unwrap_or(true);

        log::info!("Parsing source");

        match compile(source) {
            Ok(chunk) => {
                let disassembly = chunk.disassembler().style(style).disassemble();

                println!("{}", disassembly);
            }
            Err(error) => {
                eprintln!("{}", error.report());
            }
        }

        return;
    }

    match run(source) {
        Ok(Some(value)) => println!("{}", value),
        Ok(None) => {}
        Err(error) => {
            eprintln!("{}", error.report());
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }
}
