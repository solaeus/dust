use std::path::PathBuf;

use clap::{
    Args, ColorChoice, Parser, Subcommand, ValueHint,
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
    pub command: Option<Command>,

    #[command(flatten)]
    pub input: InputOptions,

    /// Set the log level
    #[arg(short, long, value_name = "LEVEL", env = "DUST_LOG")]
    pub log: Option<LevelFilter>,

    /// Display the time taken for each operation
    #[arg(short, long)]
    pub time: bool,

    /// Disable all output
    #[arg(long)]
    pub no_output: bool,

    /// Custom program name, overrides the file name
    #[arg(short, long)]
    pub name: Option<String>,

    /// Minimum heap size at which garbage collection is triggered
    #[arg(long, value_name = "BYTES", requires = "min_sweep")]
    pub min_heap: Option<usize>,

    /// Minimum bytes allocated between garbage collections
    #[arg(long, value_name = "BYTES", requires = "min_heap")]
    pub min_sweep: Option<usize>,
}

#[derive(Subcommand, Eq, PartialEq)]
pub enum Command {
    /// Parse the source code and print the syntax tree
    #[command(alias = "p")]
    Parse(InputOptions),

    /// Run a program (default)
    #[command(alias = "r")]
    Run(InputOptions),

    /// Compile and output the compiled program
    #[command(alias = "c")]
    Compile(CompileCommand),

    /// Lex the source code and print the tokens
    #[command(alias = "t")]
    Tokenize(InputOptions),

    /// Initialize a new Dust project
    #[command(alias = "i")]
    Init(InputOptions),
}

#[derive(Args, Clone, Eq, PartialEq)]
#[group(multiple = false)]
pub struct InputOptions {
    /// Source code to run instead of a file
    #[arg(short, long, value_name = "INPUT")]
    pub eval: Option<String>,

    /// Read source code from stdin
    #[arg(long)]
    pub stdin: bool,

    /// Path to a source code file
    #[arg(short, long, value_name = "PATH", value_hint = ValueHint::FilePath)]
    pub path: Option<PathBuf>,
}

#[derive(Args, Clone, Eq, PartialEq)]
pub struct CompileCommand {
    #[command(flatten)]
    pub input: InputOptions,

    /// Disable all output
    #[arg(long)]
    pub no_output: bool,

    /// Display the time taken for each operation
    #[arg(short, long)]
    pub time: bool,

    /// Dispaly disassembly with a TUI (default: true)
    #[arg(long, default_value = "true")]
    pub no_tui: bool,
}
