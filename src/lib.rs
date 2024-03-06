pub mod abstract_tree;
pub mod context;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod value;

use abstract_tree::AbstractTree;
use context::Context;
use error::Error;
use lexer::lex;
pub use parser::{parse, parser, DustParser};
pub use value::Value;

pub struct Interpreter {
    context: Context,
}

impl Interpreter {
    pub fn new(context: Context) -> Self {
        Interpreter { context }
    }

    pub fn run(&mut self, source: &str) -> Result<Value, Vec<Error>> {
        let tokens = lex(source)?;
        let statements = parse(&tokens)?;

        let mut value = Value::none();

        for (statement, _span) in statements {
            value = match statement.run(&self.context) {
                Ok(value) => value,
                Err(runtime_error) => return Err(vec![Error::Runtime(runtime_error)]),
            }
        }

        Ok(value)
    }
}
