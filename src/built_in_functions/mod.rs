use crate::{Result, Value};

mod fs;
mod output;

pub const BUILT_IN_FUNCTIONS: [&dyn BuiltInFunction; 4] =
    [&output::Output, &fs::Read, &fs::Write, &fs::Append];

pub trait BuiltInFunction {
    fn name(&self) -> &'static str;
    fn run(&self, arguments: &[Value]) -> Result<Value>;
}
