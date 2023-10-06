//! The top level of Dust's API with functions to interpret Dust code.
//!
//! You can use this library externally by calling either of the "eval"
//! functions or by constructing your own Evaluator.
use std::fmt::{self, Debug, Formatter};

use tree_sitter::{Parser, Tree as TSTree};

use crate::{abstract_tree::item::Item, language, AbstractTree, Result, Value, VariableMap};

/// Evaluate the given source code.
///
/// Returns a vector of results from evaluating the source code. Each comment
/// and statemtent will have its own result.
///
/// # Examples
///
/// ```rust
/// # use dust::*;
/// assert_eq!(evaluate("1 + 2 + 3"), vec![Ok(Value::Integer(6))]);
/// ```
pub fn evaluate(source: &str) -> Vec<Result<Value>> {
    let mut context = VariableMap::new();

    evaluate_with_context(source, &mut context)
}

/// Evaluate the given source code with the given context.
///
/// # Examples
///
/// ```rust
/// # use dust::*;
/// let mut context = VariableMap::new();
///
/// context.set_value("one".into(), 1.into());
/// context.set_value("two".into(), 2.into());
/// context.set_value("three".into(), 3.into());
///
/// let dust_code = "four = 4 one + two + three + four";
///
/// assert_eq!(
///     evaluate_with_context(dust_code, &mut context),
///     vec![Ok(Value::Empty), Ok(Value::Integer(10))]
/// );
/// ```
pub fn evaluate_with_context(source: &str, context: &mut VariableMap) -> Vec<Result<Value>> {
    let mut parser = Parser::new();
    parser.set_language(language()).unwrap();

    Evaluator::new(parser, context, source).run()
}

/// A collection of statements and comments interpreted from a syntax tree.
///
/// The Evaluator turns a tree sitter concrete syntax tree into a vector of
/// abstract trees called [Item][]s that can be run to execute the source code.
pub struct Evaluator<'context, 'code> {
    _parser: Parser,
    context: &'context mut VariableMap,
    source: &'code str,
    syntax_tree: TSTree,
}

impl Debug for Evaluator<'_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Evaluator context: {}", self.context)
    }
}

impl<'context, 'code> Evaluator<'context, 'code> {
    fn new(mut parser: Parser, context: &'context mut VariableMap, source: &'code str) -> Self {
        let syntax_tree = parser.parse(source, None).unwrap();

        Evaluator {
            _parser: parser,
            context,
            source,
            syntax_tree,
        }
    }

