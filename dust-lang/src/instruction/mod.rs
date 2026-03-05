//! The Dust instruction set.
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
mod r#return;
mod set_list;
mod small_type;
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
pub use r#return::Return;
pub use set_list::SetList;
pub use small_type::SmallType;
pub use subtract::Subtract;
pub use test::Test;
pub use to_string::ToString;

use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display, Formatter};

use crate::native_function::NativeFunction;

/// An instruction for the Dust virtual machine.
///
/// Each instruction is 64 bits and uses up to seven distinct fields.
///
/// # Layout
///
/// Bits    | Description
/// ------- | -----------
/// 0..=5   | Operation
/// 6..=7   | B memory kind ━━━━━━━━━┓
/// 8..=9   | C memory kind  ──────┐ ┃
/// 10..=15 | Type or D field      │ ┃
/// 16..=31 | A field              │ ┃
/// 48..=63 | B field ━━━━━━━━━━━━━━━┻━ B address
/// 32..=47 | C field ─────────────┴─── C address
#[derive(Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(C)]
pub struct Instruction(u64);

impl Instruction {
    pub fn inner(&self) -> u64 {
        self.0
    }

    pub fn operation(&self) -> Operation {
        Operation(self.0 as u8 & 0x1F)
    }

    pub fn operand_type(&self) -> SmallType {
        SmallType(((self.0 >> 10) & 0x1F) as u8)
    }

    pub fn b_memory_kind(&self) -> MemoryKind {
        MemoryKind(((self.0 >> 7) & 0x3) as u8)
    }

    pub fn c_memory_kind(&self) -> MemoryKind {
        MemoryKind(((self.0 >> 9) & 0x3) as u8)
    }

    pub fn d_field(&self) -> u16 {
        ((self.0 >> 10) & 0x1F) as u16
    }

    pub fn a_field(&self) -> u16 {
        ((self.0 >> 16) & 0xFFFF) as u16
    }

    pub fn b_field(&self) -> u16 {
        ((self.0 >> 32) & 0xFFFF) as u16
    }

    pub fn c_field(&self) -> u16 {
        ((self.0 >> 48) & 0xFFFF) as u16
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

    pub fn r#move(
        destination: u16,
        operand_type: SmallType,
        operand: Address,
        secondary_index: u16,
    ) -> Instruction {
        Instruction::from(Move {
            destination,
            operand_type,
            operand,
            secondary_index,
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
            source_operand: item_source,
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

    pub fn r#return(
        operand_type: SmallType,
        operand: Address,
        secondary_index: u16,
    ) -> Instruction {
        Instruction::from(Return {
            operand_type,
            operand,
            secondary_index,
        })
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
    pub operand_type: SmallType,
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
        bits |= (self.d_field as u64 & 0x1F) << 10;
        bits |= (self.operand_type.0 as u64 & 0x1F) << 10;
        bits |= (self.a_field as u64 & 0xFFFF) << 16;
        bits |= (self.b_field as u64 & 0xFFFF) << 32;
        bits |= (self.c_field as u64 & 0xFFFF) << 48;

        Instruction(bits)
    }
}

impl From<&Instruction> for InstructionFields {
    fn from(instruction: &Instruction) -> Self {
        InstructionFields {
            operation: instruction.operation(),
            operand_type: instruction.operand_type(),
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
    pub const PROTOTYPE: MemoryKind = MemoryKind(2);
    pub const COMPOUND: MemoryKind = MemoryKind(3);

    pub fn display(&self, r#type: SmallType) -> &'static str {
        match *self {
            Self::REGISTER => "reg",
            Self::CONSTANT => "const",
            Self::PROTOTYPE => "proto",
            Self::COMPOUND if matches!(r#type, SmallType::U_128 | SmallType::I_128) => "& ",
            Self::COMPOUND if r#type == SmallType::STRUCT => "..=",
            _ => "invalid",
        }
    }
}

impl Display for MemoryKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            Self::REGISTER => write!(f, "reg"),
            Self::CONSTANT => write!(f, "const"),
            Self::PROTOTYPE => write!(f, "proto"),
            Self::COMPOUND => write!(f, " ..="),
            _ => write!(f, "invalid"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CallArgument {
    pub index: u16,
    pub memory: MemoryKind,
    pub r#type: SmallType,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_instruction() -> Instruction {
        Instruction::r#move(42, SmallType::U_128, Address::constant(42), 42)
    }

    #[test]
    fn decode_operation() {
        let instruction = create_instruction();

        assert_eq!(instruction.operation(), Operation::MOVE);
    }

    #[test]
    fn decode_b_memory() {
        let instruction = create_instruction();

        assert_eq!(instruction.b_memory_kind(), MemoryKind::CONSTANT);
    }

    #[test]
    fn decode_c_memory() {
        let instruction = Instruction::add(42, Address::constant(1), Address::constant(2));

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

        assert_eq!(instruction.b_field(), 42);
    }

    #[test]
    fn decode_c_field() {
        let instruction = create_instruction();

        assert_eq!(instruction.c_field(), 42);
    }

    #[test]
    fn decode_d_field() {
        let instruction = Instruction::call(None, Address::register(666), 30, 16);

        assert_eq!(instruction.d_field(), 16);
    }
}
