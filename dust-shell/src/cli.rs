use std::{
    borrow::Cow,
    io::{self, stderr},
    path::PathBuf,
    process::Command,
    sync::Arc,
};

use ariadne::sources;
use dust_lang::{
    context::{Context, ValueData},
    *,
};
use nu_ansi_term::{Color, Style};
use reedline::{
    default_emacs_keybindings, ColumnarMenu, Completer, DefaultHinter, EditCommand, Emacs, KeyCode,
    KeyModifiers, MenuBuilder, Prompt, Reedline, ReedlineEvent, ReedlineMenu, Signal, Span,
    SqliteBackedHistory, Suggestion,
};

pub fn run_shell(context: Context) -> Result<(), io::Error> {
    let interpreter = Interpreter::new(context.clone());
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
        .with_hinter(hinter)
        .use_kitty_keyboard_enhancement(true)
        .with_completer(Box::new(completer))
        .with_menu(ReedlineMenu::EngineCompleter(Box::new(
            ColumnarMenu::default()
                .with_name("context menu")
                .with_text_style(Style::default().fg(Color::White))
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

                let run_result = interpreter.run(Arc::from("input"), Arc::from(buffer.as_str()));

                match run_result {
                    Ok(Some(value)) => {
                        println!("{value}")
                    }
                    Ok(None) => {}
                    Err(error) => {
                        let reports = error.build_reports();

                        for report in reports {
                            report
                                .write_for_stdout(sources(interpreter.sources()), stderr())
                                .unwrap();
                        }
                    }
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
                            style: None,
                        });
                    }
                }
            }
        }

        for (key, value_data) in self.context.inner().unwrap().iter() {
            let description = match value_data {
                ValueData::Value(value) => value.to_string(),
                ValueData::Type(r#type) => r#type.to_string(),
            };

            if key.as_str().contains(last_word) {
                suggestions.push(Suggestion {
                    value: key.to_string(),
                    description: Some(description),
                    extra: None,
                    span: Span::new(pos - last_word.len(), pos),
                    append_whitespace: false,
                    style: None,
                });
            }
        }

        suggestions
    }
}
