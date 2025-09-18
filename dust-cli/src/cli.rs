use std::path::PathBuf;

use clap::{
    Args, ColorChoice, Parser, Subcommand, ValueEnum, ValueHint,
    builder::{Styles, styling::AnsiColor},
    crate_authors, crate_description, crate_version,
};
use tracing::level_filters::LevelFilter;

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
        .error(AnsiColor::BrightRed.on_default())
)]
pub struct Cli {
    #[command(subcommand)]
    pub mode: Option<Mode>,

    #[command(flatten)]
    pub run_options: RunOptions,

    /// Set the log level
    #[arg(short, long, value_name = "LEVEL")]
    pub log: Option<LevelFilter>,

    /// Display the time taken for each operation
    #[arg(short, long)]
    pub time: bool,

    /// Disable all output
    #[arg(long)]
    pub no_output: bool,

    /// Disable the standard library
    #[arg(long)]
    pub no_std: bool,

    /// Custom program name, overrides the file name
    #[arg(short, long)]
    pub name: Option<String>,

    /// Format for the output, defaults to a simple text format
    #[arg(short, long, default_value = "dust", value_name = "FORMAT")]
    pub output: OutputOptions,
}

#[derive(Subcommand)]
pub enum Mode {
    #[command(alias = "p")]
    /// Parse the source code and print the syntax tree
    Parse(ParseOptions),

    /// Parse, compile and run the program (default)
    #[command(alias = "r")]
    Run(RunOptions),

    /// Compile and output the compiled program
    #[command(alias = "c")]
    Compile(CompileOptions),

    /// Lex the source code and print the tokens
    #[command(alias = "t")]
    Tokenize(InputOptions),

    #[command(alias = "i")]
    Init(InitOptions),
}

#[derive(Args)]
pub struct ParseOptions {
    #[command(flatten)]
    pub input: InputOptions,
}

#[derive(Args)]
pub struct RunOptions {
    #[command(flatten)]
    pub input: InputOptions,

    /// Minimum heap size garbage collection is triggered
    #[arg(long, value_name = "BYTES", requires = "min_sweep")]
    pub min_heap: Option<usize>,

    /// Minimum bytes allocated between garbage collections
    #[arg(long, value_name = "BYTES", requires = "min_heap")]
    pub min_sweep: Option<usize>,
}

#[derive(Args)]
pub struct CompileOptions {
    /// Print the time taken for compilation and execution, defaults to false
    #[arg(short, long)]
    pub time: bool,

    /// Disable printing, defaults to false
    #[arg(long)]
    pub no_output: bool,

    /// Disable the standard library, defaults to false
    #[arg(long)]
    pub no_std: bool,

    /// Custom program name, overrides the file name
    #[arg(short, long)]
    pub name: Option<String>,

    #[command(flatten)]
    pub input: InputOptions,

    /// Style disassembly output, defaults to true
    #[arg(short, long, default_value = "true")]
    pub style: bool,
}

#[derive(Args)]
pub struct InputOptions {
    /// Source code to run instead of a file
    #[arg(
        short,
        long,
        value_name = "INPUT",
        value_hint = ValueHint::Other,
        conflicts_with = "stdin",
        conflicts_with = "path"
    )]
    pub eval: Option<String>,

    /// Read source code from stdin
    #[arg(long, conflicts_with = "eval", conflicts_with = "path")]
    pub stdin: bool,

    /// Path to a source code file
    #[arg(
        value_name = "PATH",
        value_hint = ValueHint::FilePath,
        conflicts_with = "eval",
        conflicts_with = "stdin"
    )]
    pub path: Option<PathBuf>,
}

#[derive(ValueEnum, Clone, Copy, PartialEq)]
pub enum OutputOptions {
    Dust,
    Json,
    Ron,
    Postcard,
    Yaml,
}

#[derive(Args)]
pub struct InitOptions {
    /// Directory to create the project in, defaults to the current directory
    #[arg(value_hint = ValueHint::DirPath, value_name = "PATH")]
    pub project_path: PathBuf,
}
