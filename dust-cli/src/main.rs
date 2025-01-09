use std::{
    fs::read_to_string,
    io::{self, stdout, Read},
    path::PathBuf,
    time::{Duration, Instant},
};

use clap::{
    builder::{
        styling::{AnsiColor, Color},
        StyledStr, Styles,
    },
    crate_authors, crate_description, crate_version,
    error::ErrorKind,
    Args, ColorChoice, Error, Parser, Subcommand, ValueHint,
};
use color_print::{cformat, cstr};
use dust_lang::{CompileError, Compiler, DustError, DustString, Lexer, Span, Token, Vm};
use tracing::{subscriber::set_global_default, Level};
use tracing_subscriber::FmtSubscriber;

const ABOUT: &str = cstr!(
    r#"
<bright-magenta,bold>Dust CLI
────────</>
{about}

<bold>⚙️ Version:</> {version}
<bold>🦀 Author:</> {author}
<bold>⚖️ License:</> GPL-3.0
<bold>🔬 Repository:</> https://git.jeffa.io/jeff/dust
"#
);

const PLAIN_ABOUT: &str = r#"
{about}
"#;

const USAGE: &str = cstr!(
    r#"
<bright-magenta,bold>Usage:</> {usage}
"#
);

const SUBCOMMANDS: &str = cstr!(
    r#"
<bright-magenta,bold>Modes:</>
{subcommands}
"#
);

const OPTIONS: &str = cstr!(
    r#"
<bright-magenta,bold>Options:</>
{options}
"#
);

const CREATE_MAIN_HELP_TEMPLATE: fn() -> String =
    || cformat!("{ABOUT}{USAGE}{SUBCOMMANDS}{OPTIONS}");

const CREATE_MODE_HELP_TEMPLATE: fn(&str) -> String = |title| {
    cformat!(
        "\
        <bright-magenta,bold>{title}\n────────</>\
        {PLAIN_ABOUT}{USAGE}{OPTIONS}
        "
    )
};

const STYLES: Styles = Styles::styled()
    .literal(AnsiColor::Cyan.on_default())
    .placeholder(AnsiColor::Cyan.on_default())
    .valid(AnsiColor::BrightCyan.on_default())
    .invalid(AnsiColor::BrightYellow.on_default())
    .error(AnsiColor::BrightRed.on_default());

#[derive(Parser)]
#[clap(
    version = crate_version!(),
    author = crate_authors!(),
    about = crate_description!(),
    color = ColorChoice::Auto,
    help_template = CREATE_MAIN_HELP_TEMPLATE(),
    styles = STYLES,
)]
struct Cli {
    /// Overrides the DUST_LOG environment variable
    #[arg(
        short,
        long,
        value_parser = |input: &str| match input.to_uppercase().as_str() {
            "TRACE" => Ok(Level::TRACE),
            "DEBUG" => Ok(Level::DEBUG),
            "INFO" => Ok(Level::INFO),
            "WARN" => Ok(Level::WARN),
            "ERROR" => Ok(Level::ERROR),
            _ => Err(Error::new(ErrorKind::ValueValidation)),
        }
    )]
    log_level: Option<Level>,

    #[command(subcommand)]
    mode: Option<Mode>,

    #[command(flatten)]
    run: Run,
}

#[derive(Args)]
struct Input {
    /// Source code to run instead of a file
    #[arg(short, long, value_hint = ValueHint::Other, value_name = "INPUT")]
    command: Option<String>,

    /// Read source code from stdin
    #[arg(long)]
    stdin: bool,

    /// Path to a source code file
    #[arg(value_hint = ValueHint::FilePath)]
    file: Option<PathBuf>,
}

/// Compile and run the program (default)
#[derive(Args)]
#[command(
    short_flag = 'r',
    help_template = CREATE_MODE_HELP_TEMPLATE("Run Mode")
)]
struct Run {
    /// Print the time taken for compilation and execution
    #[arg(long)]
    time: bool,

