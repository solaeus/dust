//! The Dust instruction set.
mod add;
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
mod operand_type;
mod operation;
mod power;
mod r#return;
mod set_list;
mod subtract;
mod test;

pub use add::Add;
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
pub use operand_type::OperandType;
pub use operation::Operation;
pub use power::Power;
pub use r#return::Return;
pub use set_list::SetList;
pub use subtract::Subtract;
pub use test::Test;

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
/// 6..=7   | B memory kind
/// 8..=9   | C memory kind
/// 10..=15 | Type or D field
/// 16..=31 | A field
/// 32..=47 | B field
/// 48..=63 | C field
///
/// - Operation: The opcode of the instruction, which determines how the other fields are interpreted
/// - A field: Usually the destination register index
/// - B and C fields: Usually indices for source registers or constants
/// - B and C memory kind: Whether the B and C fields refer to a register or a constant
/// - Type: Used by most instructions to indicate the type of the operand(s)
/// - D field: Used for CALL instructions to store the argument count
/// - E field: Boolean flag used by MOVE instructions that jump to indicate the direction
/// - BC field: Combined 32-bit field spanning the B and C fields
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

    pub fn operand_type(&self) -> OperandType {
        OperandType(((self.0 >> 10) & 0x1F) as u8)
    }

    pub fn b_memory(&self) -> MemoryKind {
        MemoryKind(((self.0 >> 6) & 0x3) as u8)
    }

    pub fn c_memory(&self) -> MemoryKind {
        MemoryKind(((self.0 >> 8) & 0x3) as u8)
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

    pub fn d_field(&self) -> u16 {
        ((self.0 >> 10) & 0x3F) as u16
    }

    pub fn bc_field(&self) -> u32 {
        ((self.0 >> 32) & 0xFFFFFFFF) as u32
    }

    pub fn no_op() -> Instruction {
        Instruction(0)
    }

    pub fn r#move(
        destination: u16,
        operand_type: OperandType,
        operand_memory: MemoryKind,
        operand_index: u16,
    ) -> Instruction {
        Instruction::from(Move {
            destination,
            operand_type,
            operand_memory,
            operand_index,
            jump_distance: 0,
            jump_forward: false,
        })
    }

    pub fn move_with_jump(
        destination: u16,
        operand_type: OperandType,
        operand_memory: MemoryKind,
        operand_index: u16,
        jump_distance: u16,
        jump_forward: bool,
    ) -> Instruction {
        Instruction::from(Move {
            destination,
            operand_type,
            operand_memory,
            operand_index,
            jump_distance,
            jump_forward,
        })
    }

    pub fn drop(drop_list_start: u16, drop_list_end: u16) -> Instruction {
        Instruction::from(Drop {
            drop_list_start,
            drop_list_end,
        })
    }

    pub fn new_list(
        destination: u16,
        element_type: OperandType,
        length_memory: MemoryKind,
        length_index: u16,
        element_size: u16,
    ) -> Instruction {
        Instruction::from(NewList {
            destination,
            element_type,
            length_memory,
            length_index,
            element_size,
        })
    }

    pub fn set_list(
        destination_list: u16,
        element_type: OperandType,
        source_memory: MemoryKind,
        source_index: u16,
        index_memory: MemoryKind,
        index_index: u16,
    ) -> Instruction {
        Instruction::from(SetList {
            destination_list,
            element_type,
            source_memory,
            source_index,
            index_memory,
            index_index,
        })
    }

    pub fn get_list(
        destination: u16,
        element_type: OperandType,
        list_index: u16,
        index_memory: MemoryKind,
        index_index: u16,
    ) -> Instruction {
        Instruction::from(GetList {
            destination,
            element_type,
            list_index,
            index_memory,
            index_index,
        })
    }

    pub fn add(
        destination: u16,
        operand_type: OperandType,
        left_memory: MemoryKind,
        left_index: u16,
        right_memory: MemoryKind,
        right_index: u16,
    ) -> Instruction {
        Instruction::from(Add {
            destination,
            operand_type,
            left_memory,
            left_index,
            right_memory,
            right_index,
        })
    }

    pub fn subtract(
        destination: u16,
        operand_type: OperandType,
        left_memory: MemoryKind,
        left_index: u16,
        right_memory: MemoryKind,
        right_index: u16,
    ) -> Instruction {
        Instruction::from(Subtract {
            destination,
            operand_type,
            left_memory,
            left_index,
            right_memory,
            right_index,
        })
    }

    pub fn multiply(
        destination: u16,
        operand_type: OperandType,
        left_memory: MemoryKind,
        left_index: u16,
        right_memory: MemoryKind,
        right_index: u16,
    ) -> Instruction {
        Instruction::from(Multiply {
            destination,
            operand_type,
            left_memory,
            left_index,
            right_memory,
            right_index,
        })
    }

    pub fn divide(
        destination: u16,
        operand_type: OperandType,
        left_memory: MemoryKind,
        left_index: u16,
        right_memory: MemoryKind,
        right_index: u16,
    ) -> Instruction {
        Instruction::from(Divide {
            destination,
            operand_type,
            left_memory,
            left_index,
            right_memory,
            right_index,
        })
    }

    pub fn modulo(
        destination: u16,
        operand_type: OperandType,
        left_memory: MemoryKind,
        left_index: u16,
        right_memory: MemoryKind,
        right_index: u16,
    ) -> Instruction {
        Instruction::from(Modulo {
            destination,
            operand_type,
            left_memory,
            left_index,
            right_memory,
            right_index,
        })
    }

    pub fn power(
        destination: u16,
        operand_type: OperandType,
        base_memory: MemoryKind,
        base_index: u16,
        exponent_memory: MemoryKind,
        exponent_index: u16,
    ) -> Instruction {
        Instruction::from(Power {
            destination,
            operand_type,
            base_memory,
            base_index,
            exponent_memory,
            exponent_index,
        })
    }

    pub fn equal(
        comparator: bool,
        operand_type: OperandType,
        left_memory: MemoryKind,
        left_index: u16,
        right_memory: MemoryKind,
        right_index: u16,
    ) -> Instruction {
        Instruction::from(Equal {
            comparator,
            operand_type,
            left_memory,
            left_index,
            right_memory,
            right_index,
        })
    }

    pub fn less(
        comparator: bool,
        operand_type: OperandType,
        left_memory: MemoryKind,
        left_index: u16,
        right_memory: MemoryKind,
        right_index: u16,
    ) -> Instruction {
        Instruction::from(Less {
            comparator,
            operand_type,
            left_memory,
            left_index,
            right_memory,
            right_index,
        })
    }

    pub fn less_equal(
        comparator: bool,
        operand_type: OperandType,
        left_memory: MemoryKind,
        left_index: u16,
        right_memory: MemoryKind,
        right_index: u16,
    ) -> Instruction {
        Instruction::from(LessEqual {
            comparator,
            operand_type,
            left_memory,
            left_index,
            right_memory,
            right_index,
        })
    }

    pub fn test(
        comparator: bool,
        operand_memory: MemoryKind,
        operand_index: u16,
        jump_distance: u16,
    ) -> Instruction {
        Instruction::from(Test {
            comparator,
            operand_memory,
            operand_index,
            jump_distance,
        })
    }

    pub fn negate(
        destination: u16,
        operand_type: OperandType,
        operand_memory: MemoryKind,
        operand_index: u16,
    ) -> Instruction {
        Instruction::from(Negate {
            destination,
            operand_type,
            operand_memory,
            operand_index,
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
        destination: u16,
        callee_memory: MemoryKind,
        callee_index: u16,
        arguments_start: u16,
        argument_count: u16,
    ) -> Instruction {
        Instruction::from(Call {
            destination,
            arguments_start,
            argument_count,
            callee_memory,
            callee_index,
        })
    }

    pub fn call_native(
        destination: u16,
        function: NativeFunction,
        arguments_start: u16,
        argument_count: u16,
    ) -> Instruction {
        Instruction::from(CallNative {
            destination,
            function_id: function.id,
            arguments_start,
            argument_count,
        })
    }

    pub fn r#return(returns_value: bool, argument_count: u16) -> Instruction {
        Instruction::from(Return {
            returns_value,
            argument_count,
        })
    }

    pub fn is_coallescible_with_jump(&self, forward: bool) -> bool {
        match self.operation() {
            Operation::DROP => true,
            Operation::MOVE => {
                let Move {
                    jump_distance,
                    jump_forward,
                    ..
                } = Move::from(self);

                jump_distance == 0 || forward == jump_forward
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstructionBuilder {
    operation: Operation,
    b_memory: Option<MemoryKind>,
    c_memory: Option<MemoryKind>,
    operand_type: Option<OperandType>,
    d_field: Option<u16>,
    a_field: Option<u16>,
    b_field: Option<u16>,
    c_field: Option<u16>,
}

impl InstructionBuilder {
    pub fn new(operation: Operation) -> Self {
        Self {
            operation,
            b_memory: None,
            c_memory: None,
            operand_type: None,
            d_field: None,
            a_field: None,
            b_field: None,
            c_field: None,
        }
    }

    pub fn b_memory(mut self, memory: MemoryKind) -> Self {
        self.b_memory = Some(memory);

        self
    }

    pub fn c_memory(mut self, memory: MemoryKind) -> Self {
        self.c_memory = Some(memory);

        self
    }

    pub fn operand_type(mut self, operand_type: OperandType) -> Self {
        self.operand_type = Some(operand_type);

        self
    }

    pub fn d_field(mut self, d_field: u16) -> Self {
        self.d_field = Some(d_field);

        self
    }

    pub fn a_field(mut self, a_field: u16) -> Self {
        self.a_field = Some(a_field);

        self
    }

    pub fn b_field(mut self, b_field: u16) -> Self {
        self.b_field = Some(b_field);

        self
    }

    pub fn c_field(mut self, c_field: u16) -> Self {
        self.c_field = Some(c_field);

        self
    }

    pub fn build(self) -> Instruction {
        let mut bits = self.operation.0 as u64;

        if let Some(b_memory) = self.b_memory {
            bits |= ((b_memory.0 as u64) & 0x3) << 6;
        }

        if let Some(c_memory) = self.c_memory {
            bits |= ((c_memory.0 as u64) & 0x3) << 8;
        }

        if let Some(operand_type) = self.operand_type {
            bits |= ((operand_type.0 as u64) & 0x1F) << 10;
        }

        if let Some(d_field) = self.d_field {
            bits |= ((d_field as u64) & 0x3F) << 10;
        }

        if let Some(a_field) = self.a_field {
            bits |= ((a_field as u64) & 0xFFFF) << 16;
        }

        if let Some(b_field) = self.b_field {
            bits |= ((b_field as u64) & 0xFFFF) << 32;
        }

        if let Some(c_field) = self.c_field {
            bits |= ((c_field as u64) & 0xFFFF) << 48;
        }

        Instruction(bits)
    }
}

#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct MemoryKind(pub(super) u8);

impl MemoryKind {
    pub const REGISTER: MemoryKind = MemoryKind(0);
    pub const CONSTANT: MemoryKind = MemoryKind(1);
}

impl Display for MemoryKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            Self::REGISTER => write!(f, "reg"),
            Self::CONSTANT => write!(f, "const"),
            _ => write!(f, "invalid"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_instruction() -> Instruction {
        Instruction::move_with_jump(42, OperandType::U_128, MemoryKind::CONSTANT, 666, 777, true)
    }

    #[test]
    fn decode_operation() {
        let instruction = create_instruction();

        assert_eq!(instruction.operation(), Operation::MOVE);
    }

    #[test]
    fn decode_b_memory() {
        let instruction = create_instruction();

        assert_eq!(instruction.b_memory(), MemoryKind::CONSTANT);
    }

    #[test]
    fn decode_c_memory() {
        let instruction = Instruction::add(
            42,
            OperandType::F_64,
            MemoryKind::CONSTANT,
            1,
            MemoryKind::CONSTANT,
            2,
        );

        assert_eq!(instruction.c_memory(), MemoryKind::CONSTANT);
    }

    #[test]
    fn decode_a_field() {
        let instruction = create_instruction();

        assert_eq!(instruction.a_field(), 42);
    }

    #[test]
    fn decode_b_field() {
        let instruction = create_instruction();

        assert_eq!(instruction.b_field(), 666);
    }

    #[test]
    fn decode_c_field() {
        let instruction = create_instruction();

        assert_eq!(instruction.c_field(), 777);
    }

    #[test]
    fn decode_d_field() {
        let instruction = Instruction::call(u16::MAX, MemoryKind::REGISTER, 666, 30, 63);

        assert_eq!(instruction.d_field(), 63);
    }
}
