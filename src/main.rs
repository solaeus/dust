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

use dust_lang::{built_in_values, Context, Error, Interpreter, Value, ValueData};

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
    Syntax { path: String },
}

fn main() {
    env_logger::init();

    let args = Args::parse();
    let context = Context::new();

    if let Some(input) = args.input {
        context
            .set_value("input".to_string(), Value::string(input))
            .unwrap();
    }

    if let Some(path) = args.input_path {
        let file_contents = read_to_string(path).unwrap();

        context
            .set_value("input".to_string(), Value::string(file_contents))
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

    if let Some(CliCommand::Syntax { path }) = args.cli_command {
        let source = read_to_string(path).unwrap();
        let syntax_tree_sexp = interpreter.syntax_tree(&source).unwrap();

        println!("{syntax_tree_sexp}");

        return;
    }

    if let Some(CliCommand::Format) = args.cli_command {
        let formatted = interpreter.format(&source).unwrap();

        println!("{formatted}");

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
    context: Context,
}

impl DustHighlighter {
    fn new(context: Context) -> Self {
        Self { context }
    }
}

const HIGHLIGHT_TERMINATORS: [char; 8] = [' ', ':', '(', ')', '{', '}', '[', ']'];

impl Highlighter for DustHighlighter {
    fn highlight(&self, line: &str, _cursor: usize) -> reedline::StyledText {
        let mut styled = StyledText::new();

        for word in line.split_inclusive(&HIGHLIGHT_TERMINATORS) {
            let mut word_is_highlighted = false;

            for key in self.context.inner().unwrap().keys() {
                if key == &word {
                    styled.push((Style::new().bold(), word.to_string()));
                }

                word_is_highlighted = true;
            }

            for built_in_value in built_in_values() {
                if built_in_value.name() == word {
                    styled.push((Style::new().bold(), word.to_string()));
                }

                word_is_highlighted = true;
            }

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
            "".to_string()
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
    context: Context,
}

impl DustCompleter {
    fn new(context: Context) -> Self {
        DustCompleter { context }
    }
}

impl Completer for DustCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();
        let last_word = if let Some(word) = line.rsplit([' ', ':']).next() {
            word
        } else {
            line
        };

        if let Ok(path) = PathBuf::try_from(last_word) {
            if let Ok(read_dir) = path.read_dir() {
                for entry in read_dir {
                    if let Ok(entry) = entry {
                        let description = if let Ok(file_type) = entry.file_type() {
                            if file_type.is_dir() {
                                "directory"
                            } else if file_type.is_file() {
                                "file"
                            } else if file_type.is_symlink() {
                                "symlink"
                            } else {
                                "unknown"
                            }
                        } else {
                            "unknown"
                        };

                        suggestions.push(Suggestion {
                            value: entry.path().to_string_lossy().to_string(),
                            description: Some(description.to_string()),
                            extra: None,
                            span: Span::new(pos - last_word.len(), pos),
                            append_whitespace: false,
                        });
                    }
                }
            }
        }

        for built_in_value in built_in_values() {
            let name = built_in_value.name();
            let description = built_in_value.description();

            if built_in_value.name().contains(last_word) {
                suggestions.push(Suggestion {
                    value: name.to_string(),
                    description: Some(description.to_string()),
                    extra: None,
                    span: Span::new(pos - last_word.len(), pos),
                    append_whitespace: false,
                });
            }

            if let Value::Map(map) = built_in_value.get() {
                for (key, value) in map.iter() {
                    if key.contains(last_word) {
                        suggestions.push(Suggestion {
                            value: format!("{name}:{key}"),
                            description: Some(value.to_string()),
                            extra: None,
                            span: Span::new(pos - last_word.len(), pos),
                            append_whitespace: false,
                        });
                    }
                }
            }
        }

        for (key, value_data) in self.context.inner().unwrap().iter() {
            let value = match value_data {
                ValueData::Value { inner, .. } => inner,
                ValueData::ExpectedType { .. } => continue,
            };

            if key.contains(last_word) {
                suggestions.push(Suggestion {
                    value: key.to_string(),
                    description: Some(value.to_string()),
                    extra: None,
                    span: Span::new(pos - last_word.len(), pos),
                    append_whitespace: false,
                });
            }
        }

        suggestions
    }
}

fn run_shell(context: Context) -> Result<(), Error> {
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
        KeyModifiers::NONE,
        KeyCode::Tab,
        ReedlineEvent::Multiple(vec![
            ReedlineEvent::Menu("context menu".to_string()),
            ReedlineEvent::MenuNext,
        ]),
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
        .with_highlighter(Box::new(DustHighlighter::new(context.clone())))
        .with_hinter(hinter)
        .use_kitty_keyboard_enhancement(true)
        .with_completer(Box::new(completer))
        .with_menu(ReedlineMenu::EngineCompleter(Box::new(
            ColumnarMenu::default()
                .with_name("context menu")
                .with_text_style(Style::new().fg(Color::White))
                .with_columns(1)
                .with_column_padding(10),
        )));
    let mut prompt = StarshipPrompt::new();

    prompt.reload();

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
                    Err(error) => println!("{error}"),
                }

                prompt.reload();
            }
            Ok(Signal::CtrlD) | Ok(Signal::CtrlC) => {
                println!("\nLeaving the Dust shell.");
                break;
            }
            x => {
                println!("Unknown event: {:?}", x);
            }
        }
    }

    Ok(())
}
