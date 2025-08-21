#![feature(duration_millis_float, formatting_options, iter_intersperse)]

use std::{
    fmt::{self, Formatter, FormattingOptions},
    fs::OpenOptions,
    io::{self, Read, stdout},
    path::PathBuf,
    thread::{self},
    time::{Duration, Instant},
};

use clap::{
    Args, ColorChoice, Parser, Subcommand, ValueEnum, ValueHint,
    builder::{Styles, styling::AnsiColor},
    crate_authors, crate_description, crate_version,
};
use colored::{Color, Colorize};
use dust_lang::{Disassembler, Program, TuiDisassembler, compile, parser::parse};
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
    run_options: RunOptions,
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

    /// Disable the standard library, defaults to false
    #[arg(long)]
    no_std: bool,

    /// Custom program name, overrides the file name
    #[arg(short, long)]
    name: Option<String>,

    #[command(flatten)]
    source: Source,
}

impl SharedOptions {
    fn intersect(self, other: Self) -> Self {
        Self {
            log: self.log.or(other.log),
            pretty_log: self.pretty_log || other.pretty_log,
            time: self.time || other.time,
            no_output: self.no_output || other.no_output,
            no_std: self.no_std || other.no_std,
            name: self.name.or(other.name),
            source: Source {
                eval: self.source.eval.or(other.source.eval),
                stdin: self.source.stdin || other.source.stdin,
                file: self.source.file.or(other.source.file),
            },
        }
    }
}

#[derive(Subcommand)]
enum Mode {
    #[command(alias = "p")]
    Parse {
        #[command(flatten)]
        shared_options: SharedOptions,
    },

    /// Compile and run the program (default)
    #[command(alias = "r")]
    Run(RunOptions),

