use serde::{Deserialize, Serialize};

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, AssignmentOperator, Context, Format, Identifier, Index, IndexExpression,
    Statement, SyntaxNode, Type, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct IndexAssignment {
    index: Index,
    operator: AssignmentOperator,
    statement: Statement,
}

impl AbstractTree for IndexAssignment {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Context) -> Result<Self, SyntaxError> {
        SyntaxError::expect_syntax_node(source, "index_assignment", node)?;

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

    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        Ok(Type::None)
    }

    fn validate(&self, _source: &str, _context: &Context) -> Result<(), ValidationError> {
        self.index.validate(_source, _context)?;
        self.statement.validate(_source, _context)
    }

    fn run(&self, source: &str, context: &Context) -> Result<Value, RuntimeError> {
        let _index_collection = self.index.collection.run(source, context)?;
        let index_identifier = if let IndexExpression::Identifier(identifier) = &self.index.index {
            identifier
        } else {
            let index_run = self.index.index.run(source, context)?;
            let expected_identifier = Identifier::new(index_run.as_string()?);

            return Err(RuntimeError::VariableIdentifierNotFound(
                expected_identifier,
            ));
        };

        let value = self.statement.run(source, context)?;

        let new_value = match self.operator {
            AssignmentOperator::PlusEqual => {
                if let Some(mut previous_value) = context.get_value(index_identifier)? {
                    previous_value += value;
                    previous_value
                } else {
                    Value::none()
                }
            }
            AssignmentOperator::MinusEqual => {
                if let Some(mut previous_value) = context.get_value(index_identifier)? {
                    previous_value -= value;
                    previous_value
                } else {
                    Value::none()
                }
            }
            AssignmentOperator::Equal => value,
        };

        context.set_value(index_identifier.clone(), new_value)?;

        Ok(Value::none())
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
