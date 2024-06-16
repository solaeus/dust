use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    identifier::Identifier,
};

use super::{AbstractNode, Action, Type, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TypeAssignment {
    identifier: WithPosition<Identifier>,
    r#type: WithPosition<Type>,
}

impl TypeAssignment {
    pub fn new(identifier: WithPosition<Identifier>, r#type: WithPosition<Type>) -> Self {
        Self { identifier, r#type }
    }
}

impl AbstractNode for TypeAssignment {
    fn validate(
        &self,
        _context: &mut Context,
        _manage_memory: bool,
    ) -> Result<(), ValidationError> {
        Ok(())
    }

    fn run(self, context: &mut Context, _manage_memory: bool) -> Result<Action, RuntimeError> {
        context.set_type(self.identifier.node, self.r#type.node)?;

        Ok(Action::None)
    }
}
