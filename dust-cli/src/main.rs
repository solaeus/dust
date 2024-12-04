use std::io::{stdout, Write};
use std::time::Instant;
use std::{fs::read_to_string, path::PathBuf};

use clap::{Args, Parser};
use colored::Colorize;
use dust_lang::{compile, lex, run, write_token_list};
use log::{Level, LevelFilter};

#[derive(Parser)]
#[clap(
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
)]
struct Cli {
    /// Log level, overrides the DUST_LOG environment variable
    ///
    /// Possible values: trace, debug, info, warn, error, off
    #[arg(short, long, value_name = "LOG_LEVEL")]
    log: Option<LevelFilter>,

    #[command(flatten)]
    mode: ModeFlags,

    #[command(flatten)]
    source: Source,
}

#[derive(Args)]
#[group(multiple = false)]
struct ModeFlags {
    /// Run the source code (default)
    #[arg(short, long)]
    run: bool,

    #[arg(long, requires("run"))]
    /// Do not print the run result
    no_output: bool,

    /// Compile a chunk and show the disassembly
    #[arg(short, long)]
    disassemble: bool,

    /// Lex and display tokens from the source code
    #[arg(short, long)]
    tokenize: bool,

    /// Style disassembly or tokenization output
    #[arg(
        short,
        long,
        default_value = "true",
        requires("disassemble"),
        requires("tokenize")
    )]
    style: bool,
}

#[derive(Args)]
#[group(required = true, multiple = false)]
struct Source {
    /// Source code
    #[arg(short, long, value_name = "SOURCE")]
    command: Option<String>,

    /// Path to a source code file
    file: Option<PathBuf>,
}

fn main() {
    let start_time = Instant::now();
    let mut logger = env_logger::builder();

    logger.format(move |buf, record| {
        let elapsed = format!("T+{:.04}", start_time.elapsed().as_secs_f32()).dimmed();
        let level_display = match record.level() {
            Level::Info => "INFO".bold().white(),
            Level::Debug => "DEBUG".bold().blue(),
            Level::Warn => "WARN".bold().yellow(),
            Level::Error => "ERROR".bold().red(),
            Level::Trace => "TRACE".bold().purple(),
        };
        let display = format!("[{elapsed}] {level_display:5} {args}", args = record.args());

        writeln!(buf, "{display}")
    });

    let Cli {
        log,
        source: Source { command, file },
        mode,
    } = Cli::parse();

    if let Some(level) = log {
        logger.filter_level(level).init();
    } else {
        logger.parse_env("DUST_LOG").init();
    }

    let source = if let Some(source) = command {
        source
    } else {
        let path = file.expect("Path is required when command is not provided");

        read_to_string(path).expect("Failed to read file")
    };

    if mode.disassemble {
        let chunk = match compile(&source) {
            Ok(chunk) => chunk,
            Err(error) => {
                eprintln!("{}", error);

                return;
            }
        };
        let disassembly = chunk
            .disassembler()
            .style(mode.style)
            .source(&source)
            .disassemble();

        println!("{}", disassembly);

        return;
    }

    if mode.tokenize {
        let tokens = match lex(&source) {
            Ok(tokens) => tokens,
            Err(error) => {
                eprintln!("{}", error);

                return;
            }
        };
        let mut stdout = stdout().lock();

        write_token_list(&tokens, mode.style, &mut stdout)
    }

    let run_result = run(&source);

    match run_result {
        Ok(Some(value)) => {
            if !mode.no_output {
                println!("{}", value)
            }
        }
        Ok(None) => {}
        Err(error) => {
            eprintln!("{}", error);
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
