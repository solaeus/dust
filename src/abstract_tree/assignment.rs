use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Identifier, Map, Result, Statement, Type, TypeDefinition, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Assignment {
    identifier: Identifier,
    type_definition: Option<TypeDefinition>,
    operator: AssignmentOperator,
    statement: Statement,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum AssignmentOperator {
    Equal,
    PlusEqual,
    MinusEqual,
}

impl AbstractTree for Assignment {
    fn from_syntax_node(source: &str, node: Node, context: &Map) -> Result<Self> {
        Error::expect_syntax_node(source, "assignment", node)?;

        let child_count = node.child_count();

        let identifier_node = node.child(0).unwrap();
        let identifier = Identifier::from_syntax_node(source, identifier_node, context)?;
        let identifier_type = identifier.expected_type(context)?;

        let type_node = node.child(1).unwrap();
        let type_definition = if type_node.kind() == "type_definition" {
            Some(TypeDefinition::from_syntax_node(
                source, type_node, context,
            )?)
        } else {
            None
        };

        let operator_node = node.child(child_count - 2).unwrap().child(0).unwrap();
        let operator = match operator_node.kind() {
            "=" => AssignmentOperator::Equal,
            "+=" => AssignmentOperator::PlusEqual,
            "-=" => AssignmentOperator::MinusEqual,
            _ => {
                return Err(Error::UnexpectedSyntaxNode {
                    expected: "=, += or -=".to_string(),
                    actual: operator_node.kind().to_string(),
                    location: operator_node.start_position(),
                    relevant_source: source[operator_node.byte_range()].to_string(),
                })
            }
        };

        let statement_node = node.child(child_count - 1).unwrap();
        let statement = Statement::from_syntax_node(source, statement_node, context)?;
        let statement_type = statement.expected_type(context)?;

        if let Some(type_definition) = &type_definition {
            match operator {
                AssignmentOperator::Equal => {
                    type_definition
                        .inner()
                        .check(&statement_type)
                        .map_err(|error| error.at_node(statement_node, source))?;
                }
                AssignmentOperator::PlusEqual => {
                    if let Type::List(item_type) = type_definition.inner() {
                        item_type
                            .check(&statement_type)
                            .map_err(|error| error.at_node(statement_node, source))?;
                    } else {
                        type_definition
                            .inner()
                            .check(&identifier_type)
                            .map_err(|error| error.at_node(identifier_node, source))?;
                    }
                }
                AssignmentOperator::MinusEqual => todo!(),
            }
        } else {
            match operator {
                AssignmentOperator::Equal => {}
                AssignmentOperator::PlusEqual => {
                    if let Type::List(item_type) = identifier_type {
                        item_type
                            .check(&statement_type)
                            .map_err(|error| error.at_node(statement_node, source))?;
                    }
                }
                AssignmentOperator::MinusEqual => todo!(),
            }
        }

        let variable_key = identifier.inner().clone();
        let variable_type = if let Some(definition) = &type_definition {
            definition.inner().clone()
        } else if let Some((_, r#type)) = context.variables()?.get(identifier.inner()) {
            r#type.clone()
        } else {
            statement_type
        };

        context.set(variable_key, Value::none(), Some(variable_type))?;

        Ok(Assignment {
            identifier,
            type_definition,
            operator,
            statement,
        })
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value> {
        let key = self.identifier.inner();
        let value = self.statement.run(source, context)?;

        let new_value = match self.operator {
            AssignmentOperator::PlusEqual => {
                if let Some((mut previous_value, _)) = context.variables()?.get(key).cloned() {
                    previous_value += value;
                    previous_value
                } else {
                    return Err(Error::VariableIdentifierNotFound(key.clone()));
                }
            }
            AssignmentOperator::MinusEqual => {
                if let Some((mut previous_value, _)) = context.variables()?.get(key).cloned() {
                    previous_value -= value;
                    previous_value
                } else {
                    return Err(Error::VariableIdentifierNotFound(key.clone()));
                }
            }
            AssignmentOperator::Equal => value,
        };

        if let Some(type_defintion) = &self.type_definition {
            context.set(key.clone(), new_value, Some(type_defintion.inner().clone()))?;
        } else {
            context.set(key.clone(), new_value, None)?;
        }

        Ok(Value::none())
    }

    fn expected_type(&self, _context: &Map) -> Result<Type> {
        Ok(Type::None)
    }
}
