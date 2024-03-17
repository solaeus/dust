use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{AbstractTree, Action, Block, Expression, Positioned, Type};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct IfElse {
    if_expression: Positioned<Expression>,
    if_block: Positioned<Block>,
    else_block: Option<Positioned<Block>>,
}

impl IfElse {
    pub fn new(
        if_expression: Positioned<Expression>,
        if_block: Positioned<Block>,
        else_block: Option<Positioned<Block>>,
    ) -> Self {
        Self {
            if_expression,
            if_block,
            else_block,
        }
    }
}

impl AbstractTree for IfElse {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        self.if_block.node.expected_type(_context)
    }

    fn validate(&self, context: &Context) -> Result<(), ValidationError> {
        if let Type::Boolean = self.if_expression.node.expected_type(context)? {
            if let Some(else_block) = &self.else_block {
                let expected = self.if_block.node.expected_type(context)?;
                let actual = else_block.node.expected_type(context)?;

                expected
                    .check(&actual)
                    .map_err(|conflict| ValidationError::TypeCheck {
                        conflict,
                        actual_position: self.if_block.position,
                        expected_position: self.if_expression.position,
                    })?;
            }

            Ok(())
        } else {
            Err(ValidationError::ExpectedBoolean)
        }
    }

    fn run(self, _context: &Context) -> Result<Action, RuntimeError> {
        let if_boolean = self
            .if_expression
            .node
            .run(_context)?
            .as_return_value()?
            .as_boolean()?;

        if if_boolean {
            self.if_block.node.run(_context)
        } else if let Some(else_statement) = self.else_block {
            else_statement.node.run(_context)
        } else {
            Ok(Action::None)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        abstract_tree::{Statement, ValueNode},
        Value,
    };

    use super::*;

    #[test]
    fn simple_if() {
        assert_eq!(
            IfElse::new(
                Expression::Value(ValueNode::Boolean(true)).positioned((0..0).into()),
                Block::new(vec![Statement::Expression(
                    Expression::Value(ValueNode::String("foo".to_string()))
                        .positioned((0..0).into())
                )
                .positioned((0..0).into())])
                .positioned((0..0).into()),
                None
            )
            .run(&Context::new()),
            Ok(Action::Return(Value::string("foo".to_string())))
        )
    }

    #[test]
    fn simple_if_else() {
        assert_eq!(
            IfElse::new(
                Expression::Value(ValueNode::Boolean(false)).positioned((0..0).into()),
                Block::new(vec![Statement::Expression(
                    Expression::Value(ValueNode::String("foo".to_string()))
                        .positioned((0..0).into())
                )
                .positioned((0..0).into())])
                .positioned((0..0).into()),
                Some(
                    Block::new(vec![Statement::Expression(
                        Expression::Value(ValueNode::String("bar".to_string()))
                            .positioned((0..0).into())
                    )
                    .positioned((0..0).into())])
                    .positioned((0..0).into())
                )
            )
            .run(&Context::new()),
            Ok(Action::Return(Value::string("bar".to_string())))
        )
    }
}
