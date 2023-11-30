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
/// {
///     let mut variables = context.variables_mut().unwrap();
///
///     variables.insert("one".into(), 1.into());
///     variables.insert("two".into(), 2.into());
///     variables.insert("three".into(), 3.into());
/// }
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

#[cfg(test)]
mod tests {
    use crate::{List, Table};

    use super::*;

    #[test]
    fn evaluate_empty() {
        assert_eq!(evaluate("x = 9"), Ok(Value::Empty));
        assert_eq!(evaluate("x = 1 + 1"), Ok(Value::Empty));
    }

    #[test]
    fn evaluate_integer() {
        assert_eq!(evaluate("1"), Ok(Value::Integer(1)));
        assert_eq!(evaluate("123"), Ok(Value::Integer(123)));
        assert_eq!(evaluate("-666"), Ok(Value::Integer(-666)));
    }

    #[test]
    fn evaluate_float() {
        assert_eq!(evaluate("0.1"), Ok(Value::Float(0.1)));
        assert_eq!(evaluate("12.3"), Ok(Value::Float(12.3)));
        assert_eq!(evaluate("-6.66"), Ok(Value::Float(-6.66)));
    }

    #[test]
    fn evaluate_string() {
        assert_eq!(evaluate("\"one\""), Ok(Value::String("one".to_string())));
        assert_eq!(evaluate("'one'"), Ok(Value::String("one".to_string())));
        assert_eq!(evaluate("`one`"), Ok(Value::String("one".to_string())));
        assert_eq!(evaluate("`'one'`"), Ok(Value::String("'one'".to_string())));
        assert_eq!(evaluate("'`one`'"), Ok(Value::String("`one`".to_string())));
        assert_eq!(
            evaluate("\"'one'\""),
            Ok(Value::String("'one'".to_string()))
        );
    }

    #[test]
    fn evaluate_list() {
        assert_eq!(
            evaluate("[1, 2, 'foobar']"),
            Ok(Value::List(List::with_items(vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::String("foobar".to_string()),
            ])))
        );
    }

    #[test]
    fn evaluate_map() {
        let map = Map::new();

        {
            let mut variables = map.variables_mut().unwrap();

            variables.insert("x".to_string(), Value::Integer(1));
            variables.insert("foo".to_string(), Value::String("bar".to_string()));
        }

        assert_eq!(evaluate("{ x = 1, foo = 'bar' }"), Ok(Value::Map(map)));
    }

    #[test]
    fn evaluate_table() {
        let mut table = Table::new(vec!["messages".to_string(), "numbers".to_string()]);

        table
            .insert(vec![Value::String("hiya".to_string()), Value::Integer(42)])
            .unwrap();
        table
            .insert(vec![Value::String("foo".to_string()), Value::Integer(57)])
            .unwrap();
        table
            .insert(vec![Value::String("bar".to_string()), Value::Float(99.99)])
            .unwrap();

        assert_eq!(
            evaluate(
                "
                table |messages numbers| [
                    ['hiya', 42]
                    ['foo', 57]
                    ['bar', 99.99]
                ]
                "
            ),
            Ok(Value::Table(table))
        );
    }

    #[test]
    fn evaluate_if() {
        assert_eq!(
            evaluate("if true { 'true' }"),
            Ok(Value::String("true".to_string()))
        );
    }

    #[test]
    fn evaluate_if_else() {
        assert_eq!(evaluate("if false { 1 } else { 2 }"), Ok(Value::Integer(2)));
        assert_eq!(
            evaluate("if true { 1.0 } else { 42.0 }"),
            Ok(Value::Float(1.0))
        );
    }

    #[test]
    fn evaluate_if_else_else_if_else() {
        assert_eq!(
            evaluate(
                "
                    if false {
                        'no'
                    } else if 1 + 1 == 3 {
                        'nope'
                    } else {
                        'ok'
                    }
                "
            ),
            Ok(Value::String("ok".to_string()))
        );
    }

    #[test]
    fn evaluate_if_else_if_else_if_else_if_else() {
        assert_eq!(
            evaluate(
                "
                    if false {
                        'no'
                    } else if 1 + 1 == 1 {
                        'nope'
                    } else if 9 / 2 == 4 {
                        'nope'
                    } else if 'foo' == 'bar' {
                        'nope'
                    } else {
                        'ok'
                    }
                "
            ),
            Ok(Value::String("ok".to_string()))
        );
    }

    #[test]
    fn evaluate_function_call() {
        assert_eq!(
            evaluate(
                "
                foobar <fn str -> str> = |message| { message }
                (foobar 'Hiya')
                ",
            ),
            Ok(Value::String("Hiya".to_string()))
        );
    }

    #[test]
    fn evaluate_built_in_function_call() {
        assert_eq!(evaluate("(output 'Hiya')"), Ok(Value::Empty));
    }
}
