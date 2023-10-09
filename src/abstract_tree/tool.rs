use std::fs::read_to_string;

use serde::{Deserialize, Serialize};

use crate::{Result, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum Tool {
    Output,
    Read,
}

impl Tool {
    pub fn run(&self, value: &Value) -> Result<Value> {
        let value = match self {
            Tool::Output => {
                println!("{value}");

                Value::Empty
            }
            Tool::Read => {
                let file_contents = read_to_string(value.as_string()?)?;

                Value::String(file_contents)
            }
        };

        Ok(value)
    }
}
