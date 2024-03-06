use crate::{
    error::{RuntimeError, ValidationError},
    value::Value,
    Context,
};

use super::{AbstractTree, Identifier, Statement, Type};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Assignment<'src> {
    identifier: Identifier,
    r#type: Option<Type>,
    statement: Box<Statement<'src>>,
}

impl<'src> Assignment<'src> {
    pub fn new(identifier: Identifier, r#type: Option<Type>, statement: Statement<'src>) -> Self {
        Self {
            identifier,
            r#type,
            statement: Box::new(statement),
        }
    }
}

impl<'src> AbstractTree for Assignment<'src> {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        todo!()
    }

    fn validate(&self, context: &Context) -> Result<(), ValidationError> {
        let statement_type = self.statement.expected_type(context)?;

        if let Some(expected) = &self.r#type {
            expected.check(&statement_type)?;

            context.set_type(self.identifier.clone(), expected.clone())?;
        } else {
            context.set_type(self.identifier.clone(), statement_type)?;
        }

        Ok(())
    }

    fn run(self, context: &Context) -> Result<Value, RuntimeError> {
        let value = self.statement.run(context)?;

        context.set_value(self.identifier, value)?;

        Ok(Value::none())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        abstract_tree::{Expression, ValueNode},
        error::TypeCheckError,
    };

    use super::*;

    #[test]
    fn assign_value() {
        let context = Context::new();

        Assignment::new(
            Identifier::new("foobar"),
            None,
            Statement::Expression(Expression::Value(ValueNode::Integer(42))),
        )
        .run(&context)
        .unwrap();

        assert_eq!(
            context.get_value(&Identifier::new("foobar")),
            Ok(Some(Value::integer(42)))
        )
    }

    #[test]
    fn type_check() {
        let validation = Assignment::new(
            Identifier::new("foobar"),
            Some(Type::Boolean),
            Statement::Expression(Expression::Value(ValueNode::Integer(42))),
        )
        .validate(&Context::new());

        assert_eq!(
            validation,
            Err(ValidationError::TypeCheck(TypeCheckError {
                actual: Type::Integer,
                expected: Type::Boolean
            }))
        )
    }
}
