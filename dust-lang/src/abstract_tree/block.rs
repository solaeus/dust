use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{AbstractNode, Action, Statement, Type};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Block {
    statements: Vec<Statement>,
}

impl Block {
    pub fn new(statements: Vec<Statement>) -> Self {
        Self { statements }
    }

    pub fn last_statement(&self) -> &Statement {
        self.statements.last().unwrap()
    }
}

impl AbstractNode for Block {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        if let Some(statement) = self.statements.last() {
            statement.expected_type(_context)
        } else {
            Ok(Type::None)
        }
    }

    fn validate(&self, _context: &Context, _manage_memory: bool) -> Result<(), ValidationError> {
        for statement in &self.statements {
            statement.validate(_context, _manage_memory)?;
        }

        Ok(())
    }

    fn run(self, _context: &mut Context, _manage_memory: bool) -> Result<Action, RuntimeError> {
        let mut previous = Action::None;

        for statement in self.statements {
            previous = statement.run(_context, _manage_memory)?;
        }

        Ok(previous)
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
            block.run(&mut Context::new(), true).unwrap(),
            Action::Return(Value::integer(42))
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

        assert_eq!(block.expected_type(&Context::new()), Ok(Type::Integer))
    }
}
