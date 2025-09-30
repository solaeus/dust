#![feature(
    duration_millis_float,
    formatting_options,
    iter_intersperse,
    iterator_try_collect
)]

mod cli;

use std::{
    fmt::{self},
    fs::{File, create_dir, create_dir_all, read_to_string},
    io::{self, Read, Write},
    path::PathBuf,
    time::{Duration, Instant},
};

use clap::Parser;
use dust_lang::{
    Lexer, Source,
    project::{DEFAULT_PROGRAM_PATH, EXAMPLE_PROGRAM, PROJECT_CONFIG_PATH, ProjectConfig},
    token::Token,
};
use memmap2::MmapOptions;
use tracing::{Event, Level, Subscriber, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt::{FmtContext, FormatEvent, FormatFields, format::Writer},
    registry::LookupSpan,
};

use crate::cli::{Cli, InputOptions, Mode};

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
    } = Cli::parse();
    let mode = mode.unwrap_or(Mode::Run {
        min_heap: None,
        min_sweep: None,
    });

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

    // if mode == Mode::Parse {
    //     let source = get_source(path, name, stdin, eval);

    //     match source {
    //         Source::Script(SourceFile { name, source_code }) => {
    //             let (syntax_tree, error) = parse_main(source_code);
    //             let parse_time = start_time.elapsed();

    //             println!("{syntax_tree}");

    //             if !no_output && let Some(error) = error {
    //                 eprintln!("{}", error.report());
    //             }

    //             if time {
    //                 print_times(&[(&name, parse_time, None)]);
    //             }

    //             return;
    //         }
    //         Source::Files(_source_files) => todo!(),
    //     }
    // }

    // if mode == Mode::Compile {
    //     let source = get_source(path, name, stdin, eval);
    //     let resolver = Resolver::new(true);
    //     let compiler = Compiler::new(source.clone(), resolver);
    //     let compile_result = compiler.compile_with_extras();
    //     let compile_time = start_time.elapsed();
    //     let program_name = source.program_name().to_string();

    //     match compile_result {
    //         Ok((program, source, file_trees, resolver)) => {
    //             let disassembler = TuiDisassembler::new(&program, &source, &file_trees, &resolver);

    //             disassembler.disassemble().unwrap();
    //         }
    //         Err(error) => {
    //             if !no_output {
    //                 eprintln!("{}", error.report())
    //             }
    //         }
    //     }

    //     if time {
    //         print_times(&[(&program_name, compile_time, None)]);
    //     }

    //     return;
    // }

    if let Mode::Tokenize = mode {
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
    }

    if let Mode::Init { project_path } = mode {
        if !project_path.exists() {
            create_dir_all(&project_path).expect("Failed to create project directory");
        } else if project_path.read_dir().unwrap().next().is_some() {
            eprintln!("The directory `{}` is not empty", project_path.display());

            return;
        }

        let example_config_path = project_path.join(PROJECT_CONFIG_PATH);
        let example_project_config = toml::to_string_pretty(&ProjectConfig::example())
            .expect("Failed to serialize example project config to TOML");

        File::create(&example_config_path)
            .expect("Failed to create project config file")
            .write_all(example_project_config.as_bytes())
            .expect("Failed to write to project config file");

        let src_path = project_path.join("src");

        create_dir(&src_path).expect("Failed to create `src` directory");

        let example_program_path = src_path.join("main.ds");

        File::create(&example_program_path)
            .expect("Failed to create example program file")
            .write_all(EXAMPLE_PROGRAM.as_bytes())
            .expect("Failed to write to example program file");

        println!(
            "Initialized a new Dust project at `{}`",
            project_path.display()
        );
    }
}

fn get_source(
    path: Option<PathBuf>,
    name: Option<String>,
    stdin: bool,
    eval: Option<Vec<u8>>,
) -> Source {
    if let Some(source) = eval {
        return Source::Script {
            name: "CLI Input".to_string(),
            source,
        };
    }

    if stdin {
        let mut buffer = Vec::new();

        io::stdin()
            .read_to_end(&mut buffer)
            .expect("Failed to read from stdin");

        return Source::Script {
            name: "stdin".to_string(),
            source: buffer,
        };
    }

    match path {
        Some(path) if path.is_file() => {
            let name = name.unwrap_or_else(|| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("unknown")
                    .to_string()
            });
            let file = File::open(&path).expect("Failed to open source file");
            let mmap =
                unsafe { MmapOptions::new().map(&file) }.expect("Failed to memory map source file");
            let mut source = Source::files(1);

            source.add_file(name, mmap);

            source
        }
        Some(project_path) if project_path.is_dir() => {
            let project_path = project_path.canonicalize().expect("Invalid project path");
            let project_config_path = project_path.join(PROJECT_CONFIG_PATH);

            if !project_config_path.is_file() {
                panic!(
                    "No project config file found at `{}`",
                    project_config_path.display()
                );
            }

            let project_config_content =
                read_to_string(&project_config_path).expect("Failed to read project config file");
            let ProjectConfig {
                name: project_name,
                version: _,
                authors: _,
                program,
            } = toml::from_str(&project_config_content)
                .expect("Failed to parse project config file");

            let source_directory = project_path.join("src");

            if !source_directory.is_dir() {
                panic!("`{}` is not a directory", source_directory.display());
            }

            let file_count = source_directory
                .read_dir()
                .expect("Failed to read src directory")
                .count()
                .max(1);
            let mut source_files = Source::files(file_count);
            let main_file_path = program
                .map(|program| project_path.join(program.path))
                .unwrap_or_else(|| project_path.join(DEFAULT_PROGRAM_PATH));
            let main_file = File::open(&main_file_path).unwrap_or_else(|_| {
                panic!(
                    "Failed to open main source file at `{}`",
                    main_file_path.display()
                )
            });
            let main_file_content = unsafe {
                MmapOptions::new()
                    .map(&main_file)
                    .expect("Failed to memory map main source file")
            };

            source_files.add_file(project_name, main_file_content);

            if source_directory.is_dir() {
                for entry in source_directory
                    .read_dir()
                    .expect("Failed to read src directory")
                {
                    let entry = entry.expect("Failed to read directory entry");
                    let path = entry.path();

                    if path == main_file_path {
                        continue;
                    }

                    let module_name = path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("unknown")
                        .trim_end_matches(".ds")
                        .to_string();
                    let file = File::open(&path).expect("Failed to open source file");
                    let file_content = unsafe { MmapOptions::new().map(&file) }
                        .expect("Failed to memory map source file");

                    source_files.add_file(module_name, file_content);
                }
            }

            if source_files.is_empty() {
                panic!("No source files found");
            }

            source_files
        }
        _ => {
            panic!("No readable input source provided");
        }
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
