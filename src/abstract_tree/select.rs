use serde::{Deserialize, Serialize};

use crate::{AbstractTree, Expression, Identifier, Item, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Select {
    identifiers: Vec<Identifier>,
    expression: Expression,
    item: Option<Item>,
}

impl AbstractTree for Select {
    fn from_syntax_node(source: &str, node: tree_sitter::Node) -> crate::Result<Self> {
        let child_count = node.child_count();
        let mut identifiers = Vec::new();

        for index in 2..child_count - 4 {
            let node = node.child(index).unwrap();

            if node.kind() == "identifier" {
                let identifier = Identifier::from_syntax_node(source, node)?;
                identifiers.push(identifier);
            }

            if node.kind() == ">" {
                break;
            }
        }

        let final_node = node.child(child_count - 1).unwrap();

        let item = if final_node.kind() == "}" {
            let item_node = node.child(child_count - 2).unwrap();

            Some(Item::from_syntax_node(source, item_node)?)
        } else {
            None
        };

        let expression_node = if item.is_some() {
            node.child(child_count - 4).unwrap()
        } else {
            node.child(child_count - 1).unwrap()
        };

        let expression = Expression::from_syntax_node(source, expression_node)?;

        Ok(Select {
            identifiers,
            expression,
            item,
        })
    }

    fn run(&self, source: &str, context: &mut crate::VariableMap) -> crate::Result<crate::Value> {
        let value = self.expression.run(source, context)?;
        let table = value.as_table()?;
        let column_names = if self.identifiers.len() > 0 {
            self.identifiers
                .iter()
                .cloned()
                .map(|identifierier| identifierier.take_inner())
                .collect()
        } else {
            table.column_names().clone()
        };
        let new_table = table.select(&column_names);

        Ok(Value::Table(new_table))
    }
}
