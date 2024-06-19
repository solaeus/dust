use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    identifier::Identifier,
};

use super::{Evaluate, Evaluation, TypeConstructor, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TypeAssignment {
    identifier: WithPosition<Identifier>,
    constructor: TypeConstructor,
}

impl TypeAssignment {
    pub fn new(identifier: WithPosition<Identifier>, constructor: TypeConstructor) -> Self {
        Self {
            identifier,
            constructor,
        }
    }
}

impl Evaluate for TypeAssignment {
    fn validate(&self, context: &mut Context, _manage_memory: bool) -> Result<(), ValidationError> {
        let r#type = self.constructor.clone().construct(&context)?;

        context.set_type(self.identifier.node.clone(), r#type)?;

        Ok(())
    }

    fn evaluate(
        self,
        context: &mut Context,
        _manage_memory: bool,
    ) -> Result<Evaluation, RuntimeError> {
        let r#type = self.constructor.construct(&context)?;

        context.set_type(self.identifier.node, r#type)?;

        Ok(Evaluation::None)
    }
}
