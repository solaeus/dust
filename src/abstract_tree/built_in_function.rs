use std::{
    env::current_dir,
    fs::{copy, metadata, read_dir, read_to_string, remove_file, write, File},
    io::Write,
    path::PathBuf,
    process::Command,
};

use rand::{random, thread_rng, Rng};
use reqwest::blocking::get;
use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Expression, List, Map, Result, Table, Type, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum BuiltInFunction {
    // General
    Assert(Vec<Expression>),
    AssertEqual(Vec<Expression>),
    Download(Expression),
    Context,
    Help(Option<Expression>),
    Length(Expression),
    Output(Vec<Expression>),
    OutputError(Vec<Expression>),
    Type(Expression),
    Workdir,

    // Filesystem
    Append(Vec<Expression>),
    Metadata(Expression),
    Move(Vec<Expression>),
    Read(Option<Expression>),
    Remove(Expression),
    Write(Vec<Expression>),

    // Format conversion
    FromJson(Expression),
    ToJson(Expression),
    ToString(Expression),
    ToFloat(Expression),

    // Command
    Bash(Vec<Expression>),
    Fish(Vec<Expression>),
    Raw(Vec<Expression>),
    Sh(Vec<Expression>),
    Zsh(Vec<Expression>),

    // Random
    Random(Vec<Expression>),
    RandomBoolean,
    RandomInteger,
    RandomFloat,

    // Table
    Columns(Expression),
    Rows(Expression),

    // List
    Reverse(Expression, Option<(Expression, Expression)>),
}

impl AbstractTree for BuiltInFunction {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        debug_assert_eq!("built_in_function", node.kind());

        fn parse_expressions(source: &str, node: Node) -> Result<Vec<Expression>> {
            let mut expressions = Vec::new();

            for index in 1..node.child_count() {
                let child_node = node.child(index).unwrap();

                if child_node.kind() == "expression" {
                    let expression = Expression::from_syntax_node(source, child_node)?;

                    expressions.push(expression);
                }
            }

            Ok(expressions)
        }

