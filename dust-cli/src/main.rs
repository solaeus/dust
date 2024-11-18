use std::{
    fmt::{self, Display, Formatter},
    fs::{read_to_string, File, OpenOptions},
    io::{stdin, stdout, Read, Write},
    time::Instant,
};

use clap::{Args, Parser, Subcommand, ValueEnum};
use colored::Colorize;
use dust_lang::{compile, display_token_list, format, lex, run_source, vm::run_chunk, Chunk};
use env_logger::Target;
use log::{Level, LevelFilter};

#[derive(Parser)]
#[command(version, author, about)]
struct Cli {
    #[command(flatten)]
    input: Input,

    #[command(flatten)]
    output: Output,

    /// Log level: INFO, DEBUG, TRACE, WARN or ERROR. This overrides the DUST_LOG environment variable.
    #[arg(short, long, value_enum, value_name = "LEVEL", group = "log")]
    log_level: Option<LevelFilter>,

    /// Path to a file for log output
    #[arg(long, group = "log")]
    log_file: Option<String>,

    /// Which CLI feature to use, defaults to "run"
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Args, Clone)]
#[group(required = true, multiple = true)]
struct Input {
    /// Path to a Dust source file. If this and `input` are not provided, input will be read from
    /// stdin.
    source_file: Option<String>,

    /// Dust source code to compile. If this and `source_file` are not provided, input will be read
    /// from stdin.
    #[arg(short, long, conflicts_with = "source_file")]
    input: Option<Vec<u8>>,

    /// The format of the input
    #[arg(short, long, value_enum, default_value = "source")]
    read_format: IoFormat,
}

#[derive(Args, Clone)]
#[group(required = false, multiple = false)]
struct Output {
    /// Path to a file for output. If not provided, output will be written to stdout.
    #[arg(short, long)]
    output_file: Option<String>,

    /// The format of the output
    #[arg(short, long, value_enum, default_value = "postcard")]
    write_format: IoFormat,
}

#[derive(Subcommand, Clone, Copy)]
enum Command {
    /// Compile Dust to an intermediate format
    Compile,
    /// Compile and display the disassembled chunk
    Disassemble {
        /// Whether to style the output
        #[arg(short, long, default_value = "true")]
        style: bool,
    },
    /// Format and display the source code
    Format {
        /// Whether to color the output
        #[arg(short, long, default_value = "true")]
        color: bool,

        /// Number of spaces per indent level
        #[arg(short, long, default_value = "4")]
        indent: usize,

        /// Whether to include line numbers in the output
        #[arg(short, long, default_value = "true")]
        line_numbers: bool,
    },
    /// Compile and run the Dust code
    Run,
    /// Lex and display the token list
    Tokenize {
        /// Whether to style the output
        #[arg(short, long, default_value = "true")]
        style: bool,
    },
}

#[derive(ValueEnum, Clone, Copy)]
enum IoFormat {
    Json,
    Postcard,
    Source,
}

impl Display for IoFormat {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            IoFormat::Json => write!(f, "json"),
            IoFormat::Postcard => write!(f, "postcard"),
            IoFormat::Source => write!(f, "source"),
        }
    }
}

