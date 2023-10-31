use std::{collections::BTreeMap, ops::Range};

use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{
    AbstractTree, Block, Error, Expression, Function, Identifier, List, Map, Result, Statement,
    Table, Value, ValueType,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct ValueNode {
    value_type: ValueType,
    start_byte: usize,
    end_byte: usize,
}

impl ValueNode {
    pub fn new(value_type: ValueType, start_byte: usize, end_byte: usize) -> Self {
        Self {
            value_type,
            start_byte,
            end_byte,
        }
    }

    pub fn byte_range(&self) -> Range<usize> {
        self.start_byte..self.end_byte
    }
}

impl AbstractTree for ValueNode {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        debug_assert_eq!("value", node.kind());

        let child = node.child(0).unwrap();
        let value_type = match child.kind() {
            "integer" => ValueType::Integer,
            "float" => ValueType::Float,
            "string" => ValueType::String,
            "boolean" => ValueType::Boolean,
            "empty" => ValueType::Empty,
            "list" => {
                let mut expressions = Vec::new();

                for index in 1..child.child_count() - 1 {
                    let current_node = child.child(index).unwrap();

                    if current_node.is_named() {
                        let expression = Expression::from_syntax_node(source, current_node)?;
                        expressions.push(expression);
                    }
                }

                ValueType::List(expressions)
            }
            "table" => {
                let identifier_list_node = child.child(1).unwrap();
                let identifier_count = identifier_list_node.child_count();
                let mut column_names = Vec::with_capacity(identifier_count);

                for index in 0..identifier_count {
                    let identifier_node = identifier_list_node.child(index).unwrap();

                    if identifier_node.is_named() {
                        let identifier = Identifier::from_syntax_node(source, identifier_node)?;

                        column_names.push(identifier)
                    }
                }

                let expression_node = child.child(2).unwrap();
                let expression = Expression::from_syntax_node(source, expression_node)?;

                ValueType::Table {
                    column_names,
                    rows: Box::new(expression),
                }
            }
            "map" => {
                let mut child_nodes = BTreeMap::new();
                let mut current_key = "".to_string();

                for index in 0..child.child_count() - 1 {
                    let child_syntax_node = child.child(index).unwrap();

                    if child_syntax_node.kind() == "identifier" {
                        current_key =
                            Identifier::from_syntax_node(source, child_syntax_node)?.take_inner();
                    }

                    if child_syntax_node.kind() == "statement" {
                        let key = current_key.clone();
                        let statement = Statement::from_syntax_node(source, child_syntax_node)?;

                        child_nodes.insert(key, statement);
                    }
                }

                ValueType::Map(child_nodes)
            }
            "function" => {
                let parameters_node = child.child_by_field_name("parameters");
                let parameters = if let Some(node) = parameters_node {
                    let mut parameter_list = Vec::new();

                    for index in 0..node.child_count() {
                        let child_node = node.child(index).unwrap();

                        if child_node.is_named() {
                            let parameter = Identifier::from_syntax_node(source, child_node)?;

                            parameter_list.push(parameter);
                        }
                    }

                    Some(parameter_list)
                } else {
                    None
                };
                let body_node = child.child_by_field_name("body").unwrap();
                let body = Block::from_syntax_node(source, body_node)?;

                ValueType::Function(Function::new(parameters, body))
            }
            _ => {
                return Err(Error::UnexpectedSyntaxNode {
                    expected:
                        "string, integer, float, boolean, list, table, map, function or empty",
                    actual: child.kind(),
                    location: child.start_position(),
                    relevant_source: source[child.byte_range()].to_string(),
                })
            }
        };

        Ok(ValueNode {
            value_type,
            start_byte: child.start_byte(),
            end_byte: child.end_byte(),
        })
    }

    fn run(&self, source: &str, context: &mut Map) -> Result<Value> {
        let value_source = &source[self.byte_range()];
        let value = match &self.value_type {
            ValueType::Any => todo!(),
            ValueType::String => {
                let without_quotes = &value_source[1..value_source.len() - 1];

                Value::String(without_quotes.to_string())
            }
            ValueType::Float => Value::Float(value_source.parse().unwrap()),
            ValueType::Integer => Value::Integer(value_source.parse().unwrap()),
            ValueType::Boolean => Value::Boolean(value_source.parse().unwrap()),
            ValueType::List(nodes) => {
                let mut values = Vec::with_capacity(nodes.len());

                for node in nodes {
                    let value = node.run(source, context)?;

                    values.push(value);
                }

                Value::List(List::with_items(values))
            }
            ValueType::Empty => Value::Empty,
            ValueType::Map(nodes) => {
                let map = Map::new();

                for (key, node) in nodes {
                    let value = node.run(source, context)?;

                    map.variables_mut().insert(key.clone(), value);
                }

                Value::Map(map)
            }
            ValueType::Table {
                column_names,
                rows: row_expression,
            } => {
                let mut headers = Vec::with_capacity(column_names.len());
                let mut rows = Vec::new();

                for identifier in column_names {
                    let name = identifier.inner().clone();

                    headers.push(name)
                }

                let _row_values = row_expression.run(source, context)?;
                let row_values = _row_values.as_list()?.items();

                for value in row_values.iter() {
                    let row = value.as_list()?.items().clone();

                    rows.push(row)
                }

                let table = Table::from_raw_parts(headers, rows);

                Value::Table(table)
            }
            ValueType::Function(function) => Value::Function(function.clone()),
        };

        Ok(value)
    }
}