        let tool_node = node.child(0).unwrap();
        let tool = match tool_node.kind() {
            "assert" => {
                let expressions = parse_expressions(source, node)?;

                BuiltInFunction::Assert(expressions)
            }
            "assert_equal" => {
                let expressions = parse_expressions(source, node)?;

                BuiltInFunction::AssertEqual(expressions)
            }
            "context" => BuiltInFunction::Context,
            "download" => {
                let expression_node = node.child(1).unwrap();
                let expression = Expression::from_syntax_node(source, expression_node)?;

                BuiltInFunction::Download(expression)
            }
            "help" => {
                let child_node = node.child(1).unwrap();
                let expression = if child_node.is_named() {
                    Some(Expression::from_syntax_node(source, child_node)?)
                } else {
                    None
                };

                BuiltInFunction::Help(expression)
            }
            "length" => {
                let expression_node = node.child(1).unwrap();
                let expression = Expression::from_syntax_node(source, expression_node)?;

                BuiltInFunction::Length(expression)
            }
            "output" => {
                let expressions = parse_expressions(source, node)?;

                BuiltInFunction::Output(expressions)
            }
            "output_error" => {
                let expressions = parse_expressions(source, node)?;

                BuiltInFunction::OutputError(expressions)
            }
            "type" => {
                let expression_node = node.child(1).unwrap();
                let expression = Expression::from_syntax_node(source, expression_node)?;

                BuiltInFunction::Type(expression)
            }
            "workdir" => BuiltInFunction::Workdir,
            "append" => {
                let expressions = parse_expressions(source, node)?;

                Error::expect_tool_argument_amount("append", 2, expressions.len())?;

                BuiltInFunction::Append(expressions)
            }
            "metadata" => {
                let expression_node = node.child(1).unwrap();
                let expression = Expression::from_syntax_node(source, expression_node)?;

                BuiltInFunction::Metadata(expression)
            }
            "move" => {
                let expressions = parse_expressions(source, node)?;

                Error::expect_tool_argument_amount("move", 2, expressions.len())?;

                BuiltInFunction::Move(expressions)
            }
            "read" => {
                let expression = if let Some(node) = node.child(1) {
                    Some(Expression::from_syntax_node(source, node)?)
                } else {
                    None
                };

                BuiltInFunction::Read(expression)
            }
            "remove" => {
                let expression_node = node.child(1).unwrap();
                let expression = Expression::from_syntax_node(source, expression_node)?;

                BuiltInFunction::Remove(expression)
            }
            "write" => {
                let expressions = parse_expressions(source, node)?;

                Error::expect_tool_argument_amount("write", 2, expressions.len())?;

                BuiltInFunction::Write(expressions)
            }
            "from_json" => {
                let expression_node = node.child(1).unwrap();
                let expression = Expression::from_syntax_node(source, expression_node)?;

                BuiltInFunction::FromJson(expression)
            }
            "to_json" => {
                let expression_node = node.child(1).unwrap();
                let expression = Expression::from_syntax_node(source, expression_node)?;

                BuiltInFunction::ToJson(expression)
            }
            "to_string" => {
                let expression_node = node.child(1).unwrap();
                let expression = Expression::from_syntax_node(source, expression_node)?;

                BuiltInFunction::ToString(expression)
            }
            "to_float" => {
                let expression_node = node.child(1).unwrap();
                let expression = Expression::from_syntax_node(source, expression_node)?;

                BuiltInFunction::ToFloat(expression)
            }
            "bash" => {
                let expressions = parse_expressions(source, node)?;

                BuiltInFunction::Bash(expressions)
            }
            "fish" => {
                let expressions = parse_expressions(source, node)?;

                BuiltInFunction::Fish(expressions)
            }
            "raw" => {
                let expressions = parse_expressions(source, node)?;

                BuiltInFunction::Raw(expressions)
            }
            "sh" => {
                let expressions = parse_expressions(source, node)?;

                BuiltInFunction::Sh(expressions)
            }
            "zsh" => {
                let expressions = parse_expressions(source, node)?;

                BuiltInFunction::Zsh(expressions)
            }
            "random" => {
                let expressions = parse_expressions(source, node)?;

                BuiltInFunction::Random(expressions)
            }
            "random_boolean" => BuiltInFunction::RandomBoolean,
            "random_float" => BuiltInFunction::RandomFloat,
            "random_integer" => BuiltInFunction::RandomInteger,
            "columns" => {
                let expression_node = node.child(2).unwrap();
                let expression = Expression::from_syntax_node(source, expression_node)?;

                BuiltInFunction::Columns(expression)
            }
            "rows" => {
                let expression_node = node.child(2).unwrap();
                let expression = Expression::from_syntax_node(source, expression_node)?;

                BuiltInFunction::Rows(expression)
            }
            "reverse" => {
                let list_node = node.child(2).unwrap();
                let list_expression = Expression::from_syntax_node(source, list_node)?;

                let slice_range_nodes =
                    if let (Some(start_node), Some(end_node)) = (node.child(3), node.child(4)) {
                        let start = Expression::from_syntax_node(source, start_node)?;
                        let end = Expression::from_syntax_node(source, end_node)?;

                        Some((start, end))
                    } else {
                        None
                    };

                BuiltInFunction::Reverse(list_expression, slice_range_nodes)
            }
            _ => {
                return Err(Error::UnexpectedSyntaxNode {
                    expected: "built-in function",
                    actual: tool_node.kind(),
                    location: tool_node.start_position(),
                    relevant_source: source[tool_node.byte_range()].to_string(),
                })
            }
        };

