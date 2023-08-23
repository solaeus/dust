use std::process::Command;

use crate::{Macro, MacroInfo, Result, Value};

pub struct Sh;

impl Macro for Sh {
    fn info(&self) -> MacroInfo<'static> {
        MacroInfo {
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

impl Macro for Bash {
    fn info(&self) -> MacroInfo<'static> {
        MacroInfo {
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

impl Macro for Fish {
    fn info(&self) -> MacroInfo<'static> {
        MacroInfo {
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

impl Macro for Zsh {
    fn info(&self) -> MacroInfo<'static> {
        MacroInfo {
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

impl Macro for Raw {
    fn info(&self) -> MacroInfo<'static> {
        MacroInfo {
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
