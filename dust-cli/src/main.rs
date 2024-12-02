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
    #[arg(short, long, value_name = "LOG_LEVEL")]
    log: Option<LevelFilter>,

    /// Source code sent via command line
    #[arg(short, long, value_name = "SOURCE", conflicts_with = "path")]
    command: Option<String>,

    /// File to read source code from
    #[arg(required_unless_present = "command")]
    path: Option<PathBuf>,
}

impl GlobalArguments {
    fn set_log_and_get_source(self, start_time: Instant) -> String {
        let GlobalArguments { command, path, log } = self;
        let mut logger = env_logger::builder();

        logger.format(move |buf, record| {
            let elapsed = start_time.elapsed().as_nanos();
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
            let display = format!(
                "{elapsed} {level_display:5} {module:^6} {args}",
                args = record.args()
            );

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
            let path = path.expect("Path is required when command is not provided");

            read_to_string(path).expect("Failed to read file")
        }
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
        let source = global_arguments.set_log_and_get_source(start_time);
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
        let source = global_arguments.set_log_and_get_source(start_time);
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
        let source = global_arguments.set_log_and_get_source(start_time);
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
