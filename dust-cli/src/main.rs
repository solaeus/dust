#![feature(duration_millis_float, iter_intersperse)]

use std::{
    fmt::{self, Formatter, Write, write},
    fs::OpenOptions,
    io::{self, Read, stdout},
    path::PathBuf,
    thread::{self, scope},
    time::{Duration, Instant},
};

use clap::{
    Args, ColorChoice, Parser, Subcommand, ValueEnum, ValueHint,
    builder::{Styles, styling::AnsiColor},
    crate_authors, crate_description, crate_version,
};
use colored::{Color, ColoredString, Colorize};
use dust_lang::{
    CompileError, Compiler, DustError, DustString, Lexer, Span, Token, Vm, compiler::CompileMode,
    panic::set_dust_panic_hook,
};
use tracing::{Event, Level, Subscriber, field::Visit, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt::{
        FmtContext, FormatEvent, FormatFields, FormattedFields, format::Writer, time::FormatTime,
    },
    registry::LookupSpan,
};

const STYLES: Styles = Styles::styled()
    .header(AnsiColor::BrightMagenta.on_default().bold().underline())
    .usage(AnsiColor::BrightMagenta.on_default().bold().underline())
    .literal(AnsiColor::BrightCyan.on_default().bold())
    .placeholder(AnsiColor::BrightCyan.on_default().bold())
    .valid(AnsiColor::BrightGreen.on_default())
    .invalid(AnsiColor::BrightYellow.on_default())
    .error(AnsiColor::BrightRed.on_default());

#[derive(Parser)]
#[clap(
    version = crate_version!(),
    author = crate_authors!(),
    about = crate_description!(),
    color = ColorChoice::Auto,
    styles = STYLES,
)]
struct Cli {
    #[command(subcommand)]
    mode: Option<Mode>,

    /// Overrides the DUST_LOG environment variable
    #[arg(short, long)]
    log: Option<LevelFilter>,

    /// Print the time taken for compilation and execution
    #[arg(long)]
    time: bool,

    /// Do not print the program's return value
    #[arg(long)]
    no_output: bool,

    /// Custom program name, overrides the file name
    #[arg(long)]
    name: Option<DustString>,

    /// Input format
    #[arg(short, long, default_value = "dust")]
    input: InputFormat,

    /// Style disassembly output
    #[arg(short, long, default_value = "true")]
    style: bool,

    /// Custom program name, overrides the file name
    #[arg(short, long, default_value = "cli")]
    output: OutputFormat,

    #[command(flatten)]
    source: Source,
}

#[derive(Args)]
#[group(required = true, multiple = false)]
pub struct Source {
    /// Source code to run instead of a file
    #[arg(short, long, value_hint = ValueHint::Other, value_name = "INPUT")]
    eval: Option<String>,

    /// Read source code from stdin
    #[arg(long)]
    stdin: bool,

    /// Path to a source code file
    file: Option<PathBuf>,
}

#[derive(Subcommand, Clone)]
enum Mode {
    /// Compile and print the input
    Compile,

    /// Compile and run the program (default)
    Run,

    /// Lex the source code and print the tokens
    Tokenize,
}

#[derive(ValueEnum, Clone, Copy)]
enum InputFormat {
    Dust,
    Json,
    Ron,
    Yaml,
}

#[derive(ValueEnum, Clone, Copy)]
enum OutputFormat {
    Cli,
    Json,
    Ron,
    Yaml,
}

