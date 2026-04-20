use smallvec::smallvec;

use crate::{
    compiler::emitter::tests::emit_function,
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

#[test]
fn unit_variant() {
    let prototype =
        emit_function("enum Color { Red, Green, Blue } fn foo() -> Color { Color::Red }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::U_16, MemoryKind::ENCODED, 0),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::U_16],
            register_count: 1,
            argument_count: 0,
        }
    );
}

#[test]
fn tuple_variant() {
    let prototype = emit_function(
        "enum Shape { Circle(f64), Square(f64) } fn foo() -> Shape { Shape::Circle(3.14) }",
    );

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::U_16, MemoryKind::ENCODED, 0),
                Instruction::r#move(1, OperandType::F_64, MemoryKind::CONSTANT, 0),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::U_16, OperandType::F_64],
            register_count: 3,
            argument_count: 0,
        }
    );
}
