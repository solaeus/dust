#![feature(
    duration_millis_float,
    formatting_options,
    iter_intersperse,
    iterator_try_collect
)]

mod cli;
mod compile;
mod parse;
// mod run;
mod tokenize;

use std::{
    fmt,
    fs::{File, create_dir, create_dir_all},
    io::{self, Read, Write},
    path::PathBuf,
    time::{Duration, Instant},
};

use clap::Parser as CliParser;
use dust_lang::{
    project::{EXAMPLE_LIBRARY, EXAMPLE_PROGRAM, PROJECT_CONFIG_PATH, ProjectConfig},
    source::{Source, SourceFile},
};
use memmap2::MmapOptions;
use tracing::{Event, Level, Subscriber, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt::{FmtContext, FormatEvent, FormatFields, format::Writer},
    registry::LookupSpan,
};

use crate::{
    cli::{Cli, Command, CompileCommand, InputOptions, OutputOptions, ParseCommand},
    compile::handle_compile_command,
    parse::handle_parse_command,
    // run::handle_run_command,
    tokenize::handle_tokenize_command,
};

fn main() {
    let start_time = Instant::now();
    let Cli {
        command,
        global,
        input,
        output,
    } = Cli::parse();
    // let command = command.unwrap_or(Command::Run(input));

    if let Some(Command::Run(mut run_input)) = command {
        run_input.join(input);

        // handle_run_command(eval, path, no_output, time, start_time);

        return;
    }

    if let Some(Command::Parse(mut command)) = command {
        command.global.join(global);
        command.input.join(input);
        command.output.join(output);

        handle_logging(command.global.log, start_time);
        handle_parse_command(command, start_time);

        return;
    }

    if let Some(Command::Compile(mut command)) = command {
        command.global.join(global);
        command.input.join(input);
        command.output.join(output);

        handle_logging(command.global.log, start_time);
        handle_compile_command(command, start_time);

        return;
    }

    if let Some(Command::Tokenize(mut command)) = command {
        command.global.join(global);
        command.input.join(input);
        command.output.join(output);

        handle_logging(command.global.log, start_time);
        handle_tokenize_command(command, start_time);

        return;
    }

    if let Some(Command::Init(InputOptions { path, .. })) = command {
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

        let example_lib_path = src_path.join("lib.ds");

        File::create(&example_lib_path)
            .expect("Failed to create example library file")
            .write_all(EXAMPLE_LIBRARY.as_bytes())
            .expect("Failed to write to example library file");

        println!("Initialized a new Dust project at `{}`", path.display());
    }
}

fn handle_logging(level: Option<LevelFilter>, start_time: Instant) {
    if let Some(level) = level {
        tracing_subscriber::fmt()
            .with_env_filter(format!("none,dust_lang={level}"))
            .event_format(LogFormatter { start_time })
            .init();
    }
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

        let colorized_level = match *level {
            Level::ERROR => "ERROR".red().bold(),
            Level::WARN => "WARN".yellow().bold(),
            Level::INFO => "INFO".blue().bold(),
            Level::DEBUG => "DEBUG".green().bold(),
            Level::TRACE => "TRACE".cyan().bold(),
        };
        let time = format!("{elapsed:.5}ms").dimmed();

        write!(writer, "{colorized_level:5} {time}",)?;

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
        let total_time_display = format!("{}ms", total_time.as_millis_f64());

        println!("{source_name}: {compile_time_display}, {total_time_display} total");
    }
}

fn handle_source<'src>(
    eval: &'src Option<String>,
    path: Option<PathBuf>,
    stdin: bool,
) -> Source<'src> {
    let mut source = Source::new();

    if let Some(input) = eval {
        let eval_program = format!("fn main() -> any {{\n    {input}\n}}");
        let file = SourceFile::validated_owned("CLI Input", eval_program);

        source.add_file(file);
    }

    if let Some(path) = path {
        if path.is_dir() {
            let config_path = path.join(PROJECT_CONFIG_PATH);
            let config = if config_path.exists() {
                let mut config_file =
                    File::open(&config_path).expect("Failed to open project config file");
                let mut config_contents = String::new();

                config_file
                    .read_to_string(&mut config_contents)
                    .expect("Failed to read project config file");

                toml::from_str::<ProjectConfig>(&config_contents)
                    .expect("Failed to parse project config file")
            } else {
                panic!(
                    "No project config file found at `{}`",
                    config_path.display()
                );
            };

            let main_file_path = if let Some(program) = config.program {
                path.join(program.path)
            } else {
                path.join("src").join("main.ds")
            };
            let file = SourceFile::base_file(main_file_path).unwrap_or_else(|error| {
                panic!("Failed to create source file for main source file: {error}")
            });

            source.add_file(file);

            let lib_file_path = path.join("src").join("lib.ds");

            if lib_file_path.exists() {
                let file =
                    SourceFile::base_file(lib_file_path).unwrap_or_else(|error| panic!("{error}"));

                source.add_file(file);
            }
        } else {
            let file = SourceFile::base_file(path).unwrap_or_else(|error| panic!("{error}"));

            source.add_file(file);
        }
    }

    if stdin {
        let mut buffer = Vec::new();

        io::stdin()
            .read_to_end(&mut buffer)
            .expect("Failed to read from stdin");

        let file = SourceFile::non_validated_owned("stdin", buffer);

        source.add_file(file);
    }

    source
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