fn main() {
    let start_time = Instant::now();

    set_dust_panic_hook();

    let Cli {
        mode,
        log,
        time,
        no_output,
        name,
        input,
        source: Source { eval, stdin, file },
        style,
        output,
    } = Cli::parse();
    let mode = mode.unwrap_or(Mode::Run);

    if let Some(log_level) = log {
        start_logging(log_level, start_time);
    }

    let (source, source_name) = {
        if let Some(path) = file {
            let file_name = path
                .file_stem()
                .expect("The path `{path}` has no file name")
                .to_str()
                .map(DustString::from)
                .expect("The path `{path}` contains invalid UTF-8");
            let mut file = OpenOptions::new()
                .create(false)
                .read(true)
                .write(false)
                .open(path)
                .expect("Failed to open {path}");
            let mut file_contents = String::new();

            file.read_to_string(&mut file_contents)
                .expect("The file at `{path}` contains invalid UTF-8");

            (file_contents, file_name)
        } else {
            let source = if stdin {
                let mut source = String::new();

                io::stdin()
                    .read_to_string(&mut source)
                    .expect("The input from stdin contained invalid UTF-8");

                source
            } else {
                eval.expect("No source code provided")
            };

            (
                source,
                name.unwrap_or_else(|| DustString::from("CLI Input")),
            )
        }
    };

    if let Mode::Run = mode {
        let lexer = Lexer::new(&source);
        let chunk = match input {
            InputFormat::Dust => {
                let mut compiler = match Compiler::new(
                    lexer,
                    CompileMode::Main {
                        name: source_name.clone(),
                    },
                ) {
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

                compiler.finish()
            }
            InputFormat::Json => {
                serde_json::from_str(&source).expect("Failed to deserialize JSON into chunk")
            }
            InputFormat::Ron => {
                ron::de::from_str(&source).expect("Failed to deserialize RON into chunk")
            }
            InputFormat::Yaml => {
                serde_yaml::from_str(&source).expect("Failed to deserialize YAML into chunk")
            }
        };
        let compile_time = start_time.elapsed();
        let vm = Vm::new(chunk);
        let return_value = vm.run();
        let run_time = start_time.elapsed() - compile_time;

        if !no_output {
            if let Some(value) = return_value {
                println!("{value}")
            }
        }

        if time && !no_output {
            print_times(&[(&source_name, compile_time, Some(run_time))]);
        }
    }

    if let Mode::Compile = mode {
        let lexer = Lexer::new(&source);
        let mut compiler = match Compiler::new(
            lexer,
            CompileMode::Main {
                name: source_name.clone(),
            },
        ) {
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
        let compile_time = start_time.elapsed();

        match output {
            OutputFormat::Cli => {
                let mut stdout = stdout().lock();

                chunk
                    .disassembler(&mut stdout)
                    .width(65)
                    .style(style)
                    .source(&source)
                    .disassemble()
                    .expect("Failed to write disassembly to stdout");
            }
            OutputFormat::Json => {
                let json = serde_json::to_string_pretty(&chunk)
                    .expect("Failed to serialize chunk to JSON");

                println!("{json}");
            }
            OutputFormat::Ron => {
                let ron = ron::ser::to_string_pretty(&chunk, Default::default())
                    .expect("Failed to serialize chunk to RON");

                println!("{ron}");
            }
            OutputFormat::Yaml => {
                let yaml =
                    serde_yaml::to_string(&chunk).expect("Failed to serialize chunk to YAML");

                println!("{yaml}");
            }
        }

        if time && !no_output {
            print_times(&[(&source_name, compile_time, None)]);
        }
    }

    if let Mode::Tokenize = mode {
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
    }
}

fn start_logging(level: LevelFilter, start_time: Instant) {
    tracing_subscriber::fmt()
        .with_max_level(level)
        .event_format(LogFormatter { start_time })
        .init();
}

struct LogFormatter {
    start_time: Instant,
}

impl<S, N> FormatEvent<S, N> for LogFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,

        ctx: &FmtContext<'_, S, N>,

        mut writer: Writer<'_>,

        event: &Event<'_>,
    ) -> fmt::Result {
        let meta = event.metadata();
        let level = meta.level();
        let level_color = match *level {
            Level::ERROR => Color::Red,
            Level::WARN => Color::Yellow,
            Level::INFO => Color::White,
            Level::DEBUG => Color::Green,
            Level::TRACE => Color::Blue,
        };
        let level_display = level.as_str();
        let thread_display = thread::current().name().unwrap_or("anonymous").to_string();
        let scopes = ctx
            .event_scope()
            .map(|scope| scope.from_root())
            .unwrap()
            .map(|span| span.metadata().name())
            .collect::<Vec<_>>();

        let time = self.start_time.elapsed();
        let time_color = if time.as_secs() > 1 {
            Color::BrightRed
        } else if time.as_millis() > 1 {
            Color::BrightYellow
        } else {
            Color::BrightGreen
        };
        let time_display = format!(
            "s: {}, ms: {}, ns: {}",
            time.as_secs(),
            time.subsec_millis(),
            time.subsec_nanos()
        );

        writeln!(
            writer,
            "{}",
            "╭───────┬──────────────────────────────────────────────────────────────────────╮"
                .color(level_color)
        )?;
        writeln!(
            writer,
            "{border}{:^7}{border}{:10} {:40} {:24}{border}",
            level_display,
            thread_display.bright_cyan().to_string(),
            scopes
                .iter()
                .map(|scope| scope.to_string())
                .collect::<Vec<_>>()
                .join("->")
                .bright_magenta(),
            time_display.color(time_color),
            border = "│".color(level_color),
        )?;

        let mut message = String::new();

        ctx.format_fields(Writer::new(&mut message), event)?;
        writeln!(
            writer,
            "{border}       {border}{message:70}{border}",
            border = "│".color(level_color)
        )?;
        writeln!(
            writer,
            "{}",
            "╰───────┴──────────────────────────────────────────────────────────────────────╯"
                .color(level_color)
        )?;

        Ok(())
    }
}

fn print_times(times: &[(&str, Duration, Option<Duration>)]) {
    for (source_name, compile_time, run_time) in times {
        let total_time = run_time
            .map(|run_time| run_time + *compile_time)
            .unwrap_or(*compile_time);
        let compile_time_display = format!("{}ms", compile_time.as_millis_f64());
        let run_time_display = run_time
            .map(|run_time| format!("{}ms", run_time.as_millis_f64()))
            .unwrap_or("none".to_string());
        let total_time_display = format!("{}ms", total_time.as_millis_f64());

        println!(
            "{source_name}: Compile time = {compile_time_display} Run time = {run_time_display} Total = {total_time_display}"
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
