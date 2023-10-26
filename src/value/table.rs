use crate::{Error, Map, Result, Value};
use comfy_table::{Cell, Color, ContentArrangement, Table as ComfyTable};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    headers: Vec<String>,
    rows: Vec<Vec<Value>>,
    primary_key_index: usize,
}

impl Table {
    pub fn new(headers: Vec<String>) -> Self {
        Table {
            headers,
            rows: Vec::new(),
            primary_key_index: 0,
        }
    }

    pub fn with_capacity(capacity: usize, headers: Vec<String>) -> Self {
        Table {
            headers,
            rows: Vec::with_capacity(capacity),
            primary_key_index: 0,
        }
    }

    pub fn from_raw_parts(headers: Vec<String>, rows: Vec<Vec<Value>>) -> Self {
        Table {
            headers,
            rows,
            primary_key_index: 0,
        }
    }

    pub fn reserve(&mut self, additional: usize) {
        self.rows.reserve(additional);
    }

    pub fn headers(&self) -> &Vec<String> {
        &self.headers
    }

    pub fn rows(&self) -> &Vec<Vec<Value>> {
        &self.rows
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn sort(&mut self) {
        self.rows.sort();
    }

    pub fn insert(&mut self, row: Vec<Value>) -> Result<()> {
        if row.len() != self.headers.len() {
            return Err(Error::WrongColumnAmount {
                expected: self.headers.len(),
                actual: row.len(),
            });
        }

        self.rows.push(row);

        Ok(())
    }

    pub fn remove(&mut self, index: usize) -> Result<()> {
        self.rows.remove(index);

        Ok(())
    }

    pub fn get_row(&self, index: usize) -> Option<&Vec<Value>> {
        self.rows.get(index)
    }

    pub fn get(&self, value: &Value) -> Option<&Vec<Value>> {
        let primary_key = self.headers().get(self.primary_key_index)?;

        self.get_where(primary_key, value)
    }

    pub fn get_where(&self, column_name: &str, expected: &Value) -> Option<&Vec<Value>> {
        let column_index = self.get_column_index(column_name)?;

        for row in &self.rows {
            if let Some(actual) = row.get(column_index) {
                if actual == expected {
                    return Some(row);
                }
            }
        }

        None
    }

    pub fn filter(&self, column_name: &str, expected: &Value) -> Option<Table> {
        let mut filtered = Table::new(self.headers.clone());
        let column_index = self.get_column_index(column_name)?;

        for row in &self.rows {
            let actual = row.get(column_index).unwrap();

            if actual == expected {
                let _ = filtered.insert(row.clone());
            }
        }

        Some(filtered)
    }

    pub fn get_column_index(&self, column_name: &str) -> Option<usize> {
        let column_names = &self.headers;
        for (i, column) in column_names.iter().enumerate() {
            if column == column_name {
                return Some(i);
            }
        }
        None
    }
}

impl Display for Table {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut table = ComfyTable::new();

        table
            .load_preset("││──├─┼┤│    ┬┴╭╮╰╯")
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(&self.headers);

        for row in &self.rows {
            let row = row.iter().map(|value| {
                let text = match value {
                    Value::List(list) => {
                        let mut string = "(".to_string();

                        for (index, value) in list.into_iter().enumerate() {
                            if index > 0 {
                                string.push_str(", ");
                            }

                            string.push_str(&value.to_string());
                        }

                        string.push_str(")");

                        string
                    }
                    Value::Map(map) => format!("Map ({} items)", map.len()),
                    Value::Table(table) => format!("Table ({} items)", table.len()),
                    Value::Function(_) => "Function".to_string(),
                    Value::Empty => "Empty".to_string(),
                    value => value.to_string(),
                };

                let mut cell = Cell::new(text).bg(Color::Rgb {
                    r: 40,
                    g: 40,
                    b: 40,
                });

                if value.is_string() {
                    cell = cell.fg(Color::Green);
                }
                if value.is_integer() {
                    cell = cell.fg(Color::Blue);
                }
                if value.is_boolean() {
                    cell = cell.fg(Color::Red);
                }
                if value.is_function() {
                    cell = cell.fg(Color::Cyan);
                }

                cell
            });

            table.add_row(row);
        }

