use std::io::{self, stdout, Read, Write};
use std::time::{Duration, Instant};
use std::{fs::read_to_string, path::PathBuf};

use clap::builder::StyledStr;
use clap::{
    builder::{styling::AnsiColor, Styles},
    crate_authors, crate_description, crate_version, ArgAction, Args, ColorChoice, Parser,
    Subcommand, ValueHint,
};
use color_print::cstr;
use dust_lang::{CompileError, Compiler, DustError, DustString, Lexer, Span, Token, Vm};
use log::{Level, LevelFilter};

const HELP_TEMPLATE: &str = cstr!(
    "\
<bold,bright-magenta>Dust CLI</bold,bright-magenta>
────────
{about}
Version: {version}
Author: {author}
License: GPL-3.0 ⚖️

<bold,bright-magenta>Usage</bold,bright-magenta>
─────
{tab}{usage}

<bold,bright-magenta>Options</bold,bright-magenta>
───────
{options}

<bold,bright-magenta>Modes</bold,bright-magenta>
─────
{subcommands}

<bold,bright-magenta>Arguments</bold,bright-magenta>
─────────
{positionals}

"
);

const STYLES: Styles = Styles::styled()
    .header(AnsiColor::BrightMagenta.on_default().bold())
    .usage(AnsiColor::BrightWhite.on_default().bold())
    .literal(AnsiColor::BrightCyan.on_default())
    .placeholder(AnsiColor::BrightMagenta.on_default())
    .error(AnsiColor::BrightRed.on_default().bold())
    .valid(AnsiColor::Blue.on_default())
    .invalid(AnsiColor::BrightRed.on_default());

#[derive(Parser)]
#[clap(
    version = crate_version!(),
    author = crate_authors!(),
    about = crate_description!(),
    color = ColorChoice::Auto,
    disable_help_flag = true,
    disable_version_flag = true,
    help_template = StyledStr::from(HELP_TEMPLATE),
    styles = STYLES,
    term_width = 80,
)]
struct Cli {
    /// Print help information for this or the selected subcommand
    #[arg(short, long, action = ArgAction::Help)]
    help: bool,

    /// Print version information
    #[arg(short, long, action = ArgAction::Version)]
    version: bool,

    /// Log level, overrides the DUST_LOG environment variable
    #[arg(
        short,
        long,
        value_name = "LOG_LEVEL",
        value_parser = ["info", "trace", "debug"],
    )]
    log: Option<LevelFilter>,

    /// Source code to run instead of a file
    #[arg(short, long, value_hint = ValueHint::Other, value_name = "SOURCE")]
    command: Option<String>,

    /// Read source code from stdin
    #[arg(long)]
    stdin: bool,

    #[command(subcommand)]
    mode: Mode,

    /// Path to a source code file
    #[arg(value_hint = ValueHint::FilePath)]
    file: Option<PathBuf>,
}

#[derive(Subcommand)]
#[clap(
    help_template = StyledStr::from(HELP_TEMPLATE),
    styles = STYLES,
)]
enum Mode {
    /// Compile and run the program (default)
    #[command(short_flag = 'r')]
    Run {
        #[arg(short, long, action = ArgAction::Help)]
        #[clap(help_heading = Some("Options"))]
        help: bool,

        /// Print the time taken for compilation and execution
        #[arg(long)]
        #[clap(help_heading = Some("Run Options"))]
        time: bool,

        /// Do not print the program's return value
        #[arg(long)]
        #[clap(help_heading = Some("Run Options"))]
        no_output: bool,

        /// Custom program name, overrides the file name
        #[arg(long)]
        #[clap(help_heading = Some("Run Options"))]
        name: Option<DustString>,
    },

    /// Compile and print the bytecode disassembly
    #[command(short_flag = 'd')]
    Disassemble {
        #[arg(short, long, action = ArgAction::Help)]
        #[clap(help_heading = Some("Options"))]
        help: bool,

        /// Style disassembly output
        #[arg(short, long, default_value = "true")]
        #[clap(help_heading = Some("Disassemble Options"))]
        style: bool,

        /// Custom program name, overrides the file name
        #[arg(long)]
        #[clap(help_heading = Some("Disassemble Options"))]
        name: Option<DustString>,
    },

    /// Lex the source code and print the tokens
    #[command(short_flag = 't')]
    Tokenize {
        #[arg(short, long, action = ArgAction::Help)]
        #[clap(help_heading = Some("Options"))]
        help: bool,

        /// Style token output
        #[arg(short, long, default_value = "true")]
        #[clap(help_heading = Some("Tokenize Options"))]
        style: bool,
    },
}

#[derive(Args, Clone)]
#[group(required = true, multiple = false)]
struct Source {}

fn main() {
    let start_time = Instant::now();
    // let mut logger = env_logger::builder();

    // logger.format(move |buf, record| {
    //     let elapsed = format!("T+{:.04}", start_time.elapsed().as_secs_f32()).dimmed();
    //     let level_display = match record.level() {
    //         Level::Info => "INFO".bold().white(),
    //         Level::Debug => "DEBUG".bold().blue(),
    //         Level::Warn => "WARN".bold().yellow(),
    //         Level::Error => "ERROR".bold().red(),
    //         Level::Trace => "TRACE".bold().purple(),
    //     };
    //     let display = format!("[{elapsed}] {level_display:5} {args}", args = record.args());

    //     writeln!(buf, "{display}")
    // });

    let Cli {
        log,
        command,
        stdin,
        mode,
        file,
        ..
    } = Cli::parse();

    // if let Some(level) = log {
    //     logger.filter_level(level).init();
    // } else {
    //     logger.parse_env("DUST_LOG").init();
    // }

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
    let program_name = match &mode {
        Mode::Run { name, .. } => name,
        Mode::Disassemble { name, .. } => name,
        Mode::Tokenize { .. } => &None,
    }
    .iter()
    .next()
    .cloned()
    .or(file_name);

    if let Mode::Disassemble { style, .. } = mode {
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
            .style(style)
            .source(&source)
            .width(80)
            .disassemble()
            .expect("Failed to write disassembly to stdout");

        return;
    }

    if let Mode::Tokenize { style, .. } = mode {
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

    if let Mode::Run {
        time, no_output, ..
    } = mode
    {
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

        if time {
            print_time(compile_end);
        }

        let vm = Vm::new(chunk);
        let return_value = vm.run();
        let run_end = start_time.elapsed();

        if let Some(value) = return_value {
            if !no_output {
                println!("{}", value)
            }
        }

        if time {
            let run_time = run_end - compile_end;

            print_time(run_time);
        }
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
