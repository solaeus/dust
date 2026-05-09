use std::path::PathBuf;

use clap::{
    Args, ColorChoice, Parser, Subcommand,
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
    pub global: GlobalOptions,

    #[command(flatten)]
    pub input: InputOptions,

    #[command(flatten)]
    pub output: OutputOptions,
}

#[derive(Args)]
#[group(multiple = true)]
pub struct GlobalOptions {
    /// Set the log level
    #[arg(short, long, value_name = "LEVEL", env = "DUST_LOG")]
    pub log: Option<LevelFilter>,
}

impl GlobalOptions {
    pub fn join(&mut self, other: GlobalOptions) {
        self.log = self.log.take().or(other.log);
    }
}

#[derive(Subcommand)]
pub enum Command {
    /// Initialize a new Dust project
    #[command(alias = "i")]
    Init(InputOptions),

    /// Parse the source code and print the syntax tree
    #[command(alias = "p")]
    Parse(ParseCommand),

    /// Compile and output the compiled program
    #[command(alias = "c")]
    Compile(CompileCommand),

    /// Run a program (default)
    #[command(alias = "r")]
    Run(RunCommand),
}

#[derive(Args)]
#[group()]
pub struct InputOptions {
    /// Evaluate source code as a command-line argument
    ///
    /// Note: This wraps the input in a `main` function, so you can use statements and
    /// expressions directly. You cannot define another `main` function.
    #[arg(short, long, value_name = "INPUT")]
    pub eval: Option<String>,

    #[arg(long, value_name = "INPUT")]
    /// Evaluate source code as a command-line argument
    ///
    /// Note: This does not modify the input in any way, so you must provide a complete program with
    /// a `main` function.
    pub eval_full: Option<String>,

    /// Read source code from stdin
    #[arg(short, long)]
    pub stdin: bool,

    /// Path to a source code file
    pub path: Option<PathBuf>,
}

impl InputOptions {
    pub fn join(&mut self, other: InputOptions) {
        self.eval = self.eval.take().or(other.eval);
        self.stdin = self.stdin || other.stdin;
        self.path = self.path.take().or(other.path);
    }
}

#[derive(Args)]
pub struct OutputOptions {
    /// Print output in Rust debug format
    #[arg(long, group = "format")]
    pub debug: bool,

    /// Print output in Rusty Object Notation
    #[arg(long, group = "format")]
    pub ron: bool,

    /// Print output in pretty Rusty Object Notation
    #[arg(long, group = "format")]
    pub pretty_ron: bool,

    /// Print output in Postcard binary format
    #[arg(long, group = "format")]
    pub postcard: bool,
}

impl OutputOptions {
    pub fn join(&mut self, other: OutputOptions) {
        self.debug = self.debug || other.debug;
        self.ron = self.ron || other.ron;
        self.pretty_ron = self.pretty_ron || other.pretty_ron;
        self.postcard = self.postcard || other.postcard;
    }
}

#[derive(Args)]
pub struct ParseCommand {
    #[command(flatten)]
    pub global: GlobalOptions,

    #[command(flatten)]
    pub input: InputOptions,

    #[command(flatten)]
    pub output: OutputOptions,

    /// Print syntax trees as human-readable text trees (default: true)
    #[arg(long, default_value = "true", group = "format")]
    pub trees: bool,
}

#[derive(Args)]
pub struct CompileCommand {
    #[command(flatten)]
    pub global: GlobalOptions,

    #[command(flatten)]
    pub input: InputOptions,

    #[command(flatten)]
    pub output: OutputOptions,

    /// Launch the TUI disassembler (default: true)
    #[arg(long, default_value = "true", group = "format")]
    pub tui: bool,
}

#[derive(Args)]
pub struct RunCommand {
    #[command(flatten)]
    pub global: GlobalOptions,

    #[command(flatten)]
    pub input: InputOptions,
}
