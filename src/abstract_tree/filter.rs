use serde::{Deserialize, Serialize};

use crate::{AbstractTree, Expression, Identifier, Item, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Filter {
    identifier: Identifier,
    expression: Expression,
    item: Item,
}

impl AbstractTree for Filter {
    fn from_syntax_node(source: &str, node: tree_sitter::Node) -> crate::Result<Self> {
        let identifier_node = node.child(1).unwrap();
        let identifier = Identifier::from_syntax_node(source, identifier_node)?;

        let expression_node = node.child(3).unwrap();
        let expression = Expression::from_syntax_node(source, expression_node)?;

        let item_node = node.child(5).unwrap();
        let item = Item::from_syntax_node(source, item_node)?;

        Ok(Filter {
            identifier,
            expression,
            item,
        })
    }

    fn run(&self, source: &str, context: &mut crate::VariableMap) -> crate::Result<crate::Value> {
        let value = self.expression.run(source, context)?;
        let list = value.as_list()?;
        let key = self.identifier.inner();
        let mut context = context.clone();
        let mut new_list = Vec::with_capacity(list.len());

        for value in list {
            context.set_value(key.clone(), value.clone())?;

            let should_include = self.item.run(source, &mut context)?.as_boolean()?;

            if should_include {
                new_list.push(value.clone());
            }
        }

        Ok(Value::List(new_list))
    }
}
