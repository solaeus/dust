//! The Dust instruction set.
//!
//! Each instruction is 64 bits and uses up to seven distinct fields.
//!
//! # Layout
//!
//! Bits    | Description
//! ------- | -----------
//! 0..=5   | Operation
//! 6..=7   | B memory kind ━━━┓
//! 8..=9   | C memory kind  ┐ ┃
//! 10..=15 | D field        │ ┃
//! 16..=31 | A field        │ ┃
//! 48..=63 | C field ━━━━━━━━━┻━ B address
//! 32..=47 | B field ───────┴─── C address
mod add;
mod address;
mod call;
mod call_native;
mod divide;
mod drop;
mod equal;
mod get_list;
mod jump;
mod less;
mod less_equal;
mod modulo;
mod r#move;
mod multiply;
mod negate;
mod new_list;
mod operation;
mod power;
mod reference;
mod r#return;
mod set_list;
mod subtract;
mod test;
mod to_string;

pub use add::Add;
pub use address::Address;
pub use call::Call;
pub use call_native::CallNative;
pub use divide::Divide;
pub use drop::Drop;
pub use equal::Equal;
pub use get_list::GetList;
pub use jump::Jump;
pub use less::Less;
pub use less_equal::LessEqual;
pub use modulo::Modulo;
pub use r#move::Move;
pub use multiply::Multiply;
pub use negate::Negate;
pub use new_list::NewList;
pub use operation::Operation;
pub use power::Power;
pub use reference::Reference;
pub use r#return::Return;
pub use set_list::SetList;
pub use subtract::Subtract;
pub use test::Test;
pub use to_string::ToString;

use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display, Formatter};

use crate::native_function::NativeFunction;

/// An instruction for the Dust virtual machine.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(C)]
pub struct Instruction(pub(crate) u64);

impl Instruction {
    pub fn operation(&self) -> Operation {
        let bits_0_to_4 = (self.0 & 0x1F) as u8;

        Operation(bits_0_to_4)
    }

    pub fn b_address(&self) -> Address {
        Address {
            index: self.b_field(),
            memory: self.b_memory_kind(),
        }
    }

    pub fn c_address(&self) -> Address {
        Address {
            index: self.c_field(),
            memory: self.c_memory_kind(),
        }
    }

    pub fn b_memory_kind(&self) -> MemoryKind {
        let bits_7_to_8 = (self.0 >> 7) & 0x3;

        MemoryKind(bits_7_to_8 as u8)
    }

    pub fn c_memory_kind(&self) -> MemoryKind {
        let bits_9_to_10 = (self.0 >> 9) & 0x3;

        MemoryKind(bits_9_to_10 as u8)
    }

    pub fn a_field(&self) -> u16 {
        let bits_16_to_31 = (self.0 >> 16) & 0xFFFF;

        bits_16_to_31 as u16
    }

    pub fn b_field(&self) -> u16 {
        let bits_32_to_47 = (self.0 >> 32) & 0xFFFF;

        bits_32_to_47 as u16
    }

    pub fn c_field(&self) -> u16 {
        let bits_48_to_63 = (self.0 >> 48) & 0xFFFF;

        bits_48_to_63 as u16
    }

    pub fn d_field(&self) -> u16 {
        let bits_11_to_15 = (self.0 >> 11) & 0x1F;

        bits_11_to_15 as u16
    }

    pub fn set_b_field(&mut self, bits: u16) {
        let mut fields = InstructionFields::from(&*self);
        fields.b_field = bits;
        *self = fields.build();
    }

    pub fn set_c_field(&mut self, bits: u16) {
        let mut fields = InstructionFields::from(&*self);
        fields.c_field = bits;
        *self = fields.build();
    }

    pub fn no_op() -> Instruction {
        Instruction(0)
    }

