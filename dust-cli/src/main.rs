#![feature(duration_millis_float)]

mod cli;
mod commands;
mod error;

use std::{
    fmt,
    fs::File,
    io::{self, Read, Write, stderr},
    process::ExitCode,
    time::Instant,
};

use clap::Parser as CliParser;
use dust_lang::{
    project::{PROJECT_CONFIG_PATH, ProjectConfig},
    source::{Source, SourceCode},
};
use tracing::{Event, Level, Subscriber, info, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt::{FmtContext, FormatEvent, FormatFields, format::Writer},
    registry::LookupSpan,
};

use crate::{
    cli::{Cli, Command, InputOptions, RunCommand},
    commands::{compile::compile, init::init, parse::parse, run::run},
    error::Error,
};

fn main() -> ExitCode {
    let start_time = Instant::now();
    let Cli {
        command,
        global,
        input,
    } = Cli::parse();

    let result = match command {
        Some(Command::Run(mut command)) => {
            command = command.fill_arguments(global, input);

            handle_logging(command.global.log, start_time);
            run(command)
        }
        None => {
            handle_logging(global.log, start_time);
            run(RunCommand { global, input })
        }
        Some(Command::Parse(mut command)) => {
            command = command.fill_arguments(global, input);

            handle_logging(command.global.log, start_time);
            parse(command)
        }
        Some(Command::Compile(mut command)) => {
            command = command.fill_arguments(global, input);

            handle_logging(command.global.log, start_time);
            compile(command)
        }
        Some(Command::Init(mut command)) => {
            command = command.fill_arguments(global, input);

            handle_logging(command.global.log, start_time);
            init(command)
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let _ = stderr().write_all(error.to_string().as_bytes());

            ExitCode::FAILURE
        }
    }
}

fn handle_logging(level: Option<LevelFilter>, start_time: Instant) {
    let level = level.unwrap_or(LevelFilter::OFF);

    tracing_subscriber::fmt()
        .with_env_filter(format!("none,dust={level},dust_lang={level}"))
        .event_format(LogFormatter { start_time })
        .init();

    info!("Finished parsing command arguments and initializing logger");
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

        let level = match *level {
            Level::INFO => "INFO".blue().bold(),
            Level::DEBUG => "DEBUG".green().bold(),
            Level::TRACE => "TRACE".magenta().bold(),
            Level::WARN => "WARN".yellow().bold(),
            Level::ERROR => "ERROR".red().bold(),
        };
        let time = format!("{elapsed:.5}ms").dimmed();

        write!(writer, "{level:5} {time} ")?;

        if let Some(scopes) = scopes {
            for (index, span) in scopes.enumerate() {
                let span_name = span.metadata().name().bold();

                if index > 0 {
                    write!(writer, "::")?;
                }

                write!(writer, "{span_name}")?;
            }

            write!(writer, " ")?;
        }

        context.format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}

fn build_source<'src>(
    InputOptions {
        eval,
        eval_full,
        stdin,
        path,
    }: InputOptions,
) -> Result<Source<'src>, Error<'src>> {
    let mut source = Source::new();

    if let Some(input) = eval {
        let eval_program = format!("fn main<T>() -> T {{\n    {input}\n}}");
        let code = SourceCode::from_string("CLI Input", eval_program);

        source.add_code(code);
    } else if let Some(input) = eval_full {
        let code = SourceCode::from_string("CLI Input", input);

        source.add_code(code);
    } else if let Some(path) = path {
        if path.is_dir() {
            let config = {
                let config_path = path.join(PROJECT_CONFIG_PATH);
                let mut config_file = File::open(&config_path)?;
                let mut config_contents = String::new();

                config_file.read_to_string(&mut config_contents)?;
                toml::from_str::<ProjectConfig>(&config_contents)?
            };

            let main_file_path = if let Some(program) = config.program {
                path.join(program.path)
            } else {
                path.join("src").join("main.ds")
            };
            let code = SourceCode::file(main_file_path)?;

            source.add_code(code);

            let lib_file_path = path.join("src").join("lib.ds");

            if lib_file_path.exists() {
                let code = SourceCode::file(lib_file_path)?;

                source.add_code(code);
            }
        } else {
            let code = SourceCode::file(path)?;

            source.add_code(code);
        }
    } else if stdin {
        let mut buffer = Vec::new();

        io::stdin().read_to_end(&mut buffer)?;

        let code = SourceCode::from_owned_bytes("stdin", buffer);

        source.add_code(code);
    }

    Ok(source)
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
