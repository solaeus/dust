use std::{
    fs::read_to_string,
    io::{self, Read, stdout},
    panic,
    path::PathBuf,
    time::{Duration, Instant},
};

use clap::{
    Args, ColorChoice, Error, Parser, Subcommand, ValueEnum, ValueHint,
    builder::{Styles, styling::AnsiColor},
    crate_authors, crate_description, crate_version,
    error::ErrorKind,
};
use dust_lang::{
    CompileError, Compiler, DustError, DustString, Lexer, Span, Token, Vm, compiler::CompileMode,
};
use tracing::{Level, Subscriber, level_filters::LevelFilter, subscriber::set_global_default};
use tracing_subscriber::{
    FmtSubscriber, Layer,
    fmt::{SubscriberBuilder, layer, time::Uptime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
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
    /// Overrides the DUST_LOG environment variable
    #[arg(short, long)]
    log_level: LevelFilter,

    #[command(subcommand)]
    mode: Option<Command>,

    #[command(flatten)]
    run: Run,
}

#[derive(Args)]
struct Source {
    /// Source code to run instead of a file
    #[arg(short, long, value_hint = ValueHint::Other, value_name = "INPUT")]
    eval: Option<String>,

    /// Read source code from stdin
    #[arg(long)]
    stdin: bool,

    /// Path to a source code file
    #[arg(value_hint = ValueHint::FilePath)]
    file: Option<PathBuf>,
}

/// Compile and run the program (default)
#[derive(Args)]
#[command(short_flag = 'r')]
struct Run {
    /// Print the time taken for compilation and execution
    #[arg(long)]
    time: bool,

    /// Do not print the program's return value
    #[arg(long)]
    no_output: bool,

    /// Custom program name, overrides the file name
    #[arg(long)]
    name: Option<DustString>,

    #[command(flatten)]
    source: Source,

    /// Input format
    #[arg(short, long, default_value = "dust")]
    input: InputFormat,
}

#[derive(Subcommand)]
#[clap(subcommand_value_name = "COMMAND", flatten_help = true)]
enum Command {
    Run(Run),

    /// Compile and print the input
    #[command(short_flag = 'c')]
    Compile {
        /// Style disassembly output
        #[arg(short, long, default_value = "true")]
        style: bool,

        /// Custom program name, overrides the file name
        #[arg(short, long)]
        name: Option<DustString>,

        #[command(flatten)]
        source: Source,

        #[arg(short, long, default_value = "cli", value_name = "FORMAT")]
        output: OutputFormat,
    },

    /// Lex the source code and print the tokens
    #[command(short_flag = 't')]
    Tokenize {
        /// Style token output
        #[arg(short, long, default_value = "true")]
        style: bool,

        #[command(flatten)]
        source: Source,
    },
}

#[derive(ValueEnum, Clone, Copy)]
enum OutputFormat {
    Cli,
    Json,
    Ron,
    Yaml,
}

#[derive(ValueEnum, Clone, Copy)]
enum InputFormat {
    Dust,
    Json,
    Ron,
    Yaml,
}

fn get_source_and_file_name(source: Source) -> (String, Option<DustString>) {
    if let Some(path) = source.file {
        let source = read_to_string(&path).expect("Failed to read source file");
        let file_name = path
            .file_name()
            .and_then(|os_str| os_str.to_str())
            .map(DustString::from);

        return (source, file_name);
    }

    if source.stdin {
        let mut source = String::new();
        io::stdin()
            .read_to_string(&mut source)
            .expect("Failed to read from stdin");

        return (source, None);
    }

    let source = source.eval.expect("No source code provided");

    (source, None)
}

fn main() {
    let start_time = Instant::now();
    let start_type_clone = start_time.clone();
    let Cli {
        log_level,
        mode,
        run,
    } = Cli::parse();
    let mode = mode.unwrap_or(Command::Run(run));

    panic::set_hook(Box::new(|info| {
        println!("Dust has encountered an error and was forced to exit.",);

        if let Some(location) = info.location() {
            println!("The error occured at {location}.");
        }

        if let Some(message) = info.payload().downcast_ref::<&str>() {
            println!("Extra info: {message}");
        }
    }));

    let tracing_layer = layer()
        .with_file(false)
        .with_level(true)
        .with_target(true)
        .with_thread_names(true)
        .with_thread_ids(false)
        .with_timer(Uptime::from(start_time))
        .with_filter(log_level);

    tracing_subscriber::registry().with(tracing_layer).init();

    if let Command::Run(Run {
        time,
        no_output,
        name,
        source,
        input,
    }) = mode
    {
        let (source, file_name) = get_source_and_file_name(source);
        let lexer = Lexer::new(&source);
        let program_name = name.or(file_name);
        let chunk = match input {
            InputFormat::Dust => {
                let mut compiler =
                    match Compiler::new(lexer, CompileMode::Main { name: program_name }) {
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
        let compile_end = start_time.elapsed();

        let vm = Vm::new(chunk);
        let return_value = vm.run();
        let run_end = start_time.elapsed();

        if !no_output {
            if let Some(value) = return_value {
                println!("{value}")
            }
        }

        if time {
            let run_time = run_end - compile_end;
            let total_time = compile_end + run_time;

            print_time("Compile Time", compile_end);
            print_time("Run Time", run_time);
            print_time("Total Time", total_time);
        }

        return;
    }

    if let Command::Compile {
        style,
        name,
        source,
        output,
    } = mode
    {
        let (source, file_name) = get_source_and_file_name(source);
        let lexer = Lexer::new(&source);
        let program_name = name.or(file_name);
        let mut compiler = match Compiler::new(lexer, CompileMode::Main { name: program_name }) {
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
        let mut stdout = stdout().lock();

        match output {
            OutputFormat::Cli => {
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

        return;
    }

    if let Command::Tokenize { source, .. } = mode {
        let (source, _) = get_source_and_file_name(source);
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

fn print_time(phase: &str, instant: Duration) {
    let seconds = instant.as_secs_f64();

    match seconds {
        ..=0.001 => {
            println!(
                "{phase:12}: {microseconds}µs",
                microseconds = (seconds * 1_000_000.0).round()
            );
        }
        ..=0.199 => {
            println!(
                "{phase:12}: {milliseconds}ms",
                milliseconds = (seconds * 1000.0).round()
            );
        }
        _ => {
            println!("{phase:12}: {seconds}s");
        }
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
