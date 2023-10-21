use serde::{Deserialize, Serialize};

use crate::{AbstractTree, Expression, Identifier, Item, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Remove {
    identifier: Identifier,
    expression: Expression,
    item: Item,
}

impl AbstractTree for Remove {
    fn from_syntax_node(source: &str, node: tree_sitter::Node) -> crate::Result<Self> {
        let identifier_node = node.child(1).unwrap();
        let identifier = Identifier::from_syntax_node(source, identifier_node)?;

        let expression_node = node.child(3).unwrap();
        let expression = Expression::from_syntax_node(source, expression_node)?;

        let item_node = node.child(5).unwrap();
        let item = Item::from_syntax_node(source, item_node)?;

        Ok(Remove {
            identifier,
            expression,
            item,
        })
    }

    fn run(&self, source: &str, context: &mut crate::VariableMap) -> crate::Result<crate::Value> {
        let value = self.expression.run(source, context)?;
        let mut list = value.into_inner_list()?;
        let key = self.identifier.inner();
        let mut sub_context = context.clone();

        for (index, value) in list.clone().iter().enumerate() {
            sub_context.set_value(key.clone(), value.clone())?;

            let should_remove = self.item.run(source, &mut sub_context)?.as_boolean()?;

            if should_remove {
                list.remove(index);

                match &self.expression {
                    Expression::Identifier(identifier) => {
                        context.set_value(identifier.inner().clone(), Value::List(list))?;
                    }
                    _ => {}
                }

                return Ok(value.clone());
            }
        }

        Ok(Value::Empty)
    }
}