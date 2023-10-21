use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Expression, Result, Value, VariableMap};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum Tool {
    Output(Vec<Expression>),
    OutputError(Vec<Expression>),

    Assert(Vec<Expression>),
    AssertEqual(Vec<Expression>),

    Length(Expression),
}

impl AbstractTree for Tool {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        debug_assert_eq!("tool", node.kind());

        fn parse_expressions(source: &str, node: Node) -> Result<Vec<Expression>> {
            let mut expressions = Vec::new();

            for index in 2..node.child_count() - 1 {
                let expression_node = node.child(index).unwrap();
                let expression = Expression::from_syntax_node(source, expression_node)?;

                expressions.push(expression);
            }

            Ok(expressions)
        }

        let tool_node = node.child(1).unwrap();
        let tool = match tool_node.kind() {
            "output" => {
                let expressions = parse_expressions(source, node)?;

                Tool::Output(expressions)
            }
            "output_error" => {
                let expressions = parse_expressions(source, node)?;

                Tool::OutputError(expressions)
            }
            "assert" => {
                let expressions = parse_expressions(source, node)?;

                Tool::Assert(expressions)
            }
            "assert_equal" => {
                let expressions = parse_expressions(source, node)?;

                Tool::AssertEqual(expressions)
            }
            "length" => {
                let expression_node = node.child(2).unwrap();
                let expression = Expression::from_syntax_node(source, expression_node)?;

                Tool::Length(expression)
            }
            _ => {
                return Err(Error::UnexpectedSyntaxNode {
                    expected: "output, output_error, assert, assert_equal or length",
                    actual: tool_node.kind(),
                    location: tool_node.start_position(),
                    relevant_source: source[tool_node.byte_range()].to_string(),
                })
            }
        };

        Ok(tool)
    }

    fn run(&self, source: &str, context: &mut VariableMap) -> Result<Value> {
        match self {
            Tool::Output(expressions) => {
                for expression in expressions {
                    let value = expression.run(source, context)?;

                    println!("{value}");
                }

                Ok(Value::Empty)
            }
            Tool::OutputError(expressions) => {
                for expression in expressions {
                    let value = expression.run(source, context)?;

                    eprintln!("{value}");
                }

                Ok(Value::Empty)
            }
            Tool::Assert(expressions) => {
                for expression in expressions {
                    let value = expression.run(source, context)?;

                    if value.as_boolean()? {
                        continue;
                    } else {
                        return Err(Error::AssertFailed);
                    }
                }

                Ok(Value::Empty)
            }
            Tool::AssertEqual(expressions) => {
                let mut prev_value = None;
                for expression in expressions {
                    let value = expression.run(source, context)?;

                    if let Some(prev_value) = &prev_value {
                        if &value == prev_value {
                            continue;
                        } else {
                            return Err(Error::AssertEqualFailed {
                                expected: prev_value.clone(),
                                actual: value,
                            });
                        }
                    }

                    prev_value = Some(value);
                }

                Ok(Value::Empty)
            }
            Tool::Length(expression) => {
                let length = expression.run(source, context)?.as_list()?.len();

                Ok(Value::Integer(length as i64))
            }
        }
    }
}
