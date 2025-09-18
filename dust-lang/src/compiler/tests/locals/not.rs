use crate::{
    Address, Chunk, Instruction, OperandType, compile_main, resolver::TypeId, tests::local_cases,
};
use std::sync::Arc;

#[test]
fn local_boolean_not() {
    let source = local_cases::LOCAL_BOOLEAN_NOT;
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
                Instruction::negate(
                    Address::register(1),
                    Address::register(0),
                    OperandType::BOOLEAN
                ),
                Instruction::r#return(true, Address::register(1), OperandType::BOOLEAN)
            ],
            register_count: 2,
            ..Default::default()
        }
    );
}
