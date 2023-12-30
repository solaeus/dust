use dust_lang::Value;
use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, List, Paragraph},
    Frame,
};

use crate::{Action, Elm, Result};

pub struct ValueDisplay {
    value: Value,
}

impl ValueDisplay {
    pub fn new(value: Value) -> Self {
        ValueDisplay { value }
    }
}

impl Elm for ValueDisplay {
    fn update(&mut self, _message: Action) -> Result<Option<Action>> {
        Ok(None)
    }

    fn view(&self, frame: &mut Frame) {
        match &self.value {
            Value::List(list) => {
                let widget = List::new(list.items().iter().map(|value| value.to_string()))
                    .block(Block::default().title("list").borders(Borders::all()));

                frame.render_widget(widget, frame.size());
            }
            Value::Map(_) => todo!(),
            Value::Function(_) => todo!(),
            Value::String(string) => {
                let widget =
                    Paragraph::new(string.as_str()).style(Style::default().fg(Color::Green));

                frame.render_widget(widget, frame.size());
            }
            Value::Float(_) => todo!(),
            Value::Integer(integer) => {
                let widget =
                    Paragraph::new(integer.to_string()).style(Style::default().fg(Color::Red));

                frame.render_widget(widget, frame.size());
            }
            Value::Boolean(_) => todo!(),
            Value::Option(_) => todo!(),
        }
    }
}
