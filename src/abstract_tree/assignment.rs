use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Identifier, Map, Result, Statement, Type, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Assignment {
    identifier: Identifier,
    r#type: Option<Type>,
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
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        Error::expect_syntax_node(source, "assignment", node)?;

        let identifier_node = node.child_by_field_name("identifier").unwrap();
        let identifier = Identifier::from_syntax_node(source, identifier_node)?;

        let type_node = node.child_by_field_name("type");
        let r#type = if let Some(type_node) = type_node {
            Some(Type::from_syntax_node(source, type_node)?)
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
        let statement = Statement::from_syntax_node(source, statement_node)?;

        Ok(Assignment {
            identifier,
            r#type,
            operator,
            statement,
        })
    }

    fn run(&self, source: &str, context: &mut Map) -> Result<Value> {
        let key = self.identifier.inner().clone();
        let value = self.statement.run(source, context)?;

        let new_value = match self.operator {
            AssignmentOperator::PlusEqual => {
                if let Some(mut previous_value) = context.variables()?.get(&key).cloned() {
                    previous_value += value;
                    previous_value
                } else {
                    Value::Empty
                }
            }
            AssignmentOperator::MinusEqual => {
                if let Some(mut previous_value) = context.variables()?.get(&key).cloned() {
                    previous_value -= value;
                    previous_value
                } else {
                    Value::Empty
                }
            }
            AssignmentOperator::Equal => value,
        };

        let expected_type = self.r#type.as_ref().unwrap_or(&Type::Any);

        match (expected_type, new_value.r#type()) {
            (Type::Any, _)
            | (Type::Boolean, Type::Boolean)
            | (Type::Float, Type::Float)
            | (Type::Function, Type::Function)
            | (Type::Integer, Type::Integer)
            | (Type::List, Type::List)
            | (Type::Map, Type::Map)
            | (Type::String, Type::String)
            | (Type::Table, Type::Table) => {}
            (Type::Boolean, _) => return Err(Error::ExpectedBoolean { actual: new_value }),
            (Type::Float, _) => return Err(Error::ExpectedFloat { actual: new_value }),
            (Type::Function, _) => return Err(Error::ExpectedFunction { actual: new_value }),
            (Type::Integer, _) => return Err(Error::ExpectedInteger { actual: new_value }),
            (Type::List, _) => return Err(Error::ExpectedList { actual: new_value }),
            (Type::Map, _) => return Err(Error::ExpectedMap { actual: new_value }),
            (Type::String, _) => return Err(Error::ExpectedString { actual: new_value }),
            (Type::Table, _) => return Err(Error::ExpectedTable { actual: new_value }),
        }

        context.variables_mut()?.insert(key, new_value);

        Ok(Value::Empty)
    }
}
