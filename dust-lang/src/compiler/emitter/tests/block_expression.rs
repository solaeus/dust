use smallvec::smallvec;

use crate::{
    compiler::emitter::tests::emit_function,
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

#[test]
fn tail_expression() {
    let prototype = emit_function("fn foo() -> i32 { let x: i32 = 1; { let y: i32 = 2; x + y } }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 3),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
        }
    );
}

#[test]
fn multi_register_return() {
    let prototype = emit_function(
        "struct Pair { a: i32, b: i32 } fn foo() -> Pair { { Pair { a: 1, b: 2 } } }",
    );

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 1),
                Instruction::r#move(1, OperandType::I_32, MemoryKind::ENCODED, 2),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32, OperandType::I_32],
            register_count: 2,
            argument_count: 0,
        }
    );
}
