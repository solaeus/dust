use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Identifier, Map, Result, Statement, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct IdentifierAssignment {
    identifier: Identifier,
    operator: AssignmentOperator,
    statement: Statement,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum AssignmentOperator {
    Equal,
    PlusEqual,
    MinusEqual,
}

impl AbstractTree for IdentifierAssignment {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        let identifier_node = node.child(0).unwrap();
        let identifier = Identifier::from_syntax_node(source, identifier_node)?;

        let operator_node = node.child(1).unwrap().child(0).unwrap();
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

        let statement_node = node.child(2).unwrap();
        let statement = Statement::from_syntax_node(source, statement_node)?;

        Ok(IdentifierAssignment {
            identifier,
            operator,
            statement,
        })
    }

    fn run(&self, source: &str, context: &mut Map) -> Result<Value> {
        let key = self.identifier.inner().clone();
        let value = self.statement.run(source, context)?;
        let mut context = context.variables_mut();

        let new_value = match self.operator {
            AssignmentOperator::PlusEqual => {
                if let Some(mut previous_value) = context.get(&key).cloned() {
                    previous_value += value;
                    previous_value
                } else {
                    Value::Empty
                }
            }
            AssignmentOperator::MinusEqual => {
                if let Some(mut previous_value) = context.get(&key).cloned() {
                    previous_value -= value;
                    previous_value
                } else {
                    Value::Empty
                }
            }
            AssignmentOperator::Equal => value,
        };

        context.insert(key, new_value);

        Ok(Value::Empty)
    }
}
