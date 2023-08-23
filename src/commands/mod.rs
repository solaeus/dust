//! This module contains dust's built-in commands.
//!
//! ## Writing macros
//!
//! - snake case identifier, this is enforced by a test
//! - the description should be brief, it will display in the shell
//!
//!
//! ## Usage
//!
//! Macros can be used in Rust by passing a Value to the run method.
//!
//! ```
//! let value = Value::List(vec![1, 2,3]);
//! let count = Count.run(value).as_string().unwrap();
//!
//! assert_eq!(count, 3);
//! ```

use crate::{Result, Value, ValueType};

pub mod collections;
pub mod command;
pub mod data_formats;
pub mod disks;
pub mod filesystem;
pub mod general;
pub mod gui;
pub mod logic;
pub mod network;
pub mod package_management;
pub mod random;
pub mod system;
pub mod time;

/// Master list of all macros.
///
/// This list is used to match identifiers with macros and to provide info to
/// the shell.
pub const MACRO_LIST: [&'static dyn Macro; 57] = [
    &collections::Count,
    &collections::CreateTable,
    &collections::Get,
    &collections::Insert,
    &collections::Rows,
    &collections::Select,
    &collections::String,
    &collections::Transform,
    &collections::Where,
    &command::Bash,
    &command::Fish,
    &command::Raw,
    &command::Sh,
    &command::Zsh,
    &data_formats::FromCsv,
    &data_formats::ToCsv,
    &data_formats::FromJson,
    &data_formats::ToJson,
    &disks::ListDisks,
    &disks::Partition,
    &filesystem::Append,
    &filesystem::CreateDir,
    &filesystem::FileMetadata,
    &filesystem::MoveDir,
    &filesystem::ReadDir,
    &filesystem::ReadFile,
    &filesystem::RemoveDir,
    &filesystem::Trash,
    &filesystem::Watch,
    &filesystem::Write,
    &general::Async,
    &general::Output,
    &general::Repeat,
    &general::Run,
    &general::Wait,
    &gui::BarGraph,
    &gui::Plot,
    &gui::Gui,
    &logic::If,
    &logic::IfElse,
    &logic::Loop,
    &network::Download,
    &package_management::CoprRepositories,
    &package_management::EnableRpmRepositories,
    &package_management::InstallPackage,
    &package_management::UninstallPackage,
    &package_management::UpgradePackages,
    &random::Random,
    &random::RandomBoolean,
    &random::RandomFloat,
    &random::RandomInteger,
    &random::RandomString,
    &system::CpuSpeed,
    &logic::Assert,
    &logic::AssertEqual,
    &time::Local,
    &time::Now,
];

/// A whale macro function.
pub trait Macro: Sync + Send {
    fn info(&self) -> MacroInfo<'static>;
    fn run(&self, argument: &Value) -> Result<Value>;
}

/// Information needed for each macro.
#[derive(Clone, Debug, PartialEq)]
pub struct MacroInfo<'a> {
    /// Text pattern that triggers this macro.
    pub identifier: &'a str,

    /// User-facing information about how the macro works.
    pub description: &'a str,

    /// Category used to sort macros in the shell.
    pub group: &'a str,

    pub inputs: Vec<ValueType>,
}

// pub struct SystemInfo;

// impl Macro for SystemInfo {
//     fn info(&self) -> MacroInfo<'static> {
//         MacroInfo {
//             identifier: "system_info",
//             description: "Get information on the system.",
//         }
//     }

//     fn run(&self, argument: &Value) -> crate::Result<Value> {
//         argument.as_empty()?;

//         let mut map = VariableMap::new();

//         map.set_value("hostname", Value::String(hostname()?))?;

//         Ok(Value::Map(map))
//     }
// }

// pub struct Sort;

// impl Macro for Sort {
//     fn info(&self) -> MacroInfo<'static> {
//         MacroInfo {
//             identifier: "sort",
//             description: "Apply default ordering.",
//         }
//     }

//     fn run(&self, argument: &Value) -> Result<Value> {
//         if let Ok(mut list) = argument.as_list().cloned() {
//             list.sort();

//             Ok(Value::List(list))
//         } else if let Ok(map) = argument.as_map().cloned() {
//             Ok(Value::Map(map))
//         } else if let Ok(mut table) = argument.as_table().cloned() {
//             table.sort();

//             Ok(Value::Table(table))
//         } else {
//             Err(crate::Error::ExpectedList {
//                 actual: argument.clone(),
//             })
//         }
//     }
// }

// pub struct Map;

// impl Macro for Map {
//     fn info(&self) -> MacroInfo<'static> {
//         MacroInfo {
//             identifier: "map",
//             description: "Create a map from a value.",
//         }
//     }

