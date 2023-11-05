use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Expression, Function, FunctionCall, Result, Value, ValueNode};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Yield {
    input: Expression,
    call: FunctionCall,
}

impl AbstractTree for Yield {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        let input_node = node.child(0).unwrap();
        let input = Expression::from_syntax_node(source, input_node)?;

        let call_node = node.child(1).unwrap();
        let call = FunctionCall::from_syntax_node(source, call_node)?;

        Ok(Yield { input, call })
    }

    fn run(&self, source: &str, context: &mut crate::Map) -> Result<Value> {
        let target = self.input.run(source, context)?.as_function()?;

        self.call.run(, )
    }
}