        if self.headers.is_empty() {
            table.set_header(["empty"]);
        }

        write!(f, "{table}")
    }
}

impl From<&Value> for Table {
    fn from(value: &Value) -> Self {
        match value {
            Value::String(string) => {
                let mut table = Table::new(vec!["string".to_string()]);

                table
                    .insert(vec![Value::String(string.to_string())])
                    .unwrap();

                table
            }
            Value::Float(float) => {
                let mut table = Table::new(vec!["float".to_string()]);

                table.insert(vec![Value::Float(*float)]).unwrap();

                table
            }
            Value::Integer(integer) => {
                let mut table = Table::new(vec!["integer".to_string()]);

                table.insert(vec![Value::Integer(*integer)]).unwrap();

                table
            }
            Value::Boolean(boolean) => {
                let mut table = Table::new(vec!["boolean".to_string()]);

                table.insert(vec![Value::Boolean(*boolean)]).unwrap();

                table
            }
            Value::List(list) => Self::from(list),
            Value::Empty => Table::new(Vec::with_capacity(0)),
            Value::Map(map) => Self::from(map),
            Value::Table(table) => table.clone(),
            Value::Function(function) => {
                let mut table = Table::new(vec!["function".to_string()]);

                table
                    .insert(vec![Value::Function(function.clone())])
                    .unwrap();

                table
            }
        }
    }
}

impl From<&Vec<Value>> for Table {
    fn from(list: &Vec<Value>) -> Self {
        let mut table = Table::new(vec!["index".to_string(), "item".to_string()]);

        for (i, value) in list.iter().enumerate() {
            table
                .insert(vec![Value::Integer(i as i64), value.clone()])
                .unwrap();
        }

        table
    }
}

impl From<&mut Vec<Value>> for Table {
    fn from(list: &mut Vec<Value>) -> Self {
        let mut table = Table::new(vec!["index".to_string(), "item".to_string()]);

        for (i, value) in list.iter().enumerate() {
            if let Ok(list) = value.as_list() {
                table.insert(list.clone()).unwrap();
            } else {
                table
                    .insert(vec![Value::Integer(i as i64), value.clone()])
                    .unwrap();
            }
        }

        table
    }
}

impl From<Map> for Table {
    fn from(map: Map) -> Self {
        let inner_map = map.inner();
        let read_map = inner_map.read().unwrap();
        let keys = read_map.keys().cloned().collect();
        let values = read_map.values().cloned().collect();
        let mut table = Table::new(keys);

        table
            .insert(values)
            .expect("Failed to create Table from Map. This is a no-op.");

        table
    }
}

impl From<&Map> for Table {
    fn from(map: &Map) -> Self {
        let inner_map = map.inner();
        let read_map = inner_map.read().unwrap();
        let keys = read_map.keys().cloned().collect();
        let values = read_map.values().cloned().collect();
        let mut table = Table::new(keys);

        table
            .insert(values)
            .expect("Failed to create Table from Map. This is a no-op.");

        table
    }
}

impl From<&mut Map> for Table {
    fn from(map: &mut Map) -> Self {
        let inner_map = map.inner();
        let read_map = inner_map.read().unwrap();
        let keys = read_map.keys().cloned().collect();
        let values = read_map.values().cloned().collect();
        let mut table = Table::new(keys);

        table
            .insert(values)
            .expect("Failed to create Table from Map. This is a no-op.");

        table
    }
}

impl Eq for Table {}

impl PartialEq for Table {
    fn eq(&self, other: &Self) -> bool {
        if self.headers != other.headers {
            return false;
        }

        self.rows == other.rows
    }
}

impl PartialOrd for Table {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.headers.partial_cmp(&other.headers)
    }
}

impl Ord for Table {
    fn cmp(&self, other: &Self) -> Ordering {
        self.headers.cmp(&other.headers)
    }
}
