use crate::{
    chunk::Chunk,
    compiler::compile_main,
    instruction::{Address, Instruction, OperandType},
    tests::local_cases,
    r#type::{FunctionType, Type},
};

#[test]
fn local_boolean_or() {
    let source = local_cases::LOCAL_BOOLEAN_OR.to_string();
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            function_type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::r#move(0, Address::encoded(true as u16), OperandType::BOOLEAN),
                Instruction::r#move(1, Address::encoded(false as u16), OperandType::BOOLEAN),
                Instruction::test(Address::register(0), true),
                Instruction::jump(1, true),
                Instruction::move_with_jump(2, Address::register(0), OperandType::BOOLEAN, 1, true),
                Instruction::r#move(2, Address::register(1), OperandType::BOOLEAN),
                Instruction::r#return(true, Address::register(2), OperandType::BOOLEAN)
            ],
            register_count: 3,
            ..Default::default()
        }
    );
}
