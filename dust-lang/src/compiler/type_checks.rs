use crate::{Span, Token, Type};

use super::CompileError;

pub fn check_math_type(
    r#type: &Type,
    operator: Token,
    position: &Span,
) -> Result<(), CompileError> {
    match operator {
        Token::Plus => expect_addable_type(r#type, position),
        Token::Minus => expect_subtractable_type(r#type, position),
        Token::Star => expect_multipliable_type(r#type, position),
        Token::Slash => expect_dividable_type(r#type, position),
        Token::Percent => expect_modulable_type(r#type, position),
        _ => Ok(()),
    }
}

pub fn check_math_types(
    left: &Type,
    left_position: &Span,
    operator: Token,
    right: &Type,
    right_position: &Span,
) -> Result<(), CompileError> {
    match operator {
        Token::Plus => expect_addable_types(left, left_position, right, right_position),
        Token::Minus => expect_subtractable_types(left, left_position, right, right_position),
        Token::Star => expect_multipliable_types(left, left_position, right, right_position),
        Token::Slash => expect_dividable_types(left, left_position, right, right_position),
        Token::Percent => expect_modulable_types(left, left_position, right, right_position),
        _ => Ok(()),
    }
}

pub fn expect_addable_type(argument_type: &Type, position: &Span) -> Result<(), CompileError> {
    if matches!(
        argument_type,
        Type::Byte | Type::Character | Type::Float | Type::Integer | Type::String
    ) {
        Ok(())
    } else {
        Err(CompileError::CannotAddType {
            argument_type: argument_type.clone(),
            position: *position,
        })
    }
}

pub fn expect_addable_types(
    left: &Type,
    left_position: &Span,
    right: &Type,
    right_position: &Span,
) -> Result<(), CompileError> {
    if matches!(
        (left, right),
        (Type::Byte, Type::Byte)
            | (Type::Character, Type::String)
            | (Type::Character, Type::Character)
            | (Type::Float, Type::Float)
            | (Type::Integer, Type::Integer)
            | (Type::String, Type::Character)
            | (Type::String, Type::String),
    ) {
        Ok(())
    } else {
        Err(CompileError::CannotAddArguments {
            left_type: left.clone(),
            left_position: *left_position,
            right_type: right.clone(),
            right_position: *right_position,
        })
    }
}

pub fn expect_dividable_type(argument_type: &Type, position: &Span) -> Result<(), CompileError> {
    if matches!(argument_type, Type::Byte | Type::Float | Type::Integer) {
        Ok(())
    } else {
        Err(CompileError::CannotDivideType {
            argument_type: argument_type.clone(),
            position: *position,
        })
    }
}

pub fn expect_dividable_types(
    left: &Type,
    left_position: &Span,
    right: &Type,
    right_position: &Span,
) -> Result<(), CompileError> {
    if matches!(
        (left, right),
        (Type::Byte, Type::Byte) | (Type::Float, Type::Float) | (Type::Integer, Type::Integer)
    ) {
        Ok(())
    } else {
        Err(CompileError::CannotDivideArguments {
            left_type: left.clone(),
            right_type: right.clone(),
            position: Span(left_position.0, right_position.1),
        })
    }
}

pub fn expect_modulable_type(argument_type: &Type, position: &Span) -> Result<(), CompileError> {
    if matches!(argument_type, Type::Byte | Type::Integer | Type::Float) {
        Ok(())
    } else {
        Err(CompileError::CannotModuloType {
            argument_type: argument_type.clone(),
            position: *position,
        })
    }
}

pub fn expect_modulable_types(
    left: &Type,
    left_position: &Span,
    right: &Type,
    right_position: &Span,
) -> Result<(), CompileError> {
    if matches!(
        (left, right),
        (Type::Byte, Type::Byte) | (Type::Integer, Type::Integer) | (Type::Float, Type::Float)
    ) {
        Ok(())
    } else {
        Err(CompileError::CannotModuloArguments {
            left_type: left.clone(),
            right_type: right.clone(),
            position: Span(left_position.0, right_position.1),
        })
    }
}

pub fn expect_multipliable_type(argument_type: &Type, position: &Span) -> Result<(), CompileError> {
    if matches!(argument_type, Type::Byte | Type::Float | Type::Integer) {
        Ok(())
    } else {
        Err(CompileError::CannotMultiplyType {
            argument_type: argument_type.clone(),
            position: *position,
        })
    }
}

pub fn expect_multipliable_types(
    left: &Type,
    left_position: &Span,
    right: &Type,
    right_position: &Span,
) -> Result<(), CompileError> {
    if matches!(
        (left, right),
        (Type::Byte, Type::Byte) | (Type::Float, Type::Float) | (Type::Integer, Type::Integer)
    ) {
        Ok(())
    } else {
        Err(CompileError::CannotMultiplyArguments {
            left_type: left.clone(),
            right_type: right.clone(),
            position: Span(left_position.0, right_position.1),
        })
    }
}

pub fn expect_subtractable_type(argument_type: &Type, position: &Span) -> Result<(), CompileError> {
    if matches!(argument_type, Type::Byte | Type::Float | Type::Integer) {
        Ok(())
    } else {
        Err(CompileError::CannotSubtractType {
            argument_type: argument_type.clone(),
            position: *position,
        })
    }
}

pub fn expect_subtractable_types(
    left: &Type,
    left_position: &Span,
    right: &Type,
    right_position: &Span,
) -> Result<(), CompileError> {
    if matches!(
        (left, right),
        (Type::Byte, Type::Byte) | (Type::Float, Type::Float) | (Type::Integer, Type::Integer)
    ) {
        Ok(())
    } else {
        Err(CompileError::CannotSubtractArguments {
            left_type: left.clone(),
            right_type: right.clone(),
            position: Span(left_position.0, right_position.1),
        })
    }
}
