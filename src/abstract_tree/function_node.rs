use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Block, Context, Format, Function, Identifier, SourcePosition, SyntaxNode, Type,
    TypeSpecification, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct FunctionNode {
    parameters: Vec<Identifier>,
    body: Block,
    r#type: Type,
    syntax_position: SourcePosition,

    #[serde(skip)]
    context: Context,
}

impl FunctionNode {
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

    pub fn context(&self) -> &Context {
        &self.context
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
}

impl AbstractTree for FunctionNode {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Context) -> Result<Self, SyntaxError> {
        SyntaxError::expect_syntax_node("function", node)?;

        let child_count = node.child_count();
        let mut parameters = Vec::new();
        let mut parameter_types = Vec::new();

        for index in 1..child_count - 3 {
            let child = node.child(index).unwrap();

            if child.kind() == "identifier" {
                let identifier = Identifier::from_syntax(child, source, context)?;

                parameters.push(identifier);
            }

            if child.kind() == "type_specification" {
                let type_specification = TypeSpecification::from_syntax(child, source, context)?;

                parameter_types.push(type_specification.take_inner());
            }
        }

        let return_type_node = node.child(child_count - 2).unwrap();
        let return_type = TypeSpecification::from_syntax(return_type_node, source, context)?;

        let function_context = Context::with_variables_from(context)?;

        let body_node = node.child(child_count - 1).unwrap();
        let body = Block::from_syntax(body_node, source, &function_context)?;

        let r#type = Type::function(parameter_types, return_type.take_inner());
        let syntax_position = node.range().into();

        Ok(FunctionNode {
            parameters,
            body,
            r#type,
            syntax_position,
            context: function_context,
        })
    }

    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        Ok(self.r#type().clone())
    }

    fn validate(&self, source: &str, context: &Context) -> Result<(), ValidationError> {
        if let Type::Function {
            parameter_types,
            return_type,
        } = &self.r#type
        {
            self.context.inherit_from(context)?;

            for (parameter, r#type) in self.parameters.iter().zip(parameter_types.iter()) {
                self.context.set_type(parameter.clone(), r#type.clone())?;
            }

            let actual = self.body.expected_type(&self.context)?;

            if !return_type.accepts(&actual) {
                return Err(ValidationError::TypeCheck {
                    expected: return_type.as_ref().clone(),
                    actual,
                    position: self.syntax_position,
                });
            }

            self.body.validate(source, &self.context)?;

            Ok(())
        } else {
            Err(ValidationError::TypeCheckExpectedFunction {
                actual: self.r#type.clone(),
                position: self.syntax_position,
            })
        }
    }

    fn run(&self, _source: &str, context: &Context) -> Result<Value, RuntimeError> {
        self.context.inherit_from(context)?;

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
