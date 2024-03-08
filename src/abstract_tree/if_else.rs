use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{AbstractTree, Action, Expression, Statement, Type};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct IfElse<'src> {
    if_expression: Expression<'src>,
    if_statement: Box<Statement<'src>>,
    else_statement: Option<Box<Statement<'src>>>,
}

impl<'src> IfElse<'src> {
    pub fn new(
        if_expression: Expression<'src>,
        if_statement: Statement<'src>,
        else_statement: Option<Statement<'src>>,
    ) -> Self {
        Self {
            if_expression,
            if_statement: Box::new(if_statement),
            else_statement: else_statement.map(|statement| Box::new(statement)),
        }
    }
}

impl<'src> AbstractTree for IfElse<'src> {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        todo!()
    }

    fn validate(&self, _context: &Context) -> Result<(), ValidationError> {
        todo!()
    }

    fn run(self, _context: &Context) -> Result<Action, RuntimeError> {
        let if_boolean = self
            .if_expression
            .run(_context)?
            .as_return_value()?
            .as_boolean()?;

        if if_boolean {
            self.if_statement.run(_context)
        } else if let Some(else_statement) = self.else_statement {
            else_statement.run(_context)
        } else {
            Ok(Action::None)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        abstract_tree::{Action, ValueNode},
        context::Context,
        Value,
    };

    use super::*;

    #[test]
    fn simple_if() {
        assert_eq!(
            IfElse::new(
                Expression::Value(ValueNode::Boolean(true)),
                Statement::Expression(Expression::Value(ValueNode::String("foo"))),
                None
            )
            .run(&Context::new()),
            Ok(Action::Return(Value::string("foo")))
        )
    }

    #[test]
    fn simple_if_else() {
        assert_eq!(
            IfElse::new(
                Expression::Value(ValueNode::Boolean(false)),
                Statement::Expression(Expression::Value(ValueNode::String("foo"))),
                Some(Statement::Expression(Expression::Value(ValueNode::String(
                    "bar"
                ))))
            )
            .run(&Context::new()),
            Ok(Action::Return(Value::string("bar")))
        )
    }
}
