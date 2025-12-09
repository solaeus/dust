use std::fmt::{self, Display, Formatter};

use super::{Instruction, InstructionFields, OperandType, Operation};

pub struct Call {
    pub destination: u16,
    pub prototype_index: u16,
    pub arguments_start: u16,
    pub argument_count: u16,
    pub return_type: OperandType,
}

impl From<&Instruction> for Call {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
        let prototype_index = instruction.b_field();
        let arguments_start = instruction.c_field();
        let argument_count = instruction.d_field();
        let return_type = instruction.operand_type();

        Call {
            destination,
            prototype_index,
            arguments_start,
            argument_count,
            return_type,
        }
    }
}

impl From<Call> for Instruction {
    fn from(call: Call) -> Self {
        let operation = Operation::CALL;
        let a_field = call.destination;
        let b_field = call.prototype_index;
        let c_field = call.arguments_start;
        let d_field = Some(call.argument_count);
        let operand_type = call.return_type;

        InstructionFields {
            operation,
            a_field,
            b_field,
            c_field,
            d_field,
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
            arguments_start,
            argument_count,
            return_type,
        } = self;

        if *return_type != OperandType::NONE {
            write!(f, "reg_{destination} = ")?;
        }

        if *argument_count == 0 {
            write!(f, "proto_{prototype_index}()")
        } else {
            let arguments_end = arguments_start + argument_count;

            write!(
                f,
                "proto_{prototype_index}(args_{arguments_start}..args_{arguments_end})"
            )
        }
    }
}
