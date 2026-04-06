use crate::{
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

use super::emit_function;

#[test]
fn unit_variant() {
    let prototype =
        emit_function("enum Color { Red, Green, Blue } fn foo() -> Color { Color::Red }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::U_32, MemoryKind::CONSTANT, 0),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::U_32],
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
                Instruction::r#move(0, OperandType::U_32, MemoryKind::CONSTANT, 0),
                Instruction::r#move(1, OperandType::F_64, MemoryKind::CONSTANT, 1),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::U_32, OperandType::F_64],
            register_count: 3,
            argument_count: 0,
        }
    );
}
