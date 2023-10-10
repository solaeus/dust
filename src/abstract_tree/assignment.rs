use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Result, Value, VariableMap};

use super::{identifier::Identifier, statement::Statement};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Assignment {
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

impl AbstractTree for Assignment {
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

        Ok(Assignment {
            identifier,
            operator,
            statement,
        })
    }

    fn run(&self, source: &str, context: &mut VariableMap) -> Result<Value> {
        let key = self.identifier.inner().clone();
        let mut value = self.statement.run(source, context)?;

        match self.operator {
            AssignmentOperator::PlusEqual => {
                if let Some(previous_value) = context.get_value(&key)? {
                    value += previous_value
                }
            }
            AssignmentOperator::MinusEqual => {
                if let Some(previous_value) = context.get_value(&key)? {
                    value -= previous_value
                }
            }
            AssignmentOperator::Equal => {}
        }

        context.set_value(key, value)?;

        Ok(Value::Empty)
    }
}
