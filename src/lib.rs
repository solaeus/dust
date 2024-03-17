pub mod abstract_tree;
pub mod context;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod value;

use abstract_tree::{AbstractTree, Action, WithPosition};
use context::Context;
use error::Error;
use lexer::lex;
pub use parser::{parse, parser, DustParser};
pub use value::Value;

pub fn interpret(source: &str) -> Result<Option<Value>, Vec<Error>> {
    let context = Context::new();
    let mut interpreter = Interpreter::new(context);

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
        let statements = parse(&tokens)?;
        let errors = statements
            .iter()
            .filter_map(|WithPosition { node, position }| {
                node.validate(&self.context)
                    .err()
                    .map(|error| Error::Validation {
                        error,
                        position: position.clone(),
                    })
            })
            .collect::<Vec<Error>>();

        if !errors.is_empty() {
            return Err(errors);
        }

        let mut value = None;

        for statement in statements {
            value = match statement.node.run(&self.context) {
                Ok(action) => match action {
                    Action::Break => None,
                    Action::Return(value) => Some(value),
                    Action::None => continue,
                },
                Err(runtime_error) => {
                    return Err(vec![Error::Runtime {
                        error: runtime_error,
                        position: statement.position,
                    }])
                }
            }
        }

        Ok(value)
    }
}
