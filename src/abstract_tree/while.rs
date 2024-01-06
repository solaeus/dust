use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Block, Error, Expression, Map, Result, Type, Value};

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
        Error::expect_syntax_node(source, "while", node)?;

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

        Ok(Value::none())
    }

    fn expected_type(&self, context: &Map) -> Result<Type> {
        self.block.expected_type(context)
    }
}

impl Display for While {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let While { expression, block } = self;

        write!(f, "while {expression} {{{block}}}")
    }
}
