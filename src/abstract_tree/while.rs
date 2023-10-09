use serde::{Deserialize, Serialize};

use crate::{AbstractTree, Expression, Item, Result, Value, VariableMap};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct While {
    expression: Expression,
    items: Vec<Item>,
}

impl AbstractTree for While {
    fn from_syntax_node(node: tree_sitter::Node, source: &str) -> crate::Result<Self> {
        debug_assert_eq!("while", node.kind());

        let expression_node = node.child(1).unwrap();
        let expression = Expression::from_syntax_node(expression_node, source)?;

        let child_count = node.child_count();
        let mut items = Vec::with_capacity(child_count);

        for index in 3..child_count - 1 {
            let item_node = node.child(index).unwrap();
            let item = Item::from_syntax_node(item_node, source)?;

            items.push(item);
        }

        Ok(While { expression, items })
    }

    fn run(&self, context: &mut VariableMap) -> Result<Value> {
        while self.expression.run(context)?.as_boolean()? {
            for item in &self.items {
                item.run(context)?;
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
