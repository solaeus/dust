use std::sync::RwLock;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Block, Error, Expression, Identifier, Map, Result, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Remove {
    item_id: Identifier,
    collection: Expression,
    predicate: Block,
}

impl AbstractTree for Remove {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        let identifier_node = node.child(1).unwrap();
        let item_id = Identifier::from_syntax_node(source, identifier_node)?;

        let expression_node = node.child(3).unwrap();
        let collection = Expression::from_syntax_node(source, expression_node)?;

        let block_node = node.child(4).unwrap();
        let predicate = Block::from_syntax_node(source, block_node)?;

        Ok(Remove {
            item_id,
            collection,
            predicate,
        })
    }

    fn run(&self, source: &str, context: &mut Map) -> Result<Value> {
        let value = self.collection.run(source, context)?;
        let values = value.as_list()?;
        let key = self.item_id.inner();
        let should_remove_index = RwLock::new(None);

        values
            .items()
            .par_iter()
            .enumerate()
            .try_for_each(|(index, value)| {
                if should_remove_index.read()?.is_some() {
                    return Ok(());
                }

                let iter_context = Map::clone_from(context)?;

                iter_context
                    .variables_mut()?
                    .insert(key.clone(), value.clone());

                let should_remove = self
                    .predicate
                    .run(source, &mut iter_context.clone())?
                    .as_boolean()?;

                if should_remove {
                    let _ = should_remove_index.write()?.insert(index);
                }

                Ok::<(), Error>(())
            })?;

        let index = should_remove_index.read()?;

        if let Some(index) = *index {
            Ok(values.items_mut().remove(index))
        } else {
            Ok(Value::Empty)
        }
    }
}
