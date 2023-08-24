use std::process::Command;

use crate::{Result, Tool, ToolInfo, Value};

pub struct Sh;

impl Tool for Sh {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "sh",
            description: "Pass input to the Bourne Shell.",
            group: "command",
            inputs: vec![],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let argument = argument.as_string()?;

        Command::new("sh").arg("-c").arg(argument).spawn()?.wait()?;

        Ok(Value::Empty)
    }
}

pub struct Bash;

impl Tool for Bash {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "bash",
            description: "Pass input to the Bourne Again Shell.",
            group: "command",
            inputs: vec![],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let argument = argument.as_string()?;

        Command::new("bash")
            .arg("-c")
            .arg(argument)
            .spawn()?
            .wait()?;

        Ok(Value::Empty)
    }
}
pub struct Fish;

impl Tool for Fish {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "fish",
            description: "Pass input to the fish shell.",
            group: "command",
            inputs: vec![],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let argument = argument.as_string()?;

        Command::new("fish")
            .arg("-c")
            .arg(argument)
            .spawn()?
            .wait()?;

        Ok(Value::Empty)
    }
}

pub struct Zsh;

impl Tool for Zsh {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "zsh",
            description: "Pass input to the Z shell.",
            group: "command",
            inputs: vec![],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let argument = argument.as_string()?;

        Command::new("zsh")
            .arg("-c")
            .arg(argument)
            .spawn()?
            .wait()?;

        Ok(Value::Empty)
    }
}

pub struct Raw;

impl Tool for Raw {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "raw",
            description: "Run input as a command without a shell",
            group: "command",
            inputs: vec![],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let argument = argument.as_string()?;

        Command::new(argument).spawn()?.wait()?;

        Ok(Value::Empty)
    }
}
