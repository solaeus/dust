use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    identifier::Identifier,
};

use super::{AbstractNode, Evaluation, SourcePosition, Type, TypeConstructor, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EnumDeclaration {
    name: WithPosition<Identifier>,
    type_parameters: Option<Vec<WithPosition<Identifier>>>,
    variants: Vec<EnumVariant>,
}

impl EnumDeclaration {
    pub fn new(
        name: WithPosition<Identifier>,
        type_parameters: Option<Vec<WithPosition<Identifier>>>,
        variants: Vec<EnumVariant>,
    ) -> Self {
        Self {
            name,
            type_parameters,
            variants,
        }
    }
}

impl AbstractNode for EnumDeclaration {
    fn define_and_validate(
        &self,
        context: &Context,
        _: bool,
        _scope: SourcePosition,
    ) -> Result<(), ValidationError> {
        let EnumDeclaration {
            name,
            type_parameters,
            variants,
        } = self;

        let type_parameters = type_parameters.as_ref().map(|parameters| {
            parameters
                .iter()
                .map(|identifier| Type::Generic {
                    identifier: identifier.node.clone(),
                    concrete_type: None,
                })
                .collect()
        });
        let mut type_variants = Vec::with_capacity(variants.len());

        for EnumVariant { name, content } in variants {
            let types = if let Some(content) = content {
                let mut types = Vec::with_capacity(content.len());

                for constructor in content {
                    let r#type = constructor.construct(context)?;

                    types.push(r#type);
                }

                Some(types)
            } else {
                None
            };

            type_variants.push((name.node.clone(), types));
        }

        let r#type = Type::Enum {
            name: name.node.clone(),
            type_parameters,
            variants: type_variants,
        };
        let final_node_position = if let Some(constructors) = &self.variants.last().unwrap().content
        {
            constructors.last().unwrap().position()
        } else {
            self.variants.last().unwrap().name.position
        };
        let scope = SourcePosition(self.name.position.0, final_node_position.1);

        context.set_type(name.node.clone(), r#type, scope)?;

        Ok(())
    }

    fn evaluate(
        self,
        _: &Context,
        _: bool,
        _scope: SourcePosition,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        Ok(None)
    }

    fn expected_type(&self, _: &Context) -> Result<Option<Type>, ValidationError> {
        Ok(None)
    }
}

impl Display for EnumDeclaration {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let EnumDeclaration {
            name,
            type_parameters,
            variants,
        } = self;

        write!(f, "enum {}", name.node)?;

        if let Some(parameters) = type_parameters {
            write!(f, "<")?;
            for WithPosition { node, .. } in parameters {
                write!(f, "{node}, ")?;
            }
            write!(f, ">")?;
        }

        for EnumVariant { name, content } in variants {
            write!(f, "{}", name.node)?;

            if let Some(content) = content {
                write!(f, "(")?;

                for constructor in content {
                    write!(f, "{constructor}, ")?;
                }

                write!(f, ")")?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EnumVariant {
    pub name: WithPosition<Identifier>,
    pub content: Option<Vec<TypeConstructor>>,
}
