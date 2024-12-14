use std::io::{stdout, Write};
use std::time::Instant;
use std::{fs::read_to_string, path::PathBuf};

use clap::{Args, Parser};
use colored::Colorize;
use dust_lang::{compile, CompileError, Compiler, DustError, Lexer, Span, Token, Vm};
use log::{Level, LevelFilter};

#[derive(Parser)]
#[clap(
    name = env!("CARGO_PKG_NAME"),
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

    /// Print the time taken to compile and run the source code
    #[arg(long, requires("run"))]
    time: bool,

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
        let mut stdout = stdout().lock();

        chunk
            .disassembler(&mut stdout)
            .style(mode.style)
            .source(&source)
            .disassemble()
            .expect("Failed to write disassembly to stdout");

        return;
    }

    if mode.tokenize {
        let mut lexer = Lexer::new(&source);
        let mut next_token = || -> Option<(Token, Span, bool)> {
            match lexer.next_token() {
                Ok((token, position)) => Some((token, position, lexer.is_eof())),
                Err(error) => {
                    let report = DustError::compile(CompileError::Lex(error), &source).report();

                    eprintln!("{report}");

                    None
                }
            }
        };

        println!("{:^66}", "Tokens");

        for _ in 0..66 {
            print!("-");
        }

        println!();
        println!("{:^21}|{:^22}|{:^22}", "Kind", "Value", "Position");

        for _ in 0..66 {
            print!("-");
        }

        println!();

        while let Some((token, position, is_eof)) = next_token() {
            if is_eof {
                break;
            }

            let token_kind = token.kind().to_string();
            let token = token.to_string();
            let position = position.to_string();

            println!("{token_kind:^21}|{token:^22}|{position:^22}");
        }

        return;
    }

    let lexer = Lexer::new(&source);
    let mut compiler = match Compiler::new(lexer) {
        Ok(compiler) => compiler,
        Err(error) => {
            handle_compile_error(error, &source);

            return;
        }
    };

    match compiler.compile() {
        Ok(()) => {}
        Err(error) => {
            handle_compile_error(error, &source);

            return;
        }
    }

    let chunk = compiler.finish(None, None);
    let compile_end = start_time.elapsed();

    let vm = Vm::new(&source, &chunk, None, None);
    let return_value = vm.run();
    let run_end = start_time.elapsed();

    if let Some(value) = return_value {
        if !mode.no_output {
            println!("{}", value)
        }
    }

    if mode.time {
        let compile_time = compile_end.as_micros();
        let run_time = run_end - compile_end;

        println!(
            "Compile time: {compile_time}µs Run time: {}s{}ms{}µs",
            run_time.as_secs(),
            run_time.subsec_millis(),
            run_time.subsec_micros()
        );
    }
}

fn handle_compile_error(error: CompileError, source: &str) {
    let dust_error = DustError::compile(error, source);
    let report = dust_error.report();

    eprintln!("{report}");
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
