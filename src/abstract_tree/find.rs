use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Expression, Identifier, Map, Result, Statement, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Find {
    identifier: Identifier,
    expression: Expression,
    item: Statement,
}

impl AbstractTree for Find {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        let identifier_node = node.child(1).unwrap();
        let identifier = Identifier::from_syntax_node(source, identifier_node)?;

        let expression_node = node.child(3).unwrap();
        let expression = Expression::from_syntax_node(source, expression_node)?;

        let item_node = node.child(5).unwrap();
        let item = Statement::from_syntax_node(source, item_node)?;

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
        let mut context = context.clone();

        for value in values.iter() {
            context.variables_mut().insert(key.clone(), value.clone());

            let should_return = self.item.run(source, &mut context)?.as_boolean()?;

            if should_return {
                return Ok(value.clone());
            }
        }

        Ok(Value::Empty)
    }
}
