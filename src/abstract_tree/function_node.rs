use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Block, Format, Function, Identifier, Map, SourcePosition, SyntaxNode, Type,
    TypeSpecification, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct FunctionNode {
    parameters: Vec<Identifier>,
    body: Block,
    r#type: Type,
    syntax_position: SourcePosition,
}

impl FunctionNode {
    pub fn new(
        parameters: Vec<Identifier>,
        body: Block,
        r#type: Type,
        syntax_position: SourcePosition,
    ) -> Self {
        FunctionNode {
            parameters,
            body,
            r#type,
            syntax_position,
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

    pub fn syntax_position(&self) -> &SourcePosition {
        &self.syntax_position
    }

    pub fn return_type(&self) -> &Type {
        match &self.r#type {
            Type::Function {
                parameter_types: _,
                return_type,
            } => return_type.as_ref(),
            _ => &Type::None,
        }
    }

    pub fn call(
        &self,
        arguments: &[Value],
        source: &str,
        outer_context: &Map,
    ) -> Result<Value, RuntimeError> {
        let function_context = Map::new();
        let parameter_argument_pairs = self.parameters.iter().zip(arguments.iter());

        for (key, (value, r#type)) in outer_context.variables()?.iter() {
            if r#type.is_function() {
                function_context.set(key.clone(), value.clone())?;
            }
        }

        for (identifier, value) in parameter_argument_pairs {
            let key = identifier.inner().clone();

            function_context.set(key, value.clone())?;
        }

        let return_value = self.body.run(source, &function_context)?;

        Ok(return_value)
    }
}

impl AbstractTree for FunctionNode {
    fn from_syntax(
        node: SyntaxNode,
        source: &str,
        outer_context: &Map,
    ) -> Result<Self, SyntaxError> {
        SyntaxError::expect_syntax_node(source, "function", node)?;

        let child_count = node.child_count();
        let mut parameters = Vec::new();
        let mut parameter_types = Vec::new();

        for index in 1..child_count - 3 {
            let child = node.child(index).unwrap();

            if child.kind() == "identifier" {
                let identifier = Identifier::from_syntax(child, source, outer_context)?;

                parameters.push(identifier);
            }

            if child.kind() == "type_specification" {
                let type_specification =
                    TypeSpecification::from_syntax(child, source, outer_context)?;

                parameter_types.push(type_specification.take_inner());
            }
        }

        let return_type_node = node.child(child_count - 2).unwrap();
        let return_type = TypeSpecification::from_syntax(return_type_node, source, outer_context)?;

        let function_context = Map::new();

        for (parameter, parameter_type) in parameters.iter().zip(parameter_types.iter()) {
            function_context.set_type(parameter.inner().clone(), parameter_type.clone())?;
        }

        for (key, (value, r#type)) in outer_context.variables()?.iter() {
            if r#type.is_function() {
                function_context.set(key.clone(), value.clone())?;
            }
        }

        let body_node = node.child(child_count - 1).unwrap();
        let body = Block::from_syntax(body_node, source, &function_context)?;

        let r#type = Type::function(parameter_types, return_type.take_inner());
        let syntax_position = node.range().into();

        Ok(FunctionNode {
            parameters,
            body,
            r#type,
            syntax_position,
        })
    }

    fn expected_type(&self, _context: &Map) -> Result<Type, ValidationError> {
        Ok(self.r#type().clone())
    }

    fn check_type(&self, source: &str, context: &Map) -> Result<(), ValidationError> {
        let function_context = Map::new();

        for (key, (_value, r#type)) in context.variables()?.iter() {
            if r#type.is_function() {
                function_context.set_type(key.clone(), r#type.clone())?;
            }
        }

        if let Type::Function {
            parameter_types,
            return_type,
        } = &self.r#type
        {
            for (parameter, parameter_type) in self.parameters.iter().zip(parameter_types.iter()) {
                function_context.set_type(parameter.inner().clone(), parameter_type.clone())?;
            }

            let actual = self.body.expected_type(&function_context)?;

            if !return_type.accepts(&actual) {
                return Err(ValidationError::TypeCheck {
                    expected: return_type.as_ref().clone(),
                    actual,
                    position: self.syntax_position,
                });
            }

            self.body.check_type(source, &function_context)?;

            Ok(())
        } else {
            Err(ValidationError::TypeCheckExpectedFunction {
                actual: self.r#type.clone(),
                position: self.syntax_position,
            })
        }
    }

    fn run(&self, _source: &str, _context: &Map) -> Result<Value, RuntimeError> {
        let self_as_value = Value::Function(Function::ContextDefined(self.clone()));

        Ok(self_as_value)
    }
}

impl Format for FunctionNode {
    fn format(&self, output: &mut String, indent_level: u8) {
        let (parameter_types, return_type) = if let Type::Function {
            parameter_types,
            return_type,
        } = &self.r#type
        {
            (parameter_types, return_type)
        } else {
            return;
        };

        output.push('(');

        for (identifier, r#type) in self.parameters.iter().zip(parameter_types.iter()) {
            identifier.format(output, indent_level);
            output.push_str(" <");
            r#type.format(output, indent_level);
            output.push('>');
        }

        output.push_str(") <");
        return_type.format(output, indent_level);
        output.push_str("> ");
        self.body.format(output, indent_level);
    }
}

impl Display for FunctionNode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut string = String::new();

        self.format(&mut string, 0);
        f.write_str(&string)
    }
}
