use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{AbstractNode, Evaluation, Statement, Type};

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
    fn define_types(&self, _context: &Context) -> Result<(), ValidationError> {
        for statement in &self.statements {
            statement.define_types(_context)?;
        }

        Ok(())
    }

    fn validate(&self, _context: &Context, _manage_memory: bool) -> Result<(), ValidationError> {
        for statement in &self.statements {
            statement.validate(_context, _manage_memory)?;
        }

        Ok(())
    }

    fn evaluate(
        self,
        _context: &Context,
        _manage_memory: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        let mut previous = None;

        for statement in self.statements {
            previous = statement.run(_context, _manage_memory)?;
        }

        Ok(previous)
    }

    fn expected_type(&self, _context: &Context) -> Result<Option<Type>, ValidationError> {
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
            Statement::Expression(Expression::Value(
                ValueNode::Integer(1).with_position((0, 0)),
            )),
            Statement::Expression(Expression::Value(
                ValueNode::Integer(2).with_position((0, 0)),
            )),
            Statement::Expression(Expression::Value(
                ValueNode::Integer(42).with_position((0, 0)),
            )),
        ]);

        assert_eq!(
            block.run(&mut Context::new(None), true).unwrap(),
            Some(Evaluation::Return(Value::integer(42)))
        )
    }

    #[test]
    fn expected_type_returns_type_of_final_statement() {
        let block = Block::new(vec![
            Statement::Expression(Expression::Value(
                ValueNode::String("42".to_string()).with_position((0, 0)),
            )),
            Statement::Expression(Expression::Value(
                ValueNode::Integer(42).with_position((0, 0)),
            )),
        ]);

        assert_eq!(
            block.expected_type(&mut Context::new(None)),
            Ok(Type::Integer)
        )
    }
}
