use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, OperandType, Operation};

pub struct Call {
    pub destination: Address,
    pub function: Address,
    pub arguments_index: usize,
    pub return_type: OperandType,
}

impl From<Instruction> for Call {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.destination();
        let function_register = instruction.b_address();
        let arguments_index = instruction.c_field();
        let return_type = instruction.operand_type();

        Call {
            destination,
            function: function_register,
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
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = call.function;
        let c_field = call.arguments_index;
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
            arguments_index,
            return_type,
        } = self;

        if *return_type != OperandType::NONE {
            write!(f, "{} = ", destination.display_with_type(*return_type))?;
        }

        write!(
            f,
            "{}(args_{})",
            function.display_with_type(OperandType::FUNCTION),
            arguments_index
        )
    }
}
