use std::fs::read_to_string;

use serde::{Deserialize, Serialize};

use crate::{Error, Result, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum Tool {
    Assert,
    AssertEqual,
    Output,
    Read,
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
                    return Err(Error::AssertEqualFailed {
                        expected: Value::Boolean(true),
                        actual: values[0].clone(),
                    });
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
        };

        Ok(value)
    }
}
