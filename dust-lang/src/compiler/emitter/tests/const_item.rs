use crate::{
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
    resolver::symbols::SymbolId,
    source::{Position, SourceFileId, Span},
};

use super::emit_function;

#[test]
fn const_used_in_return() {
    let prototype = emit_function("fn foo() -> i32 { const X: i32 = 10; X }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::CONSTANT, 0),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
            debug_symbol_id: Some(SymbolId(14)),
            debug_position: Position::new(SourceFileId::MAIN, Span::new(0, 40)),
        }
    );
}

#[test]
fn const_expression_value() {
    let prototype = emit_function("fn foo() -> i32 { const X: i32 = 1 + 2; X }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::CONSTANT, 0),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
            debug_symbol_id: Some(SymbolId(14)),
            debug_position: Position::new(SourceFileId::MAIN, Span::new(0, 43)),
        }
    );
}
