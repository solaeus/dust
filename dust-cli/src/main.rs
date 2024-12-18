use std::io::{self, stdout, Read, Write};
use std::time::{Duration, Instant};
use std::{fs::read_to_string, path::PathBuf};

use clap::builder::StyledStr;
use clap::{
    builder::{styling::AnsiColor, Styles},
    ArgAction, Args, ColorChoice, Parser, ValueHint,
};
use clap::{crate_authors, crate_description, crate_version};
use colored::Colorize;
use dust_lang::{CompileError, Compiler, DustError, DustString, Lexer, Span, Token, Vm};
use log::{Level, LevelFilter};

const HELP_TEMPLATE: &str = "\
{about}
{version}
{author}

{usage-heading}
{usage}

{all-args}
";

#[derive(Parser)]
#[clap(
    version = crate_version!(),
    author = crate_authors!(),
    about = crate_description!(),
    term_width = 80,
    color = ColorChoice::Auto,
    styles = Styles::styled()
        .header(AnsiColor::BrightMagenta.on_default().bold())
        .usage(AnsiColor::BrightWhite.on_default().bold())
        .literal(AnsiColor::BrightCyan.on_default())
        .placeholder(AnsiColor::BrightGreen.on_default())
        .error(AnsiColor::BrightRed.on_default().bold())
        .valid(AnsiColor::Blue.on_default())
        .invalid(AnsiColor::BrightRed.on_default()),
    disable_help_flag = true,
    disable_version_flag = true,
    help_template = StyledStr::from(HELP_TEMPLATE.bright_white().bold().to_string()),
)]
struct Cli {
    /// Log level, overrides the DUST_LOG environment variable
    #[arg(
        short,
        long,
        value_name = "LOG_LEVEL",
        value_parser = ["info", "trace", "debug"],
    )]
    #[clap(help_heading = Some("- Options"))]
    log: Option<LevelFilter>,

    #[arg(short, long, action = ArgAction::Help)]
    #[clap(help_heading = Some("- Options"))]
    help: bool,

    #[arg(short, long, action = ArgAction::Version)]
    #[clap(help_heading = Some("- Options"))]
    version: bool,

    #[command(flatten)]
    mode: Modes,

    #[command(flatten)]
    source: Source,
}

#[derive(Args)]
#[group(multiple = true, requires = "run")]
struct RunOptions {
    /// Print the time taken for compilation and execution
    #[arg(long)]
    #[clap(help_heading = Some("- Run Options"))]
    time: bool,

    /// Do not print the run result
    #[arg(long)]
    #[clap(help_heading = Some("- Run Options"))]
    no_output: bool,

    /// Custom program name, overrides the file name
    #[arg(long)]
    #[clap(help_heading = Some("- Run Options"))]
    program_name: Option<DustString>,
}

#[derive(Args)]
#[group(multiple = false)]
struct Modes {
    /// Run the source code (default)
    ///
    /// Use the RUN OPTIONS to control this mode
    #[arg(short, long, default_value = "true")]
    #[clap(help_heading = Some("- Modes"))]
    run: bool,

    #[command(flatten)]
    run_options: RunOptions,

    /// Compile a chunk and show the disassembly
    #[arg(short, long)]
    #[clap(help_heading = Some("- Modes"))]
    disassemble: bool,

    /// Lex and display tokens from the source code
    #[arg(short, long)]
    #[clap(help_heading = Some("- Modes"))]
    tokenize: bool,

    /// Style disassembly or tokenization output
    #[arg(short, long, default_value = "true")]
    #[clap(help_heading = Some("- Modes"))]
    style: bool,
}

#[derive(Args, Clone)]
#[group(required = true, multiple = false)]
struct Source {
    /// Source code to use instead of a file
    #[arg(short, long, value_hint = ValueHint::Other, value_name = "SOURCE")]
    #[clap(help_heading = Some("- Input"))]
    command: Option<String>,

    /// Read source code from stdin
    #[arg(long)]
    #[clap(help_heading = Some("- Input"))]
    stdin: bool,

    /// Path to a source code file
    #[arg(value_hint = ValueHint::FilePath)]
    #[clap(help_heading = Some("- Input"))]
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
        source: Source {
            command,
            file,
            stdin,
        },
        mode,
        ..
    } = Cli::parse();

    if let Some(level) = log {
        logger.filter_level(level).init();
    } else {
        logger.parse_env("DUST_LOG").init();
    }

    let (source, file_name) = if let Some(source) = command {
        (source, None)
    } else if stdin {
        let mut source = String::new();

        io::stdin()
            .read_to_string(&mut source)
            .expect("Failed to read from stdin");

        (source, None)
    } else {
        let path = file.expect("Path is required when command is not provided");
        let source = read_to_string(&path).expect("Failed to read file");
        let file_name = path
            .file_name()
            .and_then(|os_str| os_str.to_str())
            .map(DustString::from);

        (source, file_name)
    };
    let program_name = mode.run_options.program_name.or(file_name);

    if mode.disassemble {
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

        let chunk = compiler.finish(program_name);
        let mut stdout = stdout().lock();

        chunk
            .disassembler(&mut stdout)
            .style(mode.style)
            .source(&source)
            .width(70)
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

    let chunk = compiler.finish(program_name);
    let compile_end = start_time.elapsed();

    if mode.run_options.time {
        print_time(compile_end);
    }

    let vm = Vm::new(chunk);
    let return_value = vm.run();
    let run_end = start_time.elapsed();

    if let Some(value) = return_value {
        if !mode.run_options.no_output {
            println!("{}", value)
        }
    }

    if mode.run_options.time {
        let run_time = run_end - compile_end;

        print_time(run_time);
    }
}

fn print_time(instant: Duration) {
    let seconds = instant.as_secs_f64();

    match seconds {
        ..=0.001 => {
            println!(
                "Compile time: {microseconds} microseconds",
                microseconds = seconds * 1_000_000.0
            );
        }
        ..=0.1 => {
            println!(
                "Compile time: {milliseconds} milliseconds",
                milliseconds = seconds * 1000.0
            );
        }
        _ => {
            println!("Compile time: {seconds} seconds");
        }
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
