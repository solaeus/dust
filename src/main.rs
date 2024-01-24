//! Command line interface for the dust programming language.

use clap::{Parser, Subcommand};
use rustyline::{
    completion::FilenameCompleter,
    error::ReadlineError,
    highlight::Highlighter,
    hint::{Hint, Hinter, HistoryHinter},
    history::DefaultHistory,
    Completer, Context, Editor, Helper, Validator,
};

use std::{borrow::Cow, fs::read_to_string};

use dust_lang::{Interpreter, Map, Value};

/// Command-line arguments to be parsed.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Dust source code to evaluate.
    #[arg(short, long)]
    command: Option<String>,

    /// Data to assign to the "input" variable.
    #[arg(short, long)]
    input: Option<String>,

    /// File whose contents will be assigned to the "input" variable.
    #[arg(short = 'p', long)]
    input_path: Option<String>,

    /// Command for alternate functionality besides running the source.
    #[command(subcommand)]
    cli_command: Option<CliCommand>,

    /// Location of the file to run.
    path: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum CliCommand {
    /// Output a formatted version of the input.
    Format,

    /// Output a concrete syntax tree of the input.
    Syntax,
}

fn main() {
    env_logger::init();

    let args = Args::parse();
    let context = Map::new();

    if let Some(input) = args.input {
        context
            .set("input".to_string(), Value::string(input))
            .unwrap();
    }

    if let Some(path) = args.input_path {
        let file_contents = read_to_string(path).unwrap();

        context
            .set("input".to_string(), Value::string(file_contents))
            .unwrap();
    }

    if args.path.is_none() && args.command.is_none() {
        return run_cli_shell(context);
    }

    let source = if let Some(path) = &args.path {
        read_to_string(path).unwrap()
    } else if let Some(command) = args.command {
        command
    } else {
        String::with_capacity(0)
    };

    let mut interpreter = Interpreter::new(context);

    if let Some(CliCommand::Syntax) = args.cli_command {
        interpreter.parse(&source).unwrap();

        println!("{}", interpreter.syntax_tree().unwrap());

        return;
    }

    if let Some(CliCommand::Format) = args.cli_command {
        println!("{}", interpreter.format());

        return;
    }

    let eval_result = interpreter.run(&source);

    match eval_result {
        Ok(value) => {
            if !value.is_none() {
                println!("{value}")
            }
        }
        Err(error) => eprintln!("{error}"),
    }
}

#[derive(Helper, Completer, Validator)]
struct DustReadline {
    #[rustyline(Completer)]
    completer: FilenameCompleter,

    tool_hints: Vec<ToolHint>,

    #[rustyline(Hinter)]
    _hinter: HistoryHinter,
}

impl DustReadline {
    fn new() -> Self {
        Self {
            completer: FilenameCompleter::new(),
            _hinter: HistoryHinter {},
            tool_hints: Vec::new(),
        }
    }
}

struct ToolHint {
    display: String,
    complete_to: usize,
}

impl Hint for ToolHint {
    fn display(&self) -> &str {
        &self.display
    }

    fn completion(&self) -> Option<&str> {
        if self.complete_to > 0 {
            Some(&self.display[..self.complete_to])
        } else {
            None
        }
    }
}

impl ToolHint {
    fn suffix(&self, strip_chars: usize) -> ToolHint {
        ToolHint {
            display: self.display[strip_chars..].to_string(),
            complete_to: self.complete_to.saturating_sub(strip_chars),
        }
    }
}

impl Hinter for DustReadline {
    type Hint = ToolHint;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context) -> Option<Self::Hint> {
        if line.is_empty() || pos < line.len() {
            return None;
        }

        self.tool_hints.iter().find_map(|tool_hint| {
            if tool_hint.display.starts_with(line) {
                Some(tool_hint.suffix(pos))
            } else {
                None
            }
        })
    }
}

impl Highlighter for DustReadline {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        let highlighted = ansi_term::Colour::Red.paint(hint).to_string();

        Cow::Owned(highlighted)
    }
}

fn run_cli_shell(context: Map) {
    let mut interpreter = Interpreter::new(context);
    let mut rl: Editor<DustReadline, DefaultHistory> = Editor::new().unwrap();

    rl.set_helper(Some(DustReadline::new()));

    if rl.load_history("target/history.txt").is_err() {
        println!("No previous history.");
    }

    loop {
        let readline = rl.readline("* ");
        match readline {
            Ok(line) => {
                let input = line.to_string();

                rl.add_history_entry(line).unwrap();

                let eval_result = interpreter.run(&input);

                match eval_result {
                    Ok(value) => println!("{value}"),
                    Err(error) => {
                        eprintln!("{error}")
                    }
                }
            }
            Err(ReadlineError::Interrupted) => break,
            Err(ReadlineError::Eof) => break,
            Err(error) => eprintln!("{error}"),
        }
    }

    rl.save_history("target/history.txt").unwrap();
}