    fn run(self) -> Vec<Result<Value>> {
        let mut cursor = self.syntax_tree.walk();
        let root_node = cursor.node();
        let item_count = root_node.child_count();
        let mut results = Vec::with_capacity(item_count);

        println!("{}", root_node.to_sexp());

        for item_node in root_node.children(&mut cursor) {
            let item_result = Item::from_syntax_node(item_node, self.source);

            match item_result {
                Ok(item) => {
                    let eval_result = item.run(self.context);

                    results.push(eval_result);
                }
                Err(error) => results.push(Err(error)),
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        abstract_tree::{expression::Expression, identifier::Identifier, statement::Statement},
        Function, Table,
    };

    use super::*;

    #[test]
    fn evaluate_empty() {
        assert_eq!(evaluate("x = 9"), vec![Ok(Value::Empty)]);
        assert_eq!(evaluate("x = 1 + 1"), vec![Ok(Value::Empty)]);
    }

    #[test]
    fn evaluate_integer() {
        assert_eq!(evaluate("1"), vec![Ok(Value::Integer(1))]);
        assert_eq!(evaluate("123"), vec![Ok(Value::Integer(123))]);
        assert_eq!(evaluate("-666"), vec![Ok(Value::Integer(-666))]);
    }

    #[test]
    fn evaluate_float() {
        assert_eq!(evaluate("0.1"), vec![Ok(Value::Float(0.1))]);
        assert_eq!(evaluate("12.3"), vec![Ok(Value::Float(12.3))]);
        assert_eq!(evaluate("-6.66"), vec![Ok(Value::Float(-6.66))]);
    }

    #[test]
    fn evaluate_string() {
        assert_eq!(
            evaluate("\"one\""),
            vec![Ok(Value::String("one".to_string()))]
        );
        assert_eq!(
            evaluate("'one'"),
            vec![Ok(Value::String("one".to_string()))]
        );
        assert_eq!(
            evaluate("`one`"),
            vec![Ok(Value::String("one".to_string()))]
        );
        assert_eq!(
            evaluate("`'one'`"),
            vec![Ok(Value::String("'one'".to_string()))]
        );
        assert_eq!(
            evaluate("'`one`'"),
            vec![Ok(Value::String("`one`".to_string()))]
        );
        assert_eq!(
            evaluate("\"'one'\""),
            vec![Ok(Value::String("'one'".to_string()))]
        );
    }

    #[test]
    fn evaluate_list() {
        assert_eq!(
            evaluate("[1, 2, 'foobar']"),
            vec![Ok(Value::List(vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::String("foobar".to_string()),
            ]))]
        );
    }

    #[test]
    fn evaluate_map() {
        let mut map = VariableMap::new();

        map.set_value("x".to_string(), Value::Integer(1)).unwrap();
        map.set_value("foo".to_string(), Value::String("bar".to_string()))
            .unwrap();

        assert_eq!(evaluate("{ x = 1 foo = 'bar' }"), vec![Ok(Value::Map(map))]);
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
                table <messages, numbers> {
                    ['hiya', 42]
                    ['foo', 57]
                    ['bar', 99.99]
                }
                "
            ),
            vec![Ok(Value::Table(table))]
        );
    }

    #[test]
    fn evaluate_if_then() {
        assert_eq!(
            evaluate("if true then 'true'"),
            vec![Ok(Value::String("true".to_string()))]
        );
    }

    #[test]
    fn evaluate_if_then_else() {
        assert_eq!(
            evaluate("if false then 1 else 2"),
            vec![Ok(Value::Integer(2))]
        );
        assert_eq!(
            evaluate("if true then 1.0 else 42.0"),
            vec![Ok(Value::Float(1.0))]
        );
    }

    #[test]
    fn evaluate_if_else_else_if_else() {
        assert_eq!(
            evaluate(
                "
                    if false
                        then 'no'
                    else if 1 + 1 == 3
                        then 'nope'
                    else
                        'ok'
                "
            ),
            vec![Ok(Value::String("ok".to_string()))]
        );
    }

    #[test]
    fn evaluate_if_else_else_if_else_if_else_if_else() {
        assert_eq!(
            evaluate(
                "
                    if false
                        then 'no'
                    else if 1 + 1 == 1
                        then 'nope'
                    else if 9 / 2 == 4 
                        then 'nope'
                    else if 'foo' == 'bar'
                        then 'nope'
                    else 'ok'
                "
            ),
            vec![Ok(Value::String("ok".to_string()))]
        );
    }

    #[test]
    fn evaluate_function() {
        let function = Function::new(
            vec![Identifier::new("message".to_string())],
            vec![Statement::Expression(Expression::Identifier(
                Identifier::new("message".to_string()),
            ))],
        );

        assert_eq!(
            evaluate("function <message> { message }"),
            vec![Ok(Value::Function(function))]
        );
    }

    #[test]
    fn evaluate_function_call() {
        let mut context = VariableMap::new();

        assert_eq!(
            evaluate_with_context(
                "
                foobar = function <message> { message }
                (foobar 'Hiya')
                ",
                &mut context
            ),
            vec![
                Ok(Value::Empty),
                Ok(Value::List(vec![Value::String("Hiya".to_string())]))
            ]
        );
    }

    #[test]
    fn evaluate_tool_call() {
        assert_eq!(evaluate("(output 'Hiya')"), vec![Ok(Value::Empty)]);
    }
}
