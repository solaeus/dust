//! Command line interface for the dust programming language.
use clap::Parser;
use crossterm::{
    event::{poll, read, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{CrosstermBackend, Rect},
    widgets::Paragraph,
    Terminal,
};
use std::{cell::RefCell, fs::read_to_string, io::stdout, time::Duration};
use tui_textarea::TextArea;

use dust_lang::{Interpreter, Map, Result, Value};

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
        run_tui().unwrap();

        return;
    }

    let source_input = if let Some(path) = &args.path {
        read_to_string(path).unwrap()
    } else if let Some(command) = &args.command {
        command.clone()
    } else {
        "(output 'Hello dust!')".to_string()
    };
    let source = RefCell::new(source_input);

    let mut context = Map::new();

    if let Some(input) = args.input {
        context
            .set("input".to_string(), Value::String(input), None)
            .unwrap();
    }

    if let Some(path) = args.input_path {
        let file_contents = read_to_string(path).unwrap();

        context
            .set("input".to_string(), Value::String(file_contents), None)
            .unwrap();
    }

    let mut interpreter = Interpreter::parse(&mut context, &source).unwrap();

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
            if !value.is_none() {
                println!("{value}")
            }
        }
        Err(error) => eprintln!("{error}"),
    }
}

fn run_tui() -> Result<()> {
    let user_input = RefCell::new("(output 'Hello dust!')".to_string());
    let mut context = Map::new();
    let mut interpreter = Interpreter::parse(&mut context, &user_input)?;

    interpreter.update()?;

    let mut interpreter_output = interpreter.run();

    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let mut textarea = TextArea::default();

    loop {
        terminal.draw(|frame| {
            let input_area = Rect {
                x: 0,
                y: 0,
                width: frame.size().width,
                height: frame.size().height / 2,
            };

            frame.render_widget(textarea.widget(), input_area);

            let output_area = Rect {
                x: input_area.left(),
                y: input_area.bottom(),
                width: frame.size().width,
                height: frame.size().height / 2,
            };

            match &interpreter_output {
                Ok(value) => frame.render_widget(Paragraph::new(value.to_string()), output_area),
                Err(error) => frame.render_widget(Paragraph::new(error.to_string()), output_area),
            }
        })?;

        if poll(Duration::from_millis(16))? {
            if let Event::Key(key) = read()? {
                if key.code == KeyCode::Esc {
                    break;
                }
                if key.code == KeyCode::Enter {
                    let input = textarea.lines().join("\n");

                    user_input.replace(input);
                    interpreter.update()?;
                    interpreter_output = interpreter.run();
                }

                textarea.input(key);
            }
        }
    }

    terminal.clear()?;
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}
