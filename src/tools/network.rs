//! Macros for network access.

use crate::{Result, Tool, ToolInfo, Value};

pub struct Download;

impl Tool for Download {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "download",
            description: "Fetch a network resource.",
            group: "network",
            inputs: vec![],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let argument = argument.as_string()?;
        let output = reqwest::blocking::get(argument)?.text()?;

        Ok(Value::String(output))
    }
}
