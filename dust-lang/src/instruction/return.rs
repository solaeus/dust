use std::fmt::{self, Display, Formatter};

use crate::r#type::TypeKind;

use super::{Address, Instruction, InstructionFields, OperandType, Operation};

pub struct Return {
    pub should_return_value: bool,
    pub return_value_address: Address,
    pub r#type: OperandType,
}

impl From<&Instruction> for Return {
    fn from(instruction: &Instruction) -> Self {
        let should_return_value = instruction.a_field() != 0;
        let return_value_address = instruction.b_address();
        let r#type = instruction.operand_type();

        Return {
            should_return_value,
            return_value_address,
            r#type,
        }
    }
}

impl From<Return> for Instruction {
    fn from(r#return: Return) -> Self {
        let operation = Operation::RETURN;
        let a_field = r#return.should_return_value as u16;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = r#return.return_value_address;

        InstructionFields {
            operation,
            a_field,
            b_field,
            b_memory_kind,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Return {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Return {
            should_return_value,
            return_value_address,
            r#type,
        } = self;
        let type_kind = match *r#type {
            OperandType::BOOLEAN => TypeKind::Boolean,
            OperandType::BYTE => TypeKind::Byte,
            OperandType::CHARACTER => TypeKind::Character,
            OperandType::FLOAT => TypeKind::Float,
            OperandType::INTEGER => TypeKind::Integer,
            OperandType::STRING => TypeKind::String,
            OperandType::LIST => TypeKind::List,
            OperandType::NONE => TypeKind::None,
            _ => return write!(f, "INVALID_RETURN_INSTRUCTION"),
        };

        write!(f, "RETURN")?;

        if *should_return_value {
            write!(f, " ")?;
            return_value_address.display(f, type_kind)?;
        }

        Ok(())
    }
}
