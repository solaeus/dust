use crate::{
    Address, DebugChunk, FunctionType, Instruction, OperandType, Path, Type, Value, compile, run,
};

#[test]
fn divide_bytes() {
    let source = "0x84 / 0x02";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Byte),
            instructions: vec![
                Instruction::divide(
                    Address::register(0),
                    Address::encoded(0x84),
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
fn divide_floats() {
    let source = "84.0 / 2.0";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Float),
            instructions: vec![
                Instruction::divide(
                    Address::register(0),
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::FLOAT
                ),
                Instruction::r#return(true, Address::register(0), OperandType::FLOAT),
            ],
            constants: vec![Value::Float(84.0), Value::Float(2.0)],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Float(42.0)));
}

#[test]
fn divide_integers() {
    let source = "84 / 2";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Integer),
            instructions: vec![
                Instruction::divide(
                    Address::register(0),
                    Address::constant(0),
                    Address::constant(1),
                    OperandType::INTEGER
                ),
                Instruction::r#return(true, Address::register(0), OperandType::INTEGER),
            ],
            constants: vec![Value::Integer(84), Value::Integer(2)],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Integer(42)));
}
