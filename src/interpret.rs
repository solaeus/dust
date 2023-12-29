//! The top level of Dust's API with functions to interpret Dust code.
//!
//! You can use this library externally by calling either of the "eval"
//! functions or by constructing your own Evaluator.
use tree_sitter::{Parser, Tree as TSTree};

use crate::{language, AbstractTree, Error, Map, Result, Root, Value};

/// Interpret the given source code.
///
/// Returns a vector of results from evaluating the source code. Each comment
/// and statemtent will have its own result.
///
/// # Examples
///
/// ```rust
/// # use dust_lang::*;
/// assert_eq!(interpret("1 + 2 + 3"), Ok(Value::Integer(6)));
/// ```
pub fn interpret(source: &str) -> Result<Value> {
    interpret_with_context(source, Map::new())
}

/// Interpret the given source code with the given context.
///
/// # Examples
///
/// ```rust
/// # use dust_lang::*;
/// let context = Map::new();
///
/// context.set("one".into(), 1.into(), None);
/// context.set("two".into(), 2.into(), None);
/// context.set("three".into(), 3.into(), None);
///
/// let dust_code = "four = 4 one + two + three + four";
///
/// assert_eq!(
///     interpret_with_context(dust_code, context),
///     Ok(Value::Integer(10))
/// );
/// ```
pub fn interpret_with_context(source: &str, context: Map) -> Result<Value> {
    let mut parser = Parser::new();
    parser.set_language(language())?;

    let mut interpreter = Interpreter::new(context)?;
    let value = interpreter.run(source)?;

    Ok(value)
}

/// A source code interpreter for the Dust language.
pub struct Interpreter {
    parser: Parser,
    context: Map,
    syntax_tree: Option<TSTree>,
    abstract_tree: Option<Root>,
}

impl Interpreter {
    pub fn new(context: Map) -> Result<Self> {
        let mut parser = Parser::new();

        parser.set_language(language())?;

        Ok(Interpreter {
            parser,
            context,
            syntax_tree: None,
            abstract_tree: None,
        })
    }

    pub fn parse_only(&mut self, source: &str) {
        self.syntax_tree = self.parser.parse(source, self.syntax_tree.as_ref());
    }

    pub fn run(&mut self, source: &str) -> Result<Value> {
        self.syntax_tree = self.parser.parse(source, self.syntax_tree.as_ref());
        self.abstract_tree = if let Some(syntax_tree) = &self.syntax_tree {
            Some(Root::from_syntax_node(
                source,
                syntax_tree.root_node(),
                &self.context,
            )?)
        } else {
            return Err(Error::ParserCancelled);
        };

        if let Some(abstract_tree) = &self.abstract_tree {
            abstract_tree.run(source, &self.context)
        } else {
            Ok(Value::Option(None))
        }
    }

    pub fn syntax_tree(&self) -> Result<String> {
        if let Some(syntax_tree) = &self.syntax_tree {
            Ok(syntax_tree.root_node().to_sexp())
        } else {
            Err(Error::ParserCancelled)
        }
    }
}
