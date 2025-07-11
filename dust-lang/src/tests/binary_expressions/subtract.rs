use crate::{
    Address, DebugChunk, FunctionType, Instruction, OperandType, Path, Type, Value, compile, run,
};

#[test]
fn subtract_bytes() {
    let source = "0x44 - 0x02";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Byte),
            instructions: vec![
                Instruction::subtract(
                    Address::register(0),
                    Address::encoded(0x44),
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
fn subtract_floats() {
    let source = "44.0 - 2.0";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Float),
            instructions: vec![
                Instruction::subtract(
                    Address::register(0),
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::FLOAT
                ),
                Instruction::r#return(true, Address::register(0), OperandType::FLOAT),
            ],
            constants: vec![Value::Float(44.0), Value::Float(2.0)],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Float(42.0)));
}

#[test]
fn subtract_integers() {
    let source = "44 - 2";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Integer),
            instructions: vec![
                Instruction::subtract(
                    Address::register(0),
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::INTEGER
                ),
                Instruction::r#return(true, Address::register(0), OperandType::INTEGER),
            ],
            constants: vec![Value::Integer(44), Value::Integer(2)],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Integer(42)));
}
