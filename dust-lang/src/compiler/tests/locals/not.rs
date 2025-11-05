use crate::{
    chunk::Chunk,
    compiler::compile_main,
    instruction::{Address, Instruction, OperandType},
    tests::local_cases,
    r#type::{FunctionType, Type},
};

#[test]
fn local_boolean_not() {
    let source = local_cases::LOCAL_BOOLEAN_NOT.to_string();
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            function_type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::r#move(
                    0,
                    Address::encoded(true as u16),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::negate(1, Address::register(0), OperandType::BOOLEAN),
                Instruction::r#return(true, Address::register(1), OperandType::BOOLEAN)
            ],
            register_count: 2,
            ..Default::default()
        }
    );
}
