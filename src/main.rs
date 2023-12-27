//! Command line interface for the dust programming language.
use clap::Parser;
use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::CrosstermBackend, style::Stylize, widgets::Paragraph, Terminal};
use tree_sitter::Parser as TSParser;

use std::{fs::read_to_string, io::stdout};

use dust_lang::{language, Interpreter, Map, Result, Value};

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
        return run_cli_shell().unwrap();
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
            .set("input".to_string(), Value::String(input), None)
            .unwrap();
    }

    if let Some(path) = args.input_path {
        let file_contents = read_to_string(path).unwrap();

        context
            .set("input".to_string(), Value::String(file_contents), None)
            .unwrap();
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
            if !value.is_none() {
                println!("{value}")
            }
        }
        Err(error) => eprintln!("{error}"),
    }
}

fn run_cli_shell() -> Result<()> {
    let mut _context = Map::new();

    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    loop {
        terminal.draw(|frame| {
            let area = frame.size();
            frame.render_widget(
                Paragraph::new("Hello Ratatui! (press 'q' to quit)")
                    .white()
                    .on_blue(),
                area,
            );
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    terminal.clear()?;
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}
