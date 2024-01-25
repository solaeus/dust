//! Command line interface for the dust programming language.

use clap::{Parser, Subcommand};
use crossterm::event::{KeyCode, KeyModifiers};
use nu_ansi_term::Style;
use reedline::{
    default_emacs_keybindings, ColumnarMenu, DefaultCompleter, DefaultPrompt, EditCommand, Emacs,
    Highlighter, Reedline, ReedlineEvent, ReedlineMenu, Signal, SqliteBackedHistory, StyledText,
};

use std::{fs::read_to_string, path::PathBuf};

use dust_lang::{built_in_values, Interpreter, Map, Result, Value};

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
        let run_shell_result = run_shell(context);

        match run_shell_result {
            Ok(_) => {}
            Err(error) => eprintln!("{error}"),
        }

        return;
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

struct DustHighlighter {
    context: Map,
}

impl DustHighlighter {
    fn new(context: Map) -> Self {
        Self { context }
    }
}

impl Highlighter for DustHighlighter {
    fn highlight(&self, line: &str, _cursor: usize) -> reedline::StyledText {
        fn highlight_identifier(styled: &mut StyledText, word: &str, map: &Map) -> bool {
            for (key, (value, _type)) in map.variables().unwrap().iter() {
                if key == &word[0..word.len() - 1] {
                    styled.push((Style::new().bold(), word.to_string()));

                    return true;
                }

                if let Value::Map(nested_map) = value {
                    return highlight_identifier(styled, key, nested_map);
                }
            }

            for built_in_value in built_in_values() {
                if built_in_value.name() == &word[0..word.len() - 1] {
                    styled.push((Style::new().bold(), word.to_string()));
                }
            }

            false
        }

        let mut styled = StyledText::new();

        for word in line.split_inclusive(&[' ', ':', '(', ')', '{', '}', '[', ']']) {
            let word_is_highlighted = highlight_identifier(&mut styled, word, &self.context);

            if !word_is_highlighted {
                styled.push((Style::new(), word.to_string()));
            }
        }

        styled
    }
}

fn run_shell(context: Map) -> Result<()> {
    let mut interpreter = Interpreter::new(context.clone());
    let prompt = DefaultPrompt::default();
    let mut keybindings = default_emacs_keybindings();

    keybindings.add_binding(
        KeyModifiers::ALT,
        KeyCode::Char('m'),
        ReedlineEvent::Edit(vec![EditCommand::BackspaceWord]),
    );
    keybindings.add_binding(
        KeyModifiers::NONE,
        KeyCode::Tab,
        ReedlineEvent::UntilFound(vec![
            ReedlineEvent::Menu("completion_menu".to_string()),
            ReedlineEvent::MenuNext,
        ]),
    );

    let edit_mode = Box::new(Emacs::new(keybindings));
    let history = Box::new(
        SqliteBackedHistory::with_file(PathBuf::from("target/history"), None, None)
            .expect("Error loading history."),
    );
    let mut commands = Vec::new();

    for built_in_value in built_in_values() {
        commands.push(built_in_value.name().to_string());
    }

    let completer = Box::new(DefaultCompleter::new_with_wordlen(commands.clone(), 0));
    let completion_menu = Box::new(ColumnarMenu::default().with_name("completion_menu"));
    let mut line_editor = Reedline::create()
        .with_edit_mode(edit_mode)
        .with_history(history)
        .with_highlighter(Box::new(DustHighlighter::new(context)))
        .with_completer(completer)
        .with_menu(ReedlineMenu::EngineCompleter(completion_menu));

    loop {
        let sig = line_editor.read_line(&prompt);
        match sig {
            Ok(Signal::Success(buffer)) => {
                if buffer.is_empty() {
                    continue;
                }

                let run_result = interpreter.run(&buffer);

                match run_result {
                    Ok(value) => {
                        if !value.is_none() {
                            println!("{value}")
                        }
                    }
                    Err(error) => println!("Error: {error}"),
                }
            }
            Ok(Signal::CtrlD) | Ok(Signal::CtrlC) => {
                println!("\nAborted!");
                break;
            }
            x => {
                println!("Event: {:?}", x);
            }
        }
    }

    Ok(())
}
