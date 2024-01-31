use serde::{Deserialize, Serialize};

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    function_expression::FunctionExpression,
    AbstractTree, Error, Expression, Format, FunctionCall, Map, SyntaxNode, Type, Value,
};

/// Abstract representation of a yield expression.
///
/// Yield is an alternate means of calling and passing values to a function.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Yield {
    call: FunctionCall,
}

impl AbstractTree for Yield {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Map) -> Result<Self, SyntaxError> {
        Error::expect_syntax_node(source, "yield", node)?;

        let input_node = node.child(0).unwrap();
        let input = Expression::from_syntax(input_node, source, context)?;

        let function_node = node.child(2).unwrap();
        let function_expression = FunctionExpression::from_syntax(function_node, source, context)?;

        let mut arguments = Vec::new();

        arguments.push(input);

        for index in 3..node.child_count() - 1 {
            let child = node.child(index).unwrap();

            if child.is_named() {
                let expression = Expression::from_syntax(child, source, context)?;

                arguments.push(expression);
            }
        }

        let call = FunctionCall::new(function_expression, arguments, node.range().into());

        Ok(Yield { call })
    }

    fn expected_type(&self, context: &Map) -> Result<Type, ValidationError> {
        self.call.expected_type(context)
    }

    fn check_type(&self, _source: &str, _context: &Map) -> Result<(), ValidationError> {
        self.call.check_type(_source, _context)
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value, RuntimeError> {
        self.call.run(source, context)
    }
}

impl Format for Yield {
    fn format(&self, output: &mut String, indent_level: u8) {
        self.call.format(output, indent_level);
    }
}
