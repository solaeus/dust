use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
};

use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{
    AbstractTree, BuiltInValue, Error, Expression, Function, Identifier, List, Map, Result,
    Statement, Structure, Type, TypeDefinition, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum ValueNode {
    Boolean(String),
    Float(String),
    Function(Function),
    Integer(String),
    String(String),
    List(Vec<Expression>),
    Option(Option<Box<Expression>>),
    Map(BTreeMap<String, (Statement, Option<Type>)>),
    BuiltInValue(BuiltInValue),
    Structure(BTreeMap<String, (Option<Statement>, Type)>),
}

impl AbstractTree for ValueNode {
    fn from_syntax_node(source: &str, node: Node, context: &Map) -> Result<Self> {
        Error::expect_syntax_node(source, "value", node)?;

        let child = node.child(0).unwrap();
        let value_node = match child.kind() {
            "boolean" => ValueNode::Boolean(source[child.byte_range()].to_string()),
            "float" => ValueNode::Float(source[child.byte_range()].to_string()),
            "function" => ValueNode::Function(Function::from_syntax_node(source, child, context)?),
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
            "map" => {
                let mut child_nodes = BTreeMap::new();
                let mut current_key = "".to_string();
                let mut current_type = None;

                for index in 0..child.child_count() - 1 {
                    let child_syntax_node = child.child(index).unwrap();

                    if child_syntax_node.kind() == "identifier" {
                        current_key =
                            Identifier::from_syntax_node(source, child_syntax_node, context)?
                                .take_inner();
                        current_type = None;
                    }

                    if child_syntax_node.kind() == "type_definition" {
                        current_type = Some(
                            TypeDefinition::from_syntax_node(source, child_syntax_node, context)?
                                .take_inner(),
                        );
                    }

                    if child_syntax_node.kind() == "statement" {
                        let statement =
                            Statement::from_syntax_node(source, child_syntax_node, context)?;

                        if let Some(type_definition) = &current_type {
                            type_definition.check(&statement.expected_type(context)?)?;
                        }

                        child_nodes.insert(current_key.clone(), (statement, current_type.clone()));
                    }
                }

                ValueNode::Map(child_nodes)
            }
            "option" => {
                let first_grandchild = child.child(0).unwrap();

                if first_grandchild.kind() == "none" {
                    ValueNode::Option(None)
                } else {
                    let expression_node = child.child(2).unwrap();
                    let expression =
                        Expression::from_syntax_node(source, expression_node, context)?;

                    ValueNode::Option(Some(Box::new(expression)))
                }
            }
            "built_in_value" => {
                let built_in_value_node = child.child(0).unwrap();

                ValueNode::BuiltInValue(BuiltInValue::from_syntax_node(
                    source,
                    built_in_value_node,
                    context,
                )?)
            }
            "structure" => {
                let mut btree_map = BTreeMap::new();
                let mut current_identifier: Option<Identifier> = None;
                let mut current_type: Option<Type> = None;
                let mut current_statement = None;

                for index in 2..child.child_count() - 1 {
                    let child_syntax_node = child.child(index).unwrap();

                    if child_syntax_node.kind() == "identifier" {
                        if current_statement.is_none() {
                            if let (Some(identifier), Some(r#type)) =
                                (&current_identifier, &current_type)
                            {
                                btree_map
                                    .insert(identifier.inner().clone(), (None, r#type.clone()));
                            }
                        }

                        current_type = None;
                        current_identifier = Some(Identifier::from_syntax_node(
                            source,
                            child_syntax_node,
                            context,
                        )?);
                    }

                    if child_syntax_node.kind() == "type_definition" {
                        current_type = Some(
                            TypeDefinition::from_syntax_node(source, child_syntax_node, context)?
                                .take_inner(),
                        );
                    }

                    if child_syntax_node.kind() == "statement" {
                        current_statement = Some(Statement::from_syntax_node(
                            source,
                            child_syntax_node,
                            context,
                        )?);

                        if let Some(identifier) = &current_identifier {
                            let r#type = if let Some(r#type) = &current_type {
                                r#type.clone()
                            } else if let Some(statement) = &current_statement {
                                statement.expected_type(context)?
                            } else {
                                Type::None
                            };

                            btree_map.insert(
                                identifier.inner().clone(),
                                (current_statement.clone(), r#type.clone()),
                            );
                        }
                    }
                }

                ValueNode::Structure(btree_map)
            }
            _ => {
                return Err(Error::UnexpectedSyntaxNode {
                    expected: "string, integer, float, boolean, list, map, option or structure"
                        .to_string(),
                    actual: child.kind().to_string(),
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
            ValueNode::Function(function) => Value::Function(function.clone()),
            ValueNode::Integer(value_source) => Value::Integer(value_source.parse().unwrap()),
            ValueNode::String(value_source) => Value::string(value_source.clone()),
            ValueNode::List(expressions) => {
                let mut values = Vec::with_capacity(expressions.len());

                for node in expressions {
                    let value = node.run(source, context)?;

                    values.push(value);
                }

                Value::List(List::with_items(values))
            }
            ValueNode::Option(option) => {
                let option_value = if let Some(expression) = option {
                    Some(Box::new(expression.run(source, context)?))
                } else {
                    None
                };

                Value::Option(option_value)
            }
            ValueNode::Map(key_statement_pairs) => {
                let map = Map::new();

                {
                    for (key, (statement, r#type)) in key_statement_pairs {
                        let value = statement.run(source, context)?;

                        map.set(key.clone(), value, r#type.clone())?;
                    }
                }

                Value::Map(map)
            }
            ValueNode::BuiltInValue(built_in_value) => built_in_value.run(source, context)?,
            ValueNode::Structure(node_map) => {
                let mut value_map = BTreeMap::new();

                for (key, (statement_option, r#type)) in node_map {
                    let value_option = if let Some(statement) = statement_option {
                        Some(statement.run(source, context)?)
                    } else {
                        None
                    };

                    value_map.insert(key.to_string(), (value_option, r#type.clone()));
                }

                Value::Structure(Structure::new(value_map))
            }
        };

        Ok(value)
    }

    fn expected_type(&self, context: &Map) -> Result<Type> {
        let r#type = match self {
            ValueNode::Boolean(_) => Type::Boolean,
            ValueNode::Float(_) => Type::Float,
            ValueNode::Function(function) => function.r#type().clone(),
            ValueNode::Integer(_) => Type::Integer,
            ValueNode::String(_) => Type::String,
            ValueNode::List(expressions) => {
                let mut previous_type = None;

                for expression in expressions {
                    let expression_type = expression.expected_type(context)?;

                    if let Some(previous) = previous_type {
                        if expression_type != previous {
                            return Ok(Type::List(Box::new(Type::Any)));
                        }
                    }

                    previous_type = Some(expression_type);
                }

                if let Some(previous) = previous_type {
                    Type::List(Box::new(previous))
                } else {
                    Type::List(Box::new(Type::Any))
                }
            }
            ValueNode::Option(option) => {
                if let Some(expression) = option {
                    Type::Option(Box::new(expression.expected_type(context)?))
                } else {
                    Type::None
                }
            }
            ValueNode::Map(_) => Type::Map(None),
            ValueNode::BuiltInValue(built_in_value) => built_in_value.expected_type(context)?,
            ValueNode::Structure(node_map) => {
                let mut value_map = BTreeMap::new();

                for (key, (_statement_option, r#type)) in node_map {
                    value_map.insert(key.to_string(), (None, r#type.clone()));
                }

                Type::Map(Some(Structure::new(value_map)))
            }
        };

        Ok(r#type)
    }
}

impl Display for ValueNode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ValueNode::Boolean(source)
            | ValueNode::Float(source)
            | ValueNode::Integer(source)
            | ValueNode::String(source) => write!(f, "{source}"),
            ValueNode::Function(function) => write!(f, "{function}"),
            ValueNode::List(expressions) => {
                for expression in expressions {
                    write!(f, "{expression}")?;
                }

                Ok(())
            }
            ValueNode::Option(option) => {
                if let Some(expression) = option {
                    write!(f, "some({})", expression)
                } else {
                    write!(f, "none")
                }
            }
            ValueNode::Map(nodes) => {
                writeln!(f, "{{")?;

                for (key, (statement, type_option)) in nodes {
                    if let Some(r#type) = type_option {
                        writeln!(f, "  {key} <{}> = {statement}", r#type)?;
                    } else {
                        writeln!(f, "  {key} = {statement}")?;
                    }
                }
                write!(f, "}}")
            }
            ValueNode::BuiltInValue(built_in_value) => write!(f, "{built_in_value}"),
            ValueNode::Structure(nodes) => {
                writeln!(f, "{{")?;

                for (key, (value_option, r#type)) in nodes {
                    if let Some(value) = value_option {
                        writeln!(f, "  {key} <{}> = {value}", r#type)?;
                    } else {
                        writeln!(f, "  {key} <{}>", r#type)?;
                    }
                }

                write!(f, "}}")
            }
        }
    }
}
