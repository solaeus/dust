use std::{collections::BTreeMap, env::args, sync::OnceLock};

use enum_iterator::{all, Sequence};
use serde::{Deserialize, Serialize};

use crate::{
    built_in_functions::{fs::fs_functions, json::json_functions, str::string_functions, Callable},
    error::{RuntimeError, SyntaxError, ValidationError},
    AbstractTree, BuiltInFunction, Format, Function, List, Map, SyntaxNode, Type, Value,
};

static ARGS: OnceLock<Value> = OnceLock::new();
static FS: OnceLock<Value> = OnceLock::new();
static JSON: OnceLock<Value> = OnceLock::new();
static RANDOM: OnceLock<Value> = OnceLock::new();
static STRING: OnceLock<Value> = OnceLock::new();

/// Returns the entire built-in value API.
pub fn built_in_values() -> impl Iterator<Item = BuiltInValue> {
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

    /// JSON format tools.
    Json,

    /// Get the length of a collection.
    Length,

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
            BuiltInValue::Json => "json",
            BuiltInValue::Length => BuiltInFunction::Length.name(),
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
            BuiltInValue::Json => "JSON formatting tools.",
            BuiltInValue::Length => BuiltInFunction::Length.description(),
            BuiltInValue::Output => "output",
            BuiltInValue::Random => "random",
            BuiltInValue::Str => "string",
        }
    }

    /// Returns the value's type.
    ///
    /// This is checked with a unit test to ensure it matches the value.
    pub fn r#type(&self) -> Type {
        match self {
            BuiltInValue::Args => Type::list(Type::String),
            BuiltInValue::AssertEqual => BuiltInFunction::AssertEqual.r#type(),
            BuiltInValue::Fs => Type::Map(None),
            BuiltInValue::Json => Type::Map(None),
            BuiltInValue::Length => BuiltInFunction::Length.r#type(),
            BuiltInValue::Output => BuiltInFunction::Output.r#type(),
            BuiltInValue::Random => Type::Map(None),
            BuiltInValue::Str => Type::Map(None),
        }
    }

    /// Returns the value by creating it or, if it has already been accessed, retrieving it from its
    /// [OnceLock][].
    pub fn get(&self) -> &Value {
        match self {
            BuiltInValue::Args => ARGS.get_or_init(|| {
                let args = args().map(|arg| Value::string(arg.to_string())).collect();

                Value::List(List::with_items(args))
            }),
            BuiltInValue::AssertEqual => {
                &Value::Function(Function::BuiltIn(BuiltInFunction::AssertEqual))
            }
            BuiltInValue::Fs => FS.get_or_init(|| {
                let mut fs_context = BTreeMap::new();

                for fs_function in fs_functions() {
                    let key = fs_function.name().to_string();
                    let value =
                        Value::Function(Function::BuiltIn(BuiltInFunction::Fs(fs_function)));
                    let r#type = value.r#type();

                    fs_context.insert(key, (value, r#type));
                }

                Value::Map(Map::with_variables(fs_context))
            }),
            BuiltInValue::Json => JSON.get_or_init(|| {
                let mut json_context = BTreeMap::new();

                for json_function in json_functions() {
                    let key = json_function.name().to_string();
                    let value =
                        Value::Function(Function::BuiltIn(BuiltInFunction::Json(json_function)));
                    let r#type = value.r#type();

                    json_context.insert(key, (value, r#type));
                }

                Value::Map(Map::with_variables(json_context))
            }),
            BuiltInValue::Length => &Value::Function(Function::BuiltIn(BuiltInFunction::Length)),
            BuiltInValue::Output => &Value::Function(Function::BuiltIn(BuiltInFunction::Output)),
            BuiltInValue::Random => RANDOM.get_or_init(|| {
                let mut random_context = BTreeMap::new();

                for built_in_function in [
                    BuiltInFunction::RandomBoolean,
                    BuiltInFunction::RandomFloat,
                    BuiltInFunction::RandomFrom,
                    BuiltInFunction::RandomInteger,
                ] {
                    let key = built_in_function.name().to_string();
                    let value = Value::Function(Function::BuiltIn(built_in_function));
                    let r#type = built_in_function.r#type();

                    random_context.insert(key, (value, r#type));
                }

                Value::Map(Map::with_variables(random_context))
            }),
            BuiltInValue::Str => STRING.get_or_init(|| {
                let mut string_context = BTreeMap::new();

                for string_function in string_functions() {
                    let key = string_function.name().to_string();
                    let value = Value::Function(Function::BuiltIn(BuiltInFunction::String(
                        string_function,
                    )));
                    let r#type = string_function.r#type();

                    string_context.insert(key, (value, r#type));
                }

                Value::Map(Map::with_variables(string_context))
            }),
        }
    }
}

impl AbstractTree for BuiltInValue {
    fn from_syntax(node: SyntaxNode, _source: &str, _context: &Map) -> Result<Self, SyntaxError> {
        let built_in_value = match node.kind() {
            "args" => BuiltInValue::Args,
            "assert_equal" => BuiltInValue::AssertEqual,
            "fs" => BuiltInValue::Fs,
            "json" => BuiltInValue::Json,
            "length" => BuiltInValue::Length,
            "output" => BuiltInValue::Output,
            "random" => BuiltInValue::Random,
            "str" => BuiltInValue::Str,
            _ => todo!(),
        };

        Ok(built_in_value)
    }

    fn expected_type(&self, _context: &Map) -> Result<Type, ValidationError> {
        Ok(self.r#type())
    }

    fn validate(&self, _source: &str, _context: &Map) -> Result<(), ValidationError> {
        Ok(())
    }

    fn run(&self, _source: &str, _context: &Map) -> Result<Value, RuntimeError> {
        Ok(self.get().clone())
    }
}

impl Format for BuiltInValue {
    fn format(&self, output: &mut String, _indent_level: u8) {
        output.push_str(&self.get().to_string());
    }
}

#[cfg(test)]
mod tests {
    use crate::built_in_values;

    #[test]
    fn check_built_in_types() {
        for built_in_value in built_in_values() {
            let expected = built_in_value.r#type();
            let actual = built_in_value.get().r#type();

            assert_eq!(expected, actual);
        }
    }
}
