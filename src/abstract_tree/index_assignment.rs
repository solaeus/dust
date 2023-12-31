use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Index, Map, Result, Statement, Type, Value};

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
    fn from_syntax_node(source: &str, node: Node, context: &Map) -> Result<Self> {
        Error::expect_syntax_node(source, "index_assignment", node)?;

        let index_node = node.child(0).unwrap();
        let index = Index::from_syntax_node(source, index_node, context)?;

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
        let statement = Statement::from_syntax_node(source, statement_node, context)?;

        Ok(IndexAssignment {
            index,
            operator,
            statement,
        })
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value> {
        let index_collection = self.index.collection.run(source, context)?;
        let index_context = index_collection.as_map().unwrap_or(context);
        let index_key = if let crate::Expression::Identifier(identifier) = &self.index.index {
            identifier.inner()
        } else {
            return Err(Error::VariableIdentifierNotFound(
                self.index.index.run(source, context)?.to_string(),
            ));
        };

        let value = self.statement.run(source, context)?;

        let new_value = match self.operator {
            AssignmentOperator::PlusEqual => {
                if let Some((mut previous_value, _)) =
                    index_context.variables()?.get(index_key).cloned()
                {
                    previous_value += value;
                    previous_value
                } else {
                    Value::none()
                }
            }
            AssignmentOperator::MinusEqual => {
                if let Some((mut previous_value, _)) =
                    index_context.variables()?.get(index_key).cloned()
                {
                    previous_value -= value;
                    previous_value
                } else {
                    Value::none()
                }
            }
            AssignmentOperator::Equal => value,
        };

        index_context.set(index_key.clone(), new_value, None)?;

        Ok(Value::none())
    }

    fn expected_type(&self, _context: &Map) -> Result<Type> {
        Ok(Type::None)
    }
}
