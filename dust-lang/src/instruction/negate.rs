use std::fmt::{self, Display, Formatter};

use crate::r#type::TypeKind;

use super::{Address, Instruction, InstructionFields, OperandType, Operation};

pub struct Negate {
    pub destination: Address,
    pub operand: Address,
    pub r#type: OperandType,
}

impl From<&Instruction> for Negate {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.destination();
        let operand = instruction.b_address();
        let r#type = instruction.operand_type();

        Negate {
            destination,
            operand,
            r#type,
        }
    }
}

impl From<Negate> for Instruction {
    fn from(negate: Negate) -> Self {
        let operation = Operation::NEGATE;
        let Address {
            index: a_field,
            memory: a_memory_kind,
        } = negate.destination;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = negate.operand;
        let operand_type = negate.r#type;

        InstructionFields {
            operation,
            a_field,
            a_memory_kind,
            b_field,
            b_memory_kind,
            operand_type,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Negate {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Negate {
            destination,
            operand,
            r#type,
        } = self;

        match *r#type {
            OperandType::BOOLEAN => {
                destination.display(f, TypeKind::Boolean)?;
                write!(f, " = !")?;
                operand.display(f, TypeKind::Boolean)
            }
            OperandType::FLOAT => {
                destination.display(f, TypeKind::Float)?;
                write!(f, " = -")?;
                operand.display(f, TypeKind::Float)
            }
            OperandType::INTEGER => {
                destination.display(f, TypeKind::Integer)?;
                write!(f, " = -")?;
                operand.display(f, TypeKind::Integer)
            }
            _ => write!(f, "INVALID_NEGATE_INSTRUCTION"),
        }
    }
}
