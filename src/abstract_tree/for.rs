use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Block, Error, Expression, Identifier, Result, Structure, Type, Value};

/// Abstract representation of a for loop statement.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct For {
    is_async: bool,
    item_id: Identifier,
    collection: Expression,
    block: Block,
    context: Structure,
}

impl AbstractTree for For {
    fn from_syntax_node(source: &str, node: Node, context: &Structure) -> Result<Self> {
        Error::expect_syntax_node(source, "for", node)?;

        let for_node = node.child(0).unwrap();
        let is_async = match for_node.kind() {
            "for" => false,
            "async for" => true,
            _ => {
                return Err(Error::UnexpectedSyntaxNode {
                    expected: "for or async for".to_string(),
                    actual: for_node.kind().to_string(),
                    location: for_node.start_position(),
                    relevant_source: source[for_node.byte_range()].to_string(),
                })
            }
        };

        let identifier_node = node.child(1).unwrap();
        let identifier = Identifier::from_syntax_node(source, identifier_node, context)?;

        let expression_node = node.child(3).unwrap();
        let expression = Expression::from_syntax_node(source, expression_node, context)?;

        let item_node = node.child(4).unwrap();
        let item = Block::from_syntax_node(source, item_node, context)?;

        Ok(For {
            is_async,
            item_id: identifier,
            collection: expression,
            block: item,
            context: Structure::clone_from(context)?,
        })
    }

    fn check_type(&self, context: &Structure) -> Result<()> {
        self.collection.check_type(context)?;

        let key = self.item_id.inner();
        let collection_type = self.collection.expected_type(context)?;

        Type::list(Type::Any).check(&collection_type)?;

        if let Type::List(item_type) = self.collection.expected_type(context)? {
            self.context
                .set(key.to_string(), Value::none(), Some(*item_type))?;
        }

        self.block.check_type(&self.context)
    }

    fn run(&self, source: &str, context: &Structure) -> Result<Value> {
        let expression_run = self.collection.run(source, context)?;
        let key = self.item_id.inner();
        let values = expression_run.as_list()?.items();

        if self.is_async {
            values.par_iter().try_for_each(|value| {
                self.context.set(key.clone(), value.clone(), None)?;

                self.block.run(source, &self.context).map(|_value| ())
            })?;
        } else {
            for value in values.iter() {
                self.context.set(key.clone(), value.clone(), None)?;

                self.block.run(source, &self.context)?;
            }
        }

        Ok(Value::none())
    }

    fn expected_type(&self, _context: &Structure) -> Result<Type> {
        Ok(Type::None)
    }
}
