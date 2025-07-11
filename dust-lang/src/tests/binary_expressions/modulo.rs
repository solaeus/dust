use crate::{
    Address, DebugChunk, FunctionType, Instruction, OperandType, Path, Type, Value, compile, run,
};

#[test]
fn modulo_bytes() {
    let source = "0x45 % 0x12";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Byte),
            instructions: vec![
                Instruction::modulo(
                    Address::register(0),
                    Address::encoded(0x45),
                    Address::encoded(0x12),
                    OperandType::BYTE
                ),
                Instruction::r#return(true, Address::register(0), OperandType::BYTE),
            ],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Byte(0x45 % 0x12)));
}

#[test]
fn modulo_floats() {
    let source = "45.5 % 3.5";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Float),
            instructions: vec![
                Instruction::modulo(
                    Address::register(0),
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::FLOAT
                ),
                Instruction::r#return(true, Address::register(0), OperandType::FLOAT),
            ],
            constants: vec![Value::Float(45.5), Value::Float(3.5)],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Float(45.5 % 3.5)));
}

#[test]
fn modulo_integers() {
    let source = "45 % 3";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Integer),
            instructions: vec![
                Instruction::modulo(
                    Address::register(0),
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::INTEGER
                ),
                Instruction::r#return(true, Address::register(0), OperandType::INTEGER),
            ],
            constants: vec![Value::Integer(45), Value::Integer(3)],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Integer(45 % 3)));
}
