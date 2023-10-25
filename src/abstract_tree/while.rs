use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Expression, Item, Map, Result, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct While {
    expression: Expression,
    items: Vec<Item>,
}

impl AbstractTree for While {
    fn from_syntax_node(source: &str, node: Node) -> crate::Result<Self> {
        debug_assert_eq!("while", node.kind());

        let expression_node = node.child(1).unwrap();
        let expression = Expression::from_syntax_node(source, expression_node)?;

        let child_count = node.child_count();
        let mut items = Vec::with_capacity(child_count);

        for index in 3..child_count - 1 {
            let item_node = node.child(index).unwrap();
            let item = Item::from_syntax_node(source, item_node)?;

            items.push(item);
        }

        Ok(While { expression, items })
    }

    fn run(&self, source: &str, context: &mut Map) -> Result<Value> {
        while self.expression.run(source, context)?.as_boolean()? {
            for item in &self.items {
                item.run(source, context)?;
            }
        }

        Ok(crate::Value::Empty)
    }
}

#[cfg(test)]
mod tests {
    use crate::evaluate;

    #[test]
    fn evalualate_while_loop() {
        assert_eq!(evaluate("while false { 'foo' }"), Ok(crate::Value::Empty))
    }
}
