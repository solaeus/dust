use crate::{Map, Result, Type, Value};

mod assert;
mod collections;
mod data_formats;
mod fs;
mod network;
mod output;
mod random;
mod r#type;

pub const BUILT_IN_FUNCTIONS: [&dyn BuiltInFunction; 14] = [
    &assert::Assert,
    &assert::AssertEqual,
    &collections::Length,
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
    &r#type::TypeFunction,
];

pub trait BuiltInFunction {
    fn name(&self) -> &'static str;
    fn run(&self, arguments: &[Value], context: &Map) -> Result<Value>;
    fn r#type(&self) -> Type;
}
