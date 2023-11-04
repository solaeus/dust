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
        let mut values = value.as_list()?.items_mut();
        let key = self.item_id.inner();
        let mut should_remove_index = None;

        values.iter().enumerate().try_for_each(|(index, value)| {
            context.variables_mut().insert(key.clone(), value.clone());

            let should_remove = self.predicate.run(source, context)?.as_boolean()?;

            if should_remove {
                should_remove_index = Some(index);
            }

            Ok::<(), Error>(())
        })?;

        if let Some(index) = should_remove_index {
            Ok(values.remove(index))
        } else {
            Ok(Value::Empty)
        }
    }
}
