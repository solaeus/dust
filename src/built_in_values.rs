use std::{env::args, sync::OnceLock};

use enum_iterator::{all, Sequence};
use serde::{Deserialize, Serialize};

use crate::{
    built_in_functions::{
        fs::all_fs_functions, io::all_io_functions, json::json_functions, str::string_functions,
        Callable,
    },
    BuiltInFunction, EnumInstance, Function, Identifier, List, Map, Value,
};

static ARGS: OnceLock<Value> = OnceLock::new();
static FS: OnceLock<Value> = OnceLock::new();
static IO: OnceLock<Value> = OnceLock::new();
static JSON: OnceLock<Value> = OnceLock::new();
static NONE: OnceLock<Value> = OnceLock::new();
static RANDOM: OnceLock<Value> = OnceLock::new();
static STR: OnceLock<Value> = OnceLock::new();

/// Returns the entire built-in value API.
pub fn all_built_in_values() -> impl Iterator<Item = BuiltInValue> {
    all()
}

/// A variable with a hard-coded key that is globally available.
#[derive(Sequence, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BuiltInValue {
    /// The arguments used to launch the current program.
    Args,

    /// Create an error if two values are not equal.
    AssertEqual,

    /// File system tools.
    Fs,

    /// Input and output tools.
    Io,

    /// JSON format tools.
    Json,

    /// Get the length of a collection.
    Length,

    /// The absence of a value.
    None,

    /// Print a value to stdout.
    Output,

    /// Random value generators.
    Random,

    /// String utilities.
    Str,
}

impl BuiltInValue {
    /// Returns the hard-coded key used to identify the value.
    pub fn name(&self) -> &'static str {
        match self {
            BuiltInValue::Args => "args",
            BuiltInValue::AssertEqual => "assert_equal",
            BuiltInValue::Fs => "fs",
            BuiltInValue::Io => "io",
            BuiltInValue::Json => "json",
            BuiltInValue::Length => BuiltInFunction::Length.name(),
            BuiltInValue::None => "None",
            BuiltInValue::Output => "output",
            BuiltInValue::Random => "random",
            BuiltInValue::Str => "str",
        }
    }

    /// Returns a brief description of the value's features.
    ///
    /// This is used by the shell when suggesting completions.
    pub fn description(&self) -> &'static str {
        match self {
            BuiltInValue::Args => "The command line arguments sent to this program.",
            BuiltInValue::AssertEqual => "Error if the two values are not equal.",
            BuiltInValue::Fs => "File and directory tools.",
            BuiltInValue::Io => "Input/output tools.",
            BuiltInValue::Json => "JSON formatting tools.",
            BuiltInValue::Length => BuiltInFunction::Length.description(),
            BuiltInValue::None => "The absence of a value.",
            BuiltInValue::Output => "output",
            BuiltInValue::Random => "random",
            BuiltInValue::Str => "string",
        }
    }

    /// Returns the value by creating it or, if it has already been accessed, retrieving it from its
    /// [OnceLock][].
    pub fn get(&self) -> Value {
        match self {
            BuiltInValue::Args => ARGS
                .get_or_init(|| {
                    let args = args().map(|arg| Value::string(arg.to_string())).collect();

                    Value::List(List::with_items(args))
                })
                .clone(),
            BuiltInValue::AssertEqual => {
                Value::Function(Function::BuiltIn(BuiltInFunction::AssertEqual))
            }
            BuiltInValue::Io => IO
                .get_or_init(|| {
                    let mut io_map = Map::new();

                    for io_function in all_io_functions() {
                        let key = io_function.name();
                        let value =
                            Value::Function(Function::BuiltIn(BuiltInFunction::Io(io_function)));

                        io_map.set(Identifier::new(key), value);
                    }

                    Value::Map(io_map)
                })
                .clone(),
            BuiltInValue::Fs => FS
                .get_or_init(|| {
                    let mut fs_map = Map::new();

                    for fs_function in all_fs_functions() {
                        let key = fs_function.name();
                        let value =
                            Value::Function(Function::BuiltIn(BuiltInFunction::Fs(fs_function)));

                        fs_map.set(Identifier::new(key), value);
                    }

                    Value::Map(fs_map)
                })
                .clone(),
            BuiltInValue::Json => JSON
                .get_or_init(|| {
                    let mut json_map = Map::new();

                    for json_function in json_functions() {
                        let key = json_function.name();
                        let value = Value::Function(Function::BuiltIn(BuiltInFunction::Json(
                            json_function,
                        )));

                        json_map.set(Identifier::new(key), value);
                    }

                    Value::Map(json_map)
                })
                .clone(),
            BuiltInValue::Length => Value::Function(Function::BuiltIn(BuiltInFunction::Length)),
            BuiltInValue::None => NONE
                .get_or_init(|| {
                    Value::Enum(EnumInstance::new(
                        Identifier::new("Option"),
                        Identifier::new("None"),
                        None,
                    ))
                })
                .clone(),
            BuiltInValue::Output => Value::Function(Function::BuiltIn(BuiltInFunction::Output)),
            BuiltInValue::Random => RANDOM
                .get_or_init(|| {
                    let mut random_map = Map::new();

                    for built_in_function in [
                        BuiltInFunction::RandomBoolean,
                        BuiltInFunction::RandomFloat,
                        BuiltInFunction::RandomFrom,
                        BuiltInFunction::RandomInteger,
                    ] {
                        let identifier = Identifier::new(built_in_function.name());
                        let value = Value::Function(Function::BuiltIn(built_in_function));

                        random_map.set(identifier, value);
                    }

                    Value::Map(random_map)
                })
                .clone(),
            BuiltInValue::Str => STR
                .get_or_init(|| {
                    let mut str_map = Map::new();

                    for string_function in string_functions() {
                        let identifier = Identifier::new(string_function.name());
                        let value = Value::Function(Function::BuiltIn(BuiltInFunction::String(
                            string_function,
                        )));

                        str_map.set(identifier, value);
                    }

                    Value::Map(str_map)
                })
                .clone(),
        }
    }
}
