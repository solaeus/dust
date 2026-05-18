use std::path::PathBuf;

use clap::{
    Args, ColorChoice, Parser, Subcommand, ValueEnum,
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
}

#[derive(Args)]
#[group(multiple = true)]
pub struct GlobalOptions {
    /// Set the log level
    #[arg(short, long, value_name = "LEVEL", env = "DUST_LOG")]
    pub log: Option<LevelFilter>,
}

impl GlobalOptions {
    pub fn join(mut self, other: GlobalOptions) -> Self {
        self.log = self.log.or(other.log);

        self
    }
}

#[derive(Subcommand)]
pub enum Command {
    /// Create a new Dust project
    #[command(alias = "i")]
    Init(InitCommand),

    /// Create syntax trees from the source code
    #[command(alias = "p")]
    Parse(ParseCommand),

    /// Create a Dust program from source code
    #[command(alias = "c")]
    Compile(CompileCommand),

    /// Run a program (default)
    #[command(alias = "r")]
    Run(RunCommand),
}

#[derive(Args)]
#[group(multiple = false)]
pub struct InputOptions {
    /// Evaluate source code as a command-line argument
    ///
    /// This wraps the input in a `main` function, so you can use statements and expressions
    /// directly. You cannot define another `main` function.
    #[arg(short, long, value_name = "INPUT")]
    pub eval: Option<String>,

    /// Evaluate source code as a command-line argument
    ///
    /// This does not modify the input in any way, so you must provide a complete program with a
    /// `main` function.
    #[arg(long, value_name = "INPUT")]
    pub eval_full: Option<String>,

    /// Name of the program to run
    #[arg(short, long)]
    pub program: Option<String>,

    /// Read source code from stdin
    #[arg(long)]
    pub stdin: bool,

    /// Path to a source code file
    pub path: Option<PathBuf>,
}

impl InputOptions {
    pub fn join(mut self, other: InputOptions) -> Self {
        self.eval = self.eval.or(other.eval);
        self.eval_full = self.eval_full.or(other.eval_full);
        self.program = self.program.or(other.program);
        self.stdin = self.stdin || other.stdin;
        self.path = self.path.or(other.path);

        self
    }
}

#[derive(Args)]
pub struct ParseCommand {
    #[command(flatten)]
    pub global: GlobalOptions,

    #[command(flatten)]
    pub input: InputOptions,

    /// Print syntax trees as pretty-printed Rusty Object Notation (.ron)
    ///
    /// Without this flag, syntax trees are rendered in an indented tree format.
    #[arg(long)]
    pub ron: bool,
}

impl ParseCommand {
    pub fn fill_arguments(mut self, global: GlobalOptions, input: InputOptions) -> Self {
        self.global = self.global.join(global);
        self.input = self.input.join(input);

        self
    }
}

#[derive(Args)]
pub struct CompileCommand {
    #[command(flatten)]
    pub global: GlobalOptions,

    #[command(flatten)]
    pub input: InputOptions,

    #[arg(short, long)]
    #[arg(value_enum, default_value = "tui")]
    pub output: CompileOutput,
}

impl CompileCommand {
    pub fn fill_arguments(mut self, global: GlobalOptions, input: InputOptions) -> Self {
        self.global = self.global.join(global);
        self.input = self.input.join(input);

        self
    }
}

#[derive(ValueEnum, Clone, Copy)]
pub enum CompileOutput {
    /// Interactive TUI disassembler (default)
    ///
    /// Navigate with arrow keys or `hjkl` and press `q` to quit.
    Tui,

    /// Generic text format that includes instruction disassembly information
    ///
    /// Unlike RON, this format includes instruction disassembly information but is not a proper
    /// serialization and cannot be parsed back into a program.
    Debug,

    /// Rusty Object Notation (.ron)
    ///
    /// This format can be parsed back into a program using the `ron` crate.
    Ron,

    /// Pretty-printed Rusty Object Notation (.ron)
    ///
    /// This format can be parsed back into a program using the `ron` crate.
    PrettyRon,
}

#[derive(Args)]
pub struct RunCommand {
    #[command(flatten)]
    pub global: GlobalOptions,

    #[command(flatten)]
    pub input: InputOptions,
}

impl RunCommand {
    pub fn fill_arguments(mut self, global: GlobalOptions, input: InputOptions) -> Self {
        self.global = self.global.join(global);
        self.input = self.input.join(input);

        self
    }
}

#[derive(Args)]
pub struct InitCommand {
    #[command(flatten)]
    pub global: GlobalOptions,

    /// Directory in which to initialize the project
    pub path: Option<PathBuf>,
}

impl InitCommand {
    pub fn fill_arguments(mut self, global: GlobalOptions, input: InputOptions) -> Self {
        self.global = self.global.join(global);
        self.path = self.path.or(input.path);

        self
    }
}
