use std::{env::args, sync::OnceLock};

use enum_iterator::{all, Sequence};
use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::{
    built_in_functions::string_functions, AbstractTree, BuiltInFunction, Function, Identifier,
    List, Result, Structure, Type, Value,
};

static ARGS: OnceLock<Value> = OnceLock::new();
static FS: OnceLock<Value> = OnceLock::new();
static JSON: OnceLock<Value> = OnceLock::new();
static RANDOM: OnceLock<Value> = OnceLock::new();
static STRING: OnceLock<Value> = OnceLock::new();

pub fn built_in_values() -> impl Iterator<Item = BuiltInValue> {
    all()
}

#[derive(Sequence, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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
    pub fn name(&self) -> &'static str {
        match self {
            BuiltInValue::Args => "args",
            BuiltInValue::AssertEqual => "assert_equal",
            BuiltInValue::Fs => "fs",
            BuiltInValue::Json => "json",
            BuiltInValue::Length => "length",
            BuiltInValue::Output => "output",
            BuiltInValue::Random => "random",
            BuiltInValue::String => "string",
        }
    }

    pub fn get(self) -> Value {
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
            BuiltInValue::Fs => FS
                .get_or_init(|| {
                    let fs_context = Structure::default();

                    fs_context
                        .set(
                            "read".to_string(),
                            Value::Function(Function::BuiltIn(BuiltInFunction::FsRead)),
                            None,
                        )
                        .unwrap();

                    Value::Structure(fs_context)
                })
                .clone(),
            BuiltInValue::Json => JSON
                .get_or_init(|| {
                    let json_context = Structure::default();

                    json_context
                        .set(
                            "parse".to_string(),
                            Value::Function(Function::BuiltIn(BuiltInFunction::JsonParse)),
                            None,
                        )
                        .unwrap();

                    Value::Structure(json_context)
                })
                .clone(),
            BuiltInValue::Length => Value::Function(Function::BuiltIn(BuiltInFunction::Length)),
            BuiltInValue::Output => Value::Function(Function::BuiltIn(BuiltInFunction::Output)),
            BuiltInValue::Random => RANDOM
                .get_or_init(|| {
                    let random_context = Structure::default();

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

                    Value::Structure(random_context)
                })
                .clone(),
            BuiltInValue::String => STRING
                .get_or_init(|| {
                    let string_context = Structure::default();

                    {
                        let mut variables = string_context.variables_mut().unwrap();

                        for string_function in string_functions() {
                            let key = string_function.name().to_string();
                            let value = Value::Function(Function::BuiltIn(
                                BuiltInFunction::String(string_function),
                            ));
                            let r#type = string_function.r#type();

                            variables.insert(key, (value, r#type));
                        }
                    }

                    Value::Structure(string_context)
                })
                .clone(),
        }
    }

    fn r#type(&self) -> Type {
        match self {
            BuiltInValue::Args => Type::list(Type::String),
            BuiltInValue::AssertEqual => BuiltInFunction::AssertEqual.r#type(),
            BuiltInValue::Fs => Type::Structure(Identifier::from(self.name())),
            BuiltInValue::Json => Type::Structure(Identifier::from(self.name())),
            BuiltInValue::Length => BuiltInFunction::Length.r#type(),
            BuiltInValue::Output => BuiltInFunction::Output.r#type(),
            BuiltInValue::Random => Type::Structure(Identifier::from(self.name())),
            BuiltInValue::String => Type::Structure(Identifier::from(self.name())),
        }
    }
}

impl AbstractTree for BuiltInValue {
    fn from_syntax_node(_source: &str, node: Node, _context: &Structure) -> Result<Self> {
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

    fn run(&self, _source: &str, _context: &Structure) -> Result<Value> {
        Ok(self.get().clone())
    }

    fn check_type(&self, _context: &Structure) -> Result<()> {
        Ok(())
    }

    fn expected_type(&self, _context: &Structure) -> Result<Type> {
        Ok(self.r#type())
    }
}
