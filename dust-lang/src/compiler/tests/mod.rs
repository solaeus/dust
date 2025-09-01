use crate::{Address, Chunk, FunctionType, Instruction, OperandType, Type, compile, tests::cases};

#[test]
fn boolean() {
    let source = cases::BOOLEAN;
    let chunk = compile(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![Instruction::load(
                Address::register(0),
                Address::encoded(true as u16),
                OperandType::BOOLEAN,
                false
            )],
            register_count: 1,
            ..Default::default()
        }
    );
}
