use std::{collections::BTreeMap, ops::Range};

use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{
    AbstractTree, Error, Expression, Function, Identifier, Item, Result, Table, Value, ValueType,
    VariableMap,
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
                let mut child_nodes = Vec::new();

                for index in 1..child.child_count() - 1 {
                    let child_syntax_node = child.child(index).unwrap();

                    if child_syntax_node.is_named() {
                        let expression = Expression::from_syntax_node(source, child_syntax_node)?;
                        child_nodes.push(expression);
                    }
                }

                ValueType::ListExact(child_nodes)
            }
            "table" => {
                let child_count = child.child_count();
                let mut column_names = Vec::new();

                let expression_node = child.child(child_count - 1).unwrap();
                let expression = Expression::from_syntax_node(source, expression_node)?;

                for index in 2..child.child_count() - 2 {
                    let node = child.child(index).unwrap();

                    if node.is_named() {
                        let identifier = Identifier::from_syntax_node(source, node)?;

                        column_names.push(identifier)
                    }
                }

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

                    if child_syntax_node.kind() == "expression" {
                        let key = current_key.clone();
                        let expression = Expression::from_syntax_node(source, child_syntax_node)?;

                        child_nodes.insert(key, expression);
                    }
                }

                ValueType::Map(child_nodes)
            }
            "function" => {
                let mut identifiers = Vec::new();

                let item_node = child.child(child.child_count() - 2).unwrap();
                let item = Item::from_syntax_node(source, item_node)?;

                for index in 1..child.child_count() - 3 {
                    let child_node = child.child(index).unwrap();

                    if child_node.kind() == "identifier" {
                        let identifier = Identifier::from_syntax_node(source, child_node)?;

                        identifiers.push(identifier);
                    }
                }

                let function = Function::new(identifiers, item);

                ValueType::Function(function)
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

    fn run(&self, source: &str, context: &mut VariableMap) -> Result<Value> {
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
            ValueType::ListExact(nodes) => {
                let mut values = Vec::with_capacity(nodes.len());

                for node in nodes {
                    let value = node.run(source, context)?;

                    values.push(value);
                }

                Value::List(values)
            }
            ValueType::Empty => Value::Empty,
            ValueType::Map(nodes) => {
                let mut values = VariableMap::new();

                for (key, node) in nodes {
                    let value = node.run(source, context)?;

                    values.set_value(key.clone(), value)?;
                }

                Value::Map(values)
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
                let row_values = _row_values.as_list()?;

                for value in row_values {
                    let row = value.as_list()?.clone();

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
