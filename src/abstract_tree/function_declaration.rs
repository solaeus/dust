use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{
    AbstractTree, Block, Error, Function, Identifier, Map, Result, Type, TypeDefinition, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct FunctionDeclaration {
    name: Identifier,
    _type_definition: TypeDefinition,
    function: Function,
}

impl AbstractTree for FunctionDeclaration {
    fn from_syntax_node(source: &str, node: Node, context: &Map) -> Result<Self> {
        Error::expect_syntax_node(source, "function_declaration", node)?;

        let name_node = node.child(0).unwrap();
        let name = Identifier::from_syntax_node(source, name_node, context)?;

        let type_node = node.child(1).unwrap();
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

        let function = {
            let function_node = node.child(2).unwrap();

            Error::expect_syntax_node(source, "function", function_node)?;

            let child_count = function_node.child_count();
            let mut parameters = Vec::new();

            for index in 1..child_count - 2 {
                let child = function_node.child(index).unwrap();

                let parameter_index = parameters.len();
                let parameter_type = parameter_types.get(parameter_index).unwrap_or(&Type::Empty);

                if child.is_named() {
                    let identifier = Identifier::from_syntax_node(source, child, context)?;
                    parameters.push((identifier, TypeDefinition::new(parameter_type.clone())));
                }
            }

            let body_node = function_node.child(child_count - 1).unwrap();
            let body = Block::from_syntax_node(source, body_node, context)?;

            Function::new(
                parameters,
                body,
                TypeDefinition::new(return_type.as_ref().clone()),
            )
        };

        Ok(FunctionDeclaration {
            name,
            _type_definition: type_definition,
            function,
        })
    }

    fn run(&self, _source: &str, context: &Map) -> Result<Value> {
        let key = self.name.inner();

        context
            .variables_mut()?
            .insert(key.clone(), Value::Function(self.function.clone()));

        Ok(Value::Empty)
    }

    fn expected_type(&self, _context: &Map) -> Result<TypeDefinition> {
        Ok(TypeDefinition::new(Type::Empty))
    }
}

#[cfg(test)]
mod tests {
    use crate::{evaluate, Value};

    #[test]
    fn simple_function_declaration() {
        let test = evaluate(
            "
                fn foo <fn int -> int> = |x| { x }
                (foo 42)
            ",
        )
        .unwrap();

        assert_eq!(Value::Integer(42), test);
    }
}
