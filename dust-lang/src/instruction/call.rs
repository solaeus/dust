use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, OperandType, Operation};

pub struct Call {
    pub destination: Address,
    pub prototype_index: usize,
    pub arguments_index: usize,
    pub return_type: OperandType,
}

impl From<Instruction> for Call {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.destination();
        let prototype_index = instruction.b_field();
        let arguments_index = instruction.c_field();
        let return_type = instruction.operand_type();

        Call {
            destination,
            prototype_index,
            arguments_index,
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
        let b_field = call.prototype_index;
        let c_field = call.arguments_index;
        let operand_type = call.return_type;

        InstructionFields {
            operation,
            a_field,
            a_memory_kind,
            b_field,
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
            prototype_index,
            arguments_index,
            return_type,
        } = self;

        if *return_type != OperandType::NONE {
            write!(f, "{destination} = ")?;
        }

        write!(f, "proto_{prototype_index}(args_{arguments_index})")
    }
}
