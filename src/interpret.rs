//! The top level of Dust's API with functions to interpret Dust code.
//!
//! You can use this library externally by calling either of the "eval"
//! functions or by constructing your own Evaluator.
use tree_sitter::{Node, Parser, Tree as TSTree, TreeCursor};

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
    let mut interpreter = Interpreter::new(context);
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
    pub fn new(context: Map) -> Self {
        let mut parser = Parser::new();

        parser
            .set_language(language())
            .expect("Language version is incompatible with tree sitter version.");

        Interpreter {
            parser,
            context,
            syntax_tree: None,
            abstract_tree: None,
        }
    }

    pub fn parse(&mut self, source: &str) -> Result<()> {
        fn check_for_error(source: &str, node: Node, cursor: &mut TreeCursor) -> Result<()> {
            if node.is_error() {
                Err(Error::Syntax {
                    source: source[node.byte_range()].to_string(),
                    location: node.start_position(),
                })
            } else {
                for child in node.children(&mut cursor.clone()) {
                    check_for_error(source, child, cursor)?;
                }

                Ok(())
            }
        }

        let syntax_tree = self.parser.parse(source, None);

        if let Some(tree) = &syntax_tree {
            let root = tree.root_node();
            let mut cursor = root.walk();

            check_for_error(source, root, &mut cursor)?;
        }

        self.syntax_tree = syntax_tree;

        Ok(())
    }

    pub fn run(&mut self, source: &str) -> Result<Value> {
        self.parse(source)?;

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
            abstract_tree.check_type(source, &self.context)?;
            abstract_tree.run(source, &self.context)
        } else {
            Ok(Value::none())
        }
    }

    pub fn syntax_tree(&self) -> Result<String> {
        if let Some(syntax_tree) = &self.syntax_tree {
            Ok(syntax_tree.root_node().to_sexp())
        } else {
            Err(Error::ParserCancelled)
        }
    }

    pub fn format(&self) -> String {
        if let Some(root_node) = &self.abstract_tree {
            root_node.to_string()
        } else {
            "".to_string()
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Interpreter::new(Map::new())
    }
}
