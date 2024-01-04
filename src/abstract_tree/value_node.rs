use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{
    AbstractTree, BuiltInValue, Error, Expression, Function, Identifier, List, Result, Structure,
    StructureInstantiator, Type, Value,
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
    Structure {
        definition_name: Identifier,
        instantiator: StructureInstantiator,
    },
    StructureDefinition(StructureInstantiator),
    BuiltInValue(BuiltInValue),
}

impl AbstractTree for ValueNode {
    fn from_syntax_node(source: &str, node: Node, context: &Structure) -> Result<Self> {
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
            "structure" => {
                let identifier_node = child.child(1).unwrap();
                let identifier = Identifier::from_syntax_node(source, identifier_node, context)?;

                let instantiator_node = child.child(2);

                if let Some(node) = instantiator_node {
                    let instantiator =
                        StructureInstantiator::from_syntax_node(source, node, context)?;

                    ValueNode::Structure {
                        definition_name: identifier,
                        instantiator,
                    }
                } else {
                    todo!()
                }
            }
            "structure_definition" => {
                let instantiator_node = child.child(1).unwrap();

                ValueNode::StructureDefinition(StructureInstantiator::from_syntax_node(
                    source,
                    instantiator_node,
                    context,
                )?)
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
            _ => {
                return Err(Error::UnexpectedSyntaxNode {
                    expected: "string, integer, float, boolean, list, map, or option".to_string(),
                    actual: child.kind().to_string(),
                    location: child.start_position(),
                    relevant_source: source[child.byte_range()].to_string(),
                })
            }
        };

        Ok(value_node)
    }

    fn check_type(&self, context: &Structure) -> Result<()> {
        match self {
            ValueNode::StructureDefinition(instantiator) => {
                for (_, (statement_option, type_definition_option)) in instantiator.iter() {
                    if let (Some(statement), Some(type_definition)) =
                        (statement_option, type_definition_option)
                    {
                        type_definition
                            .inner()
                            .check(&statement.expected_type(context)?)?;
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn run(&self, source: &str, context: &Structure) -> Result<Value> {
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
            ValueNode::Structure {
                definition_name,
                instantiator,
            } => {
                let variables = context.variables()?;
                let definition = if let Some((value, _)) = variables.get(definition_name.inner()) {
                    value.as_structure()?.instantiator()
                } else {
                    return Err(Error::VariableIdentifierNotFound(
                        definition_name.inner().clone(),
                    ));
                };

                let structure = Structure::new(BTreeMap::new(), definition.clone());

                for (key, (statement_option, type_definition_option)) in
                    definition.iter().chain(instantiator.iter())
                {
                    let value = if let Some(statement) = statement_option {
                        statement.run(source, context)?
                    } else {
                        Value::none()
                    };

                    if let Some(type_definition) = &type_definition_option {
                        structure.set(
                            key.to_string(),
                            value,
                            Some(type_definition.inner().clone()),
                        )?;
                    } else {
                        structure.set(key.to_string(), value, None)?;
                    }
                }

                Value::Structure(structure)
            }
            ValueNode::StructureDefinition(instantiator) => instantiator.run(source, context)?,
            ValueNode::BuiltInValue(built_in_value) => built_in_value.run(source, context)?,
        };

        Ok(value)
    }

    fn expected_type(&self, context: &Structure) -> Result<Type> {
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
            ValueNode::Structure {
                definition_name, ..
            } => Type::Structure(definition_name.clone()),
            ValueNode::StructureDefinition(instantiator) => {
                Type::StructureDefinition(instantiator.clone())
            }
            ValueNode::BuiltInValue(built_in_value) => built_in_value.expected_type(context)?,
        };

        Ok(r#type)
    }
}
