use async_std::task;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Expression, Identifier, Item, List, Map, Result, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Filter {
    count: Option<Expression>,
    item_id: Identifier,
    collection: Expression,
    predicate: Item,
}

impl AbstractTree for Filter {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        let count = if let Some(node) = node.child_by_field_name("count") {
            Some(Expression::from_syntax_node(source, node)?)
        } else {
            None
        };

        let identifier_node = node.child_by_field_name("item_id").unwrap();
        let identifier = Identifier::from_syntax_node(source, identifier_node)?;

        let expression_node = node.child_by_field_name("collection").unwrap();
        let expression = Expression::from_syntax_node(source, expression_node)?;

        let item_node = node.child_by_field_name("predicate").unwrap();
        let item = Item::from_syntax_node(source, item_node)?;

        Ok(Filter {
            count,
            item_id: identifier,
            collection: expression,
            predicate: item,
        })
    }

    fn run(&self, source: &str, context: &mut Map) -> Result<Value> {
        let value = self.collection.run(source, context)?;
        let values = value.as_list()?.items();
        let count = if let Some(expression) = &self.count {
            Some(expression.run(source, context)?.as_integer()? as usize)
        } else {
            None
        };
        let key = self.item_id.inner();
        let new_values = match count {
            Some(count) => List::with_capacity(count),
            None => List::new(),
        };

        async {};

        for value in values.clone().into_iter() {
            task::spawn(async {
                let new_values = new_values.clone();
                if let Some(max) = count {
                    if values.len() == max {
                        return;
                    }
                }

                let mut context = Map::new();

                context.variables_mut().insert(key.clone(), value.clone());

                let predicate_run = self.predicate.run(source, &mut context);
                let should_include = if let Ok(value) = predicate_run {
                    value.as_boolean().unwrap_or_default()
                } else {
                    return;
                };

                if should_include {
                    new_values.items_mut().push(value);
                }
            });
        }

        Ok(Value::List(new_values))
    }
}
