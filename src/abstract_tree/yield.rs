use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{
    function_expression::FunctionExpression, AbstractTree, Error, Expression, FunctionCall, Map,
    Result, Type, Value,
};

/// Abstract representation of a yield expression.
///
/// Yield is an alternate means of calling and passing values to a function.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Yield {
    call: FunctionCall,
}

impl AbstractTree for Yield {
    fn from_syntax_node(source: &str, node: Node, context: &Map) -> Result<Self> {
        Error::expect_syntax_node(source, "yield", node)?;

        let input_node = node.child(0).unwrap();
        let input = Expression::from_syntax_node(source, input_node, context)?;

        let function_node = node.child(2).unwrap();
        let function_expression =
            FunctionExpression::from_syntax_node(source, function_node, context)?;

        let mut arguments = Vec::new();

        arguments.push(input);

        for index in 3..node.child_count() - 1 {
            let child = node.child(index).unwrap();

            if child.is_named() {
                let expression = Expression::from_syntax_node(source, child, context)?;

                arguments.push(expression);
            }
        }

        let call = FunctionCall::new(function_expression, arguments, node.range().into());

        Ok(Yield { call })
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value> {
        self.call.run(source, context)
    }

    fn expected_type(&self, context: &Map) -> Result<Type> {
        self.call.expected_type(context)
    }
}
