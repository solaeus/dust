use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Block, Expression, Identifier, Map, Result, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Remove {
    identifier: Identifier,
    expression: Expression,
    item: Block,
}

impl AbstractTree for Remove {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        let identifier_node = node.child(1).unwrap();
        let identifier = Identifier::from_syntax_node(source, identifier_node)?;

        let expression_node = node.child(3).unwrap();
        let expression = Expression::from_syntax_node(source, expression_node)?;

        let item_node = node.child(4).unwrap();
        let item = Block::from_syntax_node(source, item_node)?;

        Ok(Remove {
            identifier,
            expression,
            item,
        })
    }

    fn run(&self, source: &str, context: &mut Map) -> Result<Value> {
        let expression_run = self.expression.run(source, context)?;
        let values = expression_run.into_inner_list()?;
        let key = self.identifier.inner();
        let mut sub_context = context.clone();
        let mut should_remove_index = None;

        for (index, value) in values.items().iter().enumerate() {
            sub_context
                .variables_mut()
                .insert(key.clone(), value.clone());

            let should_remove = self.item.run(source, &mut sub_context)?.as_boolean()?;

            if should_remove {
                should_remove_index = Some(index);

                match &self.expression {
                    Expression::Identifier(identifier) => {
                        sub_context
                            .variables_mut()
                            .insert(identifier.inner().clone(), Value::List(values.clone()));
                    }
                    _ => {}
                }
            }
        }

        if let Some(index) = should_remove_index {
            Ok(values.items_mut().remove(index))
        } else {
            Ok(Value::Empty)
        }
    }
}
