use crate::{
    compiler::compile_main,
    dust_type::DustType,
    instruction::{Address, Instruction},
    prototype::Prototype,
    tests::constant_cases,
};

#[test]
fn boolean() {
    let source = constant_cases::BOOLEAN;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn byte() {
    let source = constant_cases::BYTE;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Byte,
            instructions: vec![Instruction::r#return(Address::encoded(42))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn character() {
    let source = constant_cases::CHARACTER;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Character,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn float() {
    let source = constant_cases::FLOAT;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Float,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn integer() {
    let source = constant_cases::INTEGER;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn string() {
    let source = constant_cases::STRING;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::String,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_byte_addition() {
    let source = constant_cases::CONSTANT_BYTE_ADDITION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Byte,
            instructions: vec![Instruction::r#return(Address::encoded(42))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_addition() {
    let source = constant_cases::CONSTANT_FLOAT_ADDITION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Float,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_addition() {
    let source = constant_cases::CONSTANT_INTEGER_ADDITION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_byte_subtraction() {
    let source = constant_cases::CONSTANT_BYTE_SUBTRACTION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Byte,
            instructions: vec![Instruction::r#return(Address::encoded(42))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_subtraction() {
    let source = constant_cases::CONSTANT_FLOAT_SUBTRACTION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Float,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_subtraction() {
    let source = constant_cases::CONSTANT_INTEGER_SUBTRACTION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_byte_multiplication() {
    let source = constant_cases::CONSTANT_BYTE_MULTIPLICATION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Byte,
            instructions: vec![Instruction::r#return(Address::encoded(42))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_multiplication() {
    let source = constant_cases::CONSTANT_FLOAT_MULTIPLICATION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Float,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_multiplication() {
    let source = constant_cases::CONSTANT_INTEGER_MULTIPLICATION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_byte_division() {
    let source = constant_cases::CONSTANT_BYTE_DIVISION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Byte,
            instructions: vec![Instruction::r#return(Address::encoded(42))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_division() {
    let source = constant_cases::CONSTANT_FLOAT_DIVISION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Float,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_division() {
    let source = constant_cases::CONSTANT_INTEGER_DIVISION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_byte_modulo() {
    let source = constant_cases::CONSTANT_BYTE_MODULO;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Byte,
            instructions: vec![Instruction::r#return(Address::encoded(4))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_modulo() {
    let source = constant_cases::CONSTANT_FLOAT_MODULO;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Float,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_modulo() {
    let source = constant_cases::CONSTANT_INTEGER_MODULO;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_byte_exponent() {
    let source = constant_cases::CONSTANT_BYTE_EXPONENT;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Byte,
            instructions: vec![Instruction::r#return(Address::encoded(8))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_exponent() {
    let source = constant_cases::CONSTANT_FLOAT_EXPONENT;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Float,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_exponent() {
    let source = constant_cases::CONSTANT_INTEGER_EXPONENT;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_negation() {
    let source = constant_cases::CONSTANT_INTEGER_NEGATION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_negation() {
    let source = constant_cases::CONSTANT_FLOAT_NEGATION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Float,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_string_concatenation() {
    let source = constant_cases::CONSTANT_STRING_CONCATENATION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::String,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_character_concatentation() {
    let source = constant_cases::CONSTANT_CHARACTER_CONCATENATION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::String,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_string_character_concatenation() {
    let source = constant_cases::CONSTANT_STRING_CHARACTER_CONCATENATION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::String,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_character_string_concatenation() {
    let source = constant_cases::CONSTANT_CHARACTER_STRING_CONCATENATION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::String,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_boolean_and() {
    let source = constant_cases::CONSTANT_BOOLEAN_AND;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(false as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_boolean_or() {
    let source = constant_cases::CONSTANT_BOOLEAN_OR;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_boolean_not() {
    let source = constant_cases::CONSTANT_BOOLEAN_NOT;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(false as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_boolean_greater_than() {
    let source = constant_cases::CONSTANT_BOOLEAN_GREATER_THAN;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_boolean_less_than() {
    let source = constant_cases::CONSTANT_BOOLEAN_LESS_THAN;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_boolean_greater_than_or_equal() {
    let source = constant_cases::CONSTANT_BOOLEAN_GREATER_THAN_OR_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_boolean_less_than_or_equal() {
    let source = constant_cases::CONSTANT_BOOLEAN_LESS_THAN_OR_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_boolean_equal() {
    let source = constant_cases::CONSTANT_BOOLEAN_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_boolean_not_equal() {
    let source = constant_cases::CONSTANT_BOOLEAN_NOT_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_byte_less_than_or_equal() {
    let source = constant_cases::CONSTANT_BYTE_LESS_THAN_OR_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_byte_equal() {
    let source = constant_cases::CONSTANT_BYTE_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_byte_not_equal() {
    let source = constant_cases::CONSTANT_BYTE_NOT_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_character_greater_than() {
    let source = constant_cases::CONSTANT_CHARACTER_GREATER_THAN;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_character_less_than() {
    let source = constant_cases::CONSTANT_CHARACTER_LESS_THAN;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_character_greater_than_or_equal() {
    let source = constant_cases::CONSTANT_CHARACTER_GREATER_THAN_OR_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_character_less_than_or_equal() {
    let source = constant_cases::CONSTANT_CHARACTER_LESS_THAN_OR_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_character_equal() {
    let source = constant_cases::CONSTANT_CHARACTER_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_character_not_equal() {
    let source = constant_cases::CONSTANT_CHARACTER_NOT_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_greater_than() {
    let source = constant_cases::CONSTANT_FLOAT_GREATER_THAN;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_less_than() {
    let source = constant_cases::CONSTANT_FLOAT_LESS_THAN;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_greater_than_or_equal() {
    let source = constant_cases::CONSTANT_FLOAT_GREATER_THAN_OR_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_less_than_or_equal() {
    let source = constant_cases::CONSTANT_FLOAT_LESS_THAN_OR_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_equal() {
    let source = constant_cases::CONSTANT_FLOAT_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_not_equal() {
    let source = constant_cases::CONSTANT_FLOAT_NOT_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_greater_than() {
    let source = constant_cases::CONSTANT_INTEGER_GREATER_THAN;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_less_than() {
    let source = constant_cases::CONSTANT_INTEGER_LESS_THAN;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_greater_than_or_equal() {
    let source = constant_cases::CONSTANT_INTEGER_GREATER_THAN_OR_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_less_than_or_equal() {
    let source = constant_cases::CONSTANT_INTEGER_LESS_THAN_OR_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_equal() {
    let source = constant_cases::CONSTANT_INTEGER_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_not_equal() {
    let source = constant_cases::CONSTANT_INTEGER_NOT_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_string_greater_than() {
    let source = constant_cases::CONSTANT_STRING_GREATER_THAN;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(false as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_string_less_than() {
    let source = constant_cases::CONSTANT_STRING_LESS_THAN;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(false as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_string_greater_than_or_equal() {
    let source = constant_cases::CONSTANT_STRING_GREATER_THAN_OR_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_string_less_than_or_equal() {
    let source = constant_cases::CONSTANT_STRING_LESS_THAN_OR_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_string_equal() {
    let source = constant_cases::CONSTANT_STRING_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_string_not_equal() {
    let source = constant_cases::CONSTANT_STRING_NOT_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_byte_greater_than() {
    let source = constant_cases::CONSTANT_BYTE_GREATER_THAN;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_byte_less_than() {
    let source = constant_cases::CONSTANT_BYTE_LESS_THAN;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_byte_greater_than_or_equal() {
    let source = constant_cases::CONSTANT_BYTE_GREATER_THAN_OR_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}
