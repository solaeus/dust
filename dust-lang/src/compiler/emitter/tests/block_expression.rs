use crate::{
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

use super::emit_function;

#[test]
fn tail_expression() {
    let prototype = emit_function("fn foo() -> i32 { let x: i32 = 1; { let y: i32 = 2; x + y } }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::add(
                    0,
                    OperandType::I_32,
                    MemoryKind::ENCODED,
                    1,
                    MemoryKind::ENCODED,
                    2
                ),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
        }
    );
}
