use serde::{Deserialize, Serialize};

use crate::{
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, Block, Expression, Format, Map, SyntaxNode, Type, Value,
};

/// Abstract representation of a while loop.
///
/// While executes its block repeatedly until its expression evaluates to true.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct While {
    expression: Expression,
    block: Block,
}

impl AbstractTree for While {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Map) -> Result<Self, SyntaxError> {
        SyntaxError::expect_syntax_node(source, "while", node)?;

        let expression_node = node.child(1).unwrap();
        let expression = Expression::from_syntax(expression_node, source, context)?;

        let block_node = node.child(2).unwrap();
        let block = Block::from_syntax(block_node, source, context)?;

        Ok(While { expression, block })
    }

    fn expected_type(&self, context: &Map) -> Result<Type, ValidationError> {
        self.block.expected_type(context)
    }

    fn validate(&self, _source: &str, context: &Map) -> Result<(), ValidationError> {
        self.expression.validate(_source, context)?;
        self.block.validate(_source, context)
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value, RuntimeError> {
        while self.expression.run(source, context)?.as_boolean()? {
            self.block.run(source, context)?;
        }

        Ok(Value::none())
    }
}

impl Format for While {
    fn format(&self, output: &mut String, indent_level: u8) {
        output.push('\n');
        While::indent(output, indent_level);
        output.push_str("while ");
        self.expression.format(output, indent_level);
        output.push(' ');
        self.block.format(output, indent_level);
        output.push('\n');
    }
}
