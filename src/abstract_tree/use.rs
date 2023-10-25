use std::fs::read_to_string;

use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{evaluate_with_context, AbstractTree, Result, Value, ValueNode, VariableMap};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Use {
    path: ValueNode,
}

impl AbstractTree for Use {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        let path_node = node.child(1).unwrap();
        let value_node = ValueNode::from_syntax_node(source, path_node)?;

        Ok(Use { path: value_node })
    }

    fn run(&self, source: &str, context: &mut VariableMap) -> Result<Value> {
        let run_node = self.path.run(source, context)?;
        let path = run_node.as_string()?;
        let file_contents = read_to_string(path)?;
        let mut temp_context = VariableMap::new();
        let eval_result = evaluate_with_context(&file_contents, &mut temp_context)?;

        while let Some((key, value)) = temp_context.inner_mut().pop_first() {
            context.set_value(key, value)?;
        }

        Ok(eval_result)
    }
}
