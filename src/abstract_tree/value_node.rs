use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{
    AbstractTree, Error, Expression, Function, Identifier, List, Map, Result, Statement, Table,
    Type, TypeDefinition, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum ValueNode {
    Boolean(String),
    Float(String),
    Integer(String),
    String(String),
    List(Vec<Expression>),
    Empty,
    Map(BTreeMap<String, Statement>),
    Table {
        column_names: Vec<Identifier>,
        rows: Box<Expression>,
    },
    Function(Function),
}

impl AbstractTree for ValueNode {
    fn from_syntax_node(source: &str, node: Node, context: &Map) -> Result<Self> {
        debug_assert_eq!("value", node.kind());

        let child = node.child(0).unwrap();
        let value_node = match child.kind() {
            "boolean" => ValueNode::Boolean(source[child.byte_range()].to_string()),
            "float" => ValueNode::Float(source[child.byte_range()].to_string()),
            "integer" => ValueNode::Integer(source[child.byte_range()].to_string()),
            "string" => {
                let without_quotes = child.start_byte() + 1..child.end_byte() - 1;

                ValueNode::String(source[without_quotes].to_string())
            }
            "list" => {
                let mut expressions = Vec::new();

                for index in 1..child.child_count() - 1 {
                    let current_node = child.child(index).unwrap();

                    if current_node.is_named() {
                        let expression =
                            Expression::from_syntax_node(source, current_node, context)?;
                        expressions.push(expression);
                    }
                }

                ValueNode::List(expressions)
            }
            "table" => {
                let identifier_list_node = child.child(1).unwrap();
                let identifier_count = identifier_list_node.child_count();
                let mut column_names = Vec::with_capacity(identifier_count);

                for index in 0..identifier_count {
                    let identifier_node = identifier_list_node.child(index).unwrap();

                    if identifier_node.is_named() {
                        let identifier =
                            Identifier::from_syntax_node(source, identifier_node, context)?;

                        column_names.push(identifier)
                    }
                }

                let expression_node = child.child(2).unwrap();
                let expression = Expression::from_syntax_node(source, expression_node, context)?;

                ValueNode::Table {
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
                            Identifier::from_syntax_node(source, child_syntax_node, context)?
                                .take_inner();
                    }

                    if child_syntax_node.kind() == "statement" {
                        let key = current_key.clone();
                        let statement =
                            Statement::from_syntax_node(source, child_syntax_node, context)?;

                        child_nodes.insert(key, statement);
                    }
                }

                ValueNode::Map(child_nodes)
            }
            "function" => ValueNode::Function(Function::from_syntax_node(source, child, context)?),
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

        Ok(value_node)
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value> {
        let value = match self {
            ValueNode::Boolean(value_source) => Value::Boolean(value_source.parse().unwrap()),
            ValueNode::Float(value_source) => Value::Float(value_source.parse().unwrap()),
            ValueNode::Integer(value_source) => Value::Integer(value_source.parse().unwrap()),
            ValueNode::String(value_source) => Value::String(value_source.parse().unwrap()),
            ValueNode::List(expressions) => {
                let mut values = Vec::with_capacity(expressions.len());

                for node in expressions {
                    let value = node.run(source, context)?;

                    values.push(value);
                }

                Value::List(List::with_items(values))
            }
            ValueNode::Empty => Value::Empty,
            ValueNode::Map(key_statement_pairs) => {
                let map = Map::new();

                {
                    let mut variables = map.variables_mut()?;

                    for (key, statement) in key_statement_pairs {
                        let value = statement.run(source, context)?;

                        variables.insert(key.clone(), value);
                    }
                }

                Value::Map(map)
            }
            ValueNode::Table {
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
            ValueNode::Function(function) => Value::Function(function.clone()),
        };

        Ok(value)
    }

    fn expected_type(&self, context: &Map) -> Result<TypeDefinition> {
        let type_definition = match self {
            ValueNode::Boolean(_) => TypeDefinition::new(Type::Boolean),
            ValueNode::Float(_) => TypeDefinition::new(Type::Float),
            ValueNode::Integer(_) => TypeDefinition::new(Type::Integer),
            ValueNode::String(_) => TypeDefinition::new(Type::String),
            ValueNode::List(expressions) => {
                let mut previous_type = None;

                for expression in expressions {
                    let expression_type = expression.expected_type(context)?;

                    if let Some(previous) = previous_type {
                        if expression_type != previous {
                            return Ok(TypeDefinition::new(Type::Any));
                        }
                    }

                    previous_type = Some(expression_type);
                }

                if let Some(previous) = previous_type {
                    TypeDefinition::new(Type::List(Box::new(previous.take_inner())))
                } else {
                    TypeDefinition::new(Type::List(Box::new(Type::Any)))
                }
            }
            ValueNode::Empty => TypeDefinition::new(Type::Any),
            ValueNode::Map(_) => TypeDefinition::new(Type::Map),
            ValueNode::Table {
                column_names: _,
                rows: _,
            } => TypeDefinition::new(Type::Table),
            ValueNode::Function(function) => return function.expected_type(context),
        };

        Ok(type_definition)
    }
}
