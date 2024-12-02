use std::io::{stdout, Write};
use std::time::Instant;
use std::{fs::read_to_string, path::PathBuf};

use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use dust_lang::{compile, lex, run, write_token_list};
use log::{Level, LevelFilter};

#[derive(Parser)]
#[clap(
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
)]
#[command(args_conflicts_with_subcommands = true)]
struct Cli {
    #[command(flatten)]
    global_arguments: GlobalArguments,

    #[command(subcommand)]
    mode: Option<CliMode>,
}

#[derive(Subcommand)]
enum CliMode {
    /// Run the source code (default)
    #[command(short_flag = 'r')]
    Run {
        #[command(flatten)]
        global_arguments: GlobalArguments,

        /// Do not print the program's output value
        #[arg(short, long)]
        no_output: bool,
    },

    /// Compile a chunk and show the disassembly
    #[command(short_flag = 'd')]
    Disassemble {
        #[command(flatten)]
        global_arguments: GlobalArguments,

        /// Style the disassembly output
        #[arg(short, long)]
        style: bool,
    },

    /// Create and display tokens from the source code
    #[command(short_flag = 't')]
    Tokenize {
        #[command(flatten)]
        global_arguments: GlobalArguments,

        /// Style the disassembly output
        #[arg(short, long)]
        style: bool,
    },
}

#[derive(Args, Clone)]
struct GlobalArguments {
    /// Log level, overrides the DUST_LOG environment variable
    ///
    /// Possible values: trace, debug, info, warn, error
    #[arg(short, long, value_name = "LOG_LEVEL")]
    log: Option<LevelFilter>,

    /// Source code
    ///
    /// Conflicts with the file argument
    #[arg(short, long, value_name = "SOURCE", conflicts_with = "file")]
    command: Option<String>,

    /// Path to a source code file
    #[arg(required_unless_present = "command")]
    file: Option<PathBuf>,
}

fn set_log_and_get_source(arguments: GlobalArguments, start_time: Instant) -> String {
    let GlobalArguments { command, file, log } = arguments;
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

    if let Some(level) = log {
        logger.filter_level(level).init();
    } else {
        logger.parse_env("DUST_LOG").init();
    }

    if let Some(source) = command {
        source
    } else {
        let path = file.expect("Path is required when command is not provided");

        read_to_string(path).expect("Failed to read file")
    }
}

fn main() {
    let start_time = Instant::now();
    let Cli {
        global_arguments,
        mode,
    } = Cli::parse();
    let mode = mode.unwrap_or(CliMode::Run {
        global_arguments,
        no_output: false,
    });

    if let CliMode::Run {
        global_arguments,
        no_output,
    } = mode
    {
        let source = set_log_and_get_source(global_arguments, start_time);
        let run_result = run(&source);

        match run_result {
            Ok(Some(value)) => {
                if !no_output {
                    println!("{}", value)
                }
            }
            Ok(None) => {}
            Err(error) => {
                eprintln!("{}", error.report());
            }
        }

        return;
    }

    if let CliMode::Disassemble {
        global_arguments,
        style,
    } = mode
    {
        let source = set_log_and_get_source(global_arguments, start_time);
        let chunk = match compile(&source) {
            Ok(chunk) => chunk,
            Err(error) => {
                eprintln!("{}", error.report());

                return;
            }
        };
        let disassembly = chunk.disassembler().style(style).disassemble();

        println!("{}", disassembly);

        return;
    }

    if let CliMode::Tokenize {
        global_arguments,
        style,
    } = mode
    {
        let source = set_log_and_get_source(global_arguments, start_time);
        let tokens = match lex(&source) {
            Ok(tokens) => tokens,
            Err(error) => {
                eprintln!("{}", error.report());

                return;
            }
        };
        let mut stdout = stdout().lock();

        write_token_list(&tokens, style, &mut stdout)
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
