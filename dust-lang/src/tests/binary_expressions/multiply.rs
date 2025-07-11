use crate::{
    Address, DebugChunk, FunctionType, Instruction, OperandType, Path, Type, Value, compile, run,
};

#[test]
fn multiply_bytes() {
    let source = "0x21 * 0x02";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Byte),
            instructions: vec![
                Instruction::multiply(
                    Address::register(0),
                    Address::encoded(0x21),
                    Address::encoded(0x02),
                    OperandType::BYTE
                ),
                Instruction::r#return(true, Address::register(0), OperandType::BYTE),
            ],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Byte(0x42)));
}

#[test]
fn multiply_floats() {
    let source = "21.0 * 2.0";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Float),
            instructions: vec![
                Instruction::multiply(
                    Address::register(0),
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::FLOAT
                ),
                Instruction::r#return(true, Address::register(0), OperandType::FLOAT),
            ],
            constants: vec![Value::Float(21.0), Value::Float(2.0)],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Float(42.0)));
}

#[test]
fn multiply_integers() {
    let source = "21 * 2";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Integer),
            instructions: vec![
                Instruction::multiply(
                    Address::register(0),
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::INTEGER
                ),
                Instruction::r#return(true, Address::register(0), OperandType::INTEGER),
            ],
            constants: vec![Value::Integer(21), Value::Integer(2)],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Integer(42)));
}
