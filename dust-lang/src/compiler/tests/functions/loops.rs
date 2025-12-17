use crate::{
    compiler::compile,
    instruction::{Address, Instruction, OperandType},
    prototype::Prototype,
    source::{Position, SourceFileId, Span},
    tests::{create_function_case, loop_cases},
    r#type::{FunctionType, Type},
};

#[test]
fn while_loop() {
    let source = create_function_case(loop_cases::WHILE_LOOP);
    let prototypes = compile(source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            index: 1,
            name_position: Some(Position {
                file_id: SourceFileId(0),
                span: Span(16, 22)
            }),
            function_type: FunctionType::new([], [], Type::Integer),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER),
                Instruction::less(
                    true,
                    Address::register(0),
                    Address::constant(1),
                    OperandType::INTEGER
                ),
                Instruction::jump(2, true),
                Instruction::add(
                    0,
                    Address::register(0),
                    Address::constant(2),
                    OperandType::INTEGER
                ),
                Instruction::jump(2, false),
                Instruction::r#return(Address::register(0), OperandType::INTEGER),
            ],
            register_count: 1,
            ..Default::default()
        }
    );
}
