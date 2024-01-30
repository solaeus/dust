use std::{collections::BTreeMap, env::args, sync::OnceLock};

use enum_iterator::{all, Sequence};
use serde::{Deserialize, Serialize};

use crate::{
    built_in_functions::{
        fs::fs_functions, json::json_functions, string::string_functions, Callable,
    },
    AbstractTree, BuiltInFunction, Format, Function, List, Map, Result, SyntaxNode, Type, Value,
};

static ARGS: OnceLock<Value> = OnceLock::new();
static FS: OnceLock<Value> = OnceLock::new();
static JSON: OnceLock<Value> = OnceLock::new();
static RANDOM: OnceLock<Value> = OnceLock::new();
static STRING: OnceLock<Value> = OnceLock::new();

pub fn built_in_values() -> impl Iterator<Item = BuiltInValue> {
    all()
}

#[derive(Sequence, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BuiltInValue {
    Args,
    AssertEqual,
    Fs,
    Json,
    Length,
    Output,
    Random,
    Str,
}

impl BuiltInValue {
    pub fn name(&self) -> &'static str {
        match self {
            BuiltInValue::Args => "args",
            BuiltInValue::AssertEqual => "assert_equal",
            BuiltInValue::Fs => "fs",
            BuiltInValue::Json => "json",
            BuiltInValue::Length => "length",
            BuiltInValue::Output => "output",
            BuiltInValue::Random => "random",
            BuiltInValue::Str => "str",
        }
    }

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
    fn from_syntax(node: SyntaxNode, _source: &str, _context: &Map) -> Result<Self> {
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

    fn run(&self, _source: &str, _context: &Map) -> Result<Value> {
        Ok(self.get().clone())
    }

    fn expected_type(&self, _context: &Map) -> Result<Type> {
        Ok(self.r#type())
    }
}

impl Format for BuiltInValue {
    fn format(&self, output: &mut String, _indent_level: u8) {
        output.push_str(&self.get().to_string());
    }
}
