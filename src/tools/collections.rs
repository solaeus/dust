//! Macros for collection values: strings, lists, maps and tables.
//!
//! Tests for this module are written in Dust and can be found at tests/collections.ds.

use crate::{Error, Result, Table, Tool, ToolInfo, Value, ValueType, VariableMap};

pub struct Transform;

impl Tool for Transform {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "transform",
            description: "Alter a list by calling a function on each value.",
            group: "collections",
            inputs: vec![ValueType::ListExact(vec![
                ValueType::List,
                ValueType::Function,
            ])],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let argument = self.check_type(argument)?.as_list()?;
        let list = argument[0].as_list()?;
        let function = argument[1].as_function()?;
        let mut mapped_list = Vec::with_capacity(list.len());

        for value in list {
            let mut context = VariableMap::new();

            context.set_value("input", value.clone())?;

            let mapped_value = function.run_with_context(&mut context)?;

            mapped_list.push(mapped_value);
        }

        return Ok(Value::List(mapped_list));
    }
}

pub struct String;

impl Tool for String {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "string",
            description: "Stringify a value.",
            group: "collections",
            inputs: vec![
                ValueType::String,
                ValueType::Function,
                ValueType::Float,
                ValueType::Integer,
                ValueType::Boolean,
            ],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let argument = self.check_type(argument)?;

        let string = match argument {
            Value::String(string) => string.clone(),
            Value::Function(function) => function.to_string(),
            Value::Float(float) => float.to_string(),
            Value::Integer(integer) => integer.to_string(),
            Value::Boolean(boolean) => boolean.to_string(),
            _ => return self.fail(argument),
        };

        Ok(Value::String(string))
    }
}

pub struct Count;

impl Tool for Count {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "count",
            description: "Return the number of items in a collection.",
            group: "collections",
            inputs: vec![
                ValueType::String,
                ValueType::List,
                ValueType::Map,
                ValueType::Table,
            ],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let argument = self.check_type(argument)?;

        let len = match argument {
            Value::String(string) => string.chars().count(),
            Value::List(list) => list.len(),
            Value::Map(map) => map.len(),
            Value::Table(table) => table.len(),
            Value::Function(_)
            | Value::Float(_)
            | Value::Integer(_)
            | Value::Boolean(_)
            | Value::Time(_)
            | Value::Empty => return self.fail(argument),
        };

        Ok(Value::Integer(len as i64))
    }
}

pub struct CreateTable;

impl Tool for CreateTable {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "create_table",
            description: "Define a new table with a list of column names and list of rows.",
            group: "collections",
            inputs: vec![ValueType::ListExact(vec![
                ValueType::ListOf(Box::new(ValueType::String)),
                ValueType::ListOf(Box::new(ValueType::List)),
            ])],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let argument = self.check_type(argument)?.as_list()?;

        let column_name_inputs = argument[0].as_list()?;
        let mut column_names = Vec::with_capacity(column_name_inputs.len());

        for name in column_name_inputs {
            column_names.push(name.as_string()?.clone());
        }

        let column_count = column_names.len();
        let rows = argument[1].as_list()?;
        let mut table = Table::new(column_names);

        for row in rows {
            let row = row.as_fixed_len_list(column_count)?;

            table.insert(row.clone()).unwrap();
        }

        Ok(Value::Table(table))
    }
}

pub struct Rows;

impl Tool for Rows {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "rows",
            description: "Extract a table's rows as a list.",
            group: "collections",
            inputs: vec![ValueType::Table],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let argument = self.check_type(argument)?;

        if let Value::Table(table) = argument {
            let rows = table
                .rows()
                .iter()
                .map(|row| Value::List(row.clone()))
                .collect();

            Ok(Value::List(rows))
        } else {
            self.fail(argument)
        }
    }
}

pub struct Insert;

impl Tool for Insert {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "insert",
            description: "Add new rows to a table.",
            group: "collections",
            inputs: vec![ValueType::ListExact(vec![
                ValueType::Table,
                ValueType::ListOf(Box::new(ValueType::List)),
            ])],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let argument = argument.as_list()?;
        let new_rows = argument[1].as_list()?;
        let mut table = argument[0].as_table()?.clone();

        table.reserve(new_rows.len());

        for row in new_rows {
            let row = row.as_list()?.clone();

            table.insert(row)?;
        }

        Ok(Value::Table(table))
    }
}

pub struct Select;

impl Tool for Select {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "select",
            description: "Extract one or more values based on their key.",
            group: "collections",
            inputs: vec![
                ValueType::ListExact(vec![ValueType::Table, ValueType::String]),
                ValueType::ListExact(vec![
                    ValueType::Table,
                    ValueType::ListOf(Box::new(ValueType::String)),
                ]),
            ],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let arguments = argument.as_fixed_len_list(2)?;
        let collection = &arguments[0];

        if let Value::List(list) = collection {
            let mut selected = Vec::new();

            let index = arguments[1].as_int()?;
            let value = list.get(index as usize);

            if let Some(value) = value {
                selected.push(value.clone());
                return Ok(Value::List(selected));
            } else {
                return Ok(Value::List(selected));
            }
        }

        let mut column_names = Vec::new();

        if let Value::List(columns) = &arguments[1] {
            for column in columns {
                let name = column.as_string()?;

                column_names.push(name.clone());
            }
        } else if let Value::String(column) = &arguments[1] {
            column_names.push(column.clone());
        } else {
            todo!()
        };

        if let Value::Map(map) = collection {
            let mut selected = VariableMap::new();

            for (key, value) in map.inner() {
                if column_names.contains(key) {
                    selected.set_value(key, value.clone())?;
                }
            }

            return Ok(Value::Map(selected));
        }

        if let Value::Table(table) = collection {
            let selected = table.select(&column_names);

            return Ok(Value::Table(selected));
        }

        todo!()
    }
}

pub struct ForEach;

impl Tool for ForEach {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "for_each",
            description: "Run an operation on every item in a collection.",
            group: "collections",
            inputs: vec![],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let argument = argument.as_list()?;

        Error::expected_minimum_function_argument_amount(
            self.info().identifier,
            2,
            argument.len(),
        )?;

        let table = argument[0].as_table()?;
        let columns = argument[1].as_list()?;
        let mut column_names = Vec::new();

        for column in columns {
            let name = column.as_string()?;

            column_names.push(name.clone());
        }

        let selected = table.select(&column_names);

        Ok(Value::Table(selected))
    }
}

pub struct Where;

impl Tool for Where {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "where",
            description: "Keep rows matching a predicate.",
            group: "collections",
            inputs: vec![ValueType::ListExact(vec![
                ValueType::Table,
                ValueType::Function,
            ])],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let argument = self.check_type(argument)?.as_list()?;
        let table = &argument[0].as_table()?;
        let function = argument[1].as_function()?;

        let mut context = VariableMap::new();
        let mut new_table = Table::new(table.column_names().clone());

        for row in table.rows() {
            for (column_index, cell) in row.iter().enumerate() {
                let column_name = table.column_names().get(column_index).unwrap();

                context.set_value(column_name, cell.clone())?;
            }
            let keep_row = function.run_with_context(&mut context)?.as_boolean()?;

            if keep_row {
                new_table.insert(row.clone())?;
            }
        }

        Ok(Value::Table(new_table))
    }
}
