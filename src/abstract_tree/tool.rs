use std::{fs::read_to_string, process::Command};

use rand::{random, thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::{evaluate, Error, Result, Table, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum Tool {
    Assert,
    AssertEqual,
    Output,
    Run,
    Read,
    Help,

    Length,

    Raw,
    Sh,
    Bash,
    Fish,
    Zsh,

    FromCsv,
    ToCsv,
    FromJson,
    ToJson,

    Random,
    RandomFloat,
    RandomInteger,
    RandomString,
}

impl Tool {
    pub fn new(kind: &str) -> Result<Self> {
        let tool = match kind {
            "assert" => Tool::Assert,
            "assert_equal" => Tool::AssertEqual,
            "output" => Tool::Output,

            "length" => Tool::Length,

            "raw" => Tool::Raw,
            "sh" => Tool::Sh,
            "bash" => Tool::Bash,
            "fish" => Tool::Fish,
            "zsh" => Tool::Zsh,

            "from_csv" => Tool::FromCsv,
            "to_csv" => Tool::ToCsv,
            "from_json" => Tool::FromJson,
            "to_json" => Tool::ToJson,

            "random" => Tool::Random,
            "random_integer" => Tool::RandomInteger,
            "random_float" => Tool::RandomFloat,
            "random_string" => Tool::RandomString,

            "read" => Tool::Read,
            "help" => Tool::Help,
            _ => todo!("Tool name not recognized."),
        };

        Ok(tool)
    }

    pub fn run(&self, values: &[Value]) -> Result<Value> {
        let value = match self {
            Tool::Assert => {
                Error::expect_tool_argument_amount("assert", 1, values.len())?;

                if values[0].as_boolean()? {
                    Value::Empty
                } else {
                    return Err(Error::AssertFailed);
                }
            }
            Tool::AssertEqual => {
                Error::expect_tool_argument_amount("assert_equal", 2, values.len())?;

                if values[0] == values[1] {
                    Value::Empty
                } else {
                    return Err(Error::AssertEqualFailed {
                        expected: values[0].clone(),
                        actual: values[1].clone(),
                    });
                }
            }
            Tool::Run => {
                Error::expect_tool_argument_amount("run", 1, values.len())?;

                let file_path = values[0].as_string()?;
                let file_contents = read_to_string(file_path)?;

                evaluate(&file_contents)?
            }
            Tool::Output => {
                Error::expect_tool_argument_amount("output", 1, values.len())?;

                println!("{}", values[0]);

                Value::Empty
            }
            Tool::Length => {
                Error::expect_tool_argument_amount("length", 1, values.len())?;

                let length = if let Ok(list) = values[0].as_list() {
                    list.len()
                } else if let Ok(map) = values[0].as_map() {
                    map.len()
                } else if let Ok(table) = values[0].as_table() {
                    table.len()
                } else if let Ok(string) = values[0].as_string() {
                    string.chars().count()
                } else {
                    1
                };

                Value::Integer(length as i64)
            }
            Tool::Read => {
                Error::expect_tool_argument_amount("read", 1, values.len())?;

                let file_contents = read_to_string(values[0].as_string()?)?;

                Value::String(file_contents)
            }
            Tool::Random => todo!(),
            Tool::RandomFloat => todo!(),
            Tool::RandomString => todo!(),
            Tool::RandomInteger => {
                if values.len() == 0 {
                    Value::Integer(random())
                } else if values.len() == 2 {
                    let mut rng = thread_rng();
                    let range = values[0].as_int()?..=values[1].as_int()?;
                    let random = rng.gen_range(range);

                    Value::Integer(random)
                } else {
                    return Err(Error::ExpectedToolArgumentAmount {
                        tool_name: "random_integer",
                        expected: 2,
                        actual: values.len(),
                    });
                }
            }
            Tool::Help => {
                Error::expect_tool_argument_amount("help", 0, values.len())?;

                let mut help_table =
                    Table::new(vec!["name".to_string(), "description".to_string()]);

                help_table.insert(vec![
                    Value::String("help".to_string()),
                    Value::String("List available tools.".to_string()),
                ])?;
                help_table.insert(vec![
                    Value::String("assert".to_string()),
                    Value::String("Panic if an expression is false.".to_string()),
                ])?;
                help_table.insert(vec![
                    Value::String("assert_equal".to_string()),
                    Value::String("Panic if two values are not equal.".to_string()),
                ])?;
                help_table.insert(vec![
                    Value::String("output".to_string()),
                    Value::String("Emit a value to stdout.".to_string()),
                ])?;
                help_table.insert(vec![
                    Value::String("read".to_string()),
                    Value::String("Get a file's content.".to_string()),
                ])?;
                help_table.insert(vec![
                    Value::String("from_json".to_string()),
                    Value::String("Convert a JSON string to a value.".to_string()),
                ])?;
                help_table.insert(vec![
                    Value::String("to_json".to_string()),
                    Value::String("Convert a value to a JSON string.".to_string()),
                ])?;

                Value::Table(help_table)
            }
            Tool::Raw => {
                let program = values[0].as_string()?;
                let mut command = Command::new(program);

                for value in &values[1..] {
                    let arg = value.as_string()?;

                    command.arg(arg);
                }

                command.spawn()?.wait()?;

                Value::Empty
            }
            Tool::Sh => {
                let mut command = Command::new("sh");

                for value in values {
                    let arg = value.as_string()?;

                    command.arg(arg);
                }

                command.spawn()?.wait()?;

                Value::Empty
            }
            Tool::Bash => {
                let mut command = Command::new("bash");

                for value in values {
                    let arg = value.as_string()?;

                    command.arg(arg);
                }

                command.spawn()?.wait()?;

                Value::Empty
            }
            Tool::Fish => {
                let mut command = Command::new("fish");

                for value in values {
                    let arg = value.as_string()?;

                    command.arg(arg);
                }

                command.spawn()?.wait()?;

                Value::Empty
            }
            Tool::Zsh => {
                let mut command = Command::new("zsh");

                for value in values {
                    let arg = value.as_string()?;

                    command.arg(arg);
                }

                command.spawn()?.wait()?;

                Value::Empty
            }
            Tool::FromCsv => todo!(),
            Tool::ToCsv => todo!(),
            Tool::FromJson => {
                Error::expect_tool_argument_amount("from_json", 1, values.len())?;

                let json_string = values[0].as_string()?;

                serde_json::from_str(json_string)?
            }
            Tool::ToJson => {
                Error::expect_tool_argument_amount("to_json", 1, values.len())?;

                let value = &values[0];
                let json_string = serde_json::to_string(value)?;

                Value::String(json_string)
            }
        };

        Ok(value)
    }
}
