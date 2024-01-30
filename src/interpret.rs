//! Tools to interpret dust source code.
//!
//! This module has three tools to run Dust code.
//!
//! - [interpret] is the simples way to run Dust code inside of an application or library
//! - [interpret_with_context] allows you to set variables on the execution context
//! - [Interpreter] is an advanced tool that can parse, verify, run and format Dust code
//!
//! # Examples
//!
//! Run some Dust and get the result.
//!
//! ```rust
//! # use dust_lang::*;
//! assert_eq!(
//!     interpret("1 + 2 + 3"),
//!     Ok(Value::Integer(6))
//! );
//! ```
//!
//! Create a custom context with variables you can use in your code.
//!
//! ```rust
//! # use dust_lang::*;
//! let context = Map::new();
//!
//! context.set("one".into(), 1.into());
//! context.set("two".into(), 2.into());
//! context.set("three".into(), 3.into());
//!
//! let dust_code = "four = 4; one + two + three + four";
//!
//! assert_eq!(
//!     interpret_with_context(dust_code, context),
//!     Ok(Value::Integer(10))
//! );
//! ```
use tree_sitter::{Node as SyntaxNode, Parser, Tree as SyntaxTree, TreeCursor};

use crate::{language, AbstractTree, Error, Format, Map, Result, Root, Value};

/// Interpret the given source code. Returns the value of last statement or the
/// first error encountered.
///
/// See the [module-level docs][self] for more info.
pub fn interpret(source: &str) -> Result<Value> {
    interpret_with_context(source, Map::new())
}

/// Interpret the given source code with the given context.
///
/// A context is a [Map] instance, which is dust's
/// [BTreeMap][std::collections::btree_map::BTreeMap] that is used internally
/// for the `<map>` type. Any value can be set, including functions and nested
/// maps.
///
/// See the [module-level docs][self] for more info.
pub fn interpret_with_context(source: &str, context: Map) -> Result<Value> {
    let mut interpreter = Interpreter::new(context);
    let value = interpreter.run(source)?;

    Ok(value)
}

/// A source code interpreter for the Dust language.
///
/// The interpreter's most important functions are used to parse dust source code, verify it is safe
/// and run it and they are written in a way that forces them to be used safely. Each step in this
/// process contains the prior steps, meaning that the same code is always used to create the syntax /// tree, abstract tree and final evaluation. This avoids a critical logic error.
pub struct Interpreter {
    parser: Parser,
    context: Map,
}

impl Interpreter {
    /// Creates a new interpreter with the given variable context.
    pub fn new(context: Map) -> Self {
        let mut parser = Parser::new();

        parser
            .set_language(language())
            .expect("Language version is incompatible with tree sitter version.");

        parser.set_logger(Some(Box::new(|log_type, message| {
            log::info!("{}", message)
        })));

        Interpreter { parser, context }
    }

    /// Generates a syntax tree from the source. Returns an error if the the parser is cancelled for
    /// taking too long. The syntax tree may contain error nodes, which represent syntax errors.
    ///
    /// Tree sitter is designed to be run on every keystroke, so this is generally a lightweight
    /// function to call.
    pub fn parse(&mut self, source: &str) -> Result<SyntaxTree> {
        if let Some(tree) = self.parser.parse(source, None) {
            Ok(tree)
        } else {
            Err(Error::ParserCancelled)
        }
    }

    /// Checks the source for errors and generates an abstract tree.
    ///
    /// The order in which this function works is:
    ///
    /// - parse the source into a syntax tree
    /// - check the syntax tree for errors
    /// - generate an abstract tree from the source and syntax tree
    /// - check the abstract tree for type errors
    pub fn verify(&mut self, source: &str) -> Result<Root> {
        fn check_for_error(node: SyntaxNode, source: &str, cursor: &mut TreeCursor) -> Result<()> {
            if node.is_error() {
                Err(Error::Syntax {
                    source: source[node.byte_range()].to_string(),
                    location: node.start_position(),
                })
            } else {
                for child in node.children(&mut cursor.clone()) {
                    check_for_error(child, source, cursor)?;
                }

                Ok(())
            }
        }

        let syntax_tree = self.parse(source)?;
        let root = syntax_tree.root_node();
        let mut cursor = syntax_tree.root_node().walk();

        check_for_error(root, source, &mut cursor)?;

        let abstract_tree = Root::from_syntax(syntax_tree.root_node(), source, &self.context)?;

        abstract_tree.check_type(source, &self.context)?;

        Ok(abstract_tree)
    }

    /// Runs the source, returning the final statement's value or first error.
    ///
    /// This function [parses][Self::parse], [verifies][Self::verify] and [runs][Root::run] using
    /// the same source code.
    pub fn run(&mut self, source: &str) -> Result<Value> {
        self.verify(source)?.run(source, &self.context)
    }

    /// Return an s-expression displaying a syntax tree of the source, or the ParserCancelled error
    /// if the parser takes too long.
    pub fn syntax_tree(&mut self, source: &str) -> Result<String> {
        Ok(self.parse(source)?.root_node().to_sexp())
    }

    /// Return formatted Dust code generated from the current abstract tree, or None if no source
    /// code has been run successfully.
    ///
    /// You should call [verify][Interpreter::verify] before calling this function. You can only
    /// create formatted source from a valid abstract tree.
    pub fn format(&mut self, source: &str) -> Result<String> {
        let mut formatted_output = String::new();

        self.verify(source)?.format(&mut formatted_output, 0);

        Ok(formatted_output)
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Interpreter::new(Map::new())
    }
}