        Ok(tool)
    }

    fn run(&self, source: &str, context: &mut Map) -> Result<Value> {
        match self {
            BuiltInFunction::Assert(expressions) => {
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
            BuiltInFunction::AssertEqual(expressions) => {
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
            BuiltInFunction::Context => Ok(Value::Map(context.clone())),
            BuiltInFunction::Download(expression) => {
                let value = expression.run(source, context)?;
                let url = value.as_string()?;
                let data = get(url)?.text()?;

                Ok(Value::String(data))
            }
            BuiltInFunction::Length(expression) => {
                let value = expression.run(source, context)?;
                let length = match value {
                    Value::List(list) => list.items().len(),
                    Value::Map(map) => map.variables()?.len(),
                    Value::Table(table) => table.len(),
                    Value::String(string) => string.chars().count(),
                    Value::Function(_) => todo!(),
                    Value::Float(_) => todo!(),
                    Value::Integer(_) => todo!(),
                    Value::Boolean(_) => todo!(),
                    Value::Empty => todo!(),
                };

                Ok(Value::Integer(length as i64))
            }
            BuiltInFunction::Help(_expression) => {
                let mut help_table =
                    Table::new(vec!["tool".to_string(), "description".to_string()]);

                help_table.insert(vec![
                    Value::String("help".to_string()),
                    Value::String("Get info on tools.".to_string()),
                ])?;

                Ok(Value::Table(help_table))
            }
            BuiltInFunction::Output(expressions) => {
                for expression in expressions {
                    let value = expression.run(source, context)?;

                    println!("{value}");
                }

                Ok(Value::Empty)
            }
            BuiltInFunction::OutputError(expressions) => {
                for expression in expressions {
                    let value = expression.run(source, context)?;

                    eprintln!("{value}");
                }

                Ok(Value::Empty)
            }
            BuiltInFunction::Type(expression) => {
                let run_expression = expression.run(source, context);
                let value_type = if let Ok(value) = run_expression {
                    value.r#type()
                } else if let Err(Error::VariableIdentifierNotFound(_)) = run_expression {
                    Type::Any
                } else {
                    return run_expression;
                };

                Ok(Value::String(value_type.to_string()))
            }
            BuiltInFunction::Workdir => {
                let workdir = current_dir()?.to_string_lossy().to_string();

                Ok(Value::String(workdir))
            }
            BuiltInFunction::Append(expressions) => {
                let path_value = expressions[0].run(source, context)?;
                let path = path_value.as_string()?;
                let data = expressions[1].run(source, context)?.to_string();
                let mut file = File::options().append(true).open(path)?;

                file.write(data.as_bytes())?;

                Ok(Value::Empty)
            }
            BuiltInFunction::Metadata(expression) => {
                let path_value = expression.run(source, context)?;
                let path = path_value.as_string()?;
                let metadata = metadata(path)?;
                let file_type = if metadata.is_dir() {
                    "dir".to_string()
                } else if metadata.is_file() {
                    "file".to_string()
                } else if metadata.is_symlink() {
                    "link".to_string()
                } else {
                    "unknown".to_string()
                };
                let size = metadata.len() as i64;
                let created = metadata.created()?.elapsed()?.as_secs() as i64;
                let modified = metadata.modified()?.elapsed()?.as_secs() as i64;
                let accessed = metadata.accessed()?.elapsed()?.as_secs() as i64;
                let metadata_output = Map::new();

                {
                    let mut metadata_variables = metadata_output.variables_mut()?;

                    metadata_variables.insert("type".to_string(), Value::String(file_type));
                    metadata_variables.insert("size".to_string(), Value::Integer(size));
                    metadata_variables.insert("created".to_string(), Value::Integer(created));
                    metadata_variables.insert("modified".to_string(), Value::Integer(modified));
                    metadata_variables.insert("accessed".to_string(), Value::Integer(accessed));
                }

                Ok(Value::Map(metadata_output))
            }
            BuiltInFunction::Move(expressions) => {
                let from_value = expressions[0].run(source, context)?;
                let from = from_value.as_string()?;
                let to_value = expressions[1].run(source, context)?;
                let to = to_value.as_string()?;

                copy(from, to)?;
                remove_file(from)?;

                Ok(Value::Empty)
            }
            BuiltInFunction::Read(expression) => {
                let path = if let Some(expression) = expression {
                    let path_value = expression.run(source, context)?;

                    PathBuf::from(path_value.as_string()?)
                } else {
                    PathBuf::from(".")
                };
                let content = if path.is_dir() {
                    let dir = read_dir(&path)?;
                    let mut contents = Vec::new();

                    for file in dir {
                        let file = file?;
                        let file_path = file.path().to_string_lossy().to_string();

                        contents.push(Value::String(file_path));
                    }

                    Value::List(List::with_items(contents))
                } else {
                    Value::String(read_to_string(path)?)
                };

                Ok(content)
            }
            BuiltInFunction::Remove(expression) => {
                let path_value = expression.run(source, context)?;
                let path = PathBuf::from(path_value.as_string()?);

                remove_file(path)?;

                Ok(Value::Empty)
            }
            BuiltInFunction::Write(expressions) => {
                let path_value = expressions[0].run(source, context)?;
                let path = path_value.as_string()?;
                let data_value = expressions[1].run(source, context)?;
                let data = data_value.as_string()?;

                write(path, data)?;

                Ok(Value::Empty)
            }
            BuiltInFunction::FromJson(expression) => {
                let json_value = expression.run(source, context)?;
                let json = json_value.as_string()?;
                let value = serde_json::from_str(json)?;

                Ok(value)
            }
            BuiltInFunction::ToJson(expression) => {
                let value = expression.run(source, context)?;
                let json = serde_json::to_string(&value)?;

                Ok(Value::String(json))
            }
            BuiltInFunction::ToString(expression) => {
                let value = expression.run(source, context)?;
                let string = value.to_string();

                Ok(Value::String(string))
            }
            BuiltInFunction::ToFloat(expression) => {
                let value = expression.run(source, context)?;
                let float = match value {
                    Value::String(string) => string.parse()?,
                    Value::Float(float) => float,
                    Value::Integer(integer) => integer as f64,
                    _ => return Err(Error::ExpectedNumberOrString { actual: value }),
                };

                Ok(Value::Float(float))
            }
            BuiltInFunction::Bash(expressions) => {
                let mut command = Command::new("bash");

                for expression in expressions {
                    let value = expression.run(source, context)?;
                    let command_input = value.as_string()?;

                    command.arg(command_input);
                }

                let output = command.spawn()?.wait_with_output()?.stdout;

                Ok(Value::String(String::from_utf8(output)?))
            }
            BuiltInFunction::Fish(expressions) => {
                let mut command = Command::new("fish");

                for expression in expressions {
                    let value = expression.run(source, context)?;
                    let command_input = value.as_string()?;

                    command.arg(command_input);
                }

                let output = command.spawn()?.wait_with_output()?.stdout;

                Ok(Value::String(String::from_utf8(output)?))
            }
            BuiltInFunction::Raw(expressions) => {
                let raw_command = expressions[0].run(source, context)?;
                let mut command = Command::new(raw_command.as_string()?);

                for expression in &expressions[1..] {
                    let value = expression.run(source, context)?;
                    let command_input = value.as_string()?;

                    command.arg(command_input);
                }

                let output = command.spawn()?.wait_with_output()?.stdout;

                Ok(Value::String(String::from_utf8(output)?))
            }
            BuiltInFunction::Sh(expressions) => {
                let mut command = Command::new("sh");

                for expression in expressions {
                    let value = expression.run(source, context)?;
                    let command_input = value.as_string()?;

                    command.arg(command_input);
                }

                let output = command.spawn()?.wait_with_output()?.stdout;

                Ok(Value::String(String::from_utf8(output)?))
            }
            BuiltInFunction::Zsh(expressions) => {
                let mut command = Command::new("zsh");

                for expression in expressions {
                    let value = expression.run(source, context)?;
                    let command_input = value.as_string()?;

                    command.arg(command_input);
                }

                let output = command.spawn()?.wait_with_output()?.stdout;

                Ok(Value::String(String::from_utf8(output)?))
            }
            BuiltInFunction::Random(expressions) => {
                if expressions.len() == 1 {
                    let value = expressions[0].run(source, context)?;
                    let list = value.as_list()?.items();

                    if list.len() == 1 {
                        return Ok(list.first().cloned().unwrap());
                    }

                    let range = 0..list.len();
                    let random_index = thread_rng().gen_range(range);
                    let random_value = list.get(random_index).ok_or(Error::ExpectedList {
                        actual: value.clone(),
                    })?;

                    return Ok(random_value.clone());
                }

                let range = 0..expressions.len();
                let random_index = thread_rng().gen_range(range);
                let random_expression = expressions.get(random_index).unwrap();
                let value = random_expression.run(source, context)?;

                Ok(value)
            }
            BuiltInFunction::RandomBoolean => Ok(Value::Boolean(random())),
            BuiltInFunction::RandomFloat => Ok(Value::Float(random())),
            BuiltInFunction::RandomInteger => Ok(Value::Integer(random())),
            BuiltInFunction::Columns(expression) => {
                let column_names = expression
                    .run(source, context)?
                    .as_table()?
                    .headers()
                    .iter()
                    .cloned()
                    .map(Value::String)
                    .collect();

                Ok(Value::List(List::with_items(column_names)))
            }
            BuiltInFunction::Rows(expression) => {
                let rows = expression
                    .run(source, context)?
                    .as_table()?
                    .rows()
                    .iter()
                    .cloned()
                    .map(|row| Value::List(List::with_items(row)))
                    .collect();

                Ok(Value::List(List::with_items(rows)))
            }
            BuiltInFunction::Reverse(list_expression, slice_range) => {
                let expression_run = list_expression.run(source, context)?;
                let list = expression_run.as_list()?;

                if let Some((start, end)) = slice_range {
                    let start = start.run(source, context)?.as_integer()? as usize;
                    let end = end.run(source, context)?.as_integer()? as usize;

                    list.items_mut()[start..end].reverse()
                } else {
                    list.items_mut().reverse()
                };

                Ok(Value::List(list.clone()))
            }
        }
    }
}
