use crate::{Map, Result, Value};

mod assert;
mod data_formats;
mod fs;
mod output;
mod r#type;

pub const BUILT_IN_FUNCTIONS: [&dyn BuiltInFunction; 9] = [
    &assert::Assert,
    &assert::AssertEqual,
    &data_formats::FromJson,
    &data_formats::ToJson,
    &fs::Read,
    &fs::Write,
    &fs::Append,
    &output::Output,
    &r#type::Type,
];

pub trait BuiltInFunction {
    fn name(&self) -> &'static str;
    fn run(&self, arguments: &[Value], context: &Map) -> Result<Value>;
}
