use serde::{Deserialize, Serialize};

use crate::{context::Context, error::RuntimeError, identifier::Identifier};

use super::{Evaluation, Run, Type, TypeConstructor};

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

impl Run for StructureDefinition {
    fn run(
        self,
        context: &mut Context,
        _manage_memory: bool,
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

        context.set_type(self.name, struct_type)?;

        Ok(None)
    }
}
