use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{AbstractTree, Expression, Identifier, Item, Result, Value, VariableMap};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Insert {
    identifier: Identifier,
    expression: Expression,
}

impl AbstractTree for Insert {
    fn from_syntax_node(source: &str, node: Node) -> Result<Self> {
        let identifier_node = node.child(2).unwrap();
        let identifier = Identifier::from_syntax_node(source, identifier_node)?;
        let expression_node = node.child(3).unwrap();
        let expression = Expression::from_syntax_node(source, expression_node)?;

        Ok(Insert {
            identifier,
            expression,
        })
    }

    fn run(&self, source: &str, context: &mut VariableMap) -> Result<Value> {
        let table_name = self.identifier.inner().clone();
        let mut table = self.identifier.run(source, context)?.as_table()?.clone();
        let new_rows = self.expression.run(source, context)?.into_inner_list()?;

        table.reserve(new_rows.len());

        println!("{new_rows:?}");

        for row in new_rows {
            table.insert(row.into_inner_list()?)?;
        }

        context.set_value(table_name, Value::Table(table))?;

        Ok(Value::Empty)
    }
}
