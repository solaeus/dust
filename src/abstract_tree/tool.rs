use std::fs::read_to_string;

use serde::{Deserialize, Serialize};

use crate::{Error, Result, Table, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum Tool {
    Assert,
    AssertEqual,
    Output,
    Read,
    Help,
}

impl Tool {
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
        };

        Ok(value)
    }
}
