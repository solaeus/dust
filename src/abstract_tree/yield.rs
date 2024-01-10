use serde::{Deserialize, Serialize};

use crate::{
    function_expression::FunctionExpression, AbstractTree, Error, Expression, Format, FunctionCall,
    Map, Result, SyntaxNode, Type, Value,
};

/// Abstract representation of a yield expression.
///
/// Yield is an alternate means of calling and passing values to a function.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Yield {
    call: FunctionCall,
}

impl AbstractTree for Yield {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Map) -> Result<Self> {
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

    fn run(&self, source: &str, context: &Map) -> Result<Value> {
        self.call.run(source, context)
    }

    fn expected_type(&self, context: &Map) -> Result<Type> {
        self.call.expected_type(context)
    }
}

impl Format for Yield {
    fn format(&self, output: &mut String, indent_level: u8) {
        self.call.format(output, indent_level);
    }
}
