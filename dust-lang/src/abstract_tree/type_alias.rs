use serde::{Deserialize, Serialize};

use crate::{context::Context, error::RuntimeError, identifier::Identifier};

use super::{Evaluation, Run, TypeConstructor, WithPosition};

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

impl Run for TypeAlias {
    fn run(
        self,
        context: &mut Context,
        _manage_memory: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        let r#type = self.constructor.construct(&context)?;

        context.set_type(self.identifier.node, r#type)?;

        Ok(None)
    }
}
