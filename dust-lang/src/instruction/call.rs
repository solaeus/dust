use std::fmt::{self, Display, Formatter};

use crate::{Instruction, Operation};

use super::{Destination, InstructionFields, Operand, TypeCode, operand::OperandKind};

pub struct Call {
    pub destination: Destination,
    pub function: Operand,
    pub argument_list_index: u16,
    pub return_type: TypeCode,
}

impl From<Instruction> for Call {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.destination();
        let function_register = instruction.b_operand();
        let Operand {
            index: argument_list_index,
            kind: return_kind,
        } = instruction.c_operand();
        let return_type = TypeCode(return_kind.0);

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
        let Destination {
            index: a_field,
            is_register: a_is_register,
        } = call.destination;
        let Operand {
            index: b_field,
            kind: b_kind,
        } = call.function;
        let c_field = call.argument_list_index;
        let c_kind = OperandKind(call.return_type.0);

        InstructionFields {
            operation,
            a_field,
            a_is_register,
            b_field,
            b_kind,
            c_field,
            c_kind,
        }
        .build()
    }
}

impl Display for Call {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Call {
            destination,
            function: function_register,
            argument_list_index,
            return_type,
        } = self;

        match *return_type {
            TypeCode::NONE => {}
            TypeCode::BOOLEAN => write!(f, "R_BOOL_{} = ", destination.index)?,
            TypeCode::BYTE => write!(f, "R_BYTE_{} = ", destination.index)?,
            TypeCode::CHARACTER => write!(f, "R_CHAR_{} = ", destination.index)?,
            TypeCode::FLOAT => write!(f, "R_FLOAT_{} = ", destination.index)?,
            TypeCode::INTEGER => write!(f, "R_INT_{} = ", destination.index)?,
            TypeCode::STRING => write!(f, "R_STR_{} = ", destination.index)?,
            TypeCode::LIST => write!(f, "R_LIST_{} = ", destination.index)?,
            TypeCode::FUNCTION => write!(f, "R_FN_{} = ", destination.index)?,
            _ => unreachable!(),
        }

        write!(f, "R_FN_{function_register}(ARGS_{argument_list_index})")
    }
}
