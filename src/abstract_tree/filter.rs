use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Error, Expression, Identifier, Item, List, Map, Result, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Filter {
    identifier: Identifier,
    expression: Expression,
    item: Item,
}

impl AbstractTree for Filter {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
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

    fn run(&self, source: &str, context: &mut Map) -> Result<Value> {
        let value = self.expression.run(source, context)?;
        let values = value.as_list()?.items();
        let key = self.identifier.inner();
        let new_values = List::new();

        values.par_iter().try_for_each(|value| {
            let mut context = Map::new();

            context.variables_mut().insert(key.clone(), value.clone());

            let should_include = self.item.run(source, &mut context)?.as_boolean()?;

            if should_include {
                new_values.items_mut().push(value.clone());
            }

            Ok::<(), Error>(())
        })?;

        Ok(Value::List(new_values))
    }
}
