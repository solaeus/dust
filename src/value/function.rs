use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{
    AbstractTree, Block, BuiltInFunction, Error, Identifier, Map, Result, Type, TypeDefinition,
    Value,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Function {
    BuiltIn(BuiltInFunction),
    ContextDefined(ContextDefinedFunction),
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Function::BuiltIn(built_in_function) => write!(f, "{}", built_in_function.r#type()),
            Function::ContextDefined(context_defined_function) => {
                write!(f, "{}", context_defined_function.r#type())
            }
        }
    }
}

impl Function {
    pub fn call(
        &self,
        name: Option<String>,
        arguments: &[Value],
        source: &str,
        outer_context: &Map,
    ) -> Result<Value> {
        match self {
            Function::BuiltIn(built_in_function) => {
                built_in_function.call(arguments, source, outer_context)
            }
            Function::ContextDefined(context_defined_function) => {
                context_defined_function.call(name, arguments, source, outer_context)
            }
        }
    }

    pub fn r#type(&self) -> Type {
        match self {
            Function::BuiltIn(built_in_function) => built_in_function.r#type(),
            Function::ContextDefined(context_defined_function) => {
                context_defined_function.r#type().clone()
            }
        }
    }
}

impl AbstractTree for Function {
    fn from_syntax_node(source: &str, node: Node, context: &Map) -> Result<Self> {
        Error::expect_syntax_node(source, "function", node)?;

        let child_count = node.child_count();
        let mut parameters = Vec::new();
        let mut parameter_types = Vec::new();

        for index in 1..child_count - 3 {
            let child = node.child(index).unwrap();

            if child.kind() == "identifier" {
                let identifier = Identifier::from_syntax_node(source, child, context)?;

                parameters.push(identifier);
            }

            if child.kind() == "type_definition" {
                let type_definition = TypeDefinition::from_syntax_node(source, child, context)?;

                parameter_types.push(type_definition.take_inner());
            }
        }

        let function_context = Map::clone_from(context)?;

        for (parameter_name, parameter_type) in parameters.iter().zip(parameter_types.iter()) {
            function_context.set(
                parameter_name.inner().clone(),
                Value::none(),
                Some(parameter_type.clone()),
            )?;
        }

        let return_type_node = node.child(child_count - 2).unwrap();
        let return_type = TypeDefinition::from_syntax_node(source, return_type_node, context)?;

        let body_node = node.child(child_count - 1).unwrap();
        let body = Block::from_syntax_node(source, body_node, &function_context)?;

        return_type
            .inner()
            .check(&body.expected_type(&function_context)?)
            .map_err(|error| error.at_node(body_node, source))?;

        let r#type = Type::function(parameter_types, return_type.take_inner());

        Ok(Self::ContextDefined(ContextDefinedFunction::new(
            parameters, body, r#type,
        )))
    }

    fn run(&self, _source: &str, _context: &Map) -> Result<Value> {
        Ok(Value::Function(self.clone()))
    }

    fn expected_type(&self, _context: &Map) -> Result<Type> {
        match self {
            Function::BuiltIn(built_in) => Ok(built_in.r#type()),
            Function::ContextDefined(context_defined) => Ok(context_defined.r#type().clone()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ContextDefinedFunction {
    parameters: Vec<Identifier>,
    body: Block,
    r#type: Type,
}

impl ContextDefinedFunction {
    pub fn new(parameters: Vec<Identifier>, body: Block, r#type: Type) -> Self {
        Self {
            parameters,
            body,
            r#type,
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
        let parameter_argument_pairs = self.parameters.iter().zip(arguments.iter());
        let function_context = Map::clone_from(outer_context)?;

        for (identifier, value) in parameter_argument_pairs {
            let key = identifier.inner().clone();

            function_context.set(key, value.clone(), None)?;
        }

        if let Some(name) = name {
            function_context.set(
                name,
                Value::Function(Function::ContextDefined(self.clone())),
                None,
            )?;
        }

        let return_value = self.body.run(source, &function_context)?;

        Ok(return_value)
    }
}
