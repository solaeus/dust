use smallvec::smallvec;

use crate::{
    compiler::emitter::tests::emit_function,
    instruction::{Address, Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

#[test]
fn used_in_return() {
    let prototype = emit_function("fn foo() -> i32 { let x: i32 = 42; x }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 42),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
        }
    );
}

#[test]
fn function_call_binding() {
    let prototype =
        emit_function("fn bar() -> i32 { 42 } fn foo() -> i32 { let x: i32 = bar(); x + 1 }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::call(0, MemoryKind::ENCODED, 1, u16::MAX),
                Instruction::add(
                    0,
                    OperandType::I_32,
                    Address {
                        memory: MemoryKind::REGISTER,
                        index: 0,
                    },
                    Address {
                        memory: MemoryKind::ENCODED,
                        index: 1
                    }
                ),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
        }
    );
}
