use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{
    AbstractTree, Block, Error, Expression, Identifier, Map, Result, Type, TypeDefinition, Value,
};

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

    pub fn r#type(&self) -> TypeDefinition {
        let mut parameter_types = Vec::with_capacity(self.parameters.len());

        for (_, type_definition) in &self.parameters {
            parameter_types.push(type_definition.inner().clone());
        }

        TypeDefinition::new(Type::Function {
            parameter_types,
            return_type: Box::new(self.return_type.inner().clone()),
        })
    }

    pub fn call(&self, arguments: &[Expression], source: &str, context: &Map) -> Result<Value> {
        let function_context = Map::clone_from(context)?;
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

impl AbstractTree for Function {
    fn from_syntax_node(source: &str, node: Node, context: &Map) -> Result<Self> {
        Error::expect_syntax_node(source, "function", node)?;

        let type_node = node.child(0).unwrap();
        let type_definition = TypeDefinition::from_syntax_node(source, type_node, context)?;

        let (parameter_types, return_type) = if let Type::Function {
            parameter_types,
            return_type,
        } = type_definition.inner()
        {
            (parameter_types, return_type)
        } else {
            return Err(Error::TypeCheck {
                expected: TypeDefinition::new(Type::Function {
                    parameter_types: Vec::with_capacity(0),
                    return_type: Box::new(Type::Empty),
                }),
                actual: type_definition,
                location: type_node.start_position(),
                source: source[type_node.byte_range()].to_string(),
            });
        };

        let child_count = node.child_count();
        let mut parameters = Vec::new();

        for index in 2..child_count - 2 {
            let child = node.child(index).unwrap();

            let parameter_index = parameters.len();
            let parameter_type = parameter_types.get(parameter_index).unwrap_or(&Type::Empty);

            if child.is_named() {
                let identifier = Identifier::from_syntax_node(source, child, context)?;
                parameters.push((identifier, TypeDefinition::new(parameter_type.clone())));
            }
        }

        let body_node = node.child(child_count - 1).unwrap();
        let body = Block::from_syntax_node(source, body_node, context)?;

        Ok(Function::new(
            parameters,
            body,
            TypeDefinition::new(return_type.as_ref().clone()),
        ))
    }

    fn run(&self, _source: &str, _context: &Map) -> Result<Value> {
        Ok(Value::Function(self.clone()))
    }

    fn expected_type(&self, context: &Map) -> Result<TypeDefinition> {
        Value::Function(self.clone()).r#type(context)
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Value::Function(self.clone()))?;
        write!(
            f,
            "Function {{ parameters: {:?}, body: {:?} }}",
            self.parameters, self.body
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{evaluate, Value};

    #[test]
    fn simple_function_declaration() {
        let test = evaluate(
            "
                foo = <fn int -> int> |x| { x }
                (foo 42)
            ",
        )
        .unwrap();

        assert_eq!(Value::Integer(42), test);
    }
}
