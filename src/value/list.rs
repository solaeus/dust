use std::fmt::{self, Display, Formatter};

use stanza::{
    renderer::{console::Console, Renderer},
    style::Styles,
    table::{Cell, Content, Row, Table},
};

use crate::Value;

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct List(Vec<Value>);

impl Default for List {
    fn default() -> Self {
        Self::new()
    }
}

impl List {
    pub fn new() -> Self {
        List(Vec::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        List(Vec::with_capacity(capacity))
    }

    pub fn with_items(items: Vec<Value>) -> Self {
        List(items)
    }

    pub fn items(&self) -> &Vec<Value> {
        &self.0
    }

    pub fn items_mut(&mut self) -> &mut Vec<Value> {
        &mut self.0
    }

    pub fn as_text_table(&self) -> Table {
        let cells: Vec<Cell> = self
            .items()
            .iter()
            .map(|value| {
                if let Value::List(list) = value {
                    Cell::new(Styles::default(), Content::Nested(list.as_text_table()))
                } else if let Value::Map(map) = value {
                    Cell::new(Styles::default(), Content::Nested(map.as_text_table()))
                } else {
                    Cell::new(Styles::default(), Content::Label(value.to_string()))
                }
            })
            .collect();

        let row = if cells.is_empty() {
            Row::new(
                Styles::default(),
                vec![Cell::new(
                    Styles::default(),
                    Content::Label("empty list".to_string()),
                )],
            )
        } else {
            Row::new(Styles::default(), cells)
        };

        Table::default().with_row(row)
    }
}

impl Display for List {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let renderer = Console::default();

        f.write_str(&renderer.render(&self.as_text_table()))
    }
}
