use crate::{
    Address, Chunk, FunctionType, Instruction, OperandType, Path, Program, Type, Value, compile,
    run,
};

#[test]
fn boolean_true() {
    let source = "true";
    let chunk = compile(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        Program {
            main_chunk: Chunk {
                name: Some(Path::new("main").unwrap()),
                r#type: FunctionType::new([], [], Type::Boolean),
                instructions: vec![Instruction::r#return(
                    true,
                    Address::encoded(true as u16),
                    OperandType::BOOLEAN
                )],
                prototype_index: u16::MAX,
                ..Default::default()
            },
            cell_count: 0,
            prototypes: Vec::new(),
        }
    );
    assert_eq!(return_value, Some(Value::Boolean(true)));
}

#[test]
fn boolean_false() {
    let source = "false";
    let chunk = compile(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        Program {
            main_chunk: Chunk {
                name: Some(Path::new("main").unwrap()),
                r#type: FunctionType::new([], [], Type::Boolean),
                instructions: vec![Instruction::r#return(
                    true,
                    Address::encoded(false as u16),
                    OperandType::BOOLEAN
                )],
                prototype_index: u16::MAX,
                ..Default::default()
            },
            cell_count: 0,
            prototypes: Vec::new(),
        }
    );
    assert_eq!(return_value, Some(Value::Boolean(false)));
}
