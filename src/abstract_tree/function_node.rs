use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{
    AbstractTree, Block, Error, Format, Function, Identifier, Map, Result, SyntaxNode,
    SyntaxPosition, Type, TypeDefinition, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct FunctionNode {
    parameters: Vec<Identifier>,
    body: Block,
    r#type: Type,
    syntax_position: SyntaxPosition,
    context: Map,
}

impl FunctionNode {
    pub fn new(
        parameters: Vec<Identifier>,
        body: Block,
        r#type: Type,
        syntax_position: SyntaxPosition,
    ) -> Self {
        Self {
            parameters,
            body,
            r#type,
            syntax_position,
            context: Map::new(),
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
        name: Option<String>,
        arguments: &[Value],
        source: &str,
        outer_context: &Map,
    ) -> Result<Value> {
        self.context.clone_complex_values_from(outer_context)?;

        let parameter_argument_pairs = self.parameters.iter().zip(arguments.iter());

        for (identifier, value) in parameter_argument_pairs {
            let key = identifier.inner().clone();

            self.context.set(key, value.clone())?;
        }

        if let Some(name) = name {
            self.context.set(
                name,
                Value::Function(Function::ContextDefined(self.clone())),
            )?;
        }

        let return_value = self.body.run(source, &self.context)?;

        Ok(return_value)
    }
}

impl AbstractTree for FunctionNode {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Map) -> Result<Self> {
        Error::expect_syntax_node(source, "function", node)?;

        let child_count = node.child_count();
        let mut parameters = Vec::new();
        let mut parameter_types = Vec::new();

        for index in 1..child_count - 3 {
            let child = node.child(index).unwrap();

            if child.kind() == "identifier" {
                let identifier = Identifier::from_syntax(child, source, context)?;

                parameters.push(identifier);
            }

            if child.kind() == "type_definition" {
                let type_definition = TypeDefinition::from_syntax(child, source, context)?;

                parameter_types.push(type_definition.take_inner());
            }
        }

        let return_type_node = node.child(child_count - 2).unwrap();
        let return_type = TypeDefinition::from_syntax(return_type_node, source, context)?;

        let function_context = Map::new();

        function_context.clone_complex_values_from(context)?;

        for (identifier, r#type) in parameters.iter().zip(parameter_types.iter()) {
            function_context.set_type(identifier.inner().clone(), r#type.clone())?;
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
            context: function_context,
        })
    }

    fn check_type(&self, source: &str, context: &Map) -> Result<()> {
        self.context.clone_complex_values_from(context)?;
        self.return_type()
            .check(&self.body.expected_type(&self.context)?)
            .map_err(|error| error.at_source_position(source, self.syntax_position))?;
        self.body.check_type(source, &self.context)?;

        Ok(())
    }

    fn run(&self, _source: &str, _context: &Map) -> Result<Value> {
        Ok(Value::Function(Function::ContextDefined(self.clone())))
    }

    fn expected_type(&self, _context: &Map) -> Result<Type> {
        Ok(self.r#type().clone())
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