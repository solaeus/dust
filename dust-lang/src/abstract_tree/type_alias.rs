use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    identifier::Identifier,
};

use super::{AbstractNode, Evaluation, SourcePosition, Type, TypeConstructor, WithPosition};

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
    fn define_and_validate(
        &self,
        context: &Context,
        _: bool,
        scope: SourcePosition,
    ) -> Result<(), ValidationError> {
        let r#type = self.constructor.construct(context)?;

        context.set_type(self.identifier.node.clone(), r#type, scope)?;

        Ok(())
    }

    fn evaluate(
        self,
        _: &Context,
        _: bool,
        _: SourcePosition,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        Ok(None)
    }

    fn expected_type(&self, _: &Context) -> Result<Option<Type>, ValidationError> {
        Ok(None)
    }
}

impl Display for TypeAlias {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let TypeAlias {
            identifier,
            constructor,
        } = self;

        write!(f, "type {} = {constructor}", identifier.node)
    }
}
