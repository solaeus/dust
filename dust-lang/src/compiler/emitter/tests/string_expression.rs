use crate::{
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

use super::emit_function;

#[test]
fn hello() {
    let prototype = emit_function("fn foo() -> str { \"hello\" }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::POINTER, MemoryKind::CONSTANT, 0),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::POINTER],
            register_count: 1,
            argument_count: 0,
        }
    );
}
