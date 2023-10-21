use std::{
    fs::{metadata, File},
    io::Write,
};

use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Expression, Result, Table, Value, VariableMap};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum Tool {
    // General
    Assert(Vec<Expression>),
    AssertEqual(Vec<Expression>),
    Help(Option<Expression>),
    Length(Expression),
    Output(Vec<Expression>),
    OutputError(Vec<Expression>),

    // Filesystem
    Append(Vec<Expression>),
    Metadata(Expression),
    Move(Vec<Expression>),
    Read(Expression),
    Remove(Expression),
    Trash(Expression),
    Write(Vec<Expression>),

    // Format conversion
    FromJson(Expression),
    ToJson(Expression),
    ToString(Expression),

    // Command
    Bash(Expression),
    Fish(Expression),
    Raw(Expression),
    Sh(Expression),
    Zsh(Expression),
}

impl AbstractTree for Tool {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        debug_assert_eq!("tool", node.kind());

        fn parse_expressions(source: &str, node: Node) -> Result<Vec<Expression>> {
            let mut expressions = Vec::new();

            for index in 2..node.child_count() - 1 {
                let child_node = node.child(index).unwrap();

                if child_node.is_named() {
                    let expression = Expression::from_syntax_node(source, child_node)?;

                    expressions.push(expression);
                }
            }

            Ok(expressions)
        }

        let tool_node = node.child(1).unwrap();
        let tool = match tool_node.kind() {
            "assert" => {
                let expressions = parse_expressions(source, node)?;

                Tool::Assert(expressions)
            }
            "assert_equal" => {
                let expressions = parse_expressions(source, node)?;

                Tool::AssertEqual(expressions)
            }
            "help" => {
                let child_node = node.child(2).unwrap();
                let expression = if child_node.is_named() {
                    Some(Expression::from_syntax_node(source, child_node)?)
                } else {
                    None
                };

                Tool::Help(expression)
            }
            "length" => {
                let expression_node = node.child(2).unwrap();
                let expression = Expression::from_syntax_node(source, expression_node)?;

                Tool::Length(expression)
            }
            "output" => {
                let expressions = parse_expressions(source, node)?;

                Tool::Output(expressions)
            }
            "output_error" => {
                let expressions = parse_expressions(source, node)?;

                Tool::OutputError(expressions)
            }
            "append" => {
                let expressions = parse_expressions(source, node)?;

                Error::expect_tool_argument_amount("append", 2, expressions.len())?;

                Tool::Append(expressions)
            }
            "metadata" => {
                let expression_node = node.child(2).unwrap();
                let expression = Expression::from_syntax_node(source, expression_node)?;

                Tool::Metadata(expression)
            }
            "move" => {
                let expressions = parse_expressions(source, node)?;

                Error::expect_tool_argument_amount("move", 2, expressions.len())?;

                Tool::Move(expressions)
            }
            "read" => {
                let expression_node = node.child(2).unwrap();
                let expression = Expression::from_syntax_node(source, expression_node)?;

                Tool::Read(expression)
            }
            "remove" => {
                let expression_node = node.child(2).unwrap();
                let expression = Expression::from_syntax_node(source, expression_node)?;

                Tool::Remove(expression)
            }
            "trash" => {
                let expression_node = node.child(2).unwrap();
                let expression = Expression::from_syntax_node(source, expression_node)?;

                Tool::Trash(expression)
            }
            "write" => {
                let expressions = parse_expressions(source, node)?;

                Error::expect_tool_argument_amount("write", 2, expressions.len())?;

                Tool::Write(expressions)
            }
            "from_json" => {
                let expression_node = node.child(2).unwrap();
                let expression = Expression::from_syntax_node(source, expression_node)?;

                Tool::FromJson(expression)
            }
            "to_json" => {
                let expression_node = node.child(2).unwrap();
                let expression = Expression::from_syntax_node(source, expression_node)?;

                Tool::ToJson(expression)
            }
            "to_string" => {
                let expression_node = node.child(2).unwrap();
                let expression = Expression::from_syntax_node(source, expression_node)?;

                Tool::ToString(expression)
            }
            "bash" => {
                let expression_node = node.child(2).unwrap();
                let expression = Expression::from_syntax_node(source, expression_node)?;

                Tool::Bash(expression)
            }
            "fish" => {
                let expression_node = node.child(2).unwrap();
                let expression = Expression::from_syntax_node(source, expression_node)?;

                Tool::Fish(expression)
            }
            "raw" => {
                let expression_node = node.child(2).unwrap();
                let expression = Expression::from_syntax_node(source, expression_node)?;

                Tool::Raw(expression)
            }
            "sh" => {
                let expression_node = node.child(2).unwrap();
                let expression = Expression::from_syntax_node(source, expression_node)?;

                Tool::Sh(expression)
            }
            "zsh" => {
                let expression_node = node.child(2).unwrap();
                let expression = Expression::from_syntax_node(source, expression_node)?;

                Tool::Zsh(expression)
            }
            _ => {
                return Err(Error::UnexpectedSyntaxNode {
                    expected: "built-in tool",
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
            Tool::Help(_expression) => {
                let mut help_table =
                    Table::new(vec!["tool".to_string(), "description".to_string()]);

                help_table.insert(vec![
                    Value::String("help".to_string()),
                    Value::String("Get info on tools.".to_string()),
                ])?;

                Ok(Value::Table(help_table))
            }
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
            Tool::Append(expressions) => {
                let path_expression = expressions[0].run(source, context)?;
                let path = path_expression.as_string()?;
                let data = expressions[1].run(source, context)?.to_string();
                let mut file = File::options().append(true).open(path)?;

                file.write(data.as_bytes())?;

                Ok(Value::Empty)
            }
            Tool::Metadata(expression) => {
                let path_value = expression.run(source, context)?;
                let path = path_value.as_string()?;
                let metadata = metadata(path)?;
                let mut metadata_output = VariableMap::new();

                let file_type = if metadata.is_dir() {
                    "dir".to_string()
                } else if metadata.is_file() {
                    "file".to_string()
                } else if metadata.is_symlink() {
                    "link".to_string()
                } else {
                    "unknown".to_string()
                };
                let created = metadata.created()?.elapsed()?.as_secs() as i64;
                let modified = metadata.modified()?.elapsed()?.as_secs() as i64;
                let accessed = metadata.accessed()?.elapsed()?.as_secs() as i64;

                metadata_output.set_value("type".to_string(), Value::String(file_type))?;
                metadata_output.set_value("created".to_string(), Value::Integer(created))?;
                metadata_output.set_value("modified".to_string(), Value::Integer(modified))?;
                metadata_output.set_value("accessed".to_string(), Value::Integer(accessed))?;

                Ok(Value::Map(metadata_output))
            }
            Tool::Move(_) => todo!(),
            Tool::Read(_) => todo!(),
            Tool::Remove(_) => todo!(),
            Tool::Trash(_) => todo!(),
            Tool::Write(_) => todo!(),
            Tool::FromJson(_) => todo!(),
            Tool::ToJson(_) => todo!(),
            Tool::ToString(_) => todo!(),
            Tool::Bash(_) => todo!(),
            Tool::Fish(_) => todo!(),
            Tool::Raw(_) => todo!(),
            Tool::Sh(_) => todo!(),
            Tool::Zsh(_) => todo!(),
        }
    }
}