    /// Compile and output the compiled program
    #[command(alias = "c")]
    Compile {
        #[command(flatten)]
        shared_options: SharedOptions,

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

#[derive(Args)]
struct RunOptions {
    #[command(flatten)]
    shared_options: SharedOptions,

    /// Input format
    #[arg(short, long, default_value = "dust", value_name = "FORMAT")]
    input: Format,

    /// Minimum heap size garbage collection is triggered
    #[arg(long, value_name = "BYTES", requires = "min_sweep")]
    min_heap: Option<usize>,

    /// Minimum bytes allocated between garbage collections
    #[arg(long, value_name = "BYTES", requires = "min_heap")]
    min_sweep: Option<usize>,
}

impl RunOptions {
    fn intersect(self, other: Self) -> Self {
        Self {
            shared_options: self.shared_options.intersect(other.shared_options),
            input: if self.input != Format::Dust {
                self.input
            } else {
                other.input
            },
            min_heap: self.min_heap.or(other.min_heap),
            min_sweep: self.min_sweep.or(other.min_sweep),
        }
    }
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

#[derive(ValueEnum, Clone, Copy, PartialEq)]
enum Format {
    Dust,
    Json,
    Ron,
    Postcard,
    Yaml,
}

fn main() {
    let start_time = Instant::now();
    let Cli {
        mode,
        run_options: options_before_command,
    } = Cli::parse();
    let mode = match mode {
        Some(Mode::Run(run_mode_options)) => {
            Mode::Run(options_before_command.intersect(run_mode_options))
        }
        Some(Mode::Parse { shared_options }) => Mode::Parse {
            shared_options: options_before_command
                .shared_options
                .intersect(shared_options),
        },
        Some(Mode::Compile {
            shared_options,
            output,
            style,
        }) => Mode::Compile {
            shared_options: options_before_command
                .shared_options
                .intersect(shared_options),
            output,
            style,
        },
        Some(Mode::Tokenize) => Mode::Tokenize,
        None => Mode::Run(options_before_command),
    };

    // if let Mode::Run(RunOptions {
    //     input,
    //     min_heap,
    //     min_sweep,
    //     shared_options: options,
    // }) = mode
    // {
    //     let SharedOptions {
    //         log,
    //         pretty_log,
    //         time,
    //         no_output,
    //         no_std,
    //         name,
    //         source: Source { eval, stdin, file },
    //     } = options;

    //     if let Some(log_level) = log {
    //         start_logging(log_level, pretty_log, start_time);
    //     }

    //     let (source, source_name) = get_source_and_name(file, name, stdin, eval);
    //     let source_name = source_name.as_deref();

    //     let dust_program = match input {
    //         Format::Dust => {
    //             let compiler = Compiler::new();

    //             match compiler.compile_program(source_name, &source, !no_std) {
    //                 Ok(chunk) => chunk,
    //                 Err(error) => {
    //                     handle_compile_error(error, &source);

    //                     return;
    //                 }
    //             }
    //         }
    //         Format::Json => {
    //             serde_json::from_str(&source).expect("Failed to deserialize JSON into chunk")
    //         }
    //         Format::Postcard => {
    //             todo!()
    //         }
    //         Format::Ron => {
    //             ron::de::from_str(&source).expect("Failed to deserialize RON into chunk")
    //         }
    //         Format::Yaml => {
    //             serde_yaml::from_str(&source).expect("Failed to deserialize YAML into chunk")
    //         }
    //     };
    //     let compile_time = start_time.elapsed();
    //     let prototypes = dust_program.prototypes.clone();
    //     let vm = JitVm::new();
    //     let min_heap = min_heap.unwrap_or(MINIMUM_OBJECT_HEAP_DEFAULT);
    //     let min_sweep = min_sweep.unwrap_or(MINIMUM_OBJECT_SWEEP_DEFAULT);
    //     let run_result = vm.run(dust_program, min_heap, min_sweep);
    //     let run_time = start_time.elapsed() - compile_time;

    //     let return_value = match run_result {
    //         Ok(value) => value,
    //         Err(dust_error) => {
    //             let report = dust_error.report();

    //             if !no_output {
    //                 eprintln!("{report}");
    //             }

    //             return;
    //         }
    //     };

    //     if !no_output && let Some(return_value) = return_value {
    //         let mut buffer = String::new();

    //         let _ = return_value.display(
    //             &mut Formatter::new(&mut buffer, FormattingOptions::default()),
    //             &prototypes,
    //         );

    //         println!("{buffer}");
    //     }

    //     if time && !no_output {
    //         print_times(&[(source_name, compile_time, Some(run_time))]);
    //     }

    //     return;
    // }

    if let Mode::Parse { shared_options } = mode {
        let SharedOptions {
            log,
            pretty_log,
            time,
            no_output,
            no_std,
            name,
            source: Source { eval, stdin, file },
        } = shared_options;

        if let Some(log_level) = log {
            start_logging(log_level, pretty_log, start_time);
        }

        let (source, source_name) = get_source_and_name(file, name, stdin, eval);
        let (syntax_tree, error) = parse(&source);
        let parse_time = start_time.elapsed();

        println!("{syntax_tree:#?}");
        println!("{}", syntax_tree.display_node_tree());

        if let Some(error) = error
            && !no_output
        {
            eprintln!("{}", error.report());
        }

        if time && !no_output {
            print_times(&[(source_name.as_deref(), parse_time, None)]);
        }

        return;
    }

    if let Mode::Compile {
        shared_options:
            SharedOptions {
                log,
                pretty_log,
                time,
                no_output,
                no_std,
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

        let (source, source_name) = get_source_and_name(file, name, stdin, eval);
        let source_name = source_name.as_deref();
        let compile_result = compile(&source);
        let compile_time = start_time.elapsed();

        match compile_result {
            Ok(chunk) => {
                let program = Program {
                    prototypes: vec![chunk],
                    cell_count: 0,
                };
                let mut stdout = stdout();

                let mut disassembler = Disassembler::new(&program, &mut stdout);

                disassembler.disassemble().unwrap();
            }
            Err(error) => eprintln!("{}", error.report()),
        }

        // let dust_program = match compiler.compile_program(source_name, &source, !no_std) {
        //     Ok(dust_crate) => dust_crate,
        //     Err(error) => {
        //         todo!("Handle compile error: {error}");

        //         return;
        //     }
        // };

        // match output {
        //     Format::Dust => {
        //         let disassembler = TuiDisassembler::new(&dust_program, Some(&source));

        //         disassembler
        //             .disassemble()
        //             .expect("Failed to display disassembly");

        //         // disassembler
        //         //     .source(&source)
        //         //     .style(style)
        //         //     .show_type(true)
        //         //     .disassemble()
        //         //     .expect("Failed to write disassembly to stdout");
        //     }
        //     Format::Json => {
        //         let json = serde_json::to_string_pretty(&dust_program)
        //             .expect("Failed to serialize chunk to JSON");

        //         println!("{json}");
        //     }
        //     Format::Postcard => {
        //         let mut buffer = Vec::new();
        //         let postcard = postcard::to_slice_cobs(&dust_program, &mut buffer)
        //             .expect("Failed to serialize chunk to Postcard");

        //         println!("{postcard:?}");
        //     }
        //     Format::Ron => {
        //         let ron = ron::ser::to_string_pretty(
        //             &dust_program,
        //             PrettyConfig::new().struct_names(true),
        //         )
        //         .expect("Failed to serialize chunk to RON");

        //         println!("{ron}");
        //     }
        //     Format::Yaml => {
        //         let yaml = serde_yaml::to_string(&dust_program)
        //             .expect("Failed to serialize chunk to YAML");

        //         println!("{yaml}");
        //     }
        // }

        if time && !no_output {
            print_times(&[(source_name, compile_time, None)]);
        }

        return;
    }

    if let Mode::Tokenize = mode {
        todo!()
    }
}

fn get_source_and_name(
    path: Option<PathBuf>,
    name: Option<String>,
    stdin: bool,
    eval: Option<String>,
) -> (String, Option<String>) {
    if let Some(path) = &path {
        let file_name = path
            .file_stem()
            .expect("The path `{path}` contains invalid UTF-8")
            .to_string_lossy()
            .to_string();
        let mut file = OpenOptions::new()
            .create(false)
            .read(true)
            .write(false)
            .open(path)
            .expect("Failed to open {path}");
        let mut file_contents = String::new();

        file.read_to_string(&mut file_contents)
            .expect("The file at `{path}` contains invalid UTF-8");

        (file_contents, Some(file_name))
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

        (source, name)
    }
}

fn start_logging(level: LevelFilter, use_pretty: bool, start_time: Instant) {
    if use_pretty {
        tracing_subscriber::fmt()
            .with_env_filter(format!("none,dust_lang={level}"))
            .event_format(PrettyLogFormatter { start_time })
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(format!("none,dust_lang={level}"))
            .event_format(SimpleLogFormatter { start_time })
            .init();
    }
}

struct SimpleLogFormatter {
    start_time: Instant,
}

impl<S, N> FormatEvent<S, N> for SimpleLogFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        context: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        use colored::Colorize;

        let elapsed = self.start_time.elapsed().as_millis_f64();
        let level = event.metadata().level();
        let scopes = context
            .event_scope()
            .map(|scope| scope.from_root().collect::<Vec<_>>())
            .unwrap_or_default();

        let (emoji, colorized_level) = match *level {
            Level::ERROR => ("🕱", "ERROR".red().bold()),
            Level::WARN => ("⚠", "WARN".yellow().bold()),
            Level::INFO => ("🛈", "INFO".blue().bold()),
            Level::DEBUG => ("🕷", "DEBUG".green().bold()),
            Level::TRACE => ("🖙", "TRACE".cyan().bold()),
        };

        write!(
            writer,
            "{} {}  {:5}",
            format!("{elapsed:.5}ms").dimmed(),
            emoji,
            colorized_level,
        )?;

        if !scopes.is_empty() {
            let span_names = scopes
                .iter()
                .map(|span| span.metadata().name())
                .collect::<Vec<_>>();
            write!(writer, " {}", span_names.join("::").bold())?;
        }

        write!(writer, " ")?;
        context.format_fields(writer.by_ref(), event)?;
        writeln!(writer)
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

        context: &FmtContext<'_, S, N>,

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
        let scopes = context
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

        context.format_fields(Writer::new(&mut message), event)?;
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

fn print_times(times: &[(Option<&str>, Duration, Option<Duration>)]) {
    for (source_name, compile_time, run_time) in times {
        let name = source_name.unwrap_or("anonymous");
        let total_time = run_time
            .map(|run_time| run_time + *compile_time)
            .unwrap_or(*compile_time);
        let compile_time_display = format!("{}ms", compile_time.as_millis_f64());
        let run_time_display = run_time
            .map(|run_time| format!("{}ms", run_time.as_millis_f64()))
            .unwrap_or("none".to_string());
        let total_time_display = format!("{}ms", total_time.as_millis_f64());

        println!(
            "{name}: Compile time = {compile_time_display} Run time = {run_time_display} Total = {total_time_display}"
        );
    }
}

// fn handle_compile_error(error: CompileError, source: &str) {
//     let dust_error = DustError::compile(error, source);
//     let report = dust_error.report();

//     eprintln!("{report}");
// }

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }
}
