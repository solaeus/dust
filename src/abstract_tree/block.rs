use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    Value,
};

use super::{AbstractTree, Action, Statement, Type};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Block<'src> {
    statements: Vec<Statement<'src>>,
}

impl<'src> Block<'src> {
    pub fn new(statements: Vec<Statement<'src>>) -> Self {
        Self { statements }
    }
}

impl<'src> AbstractTree for Block<'src> {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        let final_statement = self.statements.last().unwrap();

        final_statement.expected_type(_context)
    }

    fn validate(&self, _context: &Context) -> Result<(), ValidationError> {
        for statement in &self.statements {
            statement.validate(_context)?;
        }

        Ok(())
    }

    fn run(self, _context: &Context) -> Result<Action, RuntimeError> {
        let mut previous = Value::none();

        for statement in self.statements {
            let action = statement.run(_context)?;
            previous = match action {
                Action::Return(value) => value,
                r#break => return Ok(r#break),
            };
        }

        Ok(Action::Return(previous))
    }
}

#[cfg(test)]
mod tests {
    use crate::abstract_tree::{Expression, ValueNode};

    use super::*;

    #[test]
    fn run_returns_value_of_final_statement() {
        let block = Block::new(vec![
            Statement::Expression(Expression::Value(ValueNode::Integer(1))),
            Statement::Expression(Expression::Value(ValueNode::Integer(2))),
            Statement::Expression(Expression::Value(ValueNode::Integer(42))),
        ]);

        assert_eq!(
            block.run(&Context::new()),
            Ok(Action::Return(Value::integer(42)))
        )
    }

    #[test]
    fn expected_type_returns_type_of_final_statement() {
        let block = Block::new(vec![
            Statement::Expression(Expression::Value(ValueNode::String("42"))),
            Statement::Expression(Expression::Value(ValueNode::Integer(42))),
        ]);

        assert_eq!(block.expected_type(&Context::new()), Ok(Type::Integer))
    }
}
