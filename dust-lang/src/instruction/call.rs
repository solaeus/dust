use std::fmt::{self, Display, Formatter};

use crate::r#type::TypeKind;

use super::{Address, Instruction, InstructionFields, OperandType, Operation};

pub struct Call {
    pub destination: Address,
    pub function: Address,
    pub argument_list_index: u16,
    pub return_type: OperandType,
}

impl From<&Instruction> for Call {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.destination();
        let function_register = instruction.b_address();
        let argument_list_index = instruction.c_field();
        let return_type = instruction.operand_type();

        Call {
            destination,
            function: function_register,
            argument_list_index,
            return_type,
        }
    }
}

impl From<Call> for Instruction {
    fn from(call: Call) -> Self {
        let operation = Operation::CALL;
        let Address {
            index: a_field,
            memory: a_memory_kind,
        } = call.destination;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = call.function;
        let c_field = call.argument_list_index;
        let operand_type = call.return_type;

        InstructionFields {
            operation,
            a_field,
            a_memory_kind,
            b_field,
            b_memory_kind,
            c_field,
            operand_type,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Call {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Call {
            destination,
            function,
            argument_list_index,
            return_type,
        } = self;
        let destination_type = match *return_type {
            OperandType::BOOLEAN => TypeKind::Boolean,
            OperandType::BYTE => TypeKind::Byte,
            OperandType::CHARACTER => TypeKind::Character,
            OperandType::FLOAT => TypeKind::Float,
            OperandType::INTEGER => TypeKind::Integer,
            OperandType::STRING => TypeKind::String,
            OperandType::LIST => TypeKind::List,
            OperandType::FUNCTION => TypeKind::Function,
            OperandType::NONE => TypeKind::None,
            _ => return write!(f, "INVALID_CALL_INSTRUCTION"),
        };
        let is_recursive = *function == Address::stack(u16::MAX);

        if destination_type != TypeKind::None {
            destination.display(f, destination_type)?;
            write!(f, " = ")?;
        }

        if is_recursive {
            function.display(f, TypeKind::FunctionSelf)?;
        } else {
            function.display(f, TypeKind::Function)?;
        }

        write!(f, "(ARGS_{argument_list_index})")
    }
}
