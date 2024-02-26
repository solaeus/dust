use crate::{error::RuntimeError, value::Value, Context};

use super::{AbstractTree, Identifier, Statement};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Assignment<'src> {
    identifier: Identifier<'src>,
    statement: Box<Statement<'src>>,
}

impl<'src> Assignment<'src> {
    pub fn new(identifier: Identifier<'src>, statement: Statement<'src>) -> Self {
        Self {
            identifier,
            statement: Box::new(statement),
        }
    }
}

impl<'src> AbstractTree for Assignment<'src> {
    fn run(self, _context: &Context) -> Result<Value, RuntimeError> {
        todo!()
        // let value = self.statement.run(context)?;

        // context.set(self.identifier, value)?;

        // Ok(Value::none())
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn assign_value() {
        todo!()
        // let context = Context::new();

        // Assignment::new(
        //     Identifier::new("foobar"),
        //     Statement::Value(Value::integer(42)),
        // )
        // .run(&context)
        // .unwrap();

        // assert_eq!(
        //     context.get(&Identifier::new("foobar")).unwrap(),
        //     Some(Value::integer(42))
        // )
    }
}
