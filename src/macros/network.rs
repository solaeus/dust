//! Macros for network access.

use crate::{Macro, MacroInfo, Result, Value};

pub struct Download;

impl Macro for Download {
    fn info(&self) -> MacroInfo<'static> {
        MacroInfo {
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
