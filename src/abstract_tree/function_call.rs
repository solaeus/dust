use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, BuiltInFunction, Error, Map, Result, Value};

use super::{expression::Expression, identifier::Identifier};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum FunctionCall {
    BuiltIn(Box<BuiltInFunction>),
    ContextDefined {
        name: Identifier,
        arguments: Vec<Expression>,
    },
}

impl AbstractTree for FunctionCall {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        debug_assert_eq!("function_call", node.kind());

        let function_node = node.child(0).unwrap();
        let mut arguments = Vec::new();

        for index in 1..node.child_count() {
            let child = node.child(index).unwrap();

            if child.kind() == "expression" {
                let expression = Expression::from_syntax_node(source, child)?;

                arguments.push(expression);
            }
        }

        let function_call = if function_node.kind() == "built_in_function" {
            let function = BuiltInFunction::from_syntax_node(source, function_node)?;

            FunctionCall::BuiltIn(Box::new(function))
        } else {
            let identifier = Identifier::from_syntax_node(source, function_node)?;

            FunctionCall::ContextDefined {
                name: identifier,
                arguments,
            }
        };

        Ok(function_call)
    }

    fn run(&self, source: &str, context: &mut Map) -> Result<Value> {
        let (name, arguments) = match self {
            FunctionCall::BuiltIn(function) => return function.run(source, context),
            FunctionCall::ContextDefined { name, arguments } => (name, arguments),
        };

        let definition = if let Some(value) = context.variables().get(name.inner()) {
            value.as_function().cloned()?
        } else {
            return Err(Error::FunctionIdentifierNotFound(name.clone()));
        };
        let mut function_context = Map::clone_from(context);
        let identifier_expression_pairs = definition.identifiers().iter().zip(arguments.iter());

        for (identifier, expression) in identifier_expression_pairs {
            let key = identifier.inner().clone();
            let value = expression.run(source, context)?;

            function_context.variables_mut().insert(key, value);
        }

        definition.body().run(source, &mut function_context)
    }
}
