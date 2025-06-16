use std::fmt::{self, Display, Formatter};

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

        if *return_type != OperandType::NONE {
            destination.display(f, *return_type)?;
            write!(f, " = ")?;
        }

        function.display(f, OperandType::FUNCTION)?;

        write!(f, "(ARGS_{argument_list_index})")
    }
}
