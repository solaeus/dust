use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Expression, Identifier, Item, Result, Value, VariableMap};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct For {
    identifier: Identifier,
    expression: Expression,
    item: Item,
}

impl AbstractTree for For {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        let identifier_node = node.child(1).unwrap();
        let identifier = Identifier::from_syntax_node(source, identifier_node)?;

        let expression_node = node.child(3).unwrap();
        let expression = Expression::from_syntax_node(source, expression_node)?;

        let item_node = node.child(5).unwrap();
        let item = Item::from_syntax_node(source, item_node)?;

        Ok(For {
            identifier,
            expression,
            item,
        })
    }

    fn run(&self, source: &str, context: &mut VariableMap) -> Result<Value> {
        let value = self.expression.run(source, context)?;
        let list = value.as_list()?;
        let key = self.identifier.inner();

        let original_value = context.get_value(key)?;

        for value in list {
            context.set_value(key.clone(), value.clone())?;

            self.item.run(source, context)?;
        }

        if let Some(original_value) = original_value {
            context.set_value(key.clone(), original_value)?;
        } else {
            context.set_value(key.clone(), Value::Empty)?;
        }

        Ok(Value::Empty)
    }
}
