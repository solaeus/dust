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
    fn from_syntax_node(node: Node, source: &str) -> Result<Self> {
        let identifier_node = node.child(0).unwrap();
        let identifier = Identifier::from_syntax_node(identifier_node, source)?;

        let operator_node = node.child(1).unwrap().child(0).unwrap();
        let operator = match operator_node.kind() {
            "=" => AssignmentOperator::Equal,
            "+=" => AssignmentOperator::PlusEqual,
            "-=" => AssignmentOperator::MinusEqual,
            _ => {
                return Err(Error::UnexpectedSyntax {
                    expected: "=, += or -=",
                    actual: operator_node.kind(),
                    location: operator_node.start_position(),
                    relevant_source: source[node.byte_range()].to_string(),
                })
            }
        };

        let statement_node = node.child(2).unwrap();
        let statement = Statement::from_syntax_node(statement_node, source)?;

        Ok(Assignment {
            identifier,
            operator,
            statement,
        })
    }

    fn run(&self, context: &mut VariableMap) -> Result<Value> {
        let key = self.identifier.clone().take_inner();
        let mut value = self.statement.run(context)?;

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
