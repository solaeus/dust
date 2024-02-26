pub mod abstract_tree;
pub mod context;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod value;

use abstract_tree::Statement;
use chumsky::{prelude::*, Parser};
use context::Context;
use error::Error;
pub use value::Value;

pub struct Interpreter<P> {
    _parser: P,
    _context: Context,
}

impl<'src, P> Interpreter<P>
where
    P: Parser<'src, &'src str, Statement<'src>, extra::Err<Rich<'src, char>>>,
{
    pub fn run(&self, _source: &'src str) -> Result<Value, Error<'src>> {
        todo!();

        // let final_value = self
        //     .parser
        //     .parse(source)
        //     .into_result()?
        //     .run(&self.context)?;

        // Ok(final_value)
    }
}
