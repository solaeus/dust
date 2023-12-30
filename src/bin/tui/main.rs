pub mod app;
pub mod interpreter_display;
pub mod log;
pub mod terminal;
pub mod value_display;

use std::path::PathBuf;

use crate::{
    app::App,
    log::{get_data_dir, initialize_logging},
};
use clap::Parser;
use color_eyre::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use dust_lang::Value;
use ratatui::Frame;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// File with source to be run and watched by the shell.
    path: Option<String>,
}

pub trait Elm {
    fn update(&mut self, message: Action) -> Result<Option<Action>>;
    fn view(&self, frame: &mut Frame);
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Action {
    Init,
    Quit,
    Error,
    Closed,
    Tick,
    Render,
    FocusGained,
    FocusLost,
    Paste(String),
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    UpdateValue(Value),
}

#[tokio::main]
async fn main() {
    initialize_logging().unwrap();

    let args = Cli::parse();
    let (action_tx, action_rx) = mpsc::unbounded_channel();
    let path = if let Some(path) = args.path {
        PathBuf::from(path)
    } else {
        PathBuf::from(format!("{}/scratch.ds", get_data_dir().to_string_lossy()))
    };
    let mut app = App::new(action_rx, action_tx, path).unwrap();
    let run_result = app.run().await;

    if let Err(report) = run_result {
        eprintln!("{report}")
    }
}
