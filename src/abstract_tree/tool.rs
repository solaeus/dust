use std::fs::read_to_string;

use rand::{random, thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::{Error, Result, Table, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum Tool {
    Assert,
    AssertEqual,
    Output,
    Read,
    Help,

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
                if values.len() != 1 {
                    return Err(Error::ExpectedToolArgumentAmount {
                        tool_name: "assert",
                        expected: 1,
                        actual: values.len(),
                    });
                }

                if values[0].as_boolean()? {
                    Value::Empty
                } else {
                    return Err(Error::AssertFailed);
                }
            }
            Tool::AssertEqual => {
                if values.len() != 2 {
                    return Err(Error::ExpectedToolArgumentAmount {
                        tool_name: "assert_equal",
                        expected: 2,
                        actual: values.len(),
                    });
                }

                if values[0] == values[1] {
                    Value::Empty
                } else {
                    return Err(Error::AssertEqualFailed {
                        expected: values[0].clone(),
                        actual: values[1].clone(),
                    });
                }
            }
            Tool::Output => {
                if values.len() != 1 {
                    return Err(Error::ExpectedToolArgumentAmount {
                        tool_name: "output",
                        expected: 1,
                        actual: values.len(),
                    });
                }

                println!("{}", values[0]);

                Value::Empty
            }
            Tool::Read => {
                if values.len() != 1 {
                    return Err(Error::ExpectedToolArgumentAmount {
                        tool_name: "read",
                        expected: 1,
                        actual: values.len(),
                    });
                }

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
                    let range = values[0].as_int()?..values[1].as_int()?;
                    let mut rng = thread_rng();
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
                if values.len() > 1 {
                    return Err(Error::ExpectedToolArgumentAmount {
                        tool_name: "help",
                        expected: 1,
                        actual: values.len(),
                    });
                }

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

                Value::Table(help_table)
            }
            Tool::Raw => todo!(),
            Tool::Sh => todo!(),
            Tool::Bash => todo!(),
            Tool::Fish => todo!(),
            Tool::Zsh => todo!(),
            Tool::FromCsv => todo!(),
            Tool::ToCsv => todo!(),
            Tool::FromJson => {
                Error::expect_tool_argument_amount("from_json", 1, values.len())?;

                let json_string = values[0].as_string()?;

                serde_json::from_str(json_string)?
            }
            Tool::ToJson => todo!(),
        };

        Ok(value)
    }
}
