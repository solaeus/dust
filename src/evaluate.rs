//! The top level of Dust's API with functions to interpret Dust code.
//!
//! You can use this library externally by calling either of the "eval"
//! functions or by constructing your own Evaluator.
use std::cell::RefCell;

use tree_sitter::{Parser, Tree as TSTree};

use crate::{language, AbstractTree, Error, Map, Result, Root, Value};

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
    Interpreter::parse(context, &RefCell::new(source.to_string()))?.run()
}

/// A source code interpreter for the Dust language.
pub struct Interpreter<'c, 's> {
    parser: Parser,
    context: &'c mut Map,
    source: &'s RefCell<String>,
    syntax_tree: Option<TSTree>,
    abstract_tree: Option<Root>,
}

impl<'c, 's> Interpreter<'c, 's> {
    pub fn parse(context: &'c mut Map, source: &'s RefCell<String>) -> Result<Self> {
        let mut parser = Parser::new();

        parser.set_language(language()).unwrap();

        let syntax_tree = parser.parse(source.borrow().as_str(), None);
        let abstract_tree = if let Some(syntax_tree) = &syntax_tree {
            Some(Root::from_syntax_node(
                source.borrow().as_str(),
                syntax_tree.root_node(),
                context,
            )?)
        } else {
            panic!()
        };

        Ok(Interpreter {
            parser,
            context,
            source,
            syntax_tree,
            abstract_tree,
        })
    }

    pub fn update(&mut self) -> Result<()> {
        let source = self.source.borrow();
        let syntax_tree = self.parser.parse(source.as_str(), None);
        let abstract_tree = if let Some(syntax_tree) = &syntax_tree {
            Some(Root::from_syntax_node(
                source.as_str(),
                syntax_tree.root_node(),
                self.context,
            )?)
        } else {
            None
        };

        self.syntax_tree = syntax_tree;
        self.abstract_tree = abstract_tree;

        Ok(())
    }

    pub fn run(&mut self) -> Result<Value> {
        if let Some(abstract_tree) = &self.abstract_tree {
            abstract_tree.run(self.source.borrow().as_ref(), self.context)
        } else {
            Err(Error::NoUserInput)
        }
    }

    pub fn syntax_tree(&self) -> String {
        if let Some(syntax_tree) = &self.syntax_tree {
            syntax_tree.root_node().to_sexp()
        } else {
            "".to_string()
        }
    }
}