fn main() -> Result<(), String> {
    let Cli {
        input,
        output,
        log_level,
        log_file,
        command,
    } = Cli::parse();
    let command = command.unwrap_or(Command::Run);
    let start = Instant::now();
    let mut logger = env_logger::builder();

    logger.format(move |buf, record| {
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
        let elapsed = start.elapsed().as_secs_f32();
        let elapsed_display = format!("T+{elapsed:0.09}").dimmed();
        let display = format!(
            "[{elapsed_display}] {level_display:5} {module:^6} {args}",
            args = record.args()
        );

        writeln!(buf, "{display}")
    });

    if let Some(path) = log_file {
        let log_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .expect("Failed to open log file");

        logger.target(Target::Pipe(Box::new(log_file)));
    }

    if let Some(level) = log_level {
        logger.filter_level(level).init();
    } else {
        logger.parse_env("DUST_LOG").init();
    }

    let input_bytes = if let Some(source_input) = input.input {
        source_input
    } else if let Some(path) = &input.source_file {
        let mut source_file = File::open(path).expect("Failed to open source file");
        let file_length = source_file
            .metadata()
            .expect("Failed to read file metadata")
            .len();
        let mut buffer = Vec::with_capacity(file_length as usize);

        source_file
            .read_to_end(&mut buffer)
            .expect("Failed to read source file");

        buffer
    } else {
        let mut buffer = Vec::new();

        stdin()
            .read_to_end(&mut buffer)
            .expect("Failed to read from stdin");

        buffer
    };

    match command {
        Command::Format {
            color,
            indent,
            line_numbers,
        } => {
            let source = String::from_utf8(input_bytes).expect("Failed to parse input as UTF-8");
            let formatted = match format(&source, color, indent, line_numbers) {
                Ok(formatted) => formatted,
                Err(dust_error) => {
                    let report = dust_error.report();

                    return Err(report);
                }
            };

            if let Some(path) = output.output_file {
                let mut file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(path)
                    .expect("Failed to open output file");

                file.write_all(formatted.as_bytes())
                    .expect("Failed to write to output file");
            } else {
                println!("{}", formatted);
            }

            return Ok(());
        }
        Command::Tokenize { style } => {
            let source = String::from_utf8(input_bytes).expect("Failed to parse input as UTF-8");
            let tokens = match lex(&source) {
                Ok(tokens) => tokens,
                Err(dust_error) => {
                    let report = dust_error.report();

                    return Err(report);
                }
            };

            if let Some(path) = output.output_file {
                let mut file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(path)
                    .expect("Failed to open output file");

                display_token_list(&tokens, style, &mut file)
            } else {
                display_token_list(&tokens, style, &mut stdout())
            }

            return Ok(());
        }
        _ => {}
    }

    let chunk = match input.read_format {
        IoFormat::Source => {
            let source = String::from_utf8(input_bytes).expect("Failed to parse input as UTF-8");

            match compile(&source) {
                Ok(chunk) => chunk,
                Err(dust_error) => {
                    let report = dust_error.report();

                    return Err(report);
                }
            }
        }
        IoFormat::Json => {
            serde_json::from_slice(&input_bytes).expect("Failed to deserialize chunk from JSON")
        }
        IoFormat::Postcard => {
            postcard::from_bytes(&input_bytes).expect("Failed to deserialize chunk from Postcard")
        }
    };

    match command {
        Command::Run => match run_chunk(&chunk) {
            Ok(Some(value)) => {
                println!("{}", value);

                return Ok(());
            }
            Ok(None) => return Ok(()),
            Err(dust_error) => {
                let report = dust_error.report();

                return Err(report);
            }
        },
        Command::Disassemble { style } => {
            let disassembly = chunk.disassembler().style(style).disassemble();

            println!("{disassembly}");

            return Ok(());
        }
        _ => {}
    }

    let output_bytes = match output.write_format {
        IoFormat::Source => {
            return Err("Invalid options, cannot compile chunk as source code.".to_string())
        }
        IoFormat::Json => serde_json::to_vec(&chunk).expect("Failed to serialize chunk as JSON"),
        IoFormat::Postcard => {
            let length = postcard::experimental::serialized_size(&chunk)
                .expect("Failed to calculate Postcard size");
            let mut buffer = vec![0_u8; length as usize];

            postcard::to_slice(&chunk, &mut buffer).expect("Failed to serialize chunk as Postcard");

            buffer
        }
    };

    if let Some(path) = output.output_file {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .expect("Failed to open output file");

        file.write_all(&output_bytes)
            .expect("Failed to write to output file");
    } else {
        stdout()
            .write_all(&output_bytes)
            .expect("Failed to write to stdout");
    }

    Ok(())
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
