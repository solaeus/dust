use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, OperandType, Operation};

pub struct List {
    pub destination: Address,
    pub start: Address,
    pub end: Address,
    pub r#type: OperandType,
}

impl From<Instruction> for List {
    fn from(instruction: Instruction) -> Self {
        let destination = instruction.destination();
        let start = instruction.b_address();
        let end = instruction.c_address();
        let r#type = instruction.operand_type();

        List {
            destination,
            start,
            end,
            r#type,
        }
    }
}

impl From<List> for Instruction {
    fn from(list: List) -> Self {
        let operation = Operation::LIST;
        let Address {
            index: a_field,
            memory: a_memory_kind,
        } = list.destination;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = list.start;
        let Address {
            index: c_field,
            memory: c_memory_kind,
        } = list.end;
        let operand_type = list.r#type;

        InstructionFields {
            operation,
            a_field,
            a_memory_kind,
            b_field,
            b_memory_kind,
            c_field,
            c_memory_kind,
            operand_type,
        }
        .build()
    }
}

impl Display for List {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let List {
            destination,
            start,
            end,
            r#type,
        } = self;

        destination.display(f, *r#type)?;
        write!(f, " = [")?;
        start.display(f, r#type.b_type())?;
        write!(f, "..=")?;
        end.display(f, r#type.c_type())?;
        write!(f, "]")
    }
}
