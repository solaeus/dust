use crate::{Map, Result, TypeDefinition, Value};

mod assert;
mod data_formats;
mod fs;
mod output;
mod random;
mod r#type;

pub const BUILT_IN_FUNCTIONS: [&dyn BuiltInFunction; 13] = [
    &assert::Assert,
    &assert::AssertEqual,
    &data_formats::FromJson,
    &data_formats::ToJson,
    &fs::Read,
    &fs::Write,
    &fs::Append,
    &output::Output,
    &random::Random,
    &random::RandomBoolean,
    &random::RandomFloat,
    &random::RandomInteger,
    &r#type::Type,
];

pub trait BuiltInFunction {
    fn name(&self) -> &'static str;
    fn run(&self, arguments: &[Value], context: &Map) -> Result<Value>;
    fn type_definition(&self) -> TypeDefinition;
}
