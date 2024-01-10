use serde::{Deserialize, Serialize};

use crate::{
    AbstractTree, AssignmentOperator, Error, Format, Index, IndexExpression, Map, Result,
    Statement, SyntaxNode, Type, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct IndexAssignment {
    index: Index,
    operator: AssignmentOperator,
    statement: Statement,
}

impl AbstractTree for IndexAssignment {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Map) -> Result<Self> {
        Error::expect_syntax_node(source, "index_assignment", node)?;

        let index_node = node.child(0).unwrap();
        let index = Index::from_syntax(index_node, source, context)?;

        let operator_node = node.child(1).unwrap();
        let operator = AssignmentOperator::from_syntax(operator_node, source, context)?;

        let statement_node = node.child(2).unwrap();
        let statement = Statement::from_syntax(statement_node, source, context)?;

        Ok(IndexAssignment {
            index,
            operator,
            statement,
        })
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value> {
        let index_collection = self.index.collection.run(source, context)?;
        let index_context = index_collection.as_map().unwrap_or(context);
        let index_key = if let IndexExpression::Identifier(identifier) = &self.index.index {
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

        index_context.set(index_key.clone(), new_value)?;

        Ok(Value::none())
    }

    fn expected_type(&self, _context: &Map) -> Result<Type> {
        Ok(Type::None)
    }
}

impl Format for IndexAssignment {
    fn format(&self, output: &mut String, indent_level: u8) {
        self.index.format(output, indent_level);
        output.push(' ');
        self.operator.format(output, indent_level);
        output.push(' ');
        self.statement.format(output, indent_level);
    }
}
