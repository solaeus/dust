use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{AbstractNode, Evaluation, ExpectedType, Statement, Type};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Block {
    statements: Vec<Statement>,
}

impl Block {
    pub fn new(statements: Vec<Statement>) -> Self {
        Self { statements }
    }

    pub fn first_statement(&self) -> &Statement {
        self.statements.first().unwrap()
    }

    pub fn last_statement(&self) -> &Statement {
        self.statements.last().unwrap()
    }
}

impl AbstractNode for Block {
    fn validate(
        &self,
        _context: &mut Context,
        _manage_memory: bool,
    ) -> Result<(), ValidationError> {
        for statement in &self.statements {
            statement.validate(_context, _manage_memory)?;
        }

        Ok(())
    }

    fn evaluate(
        self,
        _context: &mut Context,
        _manage_memory: bool,
    ) -> Result<Evaluation, RuntimeError> {
        let mut previous = Evaluation::None;

        for statement in self.statements {
            previous = statement.evaluate(_context, _manage_memory)?;
        }

        Ok(previous)
    }
}

impl ExpectedType for Block {
    fn expected_type(&self, _context: &mut Context) -> Result<Type, ValidationError> {
        self.last_statement().expected_type(_context)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        abstract_tree::{Expression, ValueNode, WithPos},
        Value,
    };

    use super::*;

    #[test]
    fn run_returns_value_of_final_statement() {
        let block = Block::new(vec![
            Statement::ValueExpression(Expression::Value(
                ValueNode::Integer(1).with_position((0, 0)),
            )),
            Statement::ValueExpression(Expression::Value(
                ValueNode::Integer(2).with_position((0, 0)),
            )),
            Statement::ValueExpression(Expression::Value(
                ValueNode::Integer(42).with_position((0, 0)),
            )),
        ]);

        assert_eq!(
            block.evaluate(&mut Context::new(None), true).unwrap(),
            Evaluation::Return(Value::integer(42))
        )
    }

    #[test]
    fn expected_type_returns_type_of_final_statement() {
        let block = Block::new(vec![
            Statement::ValueExpression(Expression::Value(
                ValueNode::String("42".to_string()).with_position((0, 0)),
            )),
            Statement::ValueExpression(Expression::Value(
                ValueNode::Integer(42).with_position((0, 0)),
            )),
        ]);

        assert_eq!(
            block.expected_type(&mut Context::new(None)),
            Ok(Type::Integer)
        )
    }
}
