use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    identifier::Identifier,
};

use super::{AbstractNode, Evaluation, SourcePosition, Type, TypeConstructor};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StructureDefinition {
    name: Identifier,
    fields: Vec<(Identifier, TypeConstructor)>,
}

impl StructureDefinition {
    pub fn new(name: Identifier, fields: Vec<(Identifier, TypeConstructor)>) -> Self {
        Self { name, fields }
    }
}

impl AbstractNode for StructureDefinition {
    fn define_and_validate(
        &self,
        context: &Context,
        _: bool,
        scope: SourcePosition,
    ) -> Result<(), ValidationError> {
        let mut fields = Vec::with_capacity(self.fields.len());

        for (identifier, constructor) in &self.fields {
            let r#type = constructor.construct(&context)?;

            fields.push((identifier.clone(), r#type));
        }

        let struct_type = Type::Structure {
            name: self.name.clone(),
            fields,
        };

        context.set_type(self.name.clone(), struct_type, scope)?;

        Ok(())
    }

    fn evaluate(
        self,
        context: &Context,
        _manage_memory: bool,
        scope: SourcePosition,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        let mut fields = Vec::with_capacity(self.fields.len());

        for (identifier, constructor) in self.fields {
            let r#type = constructor.construct(&context)?;

            fields.push((identifier, r#type));
        }

        let struct_type = Type::Structure {
            name: self.name.clone(),
            fields,
        };

        context.set_type(self.name, struct_type, scope)?;

        Ok(None)
    }

    fn expected_type(&self, _: &Context) -> Result<Option<Type>, ValidationError> {
        Ok(None)
    }
}

impl Display for StructureDefinition {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let StructureDefinition { name, fields } = self;

        write!(f, "struct {name} {{ ")?;

        for (identifer, constructor) in fields {
            write!(f, "{identifer}: {constructor}, ")?;
        }

        write!(f, " }}")
    }
}
