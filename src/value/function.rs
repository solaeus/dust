use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{AbstractTree, Block, Expression, Identifier, Map, Result, TypeDefinition, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Function {
    parameters: Vec<(Identifier, TypeDefinition)>,
    body: Block,
    return_type: TypeDefinition,
}

impl Function {
    pub fn new(
        parameters: Vec<(Identifier, TypeDefinition)>,
        body: Block,
        return_type: TypeDefinition,
    ) -> Self {
        Self {
            parameters,
            body,
            return_type,
        }
    }

    pub fn parameters(&self) -> &Vec<(Identifier, TypeDefinition)> {
        &self.parameters
    }

    pub fn body(&self) -> &Block {
        &self.body
    }

    pub fn return_type(&self) -> &TypeDefinition {
        &self.return_type
    }

    pub fn call(&self, arguments: &[Expression], source: &str, context: &Map) -> Result<Value> {
        let function_context = Map::new();
        let parameter_argument_pairs = self.parameters.iter().zip(arguments.iter());

        for ((identifier, type_definition), expression) in parameter_argument_pairs {
            let key = identifier.inner();
            let value = expression.run(source, context)?;

            println!("{key} {value}");

            type_definition.runtime_check(&value.r#type(context)?, context)?;
            function_context.variables_mut()?.insert(key.clone(), value);
        }

        let return_value = self.body.run(source, &function_context)?;

        self.return_type
            .runtime_check(&return_value.r#type(context)?, context)?;

        Ok(return_value)
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
