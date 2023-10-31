use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Block, Expression, Identifier, List, Map, Result, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Transform {
    identifier: Identifier,
    expression: Expression,
    item: Block,
}

impl AbstractTree for Transform {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        let identifier_node = node.child(1).unwrap();
        let identifier = Identifier::from_syntax_node(source, identifier_node)?;

        let expression_node = node.child(3).unwrap();
        let expression = Expression::from_syntax_node(source, expression_node)?;

        let item_node = node.child(5).unwrap();
        let item = Block::from_syntax_node(source, item_node)?;

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
        let new_values = values
            .par_iter()
            .map(|value| {
                let mut iter_context = Map::new();

                iter_context
                    .variables_mut()
                    .insert(key.clone(), value.clone());

                let item_run = self.item.run(source, &mut iter_context);

                match item_run {
                    Ok(value) => value,
                    Err(_) => Value::Empty,
                }
            })
            .filter(|value| !value.is_empty())
            .collect();

        Ok(Value::List(List::with_items(new_values)))
    }
}
