use serde::{Deserialize, Serialize};

use crate::{AbstractTree, Block, Error, Expression, Format, Map, Result, SyntaxNode, Type, Value};

/// Abstract representation of a while loop.
///
/// While executes its block repeatedly until its expression evaluates to true.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct While {
    expression: Expression,
    block: Block,
}

impl AbstractTree for While {
    fn from_syntax(node: SyntaxNode, source: &str, context: &Map) -> crate::Result<Self> {
        Error::expect_syntax_node(source, "while", node)?;

        let expression_node = node.child(1).unwrap();
        let expression = Expression::from_syntax(expression_node, source, context)?;

        let block_node = node.child(2).unwrap();
        let block = Block::from_syntax(block_node, source, context)?;

        Ok(While { expression, block })
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value> {
        while self.expression.run(source, context)?.as_boolean()? {
            self.block.run(source, context)?;
        }

        Ok(Value::none())
    }

    fn expected_type(&self, context: &Map) -> Result<Type> {
        self.block.expected_type(context)
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
