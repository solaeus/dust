//! The top level of Dust's API with functions to interpret Dust code.
//!
//! You can use this library externally by calling either of the "eval"
//! functions or by constructing your own Evaluator.
use tree_sitter::{Parser, Tree as TSTree};

use crate::{language, AbstractTree, Map, Result, Root, Value};

/// Evaluate the given source code.
///
/// Returns a vector of results from evaluating the source code. Each comment
/// and statemtent will have its own result.
///
/// # Examples
///
/// ```rust
/// # use dust_lang::*;
/// assert_eq!(evaluate("1 + 2 + 3"), Ok(Value::Integer(6)));
/// ```
pub fn evaluate(source: &str) -> Result<Value> {
    let mut context = Map::new();

    evaluate_with_context(source, &mut context)
}

/// Evaluate the given source code with the given context.
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
///     evaluate_with_context(dust_code, &mut context),
///     Ok(Value::Integer(10))
/// );
/// ```
pub fn evaluate_with_context(source: &str, context: &mut Map) -> Result<Value> {
    let mut parser = Parser::new();
    parser.set_language(language()).unwrap();

    Interpreter::parse(parser, context, source)?.run()
}

/// A source code interpreter for the Dust language.
pub struct Interpreter<'c, 's> {
    _parser: Parser,
    context: &'c mut Map,
    source: &'s str,
    syntax_tree: TSTree,
    abstract_tree: Root,
}

impl<'c, 's> Interpreter<'c, 's> {
    pub fn parse(mut parser: Parser, context: &'c mut Map, source: &'s str) -> Result<Self> {
        let syntax_tree = parser.parse(source, None).unwrap();
        let abstract_tree = Root::from_syntax_node(source, syntax_tree.root_node(), context)?;

        Ok(Interpreter {
            _parser: parser,
            context,
            source,
            syntax_tree,
            abstract_tree,
        })
    }

    pub fn run(&mut self) -> Result<Value> {
        self.abstract_tree.run(self.source, self.context)
    }

    pub fn syntax_tree(&self) -> String {
        self.syntax_tree.root_node().to_sexp()
    }
}
