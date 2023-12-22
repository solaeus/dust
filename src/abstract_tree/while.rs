use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Block, Expression, Map, Result, Type, Value};

/// Abstract representation of a while loop.
///
/// While executes its block repeatedly until its expression evaluates to true.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct While {
    expression: Expression,
    block: Block,
}

impl AbstractTree for While {
    fn from_syntax_node(source: &str, node: Node, context: &Map) -> crate::Result<Self> {
        debug_assert_eq!("while", node.kind());

        let expression_node = node.child(1).unwrap();
        let expression = Expression::from_syntax_node(source, expression_node, context)?;

        let block_node = node.child(2).unwrap();
        let block = Block::from_syntax_node(source, block_node, context)?;

        Ok(While { expression, block })
    }

    fn run(&self, source: &str, context: &Map) -> Result<Value> {
        while self.expression.run(source, context)?.as_boolean()? {
            self.block.run(source, context)?;
        }

        Ok(Value::Option(None))
    }

    fn expected_type(&self, context: &Map) -> Result<Type> {
        self.block.expected_type(context)
    }
}

#[cfg(test)]
mod tests {
    use crate::{evaluate, Value};

    #[test]
    fn evalualate_while_loop() {
        assert_eq!(evaluate("while false { 'foo' }"), Ok(Value::Option(None)))
    }
}
