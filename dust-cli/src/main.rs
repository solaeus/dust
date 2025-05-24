#![feature(duration_millis_float, iter_intersperse)]

use std::{
    fmt::{self},
    fs::OpenOptions,
    io::{self, Read, stdout},
    path::PathBuf,
    sync::Arc,
    thread::{self},
    time::{Duration, Instant},
};

use clap::{
    Args, ColorChoice, Parser, Subcommand, ValueEnum, ValueHint,
    builder::{Styles, styling::AnsiColor},
    crate_authors, crate_description, crate_version,
};
use colored::{Color, Colorize};
use dust_lang::{
    CompileError, Compiler, DEFAULT_REGISTER_COUNT, DustError, DustString, Lexer, Vm,
    compiler::CompileMode, panic::set_dust_panic_hook,
};
use ron::ser::PrettyConfig;
use tracing::{Event, Level, Subscriber, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt::{FmtContext, FormatEvent, FormatFields, format::Writer},
    registry::LookupSpan,
};

#[derive(Parser)]
#[clap(
    version = crate_version!(),
    author = crate_authors!(),
    about = crate_description!(),
    color = ColorChoice::Auto,
    styles = Styles::styled()
        .header(AnsiColor::BrightMagenta.on_default().bold().underline())
        .usage(AnsiColor::BrightMagenta.on_default().bold().underline())
        .literal(AnsiColor::BrightCyan.on_default().bold())
        .placeholder(AnsiColor::BrightCyan.on_default().bold())
        .valid(AnsiColor::BrightGreen.on_default())
        .invalid(AnsiColor::BrightYellow.on_default())
        .error(AnsiColor::BrightRed.on_default()),
)]
struct Cli {
    #[command(subcommand)]
    mode: Option<Mode>,

    #[command(flatten)]
    options: SharedOptions,
}

#[derive(Args)]
#[group(required = false, multiple = false)]
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

#[derive(Args)]
#[group(required = false, multiple = true)]
struct SharedOptions {
    /// Possible log levels: error, warn, info, debug, trace
    #[arg(short, long, value_name = "LEVEL")]
    log: Option<LevelFilter>,

    /// Use the pretty formatter for logging, defaults to false
    #[arg(short, long, default_value = "false")]
    pretty_log: bool,

    /// Print the time taken for compilation and execution, defaults to false
    #[arg(short, long)]
    time: bool,

    /// Disable printing, defaults to false
    #[arg(long)]
    no_output: bool,

    /// Custom program name, overrides the file name
    #[arg(short, long)]
    name: Option<DustString>,

    #[command(flatten)]
    source: Source,
}

#[derive(Subcommand)]
enum Mode {
    /// Compile and run the program (default)
    #[command(alias = "r")]
    Run {
        #[command(flatten)]
        options: SharedOptions,

        /// Input format
        #[arg(short, long, default_value = "dust", value_name = "FORMAT")]
        input: Format,
    },

    /// Compile and output the compiled program
    #[command(alias = "c")]
    Compile {
        #[command(flatten)]
        options: SharedOptions,

        /// Defaults to "dust", which is the disassembly output
        #[arg(short, long, default_value = "dust", value_name = "FORMAT")]
        output: Format,

        /// Style disassembly output, defaults to true
        #[arg(short, long, default_value = "true")]
        style: bool,
    },

    /// Lex the source code and print the tokens
    #[command(alias = "t")]
    Tokenize,
}

#[derive(ValueEnum, Clone, Copy)]
enum Format {
    Dust,
    Json,
    Ron,
    Postcard,
    Yaml,
}

