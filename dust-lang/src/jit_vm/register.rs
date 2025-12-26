use cranelift::prelude::{Type as CraneliftType, types::I8};

use crate::{instruction::OperandType, jit_vm::Object};

#[derive(Clone, Copy)]
#[repr(C)]
pub union Register {
    pub empty: (),
    pub boolean: bool,
    pub byte: u8,
    pub character: char,
    pub float: f64,
    pub integer: i64,
    pub prototype_index: usize,
    pub object_pointer: *mut Object,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RegisterTag(pub u8);

impl RegisterTag {
    pub const EMPTY: RegisterTag = RegisterTag(0);
    pub const SCALAR: RegisterTag = RegisterTag(1);
    pub const OBJECT: RegisterTag = RegisterTag(2);

    pub const CRANELIFT_TYPE: CraneliftType = I8;
}

impl From<OperandType> for RegisterTag {
    fn from(operand_type: OperandType) -> Self {
        match operand_type {
            OperandType::NONE => RegisterTag::EMPTY,
            OperandType::BOOLEAN
            | OperandType::BYTE
            | OperandType::CHARACTER
            | OperandType::FLOAT
            | OperandType::INTEGER => RegisterTag::SCALAR,
            _ => RegisterTag::OBJECT,
        }
    }
}
