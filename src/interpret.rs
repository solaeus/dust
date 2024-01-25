//! Tools to run and/or format dust source code.
//!
//! You can use this library externally by calling either of the "interpret"
//! functions or by constructing your own Interpreter.
use tree_sitter::{Parser, Tree as TSTree, TreeCursor};

use crate::{language, AbstractTree, Error, Format, Map, Result, Root, SyntaxNode, Value};

/// Interpret the given source code. Returns the value of last statement or the
/// first error encountered.
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
/// A context is a [Map] instance, which is dust's
/// [BTreeMap][std::collections::btree_map::BTreeMap] that is used internally
/// for the `<map>` type. Any value can be set
///
/// # Examples
///
/// ```rust
/// # use dust_lang::*;
/// let context = Map::new();
///
/// context.set("one".into(), 1.into());
/// context.set("two".into(), 2.into());
/// context.set("three".into(), 3.into());
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

        let syntax_tree = self.parser.parse(source, None);

        if let Some(tree) = &syntax_tree {
            let root = tree.root_node();
            let mut cursor = root.walk();

            check_for_error(root, source, &mut cursor)?;
        }

        self.syntax_tree = syntax_tree;

        Ok(())
    }

    pub fn run(&mut self, source: &str) -> Result<Value> {
        self.parse(source)?;

        self.abstract_tree = if let Some(syntax_tree) = &self.syntax_tree {
            Some(Root::from_syntax(
                syntax_tree.root_node(),
                source,
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
            let mut formatted_source = String::new();

            root_node.format(&mut formatted_source, 0);

            formatted_source
        } else {
            String::with_capacity(0)
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Interpreter::new(Map::new())
    }
}
