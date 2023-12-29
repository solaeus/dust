use dust_lang::Value;
use ratatui::{prelude::Rect, widgets::Paragraph, Frame};

pub struct OutputDisplay {
    values: Vec<Value>,
}

impl OutputDisplay {
    pub fn new() -> Self {
        OutputDisplay { values: Vec::new() }
    }

    pub fn add_value(&mut self, value: Value) {
        self.values.push(value);
    }

    pub fn run(&self, frame: &mut Frame, area: Rect) {
        for value in &self.values {
            match value {
                Value::List(_) => todo!(),
                Value::Map(_) => todo!(),
                Value::Function(_) => todo!(),
                Value::String(string) => frame.render_widget(Paragraph::new(string.as_str()), area),
                Value::Float(_) => todo!(),
                Value::Integer(integer) => {
                    frame.render_widget(Paragraph::new(integer.to_string()), area)
                }
                Value::Boolean(_) => todo!(),
                Value::Option(_) => todo!(),
            }
        }
    }
}