//     fn run(&self, argument: &Value) -> Result<Value> {
//         match argument {
//             Value::String(_) => todo!(),
//             Value::Float(_) => todo!(),
//             Value::Integer(_) => todo!(),
//             Value::Boolean(_) => todo!(),
//             Value::List(_) => todo!(),
//             Value::Map(_) => todo!(),
//             Value::Table(table) => Ok(Value::Map(VariableMap::from(table))),
//             Value::Function(_) => todo!(),
//             Value::Empty => todo!(),
//         }
//     }
// }

// pub struct Status;

// impl Macro for Status {
//     fn info(&self) -> MacroInfo<'static> {
//         MacroInfo {
//             identifier: "git_status",
//             description: "Get the repository status for the current directory.",
//         }
//     }

//     fn run(&self, argument: &Value) -> Result<Value> {
//         argument.as_empty()?;

//         let repo = Repository::open(".")?;
//         let mut table = Table::new(vec![
//             "path".to_string(),
//             "status".to_string(),
//             "staged".to_string(),
//         ]);

//         for entry in repo.statuses(None)?.into_iter() {
//             let (status, staged) = {
//                 if entry.status().is_wt_new() {
//                     ("created".to_string(), false)
//                 } else if entry.status().is_wt_deleted() {
//                     ("deleted".to_string(), false)
//                 } else if entry.status().is_wt_modified() {
//                     ("modified".to_string(), false)
//                 } else if entry.status().is_index_new() {
//                     ("created".to_string(), true)
//                 } else if entry.status().is_index_deleted() {
//                     ("deleted".to_string(), true)
//                 } else if entry.status().is_index_modified() {
//                     ("modified".to_string(), true)
//                 } else if entry.status().is_ignored() {
//                     continue;
//                 } else {
//                     ("".to_string(), false)
//                 }
//             };
//             let path = entry.path().unwrap().to_string();

//             table.insert(vec![
//                 Value::String(path),
//                 Value::String(status),
//                 Value::Boolean(staged),
//             ])?;
//         }

//         Ok(Value::Table(table))
//     }
// }

// pub struct DocumentConvert;

// impl Macro for DocumentConvert {
//     fn info(&self) -> MacroInfo<'static> {
//         MacroInfo {
//             identifier: "convert_document",
//             description: "Convert a file's contents to a format and set the extension.",
//         }
//     }

//     fn run(&self, argument: &Value) -> Result<Value> {
//         let argument = argument.as_list()?;

//         if argument.len() != 3 {
//             return Err(Error::WrongFunctionArgumentAmount {
//                 expected: 3,
//                 actual: argument.len(),
//             });
//         }

//         let (path, from, to) = (
//             argument[0].as_string()?,
//             argument[1].as_string()?,
//             argument[2].as_string()?,
//         );
//         let mut file_name = PathBuf::from(&path);
//         file_name.set_extension(to);
//         let new_file_name = file_name.to_str().unwrap();
//         let script = format!("pandoc --from {from} --to {to} --output {new_file_name} {path}");

//         Command::new("fish").arg("-c").arg(script).spawn()?.wait()?;

//         Ok(Value::Empty)
//     }
// }

// pub struct Trash;

// impl Macro for Trash {
//     fn info(&self) -> MacroInfo<'static> {
//         MacroInfo {
//             identifier: "trash_dir",
//             description: "Move a directory to the trash.",
//         }
//     }

//     fn run(&self, argument: &Value) -> Result<Value> {
//         let path = argument.as_string()?;

//         trash::delete(path)?;

//         Ok(Value::Empty)
//     }
// }

// pub struct Get;

// impl Macro for Get {
//     fn info(&self) -> MacroInfo<'static> {
//         MacroInfo {
//             identifier: "get",
//             description: "Extract a value from a collection.",
//         }
//     }

//     fn run(&self, argument: &Value) -> Result<Value> {
//         let argument_list = argument.as_list()?;
//         let collection = &argument_list[0];
//         let index = &argument_list[1];

//         if let Ok(list) = collection.as_list() {
//             let index = index.as_int()?;
//             let value = list.get(index as usize).unwrap_or(&Value::Empty);

//             return Ok(value.clone());
//         }

//         if let Ok(table) = collection.as_table() {
//             let index = index.as_int()?;
//             let get_row = table.get(index as usize);

//             if let Some(row) = get_row {
//                 return Ok(Value::List(row.clone()));
//             }
//         }

//         Err(Error::TypeError {
//             expected: &[
//                 ValueType::List,
//                 ValueType::Map,
//                 ValueType::Table,
//                 ValueType::String,
//             ],
//             actual: collection.clone(),
//         })
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macro_formatting() {
        for function in MACRO_LIST {
            let identifier = function.info().identifier;

            assert_eq!(identifier.to_lowercase(), identifier);
            assert!(identifier.is_ascii());
            assert!(!identifier.is_empty());
            assert!(!identifier.contains(' '));
            assert!(!identifier.contains(':'));
            assert!(!identifier.contains('.'));
            assert!(!identifier.contains('-'));
        }
    }
}
