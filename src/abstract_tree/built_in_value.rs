use std::{env::args, sync::OnceLock};

use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{
    built_in_functions::string_functions, AbstractTree, BuiltInFunction, Function, Identifier,
    List, Map, Result, Type, TypeDefinition, Value,
};

static ARGS: OnceLock<Value> = OnceLock::new();
static FS: OnceLock<Value> = OnceLock::new();
static JSON: OnceLock<Value> = OnceLock::new();
static RANDOM: OnceLock<Value> = OnceLock::new();
static STRING: OnceLock<Value> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BuiltInValue {
    Args,
    AssertEqual,
    Fs,
    Json,
    Length,
    Output,
    Random,
    String,
}

impl BuiltInValue {
    fn r#type(&self) -> Type {
        match self {
            BuiltInValue::Args => Type::list(Type::String),
            BuiltInValue::AssertEqual => BuiltInFunction::AssertEqual.r#type(),
            BuiltInValue::Fs => Type::Map(Vec::new()),
            BuiltInValue::Json => Type::Map(Vec::new()),
            BuiltInValue::Length => BuiltInFunction::Length.r#type(),
            BuiltInValue::Output => BuiltInFunction::Output.r#type(),
            BuiltInValue::Random => Type::Map(vec![
                (
                    Identifier::new("boolean".to_string()),
                    TypeDefinition::new(BuiltInFunction::RandomBoolean.r#type()),
                ),
                (
                    Identifier::new("float".to_string()),
                    TypeDefinition::new(BuiltInFunction::RandomFloat.r#type()),
                ),
                (
                    Identifier::new("from".to_string()),
                    TypeDefinition::new(BuiltInFunction::RandomFrom.r#type()),
                ),
                (
                    Identifier::new("integer".to_string()),
                    TypeDefinition::new(BuiltInFunction::RandomInteger.r#type()),
                ),
            ]),
            BuiltInValue::String => Type::Map(Vec::new()),
        }
    }

    fn get(&self) -> &Value {
        match self {
            BuiltInValue::Args => ARGS.get_or_init(|| {
                let args = args().map(|arg| Value::string(arg.to_string())).collect();

                Value::List(List::with_items(args))
            }),
            BuiltInValue::AssertEqual => {
                &Value::Function(Function::BuiltIn(BuiltInFunction::AssertEqual))
            }
            BuiltInValue::Fs => FS.get_or_init(|| {
                let fs_context = Map::new();

                fs_context
                    .set(
                        "read".to_string(),
                        Value::Function(Function::BuiltIn(BuiltInFunction::FsRead)),
                        None,
                    )
                    .unwrap();

                Value::Map(fs_context)
            }),
            BuiltInValue::Json => JSON.get_or_init(|| {
                let json_context = Map::new();

                json_context
                    .set(
                        "parse".to_string(),
                        Value::Function(Function::BuiltIn(BuiltInFunction::JsonParse)),
                        None,
                    )
                    .unwrap();

                Value::Map(json_context)
            }),
            BuiltInValue::Length => &Value::Function(Function::BuiltIn(BuiltInFunction::Length)),
            BuiltInValue::Output => &Value::Function(Function::BuiltIn(BuiltInFunction::Output)),
            BuiltInValue::Random => RANDOM.get_or_init(|| {
                let random_context = Map::new();

                {
                    let mut variables = random_context.variables_mut().unwrap();

                    for built_in_function in [
                        BuiltInFunction::RandomBoolean,
                        BuiltInFunction::RandomFloat,
                        BuiltInFunction::RandomFrom,
                        BuiltInFunction::RandomInteger,
                    ] {
                        let key = built_in_function.name().to_string();
                        let value = Value::Function(Function::BuiltIn(built_in_function));
                        let r#type = built_in_function.r#type();

                        variables.insert(key, (value, r#type));
                    }
                }

                Value::Map(random_context)
            }),
            BuiltInValue::String => STRING.get_or_init(|| {
                let string_context = Map::new();

                {
                    let mut variables = string_context.variables_mut().unwrap();

                    for string_function in string_functions() {
                        let key = string_function.name().to_string();
                        let value = Value::Function(Function::BuiltIn(BuiltInFunction::String(
                            string_function,
                        )));
                        let r#type = string_function.r#type();

                        variables.insert(key, (value, r#type));
                    }
                }

                Value::Map(string_context)
            }),
        }
    }
}

impl AbstractTree for BuiltInValue {
    fn from_syntax_node(_source: &str, node: Node, _context: &Map) -> Result<Self> {
        let built_in_value = match node.kind() {
            "args" => BuiltInValue::Args,
            "assert_equal" => BuiltInValue::AssertEqual,
            "fs" => BuiltInValue::Fs,
            "json" => BuiltInValue::Json,
            "length" => BuiltInValue::Length,
            "output" => BuiltInValue::Output,
            "random" => BuiltInValue::Random,
            "string" => BuiltInValue::String,
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
