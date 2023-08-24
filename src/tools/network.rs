//! Macros for network access.

use crate::{Result, Tool, ToolInfo, Value, ValueType};

pub struct Download;

impl Tool for Download {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "download",
            description: "Fetch a network resource.",
            group: "network",
            inputs: vec![ValueType::String],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let argument = argument.as_string()?;
        let output = reqwest::blocking::get(argument)?.text()?;

        Ok(Value::String(output))
    }
}
