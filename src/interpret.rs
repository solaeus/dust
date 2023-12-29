//! The top level of Dust's API with functions to interpret Dust code.
//!
//! You can use this library externally by calling either of the "eval"
//! functions or by constructing your own Evaluator.
use tree_sitter::{Parser, Tree as TSTree};

use crate::{language, AbstractTree, Map, Result, Root, Value};

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
    let mut context = Map::new();

    interpret_with_context(source, &mut context)
}

/// Interpret the given source code with the given context.
///
/// # Examples
///
/// ```rust
/// # use dust_lang::*;
/// let mut context = Map::new();
///
/// context.set("one".into(), 1.into(), None);
/// context.set("two".into(), 2.into(), None);
/// context.set("three".into(), 3.into(), None);
///
/// let dust_code = "four = 4 one + two + three + four";
///
/// assert_eq!(
///     interpret_with_context(dust_code, &mut context),
///     Ok(Value::Integer(10))
/// );
/// ```
pub fn interpret_with_context(source: &str, context: &mut Map) -> Result<Value> {
    let mut parser = Parser::new();
    parser.set_language(language())?;

    let mut interpreter = Interpreter::new(context, source)?;
    let value = interpreter.run()?;

    Ok(value)
}

/// A source code interpreter for the Dust language.
pub struct Interpreter<'c, 's> {
    _parser: Parser,
    context: &'c mut Map,
    source: &'s str,
    syntax_tree: Option<TSTree>,
    abstract_tree: Option<Root>,
}

impl<'c, 's> Interpreter<'c, 's> {
    pub fn new(context: &'c mut Map, source: &'s str) -> Result<Self> {
        let mut parser = Parser::new();

        parser.set_language(language())?;

        Ok(Interpreter {
            _parser: parser,
            context,
            source,
            syntax_tree: None,
            abstract_tree: None,
        })
    }

    pub fn set_source(&mut self, source: &'s str) {
        self.source = source;
    }

    pub fn run(&mut self) -> Result<Value> {
        self.syntax_tree = self._parser.parse(self.source, self.syntax_tree.as_ref());
        self.abstract_tree = if let Some(syntax_tree) = &self.syntax_tree {
            Some(Root::from_syntax_node(
                self.source,
                syntax_tree.root_node(),
                &self.context,
            )?)
        } else {
            return Err(crate::Error::ParserCancelled);
        };

        if let Some(abstract_tree) = &self.abstract_tree {
            abstract_tree.run(self.source, &self.context)
        } else {
            Ok(Value::Option(None))
        }
    }

    pub fn syntax_tree(&self) -> Option<String> {
        if let Some(syntax_tree) = &self.syntax_tree {
            Some(syntax_tree.root_node().to_sexp())
        } else {
            None
        }
    }
}
