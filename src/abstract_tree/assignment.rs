use crate::{error::RuntimeError, Context};

use super::{AbstractTree, Identifier, Statement, Value};

#[derive(Clone, Debug, PartialEq)]
pub struct Assignment {
    identifier: Identifier,
    statement: Box<Statement>,
}

impl Assignment {
    pub fn new(identifier: Identifier, statement: Statement) -> Self {
        Self {
            identifier,
            statement: Box::new(statement),
        }
    }
}

impl AbstractTree for Assignment {
    fn run(self, context: &Context) -> Result<Value, RuntimeError> {
        let value = self.statement.run(context)?;

        context.set(self.identifier, value)?;

        Ok(Value::none())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assign_value() {
        let context = Context::new();

        Assignment::new(
            Identifier::new("foobar"),
            Statement::Value(Value::integer(42)),
        )
        .run(&context)
        .unwrap();

        assert_eq!(
            context.get(&Identifier::new("foobar")).unwrap(),
            Some(Value::integer(42))
        )
    }
}
