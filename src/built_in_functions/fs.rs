use std::{
    fs::{read_dir, read_to_string, write, File},
    io::Write as IoWrite,
    path::PathBuf,
};

use crate::{BuiltInFunction, List, Map, Result, Type, Value};

pub struct Read;

impl BuiltInFunction for Read {
    fn name(&self) -> &'static str {
        "read"
    }

    fn run(&self, arguments: &[Value], _context: &Map) -> Result<Value> {
        let path_string = arguments.first().unwrap_or_default().as_string()?;
        let path = PathBuf::from(path_string);

        if path.is_dir() {
            let files = List::new();

            for entry in read_dir(&path)? {
                let entry = entry?;
                let file_data = Map::new();

                let name = entry.file_name().to_string_lossy().to_string();
                let metadata = entry.metadata()?;
                let created = metadata.created()?.elapsed()?.as_secs() as i64;
                let modified = metadata.modified()?.elapsed()?.as_secs() as i64;

                file_data.set("name".to_string(), Value::String(name), None)?;
                file_data.set("created".to_string(), Value::Integer(created), None)?;
                file_data.set("modified".to_string(), Value::Integer(modified), None)?;

                files.items_mut().push(Value::Map(file_data));
            }

            return Ok(Value::List(files));
        }

        let file_content = read_to_string(path)?;

        Ok(Value::String(file_content))
    }

    fn r#type(&self) -> Type {
        Type::Function {
            parameter_types: vec![Type::String],
            return_type: Box::new(Type::String),
        }
    }
}

pub struct Write;

impl BuiltInFunction for Write {
    fn name(&self) -> &'static str {
        "write"
    }

    fn run(&self, arguments: &[Value], _context: &Map) -> Result<Value> {
        let file_content = arguments.first().unwrap_or_default().as_string()?;
        let path = arguments.get(1).unwrap_or_default().as_string()?;

        write(path, file_content)?;

        Ok(Value::none())
    }

    fn r#type(&self) -> Type {
        Type::Function {
            parameter_types: vec![Type::String],
            return_type: Box::new(Type::None),
        }
    }
}

pub struct Append;

impl BuiltInFunction for Append {
    fn name(&self) -> &'static str {
        "append"
    }

    fn run(&self, arguments: &[Value], _context: &Map) -> Result<Value> {
        let file_content = arguments.get(0).unwrap_or_default().as_string()?;
        let path = arguments.get(1).unwrap_or_default().as_string()?;

        File::options()
            .append(true)
            .create(true)
            .open(path)?
            .write_all(file_content.as_bytes())?;

        Ok(Value::none())
    }

    fn r#type(&self) -> Type {
        Type::Function {
            parameter_types: vec![Type::String, Type::String],
            return_type: Box::new(Type::None),
        }
    }
}
