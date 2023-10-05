//! Command line interface for the whale programming language.
use clap::Parser;
use rustyline::{
    completion::FilenameCompleter,
    error::ReadlineError,
    highlight::Highlighter,
    hint::{Hint, Hinter, HistoryHinter},
    history::DefaultHistory,
    Completer, Context, Editor, Helper, Validator,
};

use std::{borrow::Cow, fs::read_to_string};

use dust_lib::{eval, eval_with_context, Value, VariableMap};

/// Command-line arguments to be parsed.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Whale source code to evaluate.
    #[arg(short, long)]
    command: Option<String>,

    /// Location of the file to run.
    path: Option<String>,
}

fn main() {
    let args = Args::parse();

    if args.path.is_none() && args.command.is_none() {
        return run_cli_shell();
    }

    let eval_results = if let Some(path) = args.path {
        let file_contents = read_to_string(path).unwrap();

        eval(&file_contents)
    } else if let Some(command) = args.command {
        eval(&command)
    } else {
        vec![Ok(Value::Empty)]
    };

    for result in eval_results {
        match result {
            Ok(value) => println!("{value}"),
            Err(error) => eprintln!("{error}"),
        }
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

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<Self::Hint> {
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
    fn highlight_hint<'h>(&self, hint: &'h str) -> std::borrow::Cow<'h, str> {
        let highlighted = ansi_term::Colour::Red.paint(hint).to_string();

        Cow::Owned(highlighted)
    }
}

fn run_cli_shell() {
    let mut context = VariableMap::new();
    let mut rl: Editor<DustReadline, DefaultHistory> = Editor::new().unwrap();

    rl.set_helper(Some(DustReadline::new()));

    if rl.load_history("target/history.txt").is_err() {
        println!("No previous history.");
    }

    loop {
        let readline = rl.readline("* ");
        match readline {
            Ok(line) => {
                let line = line.as_str();

                rl.add_history_entry(line).unwrap();

                let eval_results = eval_with_context(line, &mut context);

                for result in eval_results {
                    match result {
                        Ok(value) => println!("{value}"),
                        Err(error) => eprintln!("{error}"),
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                break;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(error) => eprintln!("{error}"),
        }
    }

    rl.save_history("target/history.txt").unwrap();
}
