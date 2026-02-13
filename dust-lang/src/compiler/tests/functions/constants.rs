use crate::{
    compiler::{Symbol, compile},
    dust_type::{DustType, FunctionDustType},
    instruction::{Address, Instruction, OperandType},
    prototype::{Prototype, PrototypeId},
    tests::{constant_cases, create_function_case},
};

#[test]
fn boolean() {
    let source = create_function_case(constant_cases::BOOLEAN, OperandType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn byte() {
    let source = create_function_case(constant_cases::BYTE, OperandType::BYTE);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Byte),
            instructions: vec![Instruction::r#return(
                Address::encoded(42),
                OperandType::BYTE
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn character() {
    let source = create_function_case(constant_cases::CHARACTER, OperandType::CHARACTER);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Character),
            instructions: vec![Instruction::r#return(
                Address::constant(0),
                OperandType::CHARACTER
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn float() {
    let source = create_function_case(constant_cases::FLOAT, OperandType::FLOAT);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Float),
            instructions: vec![Instruction::r#return(
                Address::constant(0),
                OperandType::FLOAT
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn integer() {
    let source = create_function_case(constant_cases::INTEGER, OperandType::INTEGER);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Integer),
            instructions: vec![Instruction::r#return(
                Address::constant(0),
                OperandType::INTEGER
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn string() {
    let source = create_function_case(constant_cases::STRING, OperandType::STRING);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::String),
            instructions: vec![Instruction::r#return(
                Address::constant(0),
                OperandType::STRING
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_byte_addition() {
    let source = create_function_case(constant_cases::CONSTANT_BYTE_ADDITION, OperandType::BYTE);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Byte),
            instructions: vec![Instruction::r#return(
                Address::encoded(42),
                OperandType::BYTE
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_addition() {
    let source = create_function_case(constant_cases::CONSTANT_FLOAT_ADDITION, OperandType::FLOAT);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Float),
            instructions: vec![Instruction::r#return(
                Address::constant(0),
                OperandType::FLOAT
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_addition() {
    let source = create_function_case(
        constant_cases::CONSTANT_INTEGER_ADDITION,
        OperandType::INTEGER,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Integer),
            instructions: vec![Instruction::r#return(
                Address::constant(0),
                OperandType::INTEGER
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_byte_subtraction() {
    let source = create_function_case(constant_cases::CONSTANT_BYTE_SUBTRACTION, OperandType::BYTE);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Byte),
            instructions: vec![Instruction::r#return(
                Address::encoded(42),
                OperandType::BYTE
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_subtraction() {
    let source = create_function_case(
        constant_cases::CONSTANT_FLOAT_SUBTRACTION,
        OperandType::FLOAT,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Float),
            instructions: vec![Instruction::r#return(
                Address::constant(0),
                OperandType::FLOAT
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_subtraction() {
    let source = create_function_case(
        constant_cases::CONSTANT_INTEGER_SUBTRACTION,
        OperandType::INTEGER,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Integer),
            instructions: vec![Instruction::r#return(
                Address::constant(0),
                OperandType::INTEGER
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_byte_multiplication() {
    let source = create_function_case(
        constant_cases::CONSTANT_BYTE_MULTIPLICATION,
        OperandType::BYTE,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Byte),
            instructions: vec![Instruction::r#return(
                Address::encoded(42),
                OperandType::BYTE
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_multiplication() {
    let source = create_function_case(
        constant_cases::CONSTANT_FLOAT_MULTIPLICATION,
        OperandType::FLOAT,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Float),
            instructions: vec![Instruction::r#return(
                Address::constant(0),
                OperandType::FLOAT
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_multiplication() {
    let source = create_function_case(
        constant_cases::CONSTANT_INTEGER_MULTIPLICATION,
        OperandType::INTEGER,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Integer),
            instructions: vec![Instruction::r#return(
                Address::constant(0),
                OperandType::INTEGER
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_byte_division() {
    let source = create_function_case(constant_cases::CONSTANT_BYTE_DIVISION, OperandType::BYTE);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Byte),
            instructions: vec![Instruction::r#return(
                Address::encoded(42),
                OperandType::BYTE
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_division() {
    let source = create_function_case(constant_cases::CONSTANT_FLOAT_DIVISION, OperandType::FLOAT);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Float),
            instructions: vec![Instruction::r#return(
                Address::constant(0),
                OperandType::FLOAT
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_division() {
    let source = create_function_case(
        constant_cases::CONSTANT_INTEGER_DIVISION,
        OperandType::INTEGER,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Integer),
            instructions: vec![Instruction::r#return(
                Address::constant(0),
                OperandType::INTEGER
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_byte_modulo() {
    let source = create_function_case(constant_cases::CONSTANT_BYTE_MODULO, OperandType::BYTE);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Byte),
            instructions: vec![Instruction::r#return(
                Address::encoded(4),
                OperandType::BYTE
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_modulo() {
    let source = create_function_case(constant_cases::CONSTANT_FLOAT_MODULO, OperandType::FLOAT);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Float),
            instructions: vec![Instruction::r#return(
                Address::constant(0),
                OperandType::FLOAT
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_modulo() {
    let source = create_function_case(
        constant_cases::CONSTANT_INTEGER_MODULO,
        OperandType::INTEGER,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Integer),
            instructions: vec![Instruction::r#return(
                Address::constant(0),
                OperandType::INTEGER
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_negation() {
    let source = create_function_case(
        constant_cases::CONSTANT_INTEGER_NEGATION,
        OperandType::INTEGER,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Integer),
            instructions: vec![Instruction::r#return(
                Address::constant(0),
                OperandType::INTEGER
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_negation() {
    let source = create_function_case(constant_cases::CONSTANT_FLOAT_NEGATION, OperandType::FLOAT);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Float),
            instructions: vec![Instruction::r#return(
                Address::constant(0),
                OperandType::FLOAT
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_string_concatenation() {
    let source = create_function_case(
        constant_cases::CONSTANT_STRING_CONCATENATION,
        OperandType::STRING,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::String),
            instructions: vec![Instruction::r#return(
                Address::constant(0),
                OperandType::STRING
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_character_concatentation() {
    let source = create_function_case(
        constant_cases::CONSTANT_CHARACTER_CONCATENATION,
        OperandType::STRING,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::String),
            instructions: vec![Instruction::r#return(
                Address::constant(0),
                OperandType::STRING
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_string_character_concatenation() {
    let source = create_function_case(
        constant_cases::CONSTANT_STRING_CHARACTER_CONCATENATION,
        OperandType::STRING,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::String),
            instructions: vec![Instruction::r#return(
                Address::constant(0),
                OperandType::STRING
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_character_string_concatenation() {
    let source = create_function_case(
        constant_cases::CONSTANT_CHARACTER_STRING_CONCATENATION,
        OperandType::STRING,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::String),
            instructions: vec![Instruction::r#return(
                Address::constant(0),
                OperandType::STRING
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_boolean_and() {
    let source = create_function_case(constant_cases::CONSTANT_BOOLEAN_AND, OperandType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(false as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_boolean_or() {
    let source = create_function_case(constant_cases::CONSTANT_BOOLEAN_OR, OperandType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_boolean_not() {
    let source = create_function_case(constant_cases::CONSTANT_BOOLEAN_NOT, OperandType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(false as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_boolean_greater_than() {
    let source = create_function_case(
        constant_cases::CONSTANT_BOOLEAN_GREATER_THAN,
        OperandType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_boolean_less_than() {
    let source = create_function_case(
        constant_cases::CONSTANT_BOOLEAN_LESS_THAN,
        OperandType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_boolean_greater_than_or_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_BOOLEAN_GREATER_THAN_OR_EQUAL,
        OperandType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_boolean_less_than_or_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_BOOLEAN_LESS_THAN_OR_EQUAL,
        OperandType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_boolean_equal() {
    let source = create_function_case(constant_cases::CONSTANT_BOOLEAN_EQUAL, OperandType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_boolean_not_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_BOOLEAN_NOT_EQUAL,
        OperandType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_byte_greater_than() {
    let source = create_function_case(
        constant_cases::CONSTANT_BYTE_GREATER_THAN,
        OperandType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_byte_less_than() {
    let source = create_function_case(
        constant_cases::CONSTANT_BYTE_LESS_THAN,
        OperandType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_byte_greater_than_or_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_BYTE_GREATER_THAN_OR_EQUAL,
        OperandType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_byte_less_than_or_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_BYTE_LESS_THAN_OR_EQUAL,
        OperandType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_byte_equal() {
    let source = create_function_case(constant_cases::CONSTANT_BYTE_EQUAL, OperandType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_byte_not_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_BYTE_NOT_EQUAL,
        OperandType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_character_greater_than() {
    let source = create_function_case(
        constant_cases::CONSTANT_CHARACTER_GREATER_THAN,
        OperandType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_character_less_than() {
    let source = create_function_case(
        constant_cases::CONSTANT_CHARACTER_LESS_THAN,
        OperandType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_character_greater_than_or_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_CHARACTER_GREATER_THAN_OR_EQUAL,
        OperandType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_character_less_than_or_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_CHARACTER_LESS_THAN_OR_EQUAL,
        OperandType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_character_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_CHARACTER_EQUAL,
        OperandType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_character_not_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_CHARACTER_NOT_EQUAL,
        OperandType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_greater_than() {
    let source = create_function_case(
        constant_cases::CONSTANT_FLOAT_GREATER_THAN,
        OperandType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_less_than() {
    let source = create_function_case(
        constant_cases::CONSTANT_FLOAT_LESS_THAN,
        OperandType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_greater_than_or_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_FLOAT_GREATER_THAN_OR_EQUAL,
        OperandType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_less_than_or_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_FLOAT_LESS_THAN_OR_EQUAL,
        OperandType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_equal() {
    let source = create_function_case(constant_cases::CONSTANT_FLOAT_EQUAL, OperandType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_float_not_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_FLOAT_NOT_EQUAL,
        OperandType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_greater_than() {
    let source = create_function_case(
        constant_cases::CONSTANT_INTEGER_GREATER_THAN,
        OperandType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_less_than() {
    let source = create_function_case(
        constant_cases::CONSTANT_INTEGER_LESS_THAN,
        OperandType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_greater_than_or_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_INTEGER_GREATER_THAN_OR_EQUAL,
        OperandType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_less_than_or_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_INTEGER_LESS_THAN_OR_EQUAL,
        OperandType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_equal() {
    let source = create_function_case(constant_cases::CONSTANT_INTEGER_EQUAL, OperandType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_integer_not_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_INTEGER_NOT_EQUAL,
        OperandType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_string_greater_than() {
    let source = create_function_case(
        constant_cases::CONSTANT_STRING_GREATER_THAN,
        OperandType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(false as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_string_less_than() {
    let source = create_function_case(
        constant_cases::CONSTANT_STRING_LESS_THAN,
        OperandType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(false as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_string_greater_than_or_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_STRING_GREATER_THAN_OR_EQUAL,
        OperandType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_string_less_than_or_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_STRING_LESS_THAN_OR_EQUAL,
        OperandType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_string_equal() {
    let source = create_function_case(constant_cases::CONSTANT_STRING_EQUAL, OperandType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}

#[test]
fn constant_string_not_equal() {
    let source = create_function_case(
        constant_cases::CONSTANT_STRING_NOT_EQUAL,
        OperandType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: PrototypeId(1),
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![Instruction::r#return(
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Prototype::dummy()
        }
    );
}
