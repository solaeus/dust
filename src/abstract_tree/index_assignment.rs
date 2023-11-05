use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Index, Map, Result, Statement, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct IndexAssignment {
    index: Index,
    operator: AssignmentOperator,
    statement: Statement,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum AssignmentOperator {
    Equal,
    PlusEqual,
    MinusEqual,
}

impl AbstractTree for IndexAssignment {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        let index_node = node.child(0).unwrap();
        let index = Index::from_syntax_node(source, index_node)?;

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

        Ok(IndexAssignment {
            index,
            operator,
            statement,
        })
    }

    fn run(&self, source: &str, context: &mut Map) -> Result<Value> {
        let mut left = self.index.run(source, context)?;
        let right = self.statement.run(source, context)?;

        match self.operator {
            AssignmentOperator::PlusEqual => left += right,
            AssignmentOperator::MinusEqual => left -= right,
            AssignmentOperator::Equal => left = right,
        }

        Ok(Value::Empty)
    }
}
