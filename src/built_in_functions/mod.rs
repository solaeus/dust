/// Built-in functions that are available to all Dust programs.
use crate::{Map, Result, Type, Value};

mod assert;
mod collections;
mod commands;
mod data_formats;
mod fs;
mod network;
mod output;
mod random;
mod r#type;

/// All built-in functions recognized by the interpreter.
///
/// This is the public interface to access built-in functions by iterating over
/// the references it holds.
pub const BUILT_IN_FUNCTIONS: [&dyn BuiltInFunction; 16] = [
    &assert::Assert,
    &assert::AssertEqual,
    &collections::Length,
    &commands::Sh,
    &data_formats::FromJson,
    &data_formats::ToJson,
    &fs::Read,
    &fs::Write,
    &fs::Append,
    &network::Download,
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
