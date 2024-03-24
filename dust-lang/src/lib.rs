pub mod abstract_tree;
pub mod built_in_functions;
pub mod context;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod value;

use context::Context;
use error::Error;
use lexer::lex;
use parser::parse;
pub use value::Value;

pub fn interpret(source: &str) -> Result<Option<Value>, Vec<Error>> {
    let mut interpreter = Interpreter::new(Context::new());

    interpreter.run(include_str!("../../std/io.ds"))?;
    interpreter.run(include_str!("../../std/thread.ds"))?;
    interpreter.run(source)
}

pub struct Interpreter {
    context: Context,
}

impl Interpreter {
    pub fn new(context: Context) -> Self {
        Interpreter { context }
    }

    pub fn run(&mut self, source: &str) -> Result<Option<Value>, Vec<Error>> {
        let tokens = lex(source)?;
        let abstract_tree = parse(&tokens)?;
        let value_option = abstract_tree.run(&self.context)?;

        Ok(value_option)
    }
}
