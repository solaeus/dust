use std::{fs::read_to_string, path::PathBuf, time::SystemTime};

use dust_lang::{Interpreter, Map, Value};
use ratatui::Frame;

use crate::{value_display::ValueDisplay, Action, Elm, Result};

pub struct InterpreterDisplay {
    interpreter: Interpreter,
    path: PathBuf,
    value_display: ValueDisplay,
    modified: SystemTime,
}

impl InterpreterDisplay {
    pub fn new(context: Map, path: PathBuf) -> Result<Self> {
        let interpreter = Interpreter::new(context)?;
        let value_display = ValueDisplay::new(Value::default());
        let modified = SystemTime::now();

        Ok(Self {
            interpreter,
            path,
            value_display,
            modified,
        })
    }
}

impl Elm for InterpreterDisplay {
    fn update(&mut self, message: Action) -> Result<Option<Action>> {
        match message {
            Action::Tick => {
                let last_modified = self.path.metadata()?.modified()?;

                if last_modified != self.modified {
                    let source = read_to_string(&self.path)?;
                    let value = self.interpreter.run(&source)?;

                    self.value_display = ValueDisplay::new(value);
                    self.modified = last_modified;
                }
            }
            _ => {}
        }

        self.value_display.update(message)
    }

    fn view(&self, frame: &mut Frame) {
        self.value_display.view(frame)
    }
}
