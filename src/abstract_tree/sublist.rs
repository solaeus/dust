use serde::{Deserialize, Serialize};

use crate::{AbstractTree, Expression, List, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Sublist {
    list: Expression,
    start: Expression,
    end: Expression,
}

impl AbstractTree for Sublist {
    fn from_syntax_node(source: &str, node: tree_sitter::Node) -> crate::Result<Self> {
        let list_node = node.child(0).unwrap();
        let list = Expression::from_syntax_node(source, list_node)?;

        let start_node = node.child(2).unwrap();
        let start = Expression::from_syntax_node(source, start_node)?;

        let end_node = node.child(4).unwrap();
        let end = Expression::from_syntax_node(source, end_node)?;

        Ok(Sublist { list, start, end })
    }

    fn run(&self, source: &str, context: &mut crate::Map) -> crate::Result<crate::Value> {
        let value = self.list.run(source, context)?;
        let list = value.as_list()?.items();
        let start = self.start.run(source, context)?.as_integer()? as usize;
        let end = self.end.run(source, context)?.as_integer()? as usize;
        let sublist = &list[start..=end];

        Ok(Value::List(List::with_items(sublist.to_vec())))
    }
}
