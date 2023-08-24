use std::{thread::sleep, time::Duration};

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::{Result, Tool, ToolInfo, Value, ValueType};

pub struct Output;

impl Tool for Output {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "output",
            description: "Print a value.",
            group: "general",
            inputs: vec![ValueType::Any],
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
            inputs: vec![ValueType::ListExact(vec![
                ValueType::Function,
                ValueType::Integer,
            ])],
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
            description: "Run functions in parallel.",
            group: "general",
            inputs: vec![ValueType::ListOf(Box::new(ValueType::Function))],
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
            inputs: vec![ValueType::Integer],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let argument = argument.as_int()?;

        sleep(Duration::from_millis(argument as u64));

        Ok(Value::Empty)
    }
}
