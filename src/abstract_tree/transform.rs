use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Expression, Identifier, Item, List, Map, Result, Value};

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

    fn run(&self, source: &str, context: &mut Map) -> Result<Value> {
        let expression_run = self.expression.run(source, context)?;
        let values = expression_run.as_list()?.items();
        let key = self.identifier.inner();
        let context = context.clone();
        let new_list = List::with_capacity(values.len());

        values.par_iter().try_for_each_with(
            (context, new_list.clone()),
            |(context, new_list), value| {
                context.set_value(key.clone(), value.clone()).unwrap();

                let item_run = self.item.run(source, context);

                match item_run {
                    Ok(value) => {
                        new_list.items_mut().push(value);

                        Ok(())
                    }
                    Err(error) => Err(error),
                }
            },
        )?;

        Ok(Value::List(new_list))
    }
}
