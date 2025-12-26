use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, OperandType, Operation};

pub struct Call {
    pub destination: u16,
    pub callee: Address,
    pub arguments_start: u16,
    pub argument_count: u16,
}

impl From<&Instruction> for Call {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let callee = instruction.b_address();
        let arguments_start = instruction.c_field();
        let argument_count = instruction.d_field();

        Call {
            destination,
            callee,
            arguments_start,
            argument_count,
        }
    }
}

impl From<Call> for Instruction {
    fn from(call: Call) -> Self {
        let operation = Operation::CALL;
        let a_field = call.destination;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = call.callee;
        let c_field = call.arguments_start;
        let d_field = Some(call.argument_count);

        InstructionFields {
            operation,
            a_field,
            b_field,
            b_memory_kind,
            c_field,
            d_field,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Call {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Call {
            destination,
            callee,
            arguments_start,
            argument_count,
        } = self;

        if *destination != u16::MAX {
            write!(f, "reg_{destination} = ")?;
        }

        callee.display(f, OperandType::FUNCTION)?;

        if *argument_count == 0 {
            write!(f, "()")
        } else {
            let arguments_end = arguments_start + argument_count;

            write!(f, "(args_{arguments_start}..args_{arguments_end})")
        }
    }
}
