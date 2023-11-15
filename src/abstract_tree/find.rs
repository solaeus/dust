use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Block, Error, Expression, Identifier, Map, Result, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Find {
    identifier: Identifier,
    expression: Expression,
    item: Block,
}

impl AbstractTree for Find {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        let identifier_node = node.child(1).unwrap();
        let identifier = Identifier::from_syntax_node(source, identifier_node)?;

        let expression_node = node.child(3).unwrap();
        let expression = Expression::from_syntax_node(source, expression_node)?;

        let item_node = node.child(4).unwrap();
        let item = Block::from_syntax_node(source, item_node)?;

        Ok(Find {
            identifier,
            expression,
            item,
        })
    }

    fn run(&self, source: &str, context: &mut Map) -> Result<Value> {
        let value = self.expression.run(source, context)?;
        let values = value.as_list()?.items();
        let key = self.identifier.inner();
        let loop_context = Map::clone_from(context)?;

        let find_result = values.par_iter().find_map_first(|value| {
            loop_context
                .variables_mut()
                .unwrap()
                .insert(key.clone(), (*value).clone());

            let run_result = self.item.run(source, &mut loop_context.clone());

            if let Ok(run_result_value) = run_result {
                if let Ok(should_return) = run_result_value.as_boolean() {
                    if should_return {
                        Some(Ok(value.clone()))
                    } else {
                        None
                    }
                } else {
                    Some(Err(Error::ExpectedBoolean {
                        actual: value.clone(),
                    }))
                }
            } else {
                Some(run_result)
            }
        });

        if let Some(result) = find_result {
            result
        } else {
            Ok(Value::Empty)
        }
    }
}