    pub fn r#move(destination: u16, operand: Address) -> Instruction {
        Instruction::from(Move {
            destination,
            operand,
            jump_distance: 0,
            jump_is_positive: false,
        })
    }

    pub fn move_with_jump(
        destination: u16,
        operand: Address,
        jump_distance: u16,
        jump_is_positive: bool,
    ) -> Instruction {
        Instruction::from(Move {
            destination,
            operand,
            jump_distance,
            jump_is_positive,
        })
    }

    pub fn reference(destination: u16, start: u16, length: u16) -> Instruction {
        Instruction::from(Reference {
            destination,
            start,
            length,
        })
    }

    pub fn drop(drop_list_start: u16, drop_list_end: u16) -> Instruction {
        Instruction::from(Drop {
            drop_list_start,
            drop_list_end,
        })
    }

    pub fn new_list(destination: u16, initial_length: Address) -> Instruction {
        Instruction::from(NewList {
            destination,
            initial_length,
        })
    }

    pub fn set_list(destination_list: u16, item_source: Address, index: Address) -> Instruction {
        Instruction::from(SetList {
            destination_list,
            item_source,
            index,
        })
    }

    pub fn get_list(destination: u16, list: Address, list_index: Address) -> Instruction {
        Instruction::from(GetList {
            destination,
            list,
            list_index,
        })
    }

    pub fn add(destination: u16, left: Address, right: Address) -> Instruction {
        Instruction::from(Add {
            destination,
            left,
            right,
        })
    }

    pub fn subtract(destination: u16, left: Address, right: Address) -> Instruction {
        Instruction::from(Subtract {
            destination,
            left,
            right,
        })
    }

    pub fn multiply(destination: u16, left: Address, right: Address) -> Instruction {
        Instruction::from(Multiply {
            destination,
            left,
            right,
        })
    }

    pub fn divide(destination: u16, left: Address, right: Address) -> Instruction {
        Instruction::from(Divide {
            destination,
            left,
            right,
        })
    }

    pub fn modulo(destination: u16, left: Address, right: Address) -> Instruction {
        Instruction::from(Modulo {
            destination,
            left,
            right,
        })
    }

    pub fn power(destination: u16, base: Address, exponent: Address) -> Instruction {
        Instruction::from(Power {
            destination,
            base,
            exponent,
        })
    }

    pub fn equal(comparator: bool, left: Address, right: Address) -> Instruction {
        Instruction::from(Equal {
            comparator,
            left,
            right,
        })
    }

    pub fn less(comparator: bool, left: Address, right: Address) -> Instruction {
        Instruction::from(Less {
            comparator,
            left,
            right,
        })
    }

    pub fn less_equal(comparator: bool, left: Address, right: Address) -> Instruction {
        Instruction::from(LessEqual {
            comparator,
            left,
            right,
        })
    }

    pub fn test(operand: Address, comparator: bool, jump_distance: u16) -> Instruction {
        Instruction::from(Test {
            operand,
            comparator,
            jump_distance,
        })
    }

    pub fn negate(destination: u16, operand: Address) -> Instruction {
        Instruction::from(Negate {
            destination,
            operand,
        })
    }

    pub fn jump(offset: u16, is_positive: bool) -> Instruction {
        Instruction::from(Jump {
            offset,
            is_positive,
            drop_list_start: 0,
            drop_list_end: 0,
        })
    }

    pub fn jump_with_drops(
        offset: u16,
        is_positive: bool,
        drop_list_start: u16,
        drop_list_end: u16,
    ) -> Instruction {
        Instruction::from(Jump {
            offset,
            is_positive,
            drop_list_start,
            drop_list_end,
        })
    }

    pub fn call(
        destination: Option<u16>,
        callee: Address,
        arguments_start: u16,
        argument_count: u16,
    ) -> Instruction {
        Instruction::from(Call {
            destination: destination.unwrap_or(u16::MAX),
            callee,
            arguments_start,
            argument_count,
        })
    }

    pub fn call_native(
        destination: u16,
        function: NativeFunction,
        arguments_start: u16,
    ) -> Instruction {
        Instruction::from(CallNative {
            destination,
            function_id: function.id,
            arguments_start,
        })
    }

    pub fn r#return(operand: Address) -> Instruction {
        Instruction::from(Return { operand })
    }

    pub fn to_string(destination: u16, operand: Address) -> Instruction {
        Instruction::from(ToString {
            destination,
            operand,
        })
    }

    pub fn is_coallescible_with_jump(&self, forward: bool) -> bool {
        match self.operation() {
            Operation::DROP => true,
            Operation::MOVE => {
                let Move {
                    jump_distance,
                    jump_is_positive,
                    ..
                } = Move::from(self);

                jump_distance == 0 || (forward == jump_is_positive)
            }
            Operation::TEST => {
                let Test { jump_distance, .. } = Test::from(self);

                jump_distance == 0 && forward
            }
            _ => false,
        }
    }

    pub fn disassembly_info(&self) -> String {
        let operation = self.operation();

        match operation {
            Operation::NO_OP => String::new(),
            Operation::MOVE => Move::from(self).to_string(),
            Operation::REFERENCE => Reference::from(self).to_string(),
            Operation::DROP => Drop::from(self).to_string(),
            Operation::NEW_LIST => NewList::from(self).to_string(),
            Operation::SET_LIST => SetList::from(self).to_string(),
            Operation::GET_LIST => GetList::from(self).to_string(),
            Operation::ADD => Add::from(self).to_string(),
            Operation::SUBTRACT => Subtract::from(self).to_string(),
            Operation::MULTIPLY => Multiply::from(self).to_string(),
            Operation::DIVIDE => Divide::from(self).to_string(),
            Operation::MODULO => Modulo::from(self).to_string(),
            Operation::POWER => Power::from(self).to_string(),
            Operation::NEGATE => Negate::from(self).to_string(),
            Operation::EQUAL => Equal::from(self).to_string(),
            Operation::LESS => Less::from(self).to_string(),
            Operation::LESS_EQUAL => LessEqual::from(self).to_string(),
            Operation::TEST => Test::from(self).to_string(),
            Operation::CALL => Call::from(self).to_string(),
            Operation::CALL_NATIVE => CallNative::from(self).to_string(),
            Operation::JUMP => Jump::from(self).to_string(),
            Operation::RETURN => Return::from(self).to_string(),
            Operation::TO_STRING => ToString::from(self).to_string(),
            unknown => format!("Unknown operation: {}", unknown.0),
        }
    }
}

