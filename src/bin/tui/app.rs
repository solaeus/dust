use std::path::PathBuf;

use crossterm::event::KeyCode;
use dust_lang::Map;
use ratatui::Frame;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{interpreter_display::InterpreterDisplay, terminal::Terminal, Action, Elm, Result};

pub struct App {
    action_rx: UnboundedReceiver<Action>,
    action_tx: UnboundedSender<Action>,
    interpreter_display: InterpreterDisplay,
    should_quit: bool,
}

impl App {
    pub fn new(
        action_rx: UnboundedReceiver<Action>,
        action_tx: UnboundedSender<Action>,
        path: PathBuf,
    ) -> Result<Self> {
        let interpreter_display = InterpreterDisplay::new(Map::new(), path)?;

        Ok(App {
            action_rx,
            action_tx,
            interpreter_display,
            should_quit: false,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut terminal = Terminal::new()?.tick_rate(4.0).frame_rate(30.0);

        terminal.enter()?;

        loop {
            if self.should_quit {
                break;
            }

            if let Some(action) = terminal.next().await {
                self.action_tx.send(action)?;
            } else {
                continue;
            };

            while let Ok(action) = self.action_rx.try_recv() {
                if let Action::Render = action {
                    terminal.draw(|frame| {
                        self.view(frame);
                    })?;
                }

                let next_action = self.update(action)?;

                if let Some(action) = next_action {
                    self.action_tx.send(action)?;
                }
            }
        }

        terminal.exit()?;

        Ok(())
    }
}

impl Elm for App {
    fn update(&mut self, message: Action) -> Result<Option<Action>> {
        match message {
            Action::Quit => self.should_quit = true,
            Action::Key(key_event) => {
                if let KeyCode::Esc = key_event.code {
                    return Ok(Some(Action::Quit));
                }
            }
            _ => {}
        }

        self.interpreter_display.update(message)
    }

    fn view(&self, frame: &mut Frame) {
        self.interpreter_display.view(frame)
    }
}
