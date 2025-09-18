use crate::{
    Address, Chunk, Instruction, OperandType, compile_main, resolver::TypeId, tests::local_cases,
};
use std::sync::Arc;

#[test]
fn local_boolean_and() {
    let source = local_cases::LOCAL_BOOLEAN_AND;
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            name: Arc::new("main".to_string()),
            r#type: TypeId(2),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::encoded(true as u16),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::load(
                    Address::register(1),
                    Address::encoded(false as u16),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::load(
                    Address::register(2),
                    Address::register(0),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::test(Address::register(2), true),
                Instruction::jump(1, true),
                Instruction::r#return(true, Address::register(1), OperandType::BOOLEAN)
            ],
            register_count: 3,
            ..Default::default()
        }
    );
}
