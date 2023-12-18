use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{AbstractTree, Block, Error, Identifier, Map, Result, Type, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialOrd, Ord)]
pub struct Function {
    parameters: Vec<Identifier>,
    body: Block,
    r#type: Type,
    context: Map,
}

impl Function {
    pub fn new(
        parameters: Vec<Identifier>,
        body: Block,
        r#type: Option<Type>,
        context: Map,
    ) -> Self {
        let r#type = r#type.unwrap_or(Type::Function {
            parameter_types: vec![Type::Any; parameters.len()],
            return_type: Box::new(Type::Any),
        });

        Self {
            parameters,
            body,
            r#type,
            context,
        }
    }

    pub fn parameters(&self) -> &Vec<Identifier> {
        &self.parameters
    }

    pub fn body(&self) -> &Block {
        &self.body
    }

    pub fn r#type(&self) -> &Type {
        &self.r#type
    }

    pub fn return_type(&self) -> Result<&Type> {
        match &self.r#type {
            Type::Function {
                parameter_types: _,
                return_type,
            } => Ok(return_type.as_ref()),
            _ => todo!(),
        }
    }

    pub fn call(&self, arguments: &[Value], source: &str, context: &Map) -> Result<Value> {
        if self.parameters.len() != arguments.len() {
            return Err(Error::ExpectedArgumentAmount {
                function_name: "",
                expected: self.parameters.len(),
                actual: arguments.len(),
            });
        }

        for (key, (value, r#type)) in context.variables()?.iter() {
            if self.context.variables()?.contains_key(key) {
                continue;
            }

            self.context
                .set(key.clone(), value.clone(), Some(r#type.clone()))?;
        }

        let parameter_argument_pairs = self.parameters.iter().zip(arguments.iter());

        for (identifier, value) in parameter_argument_pairs {
            let key = identifier.inner().clone();

            self.context.set(key, value.clone(), None)?;
        }

        let return_value = self.body.run(source, &self.context)?;

        Ok(return_value)
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.r#type.eq(&other.r#type)
            && self.parameters.eq(&other.parameters)
            && self.body.eq(&other.body)
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Function {{ parameters: {:?}, body: {:?} }}",
            self.parameters, self.body
        )
    }
}
