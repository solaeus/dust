//! Command line interface for the dust programming language.

use clap::{Parser, Subcommand};
use crossterm::event::{KeyCode, KeyModifiers};
use nu_ansi_term::Style;
use reedline::{
    default_emacs_keybindings, ColumnarMenu, DefaultCompleter, DefaultHinter, EditCommand, Emacs,
    Highlighter, Prompt, Reedline, ReedlineEvent, ReedlineMenu, Signal, SqliteBackedHistory,
    StyledText,
};

use std::{borrow::Cow, fs::read_to_string, path::PathBuf, time::SystemTime};

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
                if key == &word {
                    styled.push((Style::new().bold(), word.to_string()));

                    return true;
                }

                if let Value::Map(nested_map) = value {
                    return highlight_identifier(styled, key, nested_map);
                }
            }

            for built_in_value in built_in_values() {
                if built_in_value.name() == word {
                    styled.push((Style::new().bold(), word.to_string()));

                    return true;
                }
            }

            false
        }

        let mut styled = StyledText::new();
        let terminators = [' ', ':', '(', ')', '{', '}', '[', ']'];

        for word in line.split_inclusive(&terminators) {
            let word_is_highlighted =
                highlight_identifier(&mut styled, &word[0..word.len() - 1], &self.context);

            if word_is_highlighted {
                let final_char = word.chars().last().unwrap();

                if terminators.contains(&final_char) {
                    styled.push((Style::new(), final_char.to_string()));
                }
            } else {
                styled.push((Style::new(), word.to_string()));
            }
        }

        styled
    }
}

struct DustPrompt;

impl Prompt for DustPrompt {
    fn render_prompt_left(&self) -> Cow<str> {
        let path = std::env::current_dir()
            .map(|path| path.file_name().unwrap().to_string_lossy().to_string())
            .unwrap_or_else(|_| "No workdir".to_string());

        Cow::Owned(path)
    }

    fn render_prompt_right(&self) -> Cow<str> {
        let time = humantime::format_rfc3339_seconds(SystemTime::now()).to_string();

        Cow::Owned(time)
    }

    fn render_prompt_indicator(&self, _prompt_mode: reedline::PromptEditMode) -> Cow<str> {
        Cow::Borrowed(" > ")
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<str> {
        Cow::Borrowed(" > ")
    }

    fn render_prompt_history_search_indicator(
        &self,
        _history_search: reedline::PromptHistorySearch,
    ) -> Cow<str> {
        Cow::Borrowed(" ? ")
    }
}

fn run_shell(context: Map) -> Result<()> {
    let mut interpreter = Interpreter::new(context.clone());
    let mut keybindings = default_emacs_keybindings();

    keybindings.add_binding(
        KeyModifiers::CONTROL,
        KeyCode::Char(' '),
        ReedlineEvent::Edit(vec![EditCommand::InsertNewline]),
    );
    keybindings.add_binding(
        KeyModifiers::NONE,
        KeyCode::Tab,
        ReedlineEvent::UntilFound(vec![
            ReedlineEvent::Menu("completion_menu".to_string()),
            ReedlineEvent::MenuNext,
        ]),
    );
    keybindings.add_binding(
        KeyModifiers::CONTROL,
        KeyCode::Tab,
        ReedlineEvent::Edit(vec![EditCommand::InsertString("    ".to_string())]),
    );

    let edit_mode = Box::new(Emacs::new(keybindings));
    let history = Box::new(
        SqliteBackedHistory::with_file(PathBuf::from("target/history"), None, None)
            .expect("Error loading history."),
    );
    let hinter = Box::new(DefaultHinter::default());
    let mut line_editor = Reedline::create()
        .with_edit_mode(edit_mode)
        .with_history(history)
        .with_highlighter(Box::new(DustHighlighter::new(context)))
        .with_hinter(hinter)
        .with_menu(ReedlineMenu::WithCompleter {
            menu: Box::new(ColumnarMenu::default().with_name("completion_menu")),
            completer: Box::new(DefaultCompleter::new_with_wordlen(
                vec!["test".to_string()],
                2,
            )),
        });
    let prompt = DustPrompt;

    loop {
        let sig = line_editor.read_line(&prompt);
        match sig {
            Ok(Signal::Success(buffer)) => {
                if buffer.trim().is_empty() {
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
