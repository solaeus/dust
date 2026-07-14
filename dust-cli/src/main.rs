#![feature(duration_millis_float)]

mod cli;
mod commands;
mod error;
mod explorer;

use std::{
    fmt,
    io::{self, Read, Write, stderr},
    process::ExitCode,
    time::Instant,
};

use clap::Parser as CliParser;
use dust_compiler::source::{Source, SourceCode};
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

    handle_logging(global.log, start_time);

    let result = match command {
        Some(Command::Run(mut command)) => {
            command = command.fill_arguments(global, input);

            run(command)
        }
        None => {
            let command = RunCommand {
                global,
                name: None,
                input,
            };

            run(command)
        }
        Some(Command::Parse(mut command)) => {
            command = command.join(global, input);

            parse(command)
        }
        Some(Command::Compile(mut command)) => {
            command = command.fill_arguments(global, input);

            compile(command)
        }
        Some(Command::Init(mut command)) => {
            command = command.fill_arguments(global, input);

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

fn get_name(name_option: Option<String>, input: &InputOptions) -> String {
    name_option.unwrap_or_else(|| {
        if let Some(path) = &input.path
            && path.is_file()
        {
            path.file_name().unwrap().to_string_lossy().to_string()
        } else {
            "dust_program".to_string()
        }
    })
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

        source.add_code(SourceCode::validated_owned("cli_input", eval_program));
    } else if let Some(input) = eval_full {
        source.add_code(SourceCode::validated_owned("cli_input", input));
    } else if stdin {
        let mut buffer = Vec::new();

        io::stdin().read_to_end(&mut buffer)?;

        source.add_code(SourceCode::unvalidated_owned("stdin", buffer));
    } else if let Some(path) = path
        && path.is_file()
    {
        source.add_code(SourceCode::file(path)?);
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
