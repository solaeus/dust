use std::fmt::{self, Display, Formatter};

use crate::{Address, Instruction, Operation, r#type::TypeKind};

use super::{Destination, InstructionFields};

pub struct TestSet {
    pub destination: Destination,
    pub comparator: bool,
    pub operand: Address,
}

impl From<Instruction> for TestSet {
    fn from(instruction: Instruction) -> Self {
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
        let Destination {
            index: a_field,
            is_register: a_is_register,
        } = test_set.destination;
        let b_field = test_set.comparator as u16;
        let Address {
            index: c_field,
            kind: c_kind,
        } = test_set.operand;

        InstructionFields {
            operation,
            a_field,
            a_is_register,
            b_field,
            c_field,
            c_kind,
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

        write!(f, "if {bang}{operand} {{ JUMP +1 }} else {{")?;
        destination.display(f, TypeKind::Boolean)?;
        write!(f, " = {operand} }}")
    }
}
