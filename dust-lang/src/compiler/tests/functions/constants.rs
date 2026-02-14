use crate::{
    compiler::compile,
    dust_type::DustType,
    instruction::{Address, Instruction, ByteType},
    prototype::Prototype,
    tests::{constant_cases, create_function_case},
};

#[test]
fn boolean() {
    let source = create_function_case(constant_cases::BOOLEAN, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn byte() {
    let source = create_function_case(constant_cases::BYTE, ByteType::BYTE);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Byte,
            instructions: vec![Instruction::r#return(Address::encoded(42))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn character() {
    let source = create_function_case(constant_cases::CHARACTER, ByteType::CHARACTER);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Character,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn float() {
    let source = create_function_case(constant_cases::FLOAT, ByteType::FLOAT);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Float,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn integer() {
    let source = create_function_case(constant_cases::INTEGER, ByteType::INTEGER);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn string() {
    let source = create_function_case(constant_cases::STRING, ByteType::STRING);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::String,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_byte_addition() {
    let source = create_function_case(constant_cases::CONSTANT_BYTE_ADDITION, ByteType::BYTE);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Byte,
            instructions: vec![Instruction::r#return(Address::encoded(42))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_addition() {
    let source = create_function_case(constant_cases::CONSTANT_FLOAT_ADDITION, ByteType::FLOAT);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Float,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_addition() {
    let source = create_function_case(
        constant_cases::CONSTANT_INTEGER_ADDITION,
        ByteType::INTEGER,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_byte_subtraction() {
    let source = create_function_case(constant_cases::CONSTANT_BYTE_SUBTRACTION, ByteType::BYTE);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Byte,
            instructions: vec![Instruction::r#return(Address::encoded(42))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_subtraction() {
    let source = create_function_case(
        constant_cases::CONSTANT_FLOAT_SUBTRACTION,
        ByteType::FLOAT,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Float,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_subtraction() {
    let source = create_function_case(
        constant_cases::CONSTANT_INTEGER_SUBTRACTION,
        ByteType::INTEGER,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_byte_multiplication() {
    let source = create_function_case(
        constant_cases::CONSTANT_BYTE_MULTIPLICATION,
        ByteType::BYTE,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Byte,
            instructions: vec![Instruction::r#return(Address::encoded(42))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_multiplication() {
    let source = create_function_case(
        constant_cases::CONSTANT_FLOAT_MULTIPLICATION,
        ByteType::FLOAT,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Float,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_multiplication() {
    let source = create_function_case(
        constant_cases::CONSTANT_INTEGER_MULTIPLICATION,
        ByteType::INTEGER,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_byte_division() {
    let source = create_function_case(constant_cases::CONSTANT_BYTE_DIVISION, ByteType::BYTE);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Byte,
            instructions: vec![Instruction::r#return(Address::encoded(42))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_division() {
    let source = create_function_case(constant_cases::CONSTANT_FLOAT_DIVISION, ByteType::FLOAT);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Float,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_division() {
    let source = create_function_case(
        constant_cases::CONSTANT_INTEGER_DIVISION,
        ByteType::INTEGER,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_byte_modulo() {
    let source = create_function_case(constant_cases::CONSTANT_BYTE_MODULO, ByteType::BYTE);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Byte,
            instructions: vec![Instruction::r#return(Address::encoded(4))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_modulo() {
    let source = create_function_case(constant_cases::CONSTANT_FLOAT_MODULO, ByteType::FLOAT);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Float,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_modulo() {
    let source = create_function_case(
        constant_cases::CONSTANT_INTEGER_MODULO,
        ByteType::INTEGER,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_negation() {
    let source = create_function_case(
        constant_cases::CONSTANT_INTEGER_NEGATION,
        ByteType::INTEGER,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_negation() {
    let source = create_function_case(constant_cases::CONSTANT_FLOAT_NEGATION, ByteType::FLOAT);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Float,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_string_concatenation() {
    let source = create_function_case(
        constant_cases::CONSTANT_STRING_CONCATENATION,
        ByteType::STRING,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::String,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_character_concatentation() {
    let source = create_function_case(
        constant_cases::CONSTANT_CHARACTER_CONCATENATION,
        ByteType::STRING,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::String,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_string_character_concatenation() {
    let source = create_function_case(
        constant_cases::CONSTANT_STRING_CHARACTER_CONCATENATION,
        ByteType::STRING,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::String,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_character_string_concatenation() {
    let source = create_function_case(
        constant_cases::CONSTANT_CHARACTER_STRING_CONCATENATION,
        ByteType::STRING,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::String,
            instructions: vec![Instruction::r#return(Address::constant(0))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_boolean_and() {
    let source = create_function_case(constant_cases::CONSTANT_BOOLEAN_AND, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(false as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_boolean_or() {
    let source = create_function_case(constant_cases::CONSTANT_BOOLEAN_OR, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_boolean_not() {
    let source = create_function_case(constant_cases::CONSTANT_BOOLEAN_NOT, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(false as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_boolean_greater_than() {
    let source = create_function_case(
        constant_cases::CONSTANT_BOOLEAN_GREATER_THAN,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_boolean_less_than() {
    let source = create_function_case(
        constant_cases::CONSTANT_BOOLEAN_LESS_THAN,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_boolean_greater_than_or_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_BOOLEAN_GREATER_THAN_OR_EQUAL,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_boolean_less_than_or_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_BOOLEAN_LESS_THAN_OR_EQUAL,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_boolean_equal() {
    let source = create_function_case(constant_cases::CONSTANT_BOOLEAN_EQUAL, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_boolean_not_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_BOOLEAN_NOT_EQUAL,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_byte_greater_than() {
    let source = create_function_case(
        constant_cases::CONSTANT_BYTE_GREATER_THAN,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_byte_less_than() {
    let source = create_function_case(
        constant_cases::CONSTANT_BYTE_LESS_THAN,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_byte_greater_than_or_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_BYTE_GREATER_THAN_OR_EQUAL,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_byte_less_than_or_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_BYTE_LESS_THAN_OR_EQUAL,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_byte_equal() {
    let source = create_function_case(constant_cases::CONSTANT_BYTE_EQUAL, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_byte_not_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_BYTE_NOT_EQUAL,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_character_greater_than() {
    let source = create_function_case(
        constant_cases::CONSTANT_CHARACTER_GREATER_THAN,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_character_less_than() {
    let source = create_function_case(
        constant_cases::CONSTANT_CHARACTER_LESS_THAN,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_character_greater_than_or_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_CHARACTER_GREATER_THAN_OR_EQUAL,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_character_less_than_or_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_CHARACTER_LESS_THAN_OR_EQUAL,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_character_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_CHARACTER_EQUAL,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_character_not_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_CHARACTER_NOT_EQUAL,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_greater_than() {
    let source = create_function_case(
        constant_cases::CONSTANT_FLOAT_GREATER_THAN,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_less_than() {
    let source = create_function_case(
        constant_cases::CONSTANT_FLOAT_LESS_THAN,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_greater_than_or_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_FLOAT_GREATER_THAN_OR_EQUAL,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_less_than_or_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_FLOAT_LESS_THAN_OR_EQUAL,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_equal() {
    let source = create_function_case(constant_cases::CONSTANT_FLOAT_EQUAL, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_not_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_FLOAT_NOT_EQUAL,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_greater_than() {
    let source = create_function_case(
        constant_cases::CONSTANT_INTEGER_GREATER_THAN,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_less_than() {
    let source = create_function_case(
        constant_cases::CONSTANT_INTEGER_LESS_THAN,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_greater_than_or_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_INTEGER_GREATER_THAN_OR_EQUAL,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_less_than_or_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_INTEGER_LESS_THAN_OR_EQUAL,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_equal() {
    let source = create_function_case(constant_cases::CONSTANT_INTEGER_EQUAL, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_not_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_INTEGER_NOT_EQUAL,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_string_greater_than() {
    let source = create_function_case(
        constant_cases::CONSTANT_STRING_GREATER_THAN,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(false as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_string_less_than() {
    let source = create_function_case(
        constant_cases::CONSTANT_STRING_LESS_THAN,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(false as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_string_greater_than_or_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_STRING_GREATER_THAN_OR_EQUAL,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_string_less_than_or_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_STRING_LESS_THAN_OR_EQUAL,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_string_equal() {
    let source = create_function_case(constant_cases::CONSTANT_STRING_EQUAL, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_string_not_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_STRING_NOT_EQUAL,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![Instruction::r#return(Address::encoded(true as u16))],
            ..Prototype::dummy()
        }
    );
}
