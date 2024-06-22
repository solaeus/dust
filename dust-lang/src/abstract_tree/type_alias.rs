use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    identifier::Identifier,
};

use super::{AbstractNode, DefineTypes, Evaluation, Type, TypeConstructor, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TypeAlias {
    identifier: WithPosition<Identifier>,
    constructor: TypeConstructor,
}

impl TypeAlias {
    pub fn new(identifier: WithPosition<Identifier>, constructor: TypeConstructor) -> Self {
        Self {
            identifier,
            constructor,
        }
    }
}

impl AbstractNode for TypeAlias {
    fn evaluate(
        self,
        context: &mut Context,
        manage_memory: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        Ok(None)
    }

    fn expected_type(&self, context: &mut Context) -> Result<Option<Type>, ValidationError> {
        Ok(None)
    }
}

impl DefineTypes for TypeAlias {
    fn define_types(&self, context: &Context) -> Result<(), ValidationError> {
        let r#type = self.constructor.construct(&context)?;

        context.set_type(self.identifier.node, r#type)?;

        Ok(())
    }
}
