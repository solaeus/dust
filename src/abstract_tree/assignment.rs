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

        let identifier_node = node.child_by_field_name("identifier").unwrap();
        let identifier = Identifier::from_syntax_node(source, identifier_node, context)?;

        let type_node = node.child_by_field_name("type");
        let type_definition = if let Some(type_node) = type_node {
            Some(TypeDefinition::from_syntax_node(
                source, type_node, context,
            )?)
        } else {
            None
        };

        let operator_node = node
            .child_by_field_name("assignment_operator")
            .unwrap()
            .child(0)
            .unwrap();
        let operator = match operator_node.kind() {
            "=" => AssignmentOperator::Equal,
            "+=" => AssignmentOperator::PlusEqual,
            "-=" => AssignmentOperator::MinusEqual,
            _ => {
                return Err(Error::UnexpectedSyntaxNode {
                    expected: "=, += or -=",
                    actual: operator_node.kind(),
                    location: operator_node.start_position(),
                    relevant_source: source[operator_node.byte_range()].to_string(),
                })
            }
        };

        let statement_node = node.child_by_field_name("statement").unwrap();
        let statement = Statement::from_syntax_node(source, statement_node, context)?;

        if let Some(type_defintion) = &type_definition {
            let statement_type = statement.expected_type(context)?;

            type_defintion.check(&statement_type, context)?;
        }

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
                if let Some(mut previous_value) = context.variables()?.get(key).cloned() {
                    previous_value += value;
                    previous_value
                } else {
                    return Err(Error::VariableIdentifierNotFound(key.clone()));
                }
            }
            AssignmentOperator::MinusEqual => {
                if let Some(mut previous_value) = context.variables()?.get(key).cloned() {
                    previous_value -= value;
                    previous_value
                } else {
                    return Err(Error::VariableIdentifierNotFound(key.clone()));
                }
            }
            AssignmentOperator::Equal => value,
        };

        if let Some(type_definition) = &self.type_definition {
            let new_value_type = new_value.r#type(context)?;

            type_definition.check(&new_value_type, context)?;
        }

        context.variables_mut()?.insert(key.clone(), new_value);

        Ok(Value::Empty)
    }

    fn expected_type(&self, _context: &Map) -> Result<TypeDefinition> {
        Ok(TypeDefinition::new(Type::Empty))
    }
}
