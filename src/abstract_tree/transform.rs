use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Expression, Identifier, Item, Result, Value, VariableMap};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Transform {
    identifier: Identifier,
    expression: Expression,
    item: Item,
}

impl AbstractTree for Transform {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        let identifier_node = node.child(1).unwrap();
        let identifier = Identifier::from_syntax_node(source, identifier_node)?;

        let expression_node = node.child(3).unwrap();
        let expression = Expression::from_syntax_node(source, expression_node)?;

        let item_node = node.child(5).unwrap();
        let item = Item::from_syntax_node(source, item_node)?;

        Ok(Transform {
            identifier,
            expression,
            item,
        })
    }

    fn run(&self, source: &str, context: &mut VariableMap) -> Result<Value> {
        let value = self.expression.run(source, context)?;
        let list = value.as_list()?;
        let key = self.identifier.inner();
        let mut context = context.clone();
        let mut new_list = Vec::with_capacity(list.len());

        for value in list {
            context.set_value(key.clone(), value.clone())?;

            let value = self.item.run(source, &mut context)?;

            new_list.push(value);
        }

        Ok(Value::List(new_list))
    }
}
