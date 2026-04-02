use crate::{
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
    resolver::symbols::SymbolId,
    source::{Position, SourceFileId, Span},
};

use super::emit_function;

#[test]
fn add_two_variables() {
    let prototype = emit_function("fn foo() -> i32 { let a: i32 = 1; let b: i32 = 2; a + b }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::add(
                    0,
                    OperandType::I_32,
                    MemoryKind::CONSTANT,
                    0,
                    MemoryKind::CONSTANT,
                    1
                ),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
            debug_symbol_id: Some(SymbolId(14)),
            debug_position: Position::new(SourceFileId::MAIN, Span::new(0, 57)),
        }
    );
}

#[test]
fn subtract_two_variables() {
    let prototype = emit_function("fn foo() -> i32 { let a: i32 = 5; let b: i32 = 3; a - b }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::subtract(
                    0,
                    OperandType::I_32,
                    MemoryKind::CONSTANT,
                    0,
                    MemoryKind::CONSTANT,
                    1
                ),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
            debug_symbol_id: Some(SymbolId(14)),
            debug_position: Position::new(SourceFileId::MAIN, Span::new(0, 57)),
        }
    );
}

#[test]
fn multiply_two_variables() {
    let prototype = emit_function("fn foo() -> i32 { let a: i32 = 3; let b: i32 = 4; a * b }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::multiply(
                    0,
                    OperandType::I_32,
                    MemoryKind::CONSTANT,
                    0,
                    MemoryKind::CONSTANT,
                    1
                ),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
            debug_symbol_id: Some(SymbolId(14)),
            debug_position: Position::new(SourceFileId::MAIN, Span::new(0, 57)),
        }
    );
}

#[test]
fn tail_addition() {
    let prototype = emit_function("fn foo() -> i32 { 1 + 2 }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::add(
                    0,
                    OperandType::I_32,
                    MemoryKind::CONSTANT,
                    0,
                    MemoryKind::CONSTANT,
                    1
                ),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
            debug_symbol_id: Some(SymbolId(14)),
            debug_position: Position::new(SourceFileId::MAIN, Span::new(0, 25)),
        }
    );
}
