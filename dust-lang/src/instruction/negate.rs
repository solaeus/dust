use std::fmt::{self, Display, Formatter};

use super::{Address, Instruction, InstructionFields, OperandType, Operation};

pub struct Negate {
    pub destination: u16,
    pub operand: Address,
    pub r#type: OperandType,
}

impl From<&Instruction> for Negate {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.a_field();
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
        let a_field = negate.destination;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = negate.operand;
        let operand_type = negate.r#type;

        InstructionFields {
            operation,
            a_field,
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

        let operator = if *r#type == OperandType::BOOLEAN {
            "!"
        } else {
            "-"
        };

        write!(f, "reg_{destination} = {operator}")?;
        operand.display(f, *r#type)
    }
}
