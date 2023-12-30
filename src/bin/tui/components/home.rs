use std::{
    collections::HashMap,
    fs::read_to_string,
    path::PathBuf,
    time::{Duration, SystemTime},
};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use dust_lang::{interpret, interpret_with_context, Map, Value};
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use tui_tree_widget::{Tree, TreeItem, TreeState};

use super::{map_display::MapDisplay, Component, Frame};
use crate::{
    action::Action,
    config::{Config, KeyBindings},
};

pub struct Home<'a> {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    path: PathBuf,
    source: String,
    context: Map,
    context_tree_state: TreeState<String>,
    context_display: MapDisplay<'a>,
    output: dust_lang::Result<Value>,
    last_modified: SystemTime,
}

impl<'a> Home<'a> {
    pub fn new(path: PathBuf) -> Result<Self> {
        let context = Map::new();
        let context_display = MapDisplay::new(context.clone())?;

        Ok(Home {
            command_tx: None,
            config: Config::default(),
            path,
            source: String::new(),
            context,
            context_display,
            context_tree_state: TreeState::default(),
            output: Ok(Value::default()),
            last_modified: SystemTime::now(),
        })
    }
}

impl<'a> Component for Home<'a> {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => {
                let modified = self.path.metadata()?.modified()?;

                if modified != self.last_modified {
                    self.source = read_to_string(&self.path)?;
                    self.last_modified = modified;
                    self.output = interpret_with_context(&self.source, self.context.clone());
                    self.context_display = MapDisplay::new(self.context.clone())?;
                }
            }
            _ => {}
        }

        self.context_display.update(action)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        self.context_display.draw(frame, layout[0])?;

        let output_text = match &self.output {
            Ok(value) => value.to_string(),
            Err(error) => error.to_string(),
        };

        frame.render_widget(Paragraph::new(output_text), layout[1]);

        Ok(())
    }
}
