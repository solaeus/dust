use super::{AbstractNode, Expression, Type, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct As {
    expression: Expression,
    r#type: WithPosition<Type>,
}

impl As {
    pub fn new(expression: Expression, r#type: WithPosition<Type>) -> Self {
        Self { expression, r#type }
    }
}

impl AbstractNode for As {
    fn expected_type(
        &self,
        context: &mut crate::context::Context,
    ) -> Result<Type, crate::error::ValidationError> {
        todo!()
    }

    fn validate(
        &self,
        context: &mut crate::context::Context,
        manage_memory: bool,
    ) -> Result<(), crate::error::ValidationError> {
        todo!()
    }

    fn run(
        self,
        context: &mut crate::context::Context,
        manage_memory: bool,
    ) -> Result<super::Action, crate::error::RuntimeError> {
        todo!()
    }
}
