use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{AbstractTree, Action, Statement, Type};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Loop<'src> {
    statements: Vec<Statement<'src>>,
}

impl<'src> Loop<'src> {
    pub fn new(statements: Vec<Statement<'src>>) -> Self {
        Self { statements }
    }
}

impl<'src> AbstractTree for Loop<'src> {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        Ok(Type::None)
    }

    fn validate(&self, _context: &Context) -> Result<(), ValidationError> {
        for statement in &self.statements {
            statement.validate(_context)?;
        }

        Ok(())
    }

    fn run(self, _context: &Context) -> Result<Action, RuntimeError> {
        let mut index = 0;

        loop {
            if index == self.statements.len() - 1 {
                index = 0;
            } else {
                index += 1;
            }

            let statement = self.statements[index].clone();

            if let Statement::Break(expression) = statement {
                break expression.run(_context);
            } else {
                statement.run(_context)?;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        abstract_tree::{Expression, ValueNode},
        Value,
    };

    use super::*;

    #[test]
    fn basic_loop() {
        let result = Loop {
            statements: vec![Statement::Break(Expression::Value(ValueNode::Integer(42)))],
        }
        .run(&Context::new());

        assert_eq!(result, Ok(Action::Return(Value::integer(42))))
    }
}
