#![feature(duration_millis_float)]

mod cli;
mod compile;
mod parse;
// mod run;

use std::{
    fmt,
    fs::{File, create_dir, create_dir_all},
    io::{self, Read, Write},
    path::PathBuf,
    str::FromStr,
    time::Instant,
};

use clap::Parser as CliParser;
use dust_lang::prelude::*;
use tracing::{Event, Level, Subscriber, info, level_filters::LevelFilter, trace};
use tracing_subscriber::{
    EnvFilter,
    filter::Directive,
    fmt::{FmtContext, FormatEvent, FormatFields, format::Writer},
    registry::LookupSpan,
};

use crate::{
    cli::{Cli, Command, InputOptions},
    compile::handle_compile_command,
    parse::handle_parse_command,
    // run::handle_run_command,
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
        handle_parse_command(command);

        return;
    }

    if let Some(Command::Compile(mut command)) = command {
        command.global.join(global);
        command.input.join(input);
        command.output.join(output);

        handle_logging(command.global.log, start_time);
        handle_compile_command(command);

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
    let level = level.unwrap_or(LevelFilter::OFF);

    tracing_subscriber::fmt()
        .with_env_filter(format!("none,dust={level},dust_lang={level}"))
        .event_format(LogFormatter { start_time })
        .init();

    info!("Finished parsing CLI arguments and initializing logger");
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
        let scopes = context.event_scope().map(|scope| scope.from_root());

        let (symbol, level) = match *level {
            Level::ERROR => ("‼️", "ERROR".red()),
            Level::WARN => ("⚠️", "WARN".yellow()),
            Level::INFO => ("ℹ️", "INFO".blue()),
            Level::DEBUG => ("🐛", "DEBUG".green()),
            Level::TRACE => ("🔎", "TRACE".magenta()),
        };
        let time = format!("{elapsed:.5}ms").dimmed();

        write!(writer, "{symbol} ┊ {level:5} {time} ┊ ")?;

        let mut scope_length = 0;

        if let Some(scopes) = scopes {
            for (index, span) in scopes.enumerate() {
                let span_name = span.metadata().name().bold();
                let mut span_length = span_name.len();

                if index > 0 {
                    write!(writer, "::")?;

                    span_length += 2;
                }

                scope_length += span_length;

                write!(writer, "{span_name}")?;
            }

            for _ in 0..(17 - scope_length) {
                write!(writer, " ")?;
            }

            write!(writer, "┊ ")?;
        }

        context.format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}

fn build_source<'src>(
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
            let file =
                SourceFile::file(&main_file_path).unwrap_or_else(|error| error.print_and_exit());

            source.add_file(file);

            let lib_file_path = path.join("src").join("lib.ds");

            if lib_file_path.exists() {
                let file =
                    SourceFile::file(&lib_file_path).unwrap_or_else(|error| error.print_and_exit());

                source.add_file(file);
            }
        } else {
            let file = SourceFile::file(&path).unwrap_or_else(|error| error.print_and_exit());

            source.add_file(file);
        }
    }

    if stdin {
        let mut buffer = Vec::new();

        io::stdin()
            .read_to_end(&mut buffer)
            .expect("Failed to read from stdin");

        let file = SourceFile::owned("stdin", buffer);

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
