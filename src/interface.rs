use crate::{token, tree, Result, Value, VariableMap};

/// Evaluate the given expression string.
///
/// # Examples
///
/// ```rust
/// # use whale_lib::*;
/// assert_eq!(eval("1 + 2 + 3"), Ok(Value::from(6)));
/// ```
///
/// *See the [crate doc](index.html) for more examples and explanations of the expression format.*
pub fn eval(string: &str) -> Result<Value> {
    let mut context = VariableMap::new();
    let eval = eval_with_context(string, &mut context);

    match eval {
        Ok(output) => {
            if output.is_empty() {
                Ok(Value::Map(context))
            } else {
                Ok(output)
            }
        }
        Err(error) => Err(error),
    }
}

/// Evaluate the given expression string with the given context.
///
/// # Examples
///
/// ```rust
/// # use whale_lib::*;
/// let mut context = VariableMap::new();
/// context.set_value("one".into(), 1.into()).unwrap(); // Do proper error handling here
/// context.set_value("two".into(), 2.into()).unwrap(); // Do proper error handling here
/// context.set_value("three".into(), 3.into()).unwrap(); // Do proper error handling here
/// assert_eq!(eval_with_context("one + two + three", &context), Ok(Value::from(6)));
/// ```
///
/// *See the [crate doc](index.html) for more examples and explanations of the expression format.*
pub fn eval_with_context(string: &str, context: &mut VariableMap) -> Result<Value> {
    let split = string.split_once("::");

    if let Some((left, right)) = split {
        let left_result = tree::tokens_to_operator_tree(token::tokenize(left)?)?
            .eval_with_context_mut(context)?;

        context.set_value("input", left_result)?;

        let right_result = eval_with_context(right, context)?;

        Ok(right_result)
    } else {
        tree::tokens_to_operator_tree(token::tokenize(string)?)?.eval_with_context_mut(context)
    }
}
