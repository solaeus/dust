use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{AbstractTree, Action, Positioned, Statement, Type};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Loop {
    statements: Vec<Positioned<Statement>>,
}

impl Loop {
    pub fn new(statements: Vec<Positioned<Statement>>) -> Self {
        Self { statements }
    }
}

impl AbstractTree for Loop {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        Ok(Type::None)
    }

    fn validate(&self, _context: &Context) -> Result<(), ValidationError> {
        for statement in &self.statements {
            statement.node.validate(_context)?;
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
            let action = statement.node.run(_context)?;

            match action {
                Action::Return(_) => {}
                Action::None => {}
                r#break => return Ok(r#break),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_loop() {
        let result = Loop {
            statements: vec![Statement::Break.positioned((0..0).into())],
        }
        .run(&Context::new());

        assert_eq!(result, Ok(Action::Break))
    }
}
