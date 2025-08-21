use crate::{
    CompileError, Value,
    syntax_tree::{SyntaxKind, SyntaxNode},
};

pub fn fold_constants(
    _left_node: &SyntaxNode,
    left_constant: &Value,
    right_node: &SyntaxNode,
    right_constant: &Value,
    parent_node: &SyntaxNode,
) -> Result<Option<Value>, CompileError> {
    let folded_constant = match (left_constant, right_constant, parent_node.kind) {
        (Value::Float(left), Value::Float(right), SyntaxKind::AdditionExpression) => {
            Value::Float(left + right)
        }
        (Value::Float(left), Value::Float(right), SyntaxKind::SubtractionExpression) => {
            Value::Float(left - right)
        }
        (Value::Float(left), Value::Float(right), SyntaxKind::MultiplicationExpression) => {
            Value::Float(left * right)
        }
        (Value::Float(left), Value::Float(right), SyntaxKind::DivisionExpression) => {
            if *right == 0.0 {
                return Err(CompileError::DivisionByZero {
                    node_kind: parent_node.kind,
                    position: right_node.span,
                });
            }

            Value::Float(left / right)
        }
        (Value::Float(left), Value::Float(right), SyntaxKind::ModuloExpression) => {
            if *right == 0.0 {
                return Err(CompileError::DivisionByZero {
                    node_kind: parent_node.kind,
                    position: right_node.span,
                });
            }

            Value::Float(left % right)
        }
        (Value::Integer(left), Value::Integer(right), SyntaxKind::AdditionExpression) => {
            Value::Integer(left.saturating_add(*right))
        }
        (Value::Integer(left), Value::Integer(right), SyntaxKind::SubtractionExpression) => {
            Value::Integer(left.saturating_sub(*right))
        }
        (Value::Integer(left), Value::Integer(right), SyntaxKind::MultiplicationExpression) => {
            Value::Integer(left.saturating_mul(*right))
        }
        (Value::Integer(left), Value::Integer(right), SyntaxKind::DivisionExpression) => {
            if *right == 0 {
                return Err(CompileError::DivisionByZero {
                    node_kind: parent_node.kind,
                    position: right_node.span,
                });
            }
            Value::Integer(left.saturating_div(*right))
        }
        (Value::Integer(left), Value::Integer(right), SyntaxKind::ModuloExpression) => {
            if *right == 0 {
                return Err(CompileError::DivisionByZero {
                    node_kind: parent_node.kind,
                    position: right_node.span,
                });
            }
            Value::Integer(left % *right)
        }
        (Value::String(left), Value::String(right), SyntaxKind::AdditionExpression) => {
            let mut concatenated = String::with_capacity(left.len() + right.len());

            concatenated.push_str(left);
            concatenated.push_str(right);

            Value::String(concatenated)
        }
        _ => return Ok(None),
    };

    Ok(Some(folded_constant))
}
