#![feature(duration_millis_float, formatting_options, iter_intersperse)]

mod cli;

use std::{
    fmt::{self},
    fs::{File, create_dir, create_dir_all, read_to_string},
    io::{self, Read, Write},
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use clap::Parser;
use dust_lang::{
    Resolver, Source,
    chunk::TuiDisassembler,
    compiler::Compiler,
    jit_vm::{JitVm, MINIMUM_OBJECT_HEAP_DEFAULT, MINIMUM_OBJECT_SWEEP_DEFAULT},
    parser::parse_main,
    project::{DEFAULT_PROGRAM_PATH, EXAMPLE_PROGRAM, PROJECT_CONFIG_PATH, ProjectConfig},
    source::SourceFile,
    tokenize,
};
use ron::ser::PrettyConfig;
use tracing::{Event, Level, Subscriber, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt::{FmtContext, FormatEvent, FormatFields, format::Writer},
    registry::LookupSpan,
};

use crate::cli::{
    Cli, CompileOptions, InitOptions, InputOptions, Mode, OutputOptions, ParseOptions, RunOptions,
};

fn main() {
    let start_time = Instant::now();
    let Cli {
        mode,
        run_options,
        log,
        time,
        no_output,
        no_std: _,
        name,
        output: output_format,
    } = Cli::parse();
    let mode = mode.unwrap_or(Mode::Run(RunOptions {
        input: run_options.input,
        min_heap: run_options.min_heap,
        min_sweep: run_options.min_sweep,
    }));

    if let Some(log_level) = log {
        start_logging(log_level, start_time);
    }

    if let Mode::Run(RunOptions {
        input: InputOptions { eval, stdin, path },
        min_heap,
        min_sweep,
    }) = mode
    {
        let source = get_source(path, name, stdin, eval);
        let compiler = Compiler::new(source.clone());
        let resolver = Resolver::new(true);
        let compile_result = compiler.compile(resolver);
        let compile_time = start_time.elapsed();

        let program = match compile_result {
            Ok(program) => program,
            Err(error) => {
                let report = error.report();

                if !no_output {
                    eprintln!("{report}");
                }

                return;
            }
        };

        let vm = JitVm::new();
        let min_heap = min_heap.unwrap_or(MINIMUM_OBJECT_HEAP_DEFAULT);
        let min_sweep = min_sweep.unwrap_or(MINIMUM_OBJECT_SWEEP_DEFAULT);
        let run_result = vm.run(program, min_heap, min_sweep);
        let run_time = start_time.elapsed() - compile_time;

        let return_value = match run_result {
            Ok(value) => value,
            Err(dust_error) => {
                let report = dust_error.report();

                if !no_output {
                    eprintln!("{report}");
                }

                return;
            }
        };

        if !no_output && let Some(return_value) = return_value {
            println!("{return_value}");
        }

        if time && !no_output {
            print_times(&[(source.program_name().as_str(), compile_time, Some(run_time))]);
        }

        return;
    }

    if let Mode::Parse(ParseOptions {
        input: InputOptions {
            eval,
            stdin,
            path: file,
        },
    }) = mode
    {
        let source = get_source(file, name, stdin, eval);

        match source {
            Source::Script(SourceFile {
                name,
                source_code: source,
            }) => {
                let (syntax_tree, error) = parse_main(&source);
                let parse_time = start_time.elapsed();

                if no_output {
                    return;
                }

                println!("{syntax_tree}");

                if let Some(error) = error {
                    eprintln!("{}", error.report());
                }

                if time {
                    print_times(&[(&name, parse_time, None)]);
                }

                return;
            }
            Source::Files(source_files) => todo!(),
        }
    }

    if let Mode::Compile(CompileOptions {
        time,
        no_output,
        no_std: _,
        name,
        input: InputOptions { eval, stdin, path },
        style: _,
    }) = mode
    {
        let source = get_source(path, name, stdin, eval);
        let compiler = Compiler::new(source.clone());
        let resolver = Resolver::new(true);
        let compile_result = compiler.compile(resolver);
        let compile_time = start_time.elapsed();

        match compile_result {
            Ok(program) => {
                let disassembler = TuiDisassembler::new(&program, source.clone());

                disassembler.disassemble().unwrap();
            }
            Err(error) => eprintln!("{}", error.report()),
        }

        if time && !no_output {
            print_times(&[(source.program_name().as_str(), compile_time, None)]);
        }

        return;
    }

    if let Mode::Tokenize(InputOptions {
        eval,
        stdin,
        path: file,
    }) = mode
    {
        let source = match get_source(file, name, stdin, eval) {
            Source::Files(_) => {
                eprintln!("Tokenizing multiple files is not supported");

                return;
            }
            Source::Script(source_file) => source_file.source_code.clone(),
        };

        let tokens = tokenize(&source);

        match output_format {
            OutputOptions::Dust => {
                for (token, span) in tokens {
                    println!("{token} at {span}");
                }
            }
            OutputOptions::Json => {
                let json = serde_json::to_string_pretty(&tokens)
                    .expect("Failed to serialize tokens to JSON");

                println!("{json}");
            }
            OutputOptions::Postcard => {
                let mut buffer = Vec::new();
                postcard::to_slice_cobs(&tokens, &mut buffer)
                    .expect("Failed to serialize tokens to Postcard");

                println!("{buffer:?}");
            }
            OutputOptions::Ron => {
                let ron = ron::ser::to_string_pretty(&tokens, PrettyConfig::new())
                    .expect("Failed to serialize tokens to RON");

                println!("{ron}");
            }
            OutputOptions::Yaml => {
                let yaml =
                    serde_yaml::to_string(&tokens).expect("Failed to serialize tokens to YAML");

                println!("{yaml}");
            }
        }

        return;
    }

    if let Mode::Init(InitOptions { project_path }) = mode {
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
    eval: Option<String>,
) -> Source {
    if let Some(source) = eval {
        return Source::Script(SourceFile {
            name: Arc::new(name.unwrap_or_else(|| "anonymous".to_string())),
            source_code: Arc::new(source),
        });
    }

    if stdin {
        let mut buffer = String::new();

        io::stdin()
            .read_to_string(&mut buffer)
            .expect("Failed to read from stdin");

        return Source::Script(SourceFile {
            name: Arc::new(name.unwrap_or_else(|| "anonymous".to_string())),
            source_code: Arc::new(buffer),
        });
    }

    match path {
        Some(path) if path.is_file() => {
            let file_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown")
                .to_string();
            let file_content = read_to_string(&path).expect("Failed to read source file");

            Source::Script(SourceFile {
                name: Arc::new(name.unwrap_or(file_name)),
                source_code: Arc::new(file_content),
            })
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

            let mut source_files = Vec::new();
            let main_file_path = program
                .map(|program| project_path.join(program.path))
                .unwrap_or_else(|| project_path.join(DEFAULT_PROGRAM_PATH));
            let main_file_content = read_to_string(&main_file_path).unwrap_or_else(|_| {
                panic!(
                    "Failed to read main source file at `{}`",
                    main_file_path.display()
                )
            });

            source_files.push(SourceFile {
                name: Arc::new(project_name),
                source_code: Arc::new(main_file_content),
            });

            let source_directory = project_path.join("src");

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

                    let file_name = path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("unknown")
                        .trim_end_matches(".ds")
                        .to_string();
                    let file_content = read_to_string(&path).expect("Failed to read source file");

                    source_files.push(SourceFile {
                        name: Arc::new(file_name),
                        source_code: Arc::new(file_content),
                    });
                }
            }

            if source_files.is_empty() {
                panic!("No source files found");
            }

            Source::Files(source_files)
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
