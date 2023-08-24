use std::time::Instant;

use crate::{Result, Time, Tool, ToolInfo, Value, ValueType};

pub struct Now;

impl Tool for Now {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "now",
            description: "Return the current time.",
            group: "time",
            inputs: vec![ValueType::Empty],
        }
    }

    fn run(&self, argument: &crate::Value) -> Result<Value> {
        argument.as_empty()?;

        let time = Time::monotonic(Instant::now());

        Ok(Value::Time(time))
    }
}

pub struct Local;

impl Tool for Local {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "local",
            description: "Show a time value adjusted for the current time zone.",
            group: "time",
            inputs: vec![ValueType::Time],
        }
    }

    fn run(&self, argument: &crate::Value) -> Result<Value> {
        let argument = argument.as_time()?;

        Ok(Value::String(argument.as_local()))
    }
}
