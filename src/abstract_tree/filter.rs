use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Block, Error, Expression, Identifier, List, Map, Result, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Filter {
    count: Option<Expression>,
    item_id: Identifier,
    collection: Expression,
    predicate: Block,
}

impl AbstractTree for Filter {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        let count = match node.child_by_field_name("count") {
            Some(node) => Some(Expression::from_syntax_node(source, node)?),
            None => None,
        };

        let item_id_node = node.child_by_field_name("item_id").unwrap();
        let item_id = Identifier::from_syntax_node(source, item_id_node)?;

        let collection_node = node.child_by_field_name("collection").unwrap();
        let collection = Expression::from_syntax_node(source, collection_node)?;

        let predicate_node = node.child_by_field_name("predicate").unwrap();
        let predicate = Block::from_syntax_node(source, predicate_node)?;

        Ok(Filter {
            count,
            item_id,
            collection,
            predicate,
        })
    }

    fn run(&self, source: &str, context: &mut Map) -> Result<Value> {
        let value = self.collection.run(source, context)?;
        let values = value.as_list()?.items();
        let key = self.item_id.inner();
        let new_values = List::new();
        let count = match &self.count {
            Some(expression) => Some(expression.run(source, context)?.as_integer()? as usize),
            None => None,
        };
        let loop_context = Map::clone_from(context);

        values.par_iter().try_for_each(|value| {
            if let Some(max) = count {
                if new_values.items().len() == max {
                    return Ok(());
                }
            }

            let mut iter_context = loop_context.clone();

            iter_context
                .variables_mut()
                .insert(key.clone(), value.clone());

            let should_include = self
                .predicate
                .run(source, &mut iter_context)?
                .as_boolean()?;

            if should_include {
                new_values.items_mut().push(value.clone());
            }

            Ok::<(), Error>(())
        })?;

        Ok(Value::List(new_values))
    }
}
