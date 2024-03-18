use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
    Value,
};

use super::{AbstractTree, Action, Expression, Statement, Type, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct While {
    expression: WithPosition<Expression>,
    statements: Vec<WithPosition<Statement>>,
}

impl While {
    pub fn new(
        expression: WithPosition<Expression>,
        statements: Vec<WithPosition<Statement>>,
    ) -> Self {
        Self {
            expression,
            statements,
        }
    }
}

impl AbstractTree for While {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        Ok(Type::None)
    }

    fn validate(&self, _context: &Context) -> Result<(), ValidationError> {
        self.expression.node.validate(_context)?;

        for statement in &self.statements {
            statement.node.validate(_context)?;
        }

        Ok(())
    }

    fn run(self, _context: &Context) -> Result<Action, RuntimeError> {
        let get_boolean = || -> Result<Value, RuntimeError> {
            let action = self.expression.node.run(_context)?;

            if let Action::Return(value) = action {
                Ok(value)
            } else {
                Err(RuntimeError::ValidationFailure(
                    ValidationError::InterpreterExpectedReturn(self.expression.position),
                ))
            }
        };

        if let ValueInner::Boolean(boolean) = get_boolean()?.inner().as_ref() {
            while *boolean {
                for statement in &self.statements {
                    let action = statement.node.clone().run(_context)?;

                    match action {
                        Action::Return(_) => {}
                        Action::None => {}
                        Action::Break => return Ok(Action::Break),
                    }
                }
            }
        }

        Ok(Action::None)
    }
}

#[cfg(test)]
mod tests {
    use crate::abstract_tree::{
        Assignment, AssignmentOperator, Block, Identifier, Logic, ValueNode,
    };

    use super::*;

    #[test]
    fn simple_while_loop() {
        let action = Statement::Block(Block::new(vec![
            Statement::Assignment(Assignment::new(
                Identifier::new("i").with_position((0, 0)),
                None,
                AssignmentOperator::Assign,
                Statement::Expression(Expression::Value(ValueNode::Integer(3)))
                    .with_position((0, 0)),
            ))
            .with_position((0, 0)),
            Statement::While(While {
                expression: Expression::Logic(Box::new(Logic::Less(
                    Expression::Identifier(Identifier::new("i")).with_position((0, 0)),
                    Expression::Value(ValueNode::Integer(3)).with_position((0, 0)),
                )))
                .with_position((0, 0)),
                statements: vec![Statement::Assignment(Assignment::new(
                    Identifier::new("i").with_position((0, 0)),
                    None,
                    AssignmentOperator::AddAssign,
                    Statement::Expression(Expression::Value(ValueNode::Integer(1)))
                        .with_position((0, 0)),
                ))
                .with_position((0, 0))],
            })
            .with_position((0, 0)),
            Statement::Expression(Expression::Identifier(Identifier::new("i")))
                .with_position((0, 0)),
        ]))
        .run(&Context::new())
        .unwrap();

        assert_eq!(action, Action::Return(Value::integer(3)))
    }
}
