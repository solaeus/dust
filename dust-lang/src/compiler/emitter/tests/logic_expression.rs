use crate::{
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
    resolver::symbols::SymbolId,
    source::{Position, SourceFileId, Span},
};

use super::emit_function;

#[test]
fn and_expression() {
    let prototype =
        emit_function("fn foo() -> bool { let a: bool = true; let b: bool = false; a && b }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::test(false, MemoryKind::CONSTANT, 0, 1),
                Instruction::r#move(0, OperandType::BOOLEAN, MemoryKind::CONSTANT, 1),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::BOOLEAN],
            register_count: 1,
            argument_count: 0,
            debug_symbol_id: Some(SymbolId(14)),
            debug_position: Position::new(SourceFileId::MAIN, Span::new(0, 68)),
        }
    );
}

#[test]
fn or_expression() {
    let prototype =
        emit_function("fn foo() -> bool { let a: bool = true; let b: bool = false; a || b }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::test(true, MemoryKind::CONSTANT, 0, 1),
                Instruction::r#move(0, OperandType::BOOLEAN, MemoryKind::CONSTANT, 1),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::BOOLEAN],
            register_count: 1,
            argument_count: 0,
            debug_symbol_id: Some(SymbolId(14)),
            debug_position: Position::new(SourceFileId::MAIN, Span::new(0, 68)),
        }
    );
}
