//! Convert values to and from data formats like JSON and TOML.

use crate::{Result, Table, Tool, ToolInfo, Value, ValueType};

pub struct FromJson;

impl Tool for FromJson {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "from_json",
            description: "Get a whale value from a JSON string.",
            group: "data",
            inputs: vec![ValueType::String],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let argument = argument.as_string()?;
        let value = serde_json::from_str(argument)?;

        Ok(value)
    }
}

pub struct ToJson;

impl Tool for ToJson {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "to_json",
            description: "Create a JSON string from a whale value.",
            group: "data",
            inputs: vec![],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let json = serde_json::to_string(argument)?;

        Ok(Value::String(json))
    }
}

pub struct FromCsv;

impl Tool for FromCsv {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "from_csv",
            description: "Create a whale value from a CSV string.",
            group: "data",
            inputs: vec![ValueType::String],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let csv = argument.as_string()?;
        let mut reader = csv::Reader::from_reader(csv.as_bytes());

        let headers = reader
            .headers()?
            .iter()
            .map(|header| header.trim().trim_matches('"').to_string())
            .collect();

        let mut table = Table::new(headers);

        for result in reader.records() {
            let row = result?
                .iter()
                .map(|column| {
                    let column = column.trim().trim_matches('"').trim_matches('\'');

                    if let Ok(integer) = column.parse::<i64>() {
                        Value::Integer(integer)
                    } else if let Ok(float) = column.parse::<f64>() {
                        Value::Float(float)
                    } else {
                        Value::String(column.to_string())
                    }
                })
                .collect();

            table.insert(row)?;
        }

        Ok(Value::Table(table))
    }
}

pub struct ToCsv;

impl Tool for ToCsv {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "to_csv",
            description: "Convert a value to a string of comma-separated values.",
            group: "data",
            inputs: vec![],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let mut buffer = Vec::new();
        let mut writer = csv::Writer::from_writer(&mut buffer);

        match argument {
            Value::String(string) => {
                writer.write_record([string])?;
            }
            Value::Float(float) => {
                writer.write_record(&[float.to_string()])?;
            }
            Value::Integer(integer) => {
                writer.write_record(&[integer.to_string()])?;
            }
            Value::Boolean(boolean) => {
                writer.write_record(&[boolean.to_string()])?;
            }
            Value::List(list) => {
                let string_list = list.iter().map(|value| value.to_string());

                writer.write_record(string_list)?;
            }
            Value::Empty => {}
            Value::Map(map) => {
                writer.write_record(map.inner().keys())?;
                writer.write_record(map.inner().values().map(|value| value.to_string()))?;
            }
            Value::Table(table) => {
                writer.write_record(table.column_names())?;

                for row in table.rows() {
                    let row_string = row.iter().map(|value| value.to_string());

                    writer.write_record(row_string)?;
                }
            }
            Value::Function(_) => todo!(),
            Value::Time(time) => {
                writer.write_record(&[time.to_string()])?;
            }
        }

        writer.flush()?;

        Ok(Value::String(
            String::from_utf8_lossy(writer.get_ref()).to_string(),
        ))
    }
}
