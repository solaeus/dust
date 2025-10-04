#![feature(
    duration_millis_float,
    formatting_options,
    iter_intersperse,
    iterator_try_collect
)]

mod cli;
mod compile;
mod parse;

use std::{
    fmt::{self},
    fs::{File, create_dir, create_dir_all},
    io::{self, Read, Write},
    path::PathBuf,
    time::{Duration, Instant},
};

use clap::Parser as CliParser;
use dust_lang::{
    lexer::Lexer,
    project::{EXAMPLE_PROGRAM, PROJECT_CONFIG_PATH, ProjectConfig},
    token::Token,
};
use memmap2::MmapOptions;
use tracing::{Event, Level, Subscriber, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt::{FmtContext, FormatEvent, FormatFields, format::Writer},
    registry::LookupSpan,
};

use crate::{
    cli::{Cli, InputOptions, Mode},
    compile::handle_compile_command,
    parse::handle_parse_command,
};

fn main() {
    let start_time = Instant::now();
    let Cli {
        mode,
        input: InputOptions { eval, stdin, path },
        log,
        time,
        no_output,
        no_std: _,
        name: _,
        min_heap: _,
        min_sweep: _,
    } = Cli::parse();

    if let Some(log_level) = log {
        start_logging(log_level, start_time);
    }

    // if let Mode::Run {
    //     min_heap,
    //     min_sweep,
    // } = mode
    // {
    //     let source = get_source(path, name, stdin, eval);
    //     let resolver = Resolver::new(true);
    //     let compiler = Compiler::new(source.clone(), resolver);
    //     let compile_result = compiler.compile();
    //     let compile_time = start_time.elapsed();

    //     let program = match compile_result {
    //         Ok(program) => Arc::new(program),
    //         Err(error) => {
    //             let report = error.report();

    //             if !no_output {
    //                 eprintln!("{report}");
    //             }

    //             return;
    //         }
    //     };

    //     let vm = JitVm::new();
    //     let min_heap = min_heap.unwrap_or(MINIMUM_OBJECT_HEAP_DEFAULT);
    //     let min_sweep = min_sweep.unwrap_or(MINIMUM_OBJECT_SWEEP_DEFAULT);
    //     let run_result = vm.run(program, min_heap, min_sweep);
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
    //         println!("{return_value}");
    //     }

    //     if time {
    //         print_times(&[(source.program_name(), compile_time, Some(run_time))]);
    //     }

    //     return;
    // }

    if mode == Mode::Parse {
        handle_parse_command(eval, no_output, time, start_time);

        return;
    }

    if mode == Mode::Compile {
        handle_compile_command(eval, no_output, time, start_time);

        return;
    }

    if mode == Mode::Tokenize {
        let tokenize_bytes = |source: &[u8]| {
            let mut lexer = Lexer::new(source);
            let tokens = lexer
                .try_collect::<Vec<Token>>()
                .expect("Failed to tokenize source");
            let tokenize_time = start_time.elapsed();

            if !no_output {
                for token in &tokens {
                    println!("{token}");
                }
            }

            if time {
                print_times(&[("Tokenization", tokenize_time, None)]);
            }
        };

        if let Some(path) = path {
            let file = File::open(&path).expect("Failed to open source file");
            let mmap =
                unsafe { MmapOptions::new().map(&file) }.expect("Failed to memory map source file");

            tokenize_bytes(&mmap);
        } else if stdin {
            let mut buffer = Vec::new();

            io::stdin()
                .read_to_end(&mut buffer)
                .expect("Failed to read from stdin");

            tokenize_bytes(&buffer);
        } else if let Some(eval) = eval {
            let mut lexer = Lexer::from_str(&eval);
            let tokens = lexer
                .try_collect::<Vec<Token>>()
                .expect("Failed to tokenize source");
            let tokenize_time = start_time.elapsed();

            if !no_output {
                for token in &tokens {
                    println!("{token}");
                }
            }

            if time {
                print_times(&[("Tokenization", tokenize_time, None)]);
            }
        } else {
            panic!("No readable input source provided");
        };

        return;
    }

    if mode == Mode::Init {
        let path = path.unwrap_or_else(|| PathBuf::from("."));

        if !path.exists() {
            create_dir_all(&path).expect("Failed to create project directory");
        } else if path.read_dir().unwrap().next().is_some() {
            eprintln!("The directory `{}` is not empty", path.display());

            return;
        }

        let example_config_path = path.join(PROJECT_CONFIG_PATH);
        let example_project_config = toml::to_string_pretty(&ProjectConfig::example())
            .expect("Failed to serialize example project config to TOML");

        File::create(&example_config_path)
            .expect("Failed to create project config file")
            .write_all(example_project_config.as_bytes())
            .expect("Failed to write to project config file");

        let src_path = path.join("src");

        create_dir(&src_path).expect("Failed to create `src` directory");

        let example_program_path = src_path.join("main.ds");

        File::create(&example_program_path)
            .expect("Failed to create example program file")
            .write_all(EXAMPLE_PROGRAM.as_bytes())
            .expect("Failed to write to example program file");

        println!("Initialized a new Dust project at `{}`", path.display());
    }
}

fn start_logging(level: LevelFilter, start_time: Instant) {
    tracing_subscriber::fmt()
        .with_env_filter(format!("none,dust_lang={level}"))
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

pub fn print_times(times: &[(&str, Duration, Option<Duration>)]) {
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
