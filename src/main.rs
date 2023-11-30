//! Command line interface for the dust programming language.
use clap::Parser;
use rustyline::{
    completion::FilenameCompleter,
    error::ReadlineError,
    highlight::Highlighter,
    hint::{Hint, Hinter, HistoryHinter},
    history::DefaultHistory,
    Completer, Context, Editor, Helper, Validator,
};
use tree_sitter::Parser as TSParser;

use std::{borrow::Cow, fs::read_to_string};

use dust_lang::{evaluate_with_context, language, Interpreter, Map, Value};

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

    /// A path to file whose contents will be assigned to the "input" variable.
    #[arg(short = 'p', long)]
    input_path: Option<String>,

    /// Show the syntax tree.
    #[arg(short = 't', long = "tree")]
    show_syntax_tree: bool,

    /// Launch in interactive mode.
    #[arg(short = 'n', long)]
    interactive: bool,

    /// Location of the file to run.
    path: Option<String>,
}

fn main() {
    let args = Args::parse();

    if args.path.is_none() && args.command.is_none() {
        return run_cli_shell();
    }

    let source = if let Some(path) = &args.path {
        read_to_string(path).unwrap()
    } else if let Some(command) = &args.command {
        command.clone()
    } else {
        "".to_string()
    };

    let mut context = Map::new();

    if let Some(input) = args.input {
        context
            .variables_mut()
            .unwrap()
            .insert("input".to_string(), Value::String(input));
    }

    if let Some(path) = args.input_path {
        let file_contents = read_to_string(path).unwrap();

        context
            .variables_mut()
            .unwrap()
            .insert("input".to_string(), Value::String(file_contents));
    }

    let mut parser = TSParser::new();
    parser.set_language(language()).unwrap();

    let mut interpreter = Interpreter::parse(parser, &mut context, &source).unwrap();

    if args.interactive {
        loop {
            let result = interpreter.run();

            println!("{result:?}")
        }
    }

    if args.show_syntax_tree {
        println!("{}", interpreter.syntax_tree());
    }

    let eval_result = interpreter.run();

    match eval_result {
        Ok(value) => {
            if !value.is_empty() {
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
    let mut context = Map::new();
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

                let eval_result = evaluate_with_context(line, &mut context);

                match eval_result {
                    Ok(value) => println!("{value}"),
                    Err(error) => eprintln!("{error}"),
                }
            }
            Err(ReadlineError::Interrupted) => break,
            Err(ReadlineError::Eof) => break,
            Err(error) => eprintln!("{error}"),
        }
    }

    rl.save_history("target/history.txt").unwrap();
}
