use std::{fs::read_to_string, io::Write};

use clap::Parser;
use colored::Colorize;
use dust_lang::{format, parse, run, Chunk, DustError, Vm};
use log::Level;

#[derive(Parser)]
struct Cli {
    /// Source code send via command line
    #[arg(short, long)]
    command: Option<String>,

    /// Whether to output formatted source code
    #[arg(short, long)]
    format: bool,

    /// Whether to output line numbers in formatted source code
    #[arg(short = 'l', long)]
    format_line_numbers: bool,

    /// Whether to output colors in formatted source code
    #[arg(short = 'o', long)]
    format_colored: bool,

    /// Whether to run the source code
    #[arg(short, long)]
    no_run: bool,

    /// Whether to output the disassembled chunk
    #[arg(short, long)]
    parse: bool,

    /// Whether to style the disassembled chunk
    #[arg(short, long)]
    style_disassembly: bool,

    /// Path to a source code file
    path: Option<String>,
}

fn main() {
    env_logger::builder()
        .parse_env("DUST_LOG")
        .format(|buf, record| {
            let level_display = match record.level() {
                Level::Info => "INFO".bold().white(),
                Level::Debug => "DEBUG".bold().blue(),
                Level::Warn => "WARN".bold().yellow(),
                Level::Error => "ERROR".bold().red(),
                Level::Trace => "TRACE".bold().purple(),
            };
            let module = record
                .module_path()
                .map(|path| path.split("::").last().unwrap_or(path))
                .unwrap_or("unknown")
                .dimmed();
            let display = format!("{level_display:5} {module:^6} {args}", args = record.args());

            writeln!(buf, "{display}")
        })
        .init();

    let args = Cli::parse();
    let source = if let Some(path) = &args.path {
        &read_to_string(path).expect("Failed to read file")
    } else if let Some(command) = &args.command {
        command
    } else {
        eprintln!("No input provided");

        return;
    };

    if !args.no_run {
        if args.format {
            format_source(source, args.format_line_numbers, args.format_colored);
        }

        let run_result = if args.parse {
            let chunk = parse(source).unwrap();
            let disassembly = chunk
                .disassembler("Dust CLI Input")
                .source(source)
                .styled(args.style_disassembly)
                .disassemble();

            println!("{}", disassembly);

            let mut vm = Vm::new(chunk);

            vm.run()
                .map_err(|error| DustError::Runtime { error, source })
        } else {
            run(source)
        };

        match run_result {
            Ok(Some(value)) => println!("{}", value),
            Ok(_) => {}
            Err(error) => {
                eprintln!("{}", error.report());
            }
        }

        return;
    }

    if args.format {
        format_source(source, args.format_line_numbers, args.format_colored);
    }

    if args.parse {
        parse_source(source, args.style_disassembly);
    }
}

pub fn format_source(source: &str, line_numbers: bool, colored: bool) {
    log::info!("Formatting source");

    match format(source, line_numbers, colored) {
        Ok(formatted) => println!("{}", formatted),
        Err(error) => {
            eprintln!("{}", error.report());
        }
    }
}

fn parse_source(source: &str, styled: bool) -> Option<Chunk> {
    parse(source)
        .inspect(|chunk| {
            let disassembly = chunk
                .disassembler("Dust CLI Input")
                .source(source)
                .styled(styled)
                .disassemble();

            println!("{disassembly}",);
        })
        .inspect_err(|error| {
            eprintln!("{}", error.report());
        })
        .ok()
}
