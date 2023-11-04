use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Block, Error, Expression, Identifier, Map, Result, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct For {
    is_async: bool,
    item_id: Identifier,
    collection: Expression,
    block: Block,
}

impl AbstractTree for For {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        let for_node = node.child(0).unwrap();
        let is_async = match for_node.kind() {
            "for" => false,
            "async for" => true,
            _ => {
                return Err(Error::UnexpectedSyntaxNode {
                    expected: "for or async for",
                    actual: for_node.kind(),
                    location: for_node.start_position(),
                    relevant_source: source[for_node.byte_range()].to_string(),
                })
            }
        };

        let identifier_node = node.child(1).unwrap();
        let identifier = Identifier::from_syntax_node(source, identifier_node)?;

        let expression_node = node.child(3).unwrap();
        let expression = Expression::from_syntax_node(source, expression_node)?;

        let item_node = node.child(4).unwrap();
        let item = Block::from_syntax_node(source, item_node)?;

        Ok(For {
            is_async,
            item_id: identifier,
            collection: expression,
            block: item,
        })
    }

    fn run(&self, source: &str, context: &mut Map) -> Result<Value> {
        let expression_run = self.collection.run(source, context)?;
        let values = expression_run.as_list()?.items();
        let key = self.item_id.inner();
        let mut loop_context = Map::clone_from(context);

        if self.is_async {
            values.par_iter().try_for_each(|value| {
                let mut iter_context = loop_context.clone();

                iter_context
                    .variables_mut()
                    .insert(key.clone(), value.clone());

                self.block.run(source, &mut iter_context).map(|_value| ())
            })?;
        } else {
            for value in values.iter() {
                loop_context
                    .variables_mut()
                    .insert(key.clone(), value.clone());

                self.block.run(source, &mut loop_context)?;
            }
        }

        Ok(Value::Empty)
    }
}
