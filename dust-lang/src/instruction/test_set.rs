use std::fmt::{self, Display, Formatter};

use crate::OperandType;

use super::{Address, Instruction, InstructionFields, Operation};

pub struct TestSet {
    pub destination: Address,
    pub comparator: bool,
    pub operand: Address,
}

impl From<&Instruction> for TestSet {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.destination();
        let comparator = instruction.b_field() != 0;
        let operand = instruction.c_address();

        TestSet {
            destination,
            operand,
            comparator,
        }
    }
}

impl From<TestSet> for Instruction {
    fn from(test_set: TestSet) -> Self {
        let operation = Operation::TEST;
        let Address {
            index: a_field,
            memory: a_memory_kind,
        } = test_set.destination;
        let b_field = test_set.comparator as u16;
        let Address {
            index: c_field,
            memory: c_memory_kind,
        } = test_set.operand;

        InstructionFields {
            operation,
            a_field,
            a_memory_kind,
            b_field,
            c_field,
            c_memory_kind,
            ..Default::default()
        }
        .build()
    }
}

impl Display for TestSet {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let TestSet {
            destination,
            operand,
            comparator,
        } = self;
        let bang = if *comparator { "" } else { "!" };

        write!(f, "if {bang}")?;
        operand.display(f, OperandType::BOOLEAN)?;
        write!(f, " {{ JUMP +1 }} else {{")?;
        destination.display(f, OperandType::BOOLEAN)?;
        write!(f, " = ")?;
        operand.display(f, OperandType::BOOLEAN)
    }
}
