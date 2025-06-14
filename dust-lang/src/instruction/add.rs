use std::fmt::{self, Display, Formatter};

use crate::r#type::TypeKind;

use super::{Address, Instruction, InstructionFields, OperandType, Operation};

pub struct Add {
    pub destination: Address,
    pub left: Address,
    pub right: Address,
    pub r#type: OperandType,
}

impl From<&Instruction> for Add {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.destination();
        let left = instruction.b_address();
        let right = instruction.c_address();
        let r#type = instruction.operand_type();

        Add {
            destination,
            left,
            right,
            r#type,
        }
    }
}

impl From<Add> for Instruction {
    fn from(add: Add) -> Self {
        let operation = Operation::ADD;
        let Address {
            index: a_field,
            memory: a_memory_kind,
        } = add.destination;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = add.left;
        let Address {
            index: c_field,
            memory: c_memory_kind,
        } = add.right;

        InstructionFields {
            operation,
            a_field,
            a_memory_kind,
            b_field,
            b_memory_kind,
            c_field,
            c_memory_kind,
            ..InstructionFields::default()
        }
        .build()
    }
}

impl Display for Add {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Add {
            destination,
            left,
            right,
            r#type,
        } = self;

        match *r#type {
            OperandType::BYTE => {
                destination.display(f, TypeKind::Byte)?;
                write!(f, " = ")?;
                left.display(f, TypeKind::Byte)?;
                write!(f, " + ")?;
                right.display(f, TypeKind::Byte)?;
            }
            OperandType::CHARACTER => {
                destination.display(f, TypeKind::Character)?;
                write!(f, " = ")?;
                left.display(f, TypeKind::Character)?;
                write!(f, " + ")?;
                right.display(f, TypeKind::Character)?;
            }
            OperandType::INTEGER => {
                destination.display(f, TypeKind::Integer)?;
                write!(f, " = ")?;
                left.display(f, TypeKind::Integer)?;
                write!(f, " + ")?;
                right.display(f, TypeKind::Integer)?;
            }
            OperandType::STRING => {
                destination.display(f, TypeKind::String)?;
                write!(f, " = ")?;
                left.display(f, TypeKind::String)?;
                write!(f, " + ")?;
                right.display(f, TypeKind::String)?;
            }
            OperandType::CHARACTER_STRING => {
                destination.display(f, TypeKind::String)?;
                write!(f, " = ")?;
                left.display(f, TypeKind::Character)?;
                write!(f, " + ")?;
                right.display(f, TypeKind::String)?;
            }
            OperandType::STRING_CHARACTER => {
                destination.display(f, TypeKind::String)?;
                write!(f, " = ")?;
                left.display(f, TypeKind::String)?;
                write!(f, " + ")?;
                right.display(f, TypeKind::Character)?;
            }
            invalid => invalid.invalid_panic(Operation::ADD),
        };

        Ok(())
    }
}
