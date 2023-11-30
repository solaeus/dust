use crate::{Map, Result, Value};

mod assert;
mod fs;
mod output;
mod r#type;

pub const BUILT_IN_FUNCTIONS: [&dyn BuiltInFunction; 7] = [
    &assert::Assert,
    &assert::AssertEqual,
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
