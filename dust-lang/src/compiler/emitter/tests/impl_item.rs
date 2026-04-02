use crate::{
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
    resolver::symbols::SymbolId,
    source::{Position, SourceFileId, Span},
};

use super::emit_function;

#[test]
fn impl_method_returning_value() {
    let prototype = emit_function(
        "struct Foo {} impl Foo { fn value() -> i32 { 42 } } fn foo() -> i32 { Foo::value() }",
    );

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::call(0, MemoryKind::CONSTANT, 0, u16::MAX),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
            debug_symbol_id: Some(SymbolId(16)),
            debug_position: Position::new(SourceFileId::MAIN, Span::new(52, 83)),
        }
    );
}