impl Debug for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.operation(), self.disassembly_info())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InstructionFields {
    pub operation: Operation,
    pub b_memory_kind: MemoryKind,
    pub c_memory_kind: MemoryKind,
    pub d_field: u16,
    pub a_field: u16,
    pub b_field: u16,
    pub c_field: u16,
}

impl InstructionFields {
    pub fn build(self) -> Instruction {
        let mut bits = 0_u64;

        bits |= (self.operation.0 as u64) & 0x1F;
        bits |= ((self.b_memory_kind.0 as u64) & 0x3) << 7;
        bits |= ((self.c_memory_kind.0 as u64) & 0x3) << 9;
        bits |= ((self.d_field as u64) & 0x1F) << 11;
        bits |= ((self.a_field as u64) & 0xFFFF) << 16;
        bits |= ((self.b_field as u64) & 0xFFFF) << 32;
        bits |= ((self.c_field as u64) & 0xFFFF) << 48;

        Instruction(bits)
    }
}

impl From<&Instruction> for InstructionFields {
    fn from(instruction: &Instruction) -> Self {
        InstructionFields {
            operation: instruction.operation(),
            b_memory_kind: instruction.b_memory_kind(),
            c_memory_kind: instruction.c_memory_kind(),
            d_field: instruction.d_field(),
            a_field: instruction.a_field(),
            b_field: instruction.b_field(),
            c_field: instruction.c_field(),
        }
    }
}

#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct MemoryKind(pub u8);

impl MemoryKind {
    pub const REGISTER: MemoryKind = MemoryKind(0);
    pub const CONSTANT: MemoryKind = MemoryKind(1);
    pub const ENCODED: MemoryKind = MemoryKind(2);
    pub const PROTOTYPE: MemoryKind = MemoryKind(3);
}

impl Display for MemoryKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            Self::REGISTER => write!(f, "reg"),
            Self::CONSTANT => write!(f, "const"),
            Self::ENCODED => write!(f, "enc"),
            Self::PROTOTYPE => write!(f, "proto"),
            _ => write!(f, "invalid"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_instruction() -> Instruction {
        Instruction::add(42, Address::register(1), Address::constant(2))
    }

    #[test]
    fn decode_operation() {
        let instruction = create_instruction();

        assert_eq!(instruction.operation(), Operation::ADD);
    }

    #[test]
    fn decode_b_memory() {
        let instruction = create_instruction();

        assert_eq!(instruction.b_memory_kind(), MemoryKind::REGISTER);
    }

    #[test]
    fn decode_c_memory() {
        let instruction = create_instruction();

        assert_eq!(instruction.c_memory_kind(), MemoryKind::CONSTANT);
    }

    #[test]
    fn decode_a_field() {
        let instruction = create_instruction();

        assert_eq!(instruction.a_field(), 42);
    }

    #[test]
    fn decode_b_field() {
        let instruction = create_instruction();

        assert_eq!(instruction.b_field(), 1);
    }

    #[test]
    fn decode_c_field() {
        let instruction = create_instruction();

        assert_eq!(instruction.c_field(), 2);
    }

    #[test]
    fn decode_d_field() {
        let instruction = Instruction::call(Some(5), Address::constant(10), 25, 0);

        assert_eq!(instruction.d_field(), 0);

        let instruction = Instruction::call(None, Address::register(42), 30, 2);

        assert_eq!(instruction.d_field(), 2);
    }
}
