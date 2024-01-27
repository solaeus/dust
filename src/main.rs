//! Command line interface for the dust programming language.

use clap::{Parser, Subcommand};
use crossterm::event::{KeyCode, KeyModifiers};
use nu_ansi_term::{Color, Style};
use reedline::{
    default_emacs_keybindings, ColumnarMenu, Completer, DefaultHinter, EditCommand, Emacs,
    Highlighter, Prompt, Reedline, ReedlineEvent, ReedlineMenu, Signal, Span, SqliteBackedHistory,
    StyledText, Suggestion,
};

use std::{borrow::Cow, fs::read_to_string, path::PathBuf, process::Command};

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

const HIGHLIGHT_TERMINATORS: [char; 8] = [' ', ':', '(', ')', '{', '}', '[', ']'];

impl Highlighter for DustHighlighter {
    fn highlight(&self, line: &str, _cursor: usize) -> reedline::StyledText {
        fn highlight_identifier(styled: &mut StyledText, word: &str, map: &Map) -> bool {
            for (key, (value, _type)) in map.variables().unwrap().iter() {
                if key == &word {
                    styled.push((Style::new().bold(), word.to_string()));

                    return true;
                }

                if let Value::Map(nested_map) = value {
                    return highlight_identifier(styled, word, nested_map);
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

        for word in line.split_inclusive(&HIGHLIGHT_TERMINATORS) {
            let word_is_highlighted =
                highlight_identifier(&mut styled, &word[0..word.len() - 1], &self.context);

            if word_is_highlighted {
                let final_char = word.chars().last().unwrap();

                if HIGHLIGHT_TERMINATORS.contains(&final_char) {
                    let mut terminator_style = Style::new();

                    terminator_style.foreground = Some(Color::Cyan);

                    styled.push((terminator_style, final_char.to_string()));
                }
            } else {
                styled.push((Style::new(), word.to_string()));
            }
        }

        styled
    }
}

struct StarshipPrompt {
    left: String,
    right: String,
}

impl StarshipPrompt {
    fn new() -> Self {
        Self {
            left: String::new(),
            right: String::new(),
        }
    }

    fn reload(&mut self) {
        let run_starship_left = Command::new("starship").arg("prompt").output();
        let run_starship_right = Command::new("starship")
            .args(["prompt", "--right"])
            .output();
        let left_prompt = if let Ok(output) = &run_starship_left {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        } else {
            ">".to_string()
        };
        let right_prompt = if let Ok(output) = &run_starship_right {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        } else {
            ">".to_string()
        };

        self.left = left_prompt;
        self.right = right_prompt;
    }
}

impl Prompt for StarshipPrompt {
    fn render_prompt_left(&self) -> Cow<str> {
        Cow::Borrowed(&self.left)
    }

    fn render_prompt_right(&self) -> Cow<str> {
        Cow::Borrowed(&self.right)
    }

    fn render_prompt_indicator(&self, _prompt_mode: reedline::PromptEditMode) -> Cow<str> {
        Cow::Borrowed(" ")
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<str> {
        Cow::Borrowed("")
    }

    fn render_prompt_history_search_indicator(
        &self,
        _history_search: reedline::PromptHistorySearch,
    ) -> Cow<str> {
        Cow::Borrowed("")
    }
}

pub struct DustCompleter {
    context: Map,
}

impl DustCompleter {
    fn new(context: Map) -> Self {
        DustCompleter { context }
    }
}

impl Completer for DustCompleter {
    fn complete(&mut self, _line: &str, pos: usize) -> Vec<Suggestion> {
        let variables = self.context.variables().unwrap();
        let mut suggestions = Vec::with_capacity(variables.len());

        for (key, (value, r#type)) in variables.iter() {
            suggestions.push(Suggestion {
                value: key.clone(),
                description: Some(value.to_string()),
                extra: Some(vec![r#type.to_string()]),
                span: Span::new(pos, pos),
                append_whitespace: false,
            });
        }

        suggestions
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
        KeyCode::Enter,
        ReedlineEvent::SubmitOrNewline,
    );
    keybindings.add_binding(
        KeyModifiers::NONE,
        KeyCode::Tab,
        ReedlineEvent::Edit(vec![EditCommand::InsertString("    ".to_string())]),
    );
    keybindings.add_binding(
        KeyModifiers::CONTROL,
        KeyCode::Char('h'),
        ReedlineEvent::Menu("help menu".to_string()),
    );

    let edit_mode = Box::new(Emacs::new(keybindings));
    let history = Box::new(
        SqliteBackedHistory::with_file(PathBuf::from("target/history"), None, None)
            .expect("Error loading history."),
    );
    let hinter = Box::new(DefaultHinter::default().with_style(Style::new().dimmed()));
    let completer = DustCompleter::new(context.clone());
    let mut line_editor = Reedline::create()
        .with_edit_mode(edit_mode)
        .with_history(history)
        .with_highlighter(Box::new(DustHighlighter::new(context)))
        .with_hinter(hinter)
        .use_kitty_keyboard_enhancement(true)
        .with_completer(Box::new(completer))
        .with_menu(ReedlineMenu::EngineCompleter(Box::new(
            ColumnarMenu::default().with_name("help menu"),
        )));
    let mut prompt = StarshipPrompt::new();

    prompt.reload();

    loop {
        let sig = line_editor.read_line(&prompt);

        match sig {
            Ok(Signal::Success(buffer)) => {
                prompt.reload();

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