    /// Do not print the program's return value
    #[arg(long)]
    no_output: bool,

    /// Custom program name, overrides the file name
    #[arg(long)]
    name: Option<DustString>,

    #[command(flatten)]
    input: Input,
}

#[derive(Subcommand)]
#[clap(subcommand_value_name = "MODE", flatten_help = true)]
enum Mode {
    Run(Run),

    /// Compile and print the bytecode disassembly
    #[command(
        short_flag = 'd',
        help_template = CREATE_MODE_HELP_TEMPLATE("Disassemble Mode")
    )]
    Disassemble {
        /// Style disassembly output
        #[arg(short, long, default_value = "true")]
        style: bool,

        /// Custom program name, overrides the file name
        #[arg(long)]
        name: Option<DustString>,

        #[command(flatten)]
        input: Input,
    },

    /// Lex the source code and print the tokens
    #[command(
        short_flag = 't',
        help_template = CREATE_MODE_HELP_TEMPLATE("Tokenize Mode")
    )]
    Tokenize {
        /// Style token output
        #[arg(short, long, default_value = "true")]
        style: bool,

        #[command(flatten)]
        input: Input,
    },
}

fn get_source_and_file_name(input: Input) -> (String, Option<DustString>) {
    if let Some(path) = input.file {
        let source = read_to_string(&path).expect("Failed to read source file");
        let file_name = path
            .file_name()
            .and_then(|os_str| os_str.to_str())
            .map(DustString::from);

        return (source, file_name);
    }

    if input.stdin {
        let mut source = String::new();
        io::stdin()
            .read_to_string(&mut source)
            .expect("Failed to read from stdin");

        return (source, None);
    }

    let source = input.command.expect("No source code provided");

    (source, None)
}

fn main() {
    let start_time = Instant::now();
    let Cli {
        log_level,
        mode,
        run,
    } = Cli::parse();
    let mode = mode.unwrap_or(Mode::Run(run));
    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_thread_names(true)
        .with_file(false)
        .finish();

    set_global_default(subscriber).expect("Failed to set tracing subscriber");

    if let Mode::Disassemble { style, name, input } = mode {
        let (source, file_name) = get_source_and_file_name(input);
        let lexer = Lexer::new(&source);
        let program_name = name.or(file_name);
        let mut compiler = match Compiler::new(lexer, program_name, true) {
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

        let chunk = compiler.finish();
        let mut stdout = stdout().lock();

        chunk
            .disassembler(&mut stdout)
            .width(65)
            .style(style)
            .source(&source)
            .disassemble()
            .expect("Failed to write disassembly to stdout");

        return;
    }

    if let Mode::Tokenize { input, .. } = mode {
        let (source, _) = get_source_and_file_name(input);
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

    if let Mode::Run(Run {
        time,
        no_output,
        name,
        input,
    }) = mode
    {
        let (source, file_name) = get_source_and_file_name(input);
        let lexer = Lexer::new(&source);
        let program_name = name.or(file_name);
        let mut compiler = match Compiler::new(lexer, program_name, true) {
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

        let chunk = compiler.finish();
        let compile_end = start_time.elapsed();

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
            let total_time = compile_end + run_time;

            print_time("Compile Time", compile_end);
            print_time("Run Time", run_time);
            print_time("Total Time", total_time);
        }
    }
}

fn print_time(phase: &str, instant: Duration) {
    let seconds = instant.as_secs_f64();

    match seconds {
        ..=0.001 => {
            println!(
                "{phase:12}: {microseconds}µs",
                microseconds = (seconds * 1_000_000.0).round()
            );
        }
        ..=0.199 => {
            println!(
                "{phase:12}: {milliseconds}ms",
                milliseconds = (seconds * 1000.0).round()
            );
        }
        _ => {
            println!("{phase:12}: {seconds}s");
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