fn main() {
    let start_time = Instant::now();

    set_dust_panic_hook();

    let Cli { mode, options } = Cli::parse();
    let mode = mode.unwrap_or(Mode::Run {
        input: Format::Dust,
        options,
    });

    if let Mode::Run {
        input,
        options:
            SharedOptions {
                log,
                pretty_log,
                time,
                no_output,
                name,
                source: Source { eval, stdin, file },
            },
    } = mode
    {
        if let Some(log_level) = log {
            start_logging(log_level, pretty_log, start_time);
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
        let lexer = Lexer::new(&source);
        let chunk = match input {
            Format::Dust => {
                let mut compiler = match Compiler::<DEFAULT_REGISTER_COUNT>::new(
                    lexer,
                    CompileMode::Main {
                        name: Some(source_name.clone()),
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
            Format::Json => {
                serde_json::from_str(&source).expect("Failed to deserialize JSON into chunk")
            }
            Format::Postcard => {
                todo!()
            }
            Format::Ron => {
                ron::de::from_str(&source).expect("Failed to deserialize RON into chunk")
            }
            Format::Yaml => {
                serde_yaml::from_str(&source).expect("Failed to deserialize YAML into chunk")
            }
        };
        let compile_time = start_time.elapsed();
        let vm = Vm::<DEFAULT_REGISTER_COUNT>::new(Arc::new(chunk));
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

        return;
    }

    if let Mode::Compile {
        options:
            SharedOptions {
                log,
                pretty_log,
                time,
                no_output,
                name,
                source: Source { eval, stdin, file },
            },
        output,
        style,
    } = mode
    {
        if let Some(log_level) = log {
            start_logging(log_level, pretty_log, start_time);
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
        let lexer = Lexer::new(&source);
        let mut compiler = match Compiler::<DEFAULT_REGISTER_COUNT>::new(
            lexer,
            CompileMode::Main {
                name: Some(source_name.clone()),
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
            Format::Dust => {
                let mut stdout = stdout().lock();

                chunk
                    .disassembler(&mut stdout)
                    .width(65)
                    .style(style)
                    .source(&source)
                    .disassemble()
                    .expect("Failed to write disassembly to stdout");
            }
            Format::Json => {
                let json = serde_json::to_string_pretty(&chunk)
                    .expect("Failed to serialize chunk to JSON");

                println!("{json}");
            }
            Format::Postcard => {
                let mut buffer = Vec::new();
                let postcard = postcard::to_slice_cobs(&chunk, &mut buffer)
                    .expect("Failed to serialize chunk to Postcard");

                println!("{postcard:?}");
            }
            Format::Ron => {
                let ron =
                    ron::ser::to_string_pretty(&chunk, PrettyConfig::new().struct_names(true))
                        .expect("Failed to serialize chunk to RON");

                println!("{ron}");
            }
            Format::Yaml => {
                let yaml =
                    serde_yaml::to_string(&chunk).expect("Failed to serialize chunk to YAML");

                println!("{yaml}");
            }
        }

        if time && !no_output {
            print_times(&[(&source_name, compile_time, None)]);
        }

        return;
    }

    if let Mode::Tokenize = mode {
        todo!()
    }
}

fn start_logging(level: LevelFilter, use_pretty: bool, start_time: Instant) {
    if use_pretty {
        tracing_subscriber::fmt()
            .with_max_level(level)
            .event_format(PrettyLogFormatter { start_time })
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(level)
            .with_writer(io::stdout)
            .init();
    }
}

struct PrettyLogFormatter {
    start_time: Instant,
}

impl<S, N> FormatEvent<S, N> for PrettyLogFormatter
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
        let level = event.metadata().level();
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
            "{left_aligned:<20} {scopes:40} {right_aligned:>32}",
            left_aligned = format!(
                "{border}{:^7}{border}{}",
                level_display,
                thread_display,
                border = "│".color(level_color),
            ),
            scopes = scopes
                .iter()
                .map(|scope| scope.to_string())
                .collect::<Vec<_>>()
                .join("->")
                .to_string(),
            right_aligned = format!("{} {}", time_display.to_string(), "│".color(level_color),)
        )?;
        writeln!(
            writer,
            "{border}       {border}{border:>71}",
            border = "│".color(level_color),
        )?;

        let mut message = String::new();

        ctx.format_fields(Writer::new(&mut message), event)?;
        writeln!(
            writer,
            "{border}       {border}{message:len$}{border}",
            border = "│".color(level_color),
            len = message.len().max(70),
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
