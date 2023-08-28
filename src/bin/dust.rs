//! Command line interface for the whale programming language.
use clap::Parser;
use nu_ansi_term::{Color, Style};
use reedline::{
    default_emacs_keybindings, ColumnarMenu, Completer, DefaultHinter, DefaultPrompt,
    DefaultPromptSegment, EditCommand, Emacs, FileBackedHistory, KeyCode, KeyModifiers, Reedline,
    ReedlineEvent, ReedlineMenu, Signal, Span, Suggestion,
};

use std::{
    fs::{self, read_to_string},
    path::PathBuf,
};

use dust_lib::{eval, eval_with_context, Tool, ToolInfo, Value, VariableMap, TOOL_LIST};

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

    let eval_result = if let Some(path) = args.path {
        let file_contents = read_to_string(path).unwrap();

        eval(&file_contents)
    } else if let Some(command) = args.command {
        eval(&command)
    } else {
        Ok(Value::Empty)
    };

    match eval_result {
        Ok(value) => println!("{value}"),
        Err(error) => eprintln!("{error}"),
    }
}

fn run_cli_shell() {
    let mut context = VariableMap::new();
    let mut line_editor = setup_reedline();
    let prompt = DefaultPrompt {
        left_prompt: DefaultPromptSegment::WorkingDirectory,
        right_prompt: DefaultPromptSegment::CurrentDateTime,
    };

    loop {
        let sig = line_editor.read_line(&prompt);

        match sig {
            Ok(Signal::Success(buffer)) => {
                let eval_result = eval_with_context(&buffer, &mut context);

                match eval_result {
                    Ok(value) => println!("{value}"),
                    Err(error) => eprintln!("{error}"),
                }
            }
            Ok(Signal::CtrlD) | Ok(Signal::CtrlC) => {
                println!("\nExit");
                break;
            }
            signal => {
                println!("Unhandled signal: {:?}", signal);
            }
        }
    }
}

struct WhaleCompeleter {
    macro_list: Vec<Suggestion>,
    files: Vec<Suggestion>,
}

impl WhaleCompeleter {
    pub fn new() -> Self {
        WhaleCompeleter {
            macro_list: Vec::new(),
            files: Vec::new(),
        }
    }

    pub fn set_command_list(&mut self, macro_list: Vec<&'static dyn Tool>) -> &mut Self {
        self.macro_list = macro_list
            .iter()
            .map(|r#macro| {
                let ToolInfo {
                    identifier,
                    description,
                    group,
                    inputs,
                } = r#macro.info();

                let description = format!("{description} | {group}");
                let inputs = inputs
                    .iter()
                    .map(|value_type| value_type.to_string())
                    .collect();

                Suggestion {
                    value: identifier.to_string() + "()",
                    description: Some(description),
                    extra: Some(inputs),
                    ..Default::default()
                }
            })
            .collect();

        self.macro_list
            .sort_by_key(|suggestion| suggestion.extra.clone());

        self
    }

    pub fn get_suggestions(&mut self, start: usize, end: usize) -> Vec<Suggestion> {
        let macro_suggestions = self
            .macro_list
            .iter()
            .cloned()
            .map(|suggestion| Suggestion {
                span: Span { start, end },
                ..suggestion
            });
        let file_suggestions = self.files.iter().cloned().map(|suggestion| Suggestion {
            span: Span { start, end },
            ..suggestion
        });

        file_suggestions.chain(macro_suggestions).collect()
    }

    pub fn update_files(&mut self, mut path: &str) {
        if path.starts_with('\"') {
            path = &path[1..];
        }

        let path = PathBuf::from(path);

        if !path.is_dir() {
            return;
        }

        self.files = fs::read_dir(path)
            .unwrap()
            .map(|entry| {
                let path = entry.unwrap().path();
                let path = path.to_string_lossy();

                Suggestion {
                    value: format!("\"{path}\""),
                    description: None,
                    ..Default::default()
                }
            })
            .collect();
    }
}

impl Completer for WhaleCompeleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        let split = line.split(' ');
        let current_word = split.last().unwrap_or("");
        let start = pos.saturating_sub(current_word.len());
        let end = line.len();

        self.update_files(current_word);
        self.get_suggestions(start, end)
    }
}

fn setup_reedline() -> Reedline {
    let mut completer = Box::new(WhaleCompeleter::new());

    completer.set_command_list(TOOL_LIST.to_vec());

    let completion_menu = Box::new(
        ColumnarMenu::default()
            .with_name("completion_menu")
            .with_columns(1)
            .with_text_style(Style {
                foreground: Some(Color::White),
                is_dimmed: false,
                ..Default::default()
            })
            .with_description_text_style(Style {
                is_dimmed: true,
                ..Default::default()
            })
            .with_selected_text_style(Style {
                is_bold: true,
                background: Some(Color::Black),
                foreground: Some(Color::White),
                ..Default::default()
            }),
    );

    let mut keybindings = default_emacs_keybindings();
    keybindings.add_binding(
        KeyModifiers::NONE,
        KeyCode::Tab,
        ReedlineEvent::UntilFound(vec![
            ReedlineEvent::Menu("completion_menu".to_string()),
            ReedlineEvent::MenuNext,
        ]),
    );
    keybindings.add_binding(
        KeyModifiers::SHIFT,
        KeyCode::Tab,
        ReedlineEvent::UntilFound(vec![
            ReedlineEvent::Menu("completion_menu".to_string()),
            ReedlineEvent::MenuPrevious,
        ]),
    );
    keybindings.add_binding(
        KeyModifiers::ALT,
        KeyCode::Enter,
        ReedlineEvent::Edit(vec![EditCommand::InsertNewline]),
    );

    let edit_mode = Box::new(Emacs::new(keybindings));
    let history = Box::new(
        FileBackedHistory::with_file(100, "target/history.txt".into())
            .expect("Error configuring shell history file."),
    );
    let mut hinter = DefaultHinter::default();

    hinter = hinter.with_style(Style {
        foreground: Some(Color::Yellow),
        ..Default::default()
    });

    Reedline::create()
        .with_completer(completer)
        .with_menu(ReedlineMenu::EngineCompleter(completion_menu))
        .with_edit_mode(edit_mode)
        .with_history(history)
        .with_hinter(Box::new(hinter))
        .with_partial_completions(true)
        .with_quick_completions(true)
}
