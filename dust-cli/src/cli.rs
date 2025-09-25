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

    /// Disable the standard library
    #[arg(long)]
    pub no_std: bool,

    /// Custom program name, overrides the file name
    #[arg(short, long)]
    pub name: Option<String>,
}

#[derive(Subcommand, Eq, PartialEq)]
pub enum Mode {
    /// Parse the source code and print the syntax tree
    #[command(alias = "p")]
    Parse,

    /// Parse, compile and run the program (default)
    #[command(alias = "r")]
    Run {
        /// Minimum heap size at which garbage collection is triggered
        #[arg(long, value_name = "BYTES", requires = "min_sweep")]
        min_heap: Option<usize>,

        /// Minimum bytes allocated between garbage collections
        #[arg(long, value_name = "BYTES", requires = "min_heap")]
        min_sweep: Option<usize>,
    },

    /// Compile and output the compiled program
    #[command(alias = "c")]
    Compile,

    /// Lex the source code and print the tokens
    #[command(alias = "t")]
    Tokenize {
        /// Format for the output, defaults to a simple text format
        #[arg(short, long, default_value = "dust", value_name = "FORMAT")]
        output: OutputOptions,
    },

    #[command(alias = "i")]
    Init {
        /// Directory to create the project in, defaults to the current directory
        #[arg(value_hint = ValueHint::DirPath, value_name = "PATH")]
        project_path: PathBuf,
    },
}

#[derive(Args)]
#[group(required = true, multiple = false)]
pub struct InputOptions {
    /// Source code to run instead of a file
    #[arg(short, long, value_name = "INPUT", value_hint = ValueHint::Other)]
    pub eval: Option<Vec<u8>>,

    /// Read source code from stdin
    #[arg(long)]
    pub stdin: bool,

    /// Path to a source code file
    #[arg(value_name = "PATH", value_hint = ValueHint::FilePath)]
    pub path: Option<PathBuf>,
}

#[derive(ValueEnum, Clone, Copy, Eq, PartialEq)]
pub enum OutputOptions {
    Dust,
    Json,
    Ron,
    Postcard,
    Yaml,
}
