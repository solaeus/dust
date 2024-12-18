use std::io::{self, stdout, Read, Write};
use std::time::{Duration, Instant};
use std::{fs::read_to_string, path::PathBuf};

use clap::builder::StyledStr;
use clap::Args;
use clap::{
    builder::{styling::AnsiColor, Styles},
    crate_authors, crate_description, crate_version, ColorChoice, Parser, Subcommand, ValueHint,
};
use color_print::cstr;
use dust_lang::{CompileError, Compiler, DustError, DustString, Lexer, Span, Token, Vm};
use log::{Level, LevelFilter};

const CLI_HELP_TEMPLATE: &str = cstr!(
    r#"
<bright-magenta><bold>Dust CLI
────────</bold>
{about}</bright-magenta>

 ☑  Version: {version}
 ✎  Author: {author}
 ⚖️ License: GPL-3.0
 🌐 Repository: git.jeffa.io/jeff/dust

<bright-magenta,bold>Usage
─────</bright-magenta,bold>
{tab}{usage}

<bright-magenta,bold>Modes
─────</bright-magenta,bold>
{subcommands}

<bright-magenta,bold>Options
───────</bright-magenta,bold>
{options}
"#
);

const MODE_HELP_TEMPLATE: &str = cstr!(
    r#"
<bright-magenta><bold>Dust CLI
────────</bold>
{about}</bright-magenta>

 ☑  Version: {version}
 ✎  Author: {author}
 ⚖️ License: GPL-3.0
 🌐 Repository: git.jeffa.io/jeff/dust

<bright-magenta,bold>Usage
─────</bright-magenta,bold>
{tab}{usage}

<bright-magenta,bold>Options
───────</bright-magenta,bold>
{options}
"#
);

const STYLES: Styles = Styles::styled()
    .header(AnsiColor::BrightMagenta.on_default().bold())
    .usage(AnsiColor::BrightCyan.on_default().bold())
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
    help_template = StyledStr::from(CLI_HELP_TEMPLATE),
    styles = STYLES,
)]
struct Cli {
    /// Overrides the DUST_LOG environment variable
    #[arg(short, long, value_name = "LOG_LEVEL")]
    log: Option<LevelFilter>,

    #[command(subcommand)]
    mode: Mode,
}

#[derive(Args)]
#[clap(
    styles = STYLES,
)]
#[group()]
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

#[derive(Subcommand)]
#[clap(subcommand_value_name = "MODE", flatten_help = true)]
enum Mode {
    /// Compile and run the program (default)
    #[command(
        short_flag = 'r',
        help_template = MODE_HELP_TEMPLATE
    )]
    Run {
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
    },

    /// Compile and print the bytecode disassembly
    #[command(
        short_flag = 'd',
        help_template = MODE_HELP_TEMPLATE
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
        help_template = MODE_HELP_TEMPLATE
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
    let mut logger = env_logger::builder();

    logger.format(move |buf, record| {
        let elapsed = format!("T+{:.04}", start_time.elapsed().as_secs_f32());
        let level_display = match record.level() {
            Level::Info => cstr!("<bright-magenta,bold>INFO<bright-magenta,bold>"),
            Level::Trace => cstr!("<bright-cyan,bold>TRACE<bright-cyan,bold>"),
            Level::Debug => cstr!("<bright-blue,bold>DEBUG<bright-blue,bold>"),
            Level::Warn => cstr!("<bright-yellow,bold>WARN<bright-yellow,bold>"),
            Level::Error => cstr!("<bright-red,bold>ERROR<bright-red,bold>"),
        };
        let display = format!("[{elapsed}] {level_display:5} {args}", args = record.args());

        writeln!(buf, "{display}")
    });

    let Cli { log, mode } = Cli::parse();

    if let Some(level) = log {
        logger.filter_level(level).init();
    } else {
        logger.parse_env("DUST_LOG").init();
    }

    if let Mode::Disassemble { style, name, input } = mode {
        let (source, file_name) = get_source_and_file_name(input);
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

        let chunk = compiler.finish(file_name);
        let mut stdout = stdout().lock();

        chunk
            .disassembler(&mut stdout)
            .style(style)
            .source(&source)
            .width(65)
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

    if let Mode::Run {
        name,
        time,
        no_output,
        input,
    } = mode
    {
        let (source, file_name) = get_source_and_file_name(input);
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

        let chunk = compiler.finish(name.or(file_name));
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
