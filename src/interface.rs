//! The top level of Dust's API with functions in interpret Dust code.

use crate::{token, tree, Result, Value, VariableMap};

/// Evaluate the given expression string.
///
/// # Examples
///
/// ```rust
/// # use dust_lib::*;
/// assert_eq!(eval("1 + 2 + 3"), Ok(Value::from(6)));
/// ```
///
/// *See the [crate doc](index.html) for more examples and explanations of the expression format.*
pub fn eval(string: &str) -> Result<Value> {
    let mut context = VariableMap::new();
    eval_with_context(string, &mut context)
}

/// Evaluate the given expression string with the given context.
///
/// # Examples
///
/// ```rust
/// # use dust_lib::*;
/// let mut context = VariableMap::new();
/// context.set_value("one".into(), 1.into()).unwrap(); // Do proper error handling here
/// context.set_value("two".into(), 2.into()).unwrap(); // Do proper error handling here
/// context.set_value("three".into(), 3.into()).unwrap(); // Do proper error handling here
/// assert_eq!(eval_with_context("one + two + three", &mut context), Ok(Value::from(6)));
/// ```
pub fn eval_with_context(input: &str, context: &mut VariableMap) -> Result<Value> {
    let without_comments = input
        .lines()
        .map(|line| {
            let split = line.split_once('#');

            if let Some((code, _comment)) = split {
                code
            } else {
                line
            }
        })
        .collect::<String>();

    let split = without_comments.split_once("->");

    if let Some((left, right)) = split {
        let left_result = tree::tokens_to_operator_tree(token::tokenize(left)?)?
            .eval_with_context_mut(context)?;

        context.set_value("input", left_result)?;

        let right_result = eval_with_context(right, context)?;

        Ok(right_result)
    } else {
        tree::tokens_to_operator_tree(token::tokenize(&without_comments)?)?
            .eval_with_context_mut(context)
    }
}
