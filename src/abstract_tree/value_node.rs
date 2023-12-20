use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{
    AbstractTree, Block, Error, Expression, Function, Identifier, List, Map, Result, Statement,
    Type, TypeDefinition, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum ValueNode {
    Boolean(String),
    Float(String),
    Function(Function),
    Integer(String),
    String(String),
    List(Vec<Expression>),
    Empty,
    Map(BTreeMap<String, (Statement, Option<Type>)>),
}

impl AbstractTree for ValueNode {
    fn from_syntax_node(source: &str, node: Node, context: &Map) -> Result<Self> {
        Error::expect_syntax_node(source, "value", node)?;

        let child = node.child(0).unwrap();
        let value_node = match child.kind() {
            "boolean" => ValueNode::Boolean(source[child.byte_range()].to_string()),
            "float" => ValueNode::Float(source[child.byte_range()].to_string()),
            "function" => {
                let child_count = child.child_count();
                let mut parameters = Vec::new();
                let mut parameter_types = Vec::new();

                for index in 2..child_count - 2 {
                    let child = child.child(index).unwrap();

                    if child.kind() == "identifier" {
                        let identifier = Identifier::from_syntax_node(source, child, context)?;

                        parameters.push(identifier);
                    }

                    if child.kind() == "type_definition" {
                        let type_definition =
                            TypeDefinition::from_syntax_node(source, child, context)?;

                        parameter_types.push(type_definition.take_inner());
                    }
                }

                let function_context = Map::clone_from(context)?;

                for (parameter_name, parameter_type) in
                    parameters.iter().zip(parameter_types.iter())
                {
                    function_context.set(
                        parameter_name.inner().clone(),
                        Value::Empty,
                        Some(parameter_type.clone()),
                    )?;
                }

                let return_type_node = child.child(child_count - 2).unwrap();
                let return_type =
                    TypeDefinition::from_syntax_node(source, return_type_node, context)?;

                let body_node = child.child(child_count - 1).unwrap();
                let body = Block::from_syntax_node(source, body_node, &function_context)?;

                let r#type = Type::Function {
                    parameter_types,
                    return_type: Box::new(return_type.take_inner()),
                };

                ValueNode::Function(Function::new(
                    parameters,
                    body,
                    Some(r#type),
                    function_context,
                ))
            }
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
            _ => {
                return Err(Error::UnexpectedSyntaxNode {
                    expected: "string, integer, float, boolean, list, map, or empty",
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
            ValueNode::Function(function) => Value::Function(function.clone()),
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
                    for (key, (statement, r#type)) in key_statement_pairs {
                        let value = statement.run(source, context)?;

                        map.set(key.clone(), value, r#type.clone())?;
                    }
                }

                Value::Map(map)
            }
        };

        Ok(value)
    }

    fn expected_type(&self, context: &Map) -> Result<Type> {
        let type_definition = match self {
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
            ValueNode::Empty => Type::Any,
            ValueNode::Map(_) => Type::Map,
        };

        Ok(type_definition)
    }
}
#[cfg(test)]
mod tests {
    use crate::{evaluate, List};

    use super::*;

    #[test]
    fn evaluate_empty() {
        assert_eq!(evaluate("x = 9"), Ok(Value::Empty));
        assert_eq!(evaluate("x = 1 + 1"), Ok(Value::Empty));
    }

    #[test]
    fn evaluate_integer() {
        assert_eq!(evaluate("1"), Ok(Value::Integer(1)));
        assert_eq!(evaluate("123"), Ok(Value::Integer(123)));
        assert_eq!(evaluate("-666"), Ok(Value::Integer(-666)));
    }

    #[test]
    fn evaluate_float() {
        assert_eq!(evaluate("0.1"), Ok(Value::Float(0.1)));
        assert_eq!(evaluate("12.3"), Ok(Value::Float(12.3)));
        assert_eq!(evaluate("-6.66"), Ok(Value::Float(-6.66)));
    }

    #[test]
    fn evaluate_string() {
        assert_eq!(evaluate("\"one\""), Ok(Value::String("one".to_string())));
        assert_eq!(evaluate("'one'"), Ok(Value::String("one".to_string())));
        assert_eq!(evaluate("`one`"), Ok(Value::String("one".to_string())));
        assert_eq!(evaluate("`'one'`"), Ok(Value::String("'one'".to_string())));
        assert_eq!(evaluate("'`one`'"), Ok(Value::String("`one`".to_string())));
        assert_eq!(
            evaluate("\"'one'\""),
            Ok(Value::String("'one'".to_string()))
        );
    }

    #[test]
    fn evaluate_list() {
        assert_eq!(
            evaluate("[1, 2, 'foobar']"),
            Ok(Value::List(List::with_items(vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::String("foobar".to_string()),
            ])))
        );
    }

    #[test]
    fn evaluate_map() {
        let map = Map::new();

        map.set("x".to_string(), Value::Integer(1), None).unwrap();
        map.set("foo".to_string(), Value::String("bar".to_string()), None)
            .unwrap();

        assert_eq!(evaluate("{ x = 1, foo = 'bar' }"), Ok(Value::Map(map)));
    }

    #[test]
    fn evaluate_map_types() {
        let map = Map::new();

        map.set("x".to_string(), Value::Integer(1), Some(Type::Integer))
            .unwrap();
        map.set(
            "foo".to_string(),
            Value::String("bar".to_string()),
            Some(Type::String),
        )
        .unwrap();

        assert_eq!(
            evaluate("{ x <int> = 1, foo <str> = 'bar' }"),
            Ok(Value::Map(map))
        );
    }

    #[test]
    fn evaluate_map_type_errors() {
        assert!(evaluate("{ foo <bool> = 'bar' }")
            .unwrap_err()
            .is_type_check_error(&Error::TypeCheck {
                expected: Type::Boolean,
                actual: Type::String
            }))
    }

    #[test]
    fn evaluate_function() {
        let result = evaluate("(fn) <int> { 1 }");
        let value = result.unwrap();
        let function = value.as_function().unwrap();

        assert_eq!(&Vec::<Identifier>::with_capacity(0), function.parameters());
        assert_eq!(Ok(&Type::Integer), function.return_type());

        let result = evaluate("(fn x <bool>) <bool> {true}");
        let value = result.unwrap();
        let function = value.as_function().unwrap();

        assert_eq!(
            &vec![Identifier::new("x".to_string())],
            function.parameters()
        );
        assert_eq!(Ok(&Type::Boolean), function.return_type());
    }
}
