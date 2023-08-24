use std::{fs, thread::sleep, time::Duration};

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::{Function, Result, Tool, ToolInfo, Value};

pub struct Output;

impl Tool for Output {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "output",
            description: "Print a value.",
            group: "general",
            inputs: vec![],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        println!("{argument}");

        Ok(Value::Empty)
    }
}
pub struct Repeat;

impl Tool for Repeat {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "repeat",
            description: "Run a function the given number of times.",
            group: "general",
            inputs: vec![],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let argument = argument.as_list()?;
        let function = argument[0].as_function()?;
        let count = argument[1].as_int()?;
        let mut result_list = Vec::with_capacity(count as usize);

        for _ in 0..count {
            let result = function.run()?;

            result_list.push(result);
        }

        Ok(Value::List(result_list))
    }
}

pub struct Run;

impl Tool for Run {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "run",
            description: "Run a whale file.",
            group: "general",
            inputs: vec![],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let path = argument.as_string()?;
        let file_contents = fs::read_to_string(path)?;

        Function::new(&file_contents).run()
    }
}

pub struct Async;

impl Tool for Async {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "async",
            description: "Run functions in parallel.",
            group: "general",
            inputs: vec![],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let argument_list = argument.as_list()?;
        let results = argument_list
            .par_iter()
            .map(|value| {
                let function = if let Ok(function) = value.as_function() {
                    function
                } else {
                    return value.clone();
                };

                match function.run() {
                    Ok(value) => value,
                    Err(error) => Value::String(error.to_string()),
                }
            })
            .collect();

        Ok(Value::List(results))
    }
}

pub struct Wait;

impl Tool for Wait {
    fn info(&self) -> crate::ToolInfo<'static> {
        ToolInfo {
            identifier: "wait",
            description: "Wait for the given number of milliseconds.",
            group: "general",
            inputs: vec![],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let argument = argument.as_int()?;

        sleep(Duration::from_millis(argument as u64));

        Ok(Value::Empty)
    }
}
