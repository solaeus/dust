//! The Dust instruction set.
mod add;
mod call;
mod call_native;
pub mod dispatch_keys;
mod divide;
mod drop;
mod equal;
mod exponent;
mod get;
mod jump;
mod less;
mod less_equal;
mod modulo;
mod r#move;
mod multiply;
mod native_function;
mod negate;
mod operand_type;
mod operation;
mod reference;
mod r#return;
mod set;
mod subtract;
mod test;

pub use add::Add;
pub use call::Call;
pub use call_native::CallNative;
pub use divide::Divide;
pub use drop::Drop;
pub use equal::Equal;
pub use exponent::Exponent;
pub use get::Get;
pub use jump::Jump;
pub use less::Less;
pub use less_equal::LessEqual;
pub use modulo::Modulo;
pub use r#move::Move;
pub use multiply::Multiply;
pub use native_function::NativeFunction;
pub use negate::Negate;
pub use operand_type::{OperandType, RegisterWidth};
pub use operation::Operation;
pub use reference::Reference;
pub use r#return::Return;
pub use set::Set;
pub use subtract::Subtract;
pub use test::Test;

use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display, Formatter};

/// An instruction for the Dust virtual machine.
///
/// Each instruction is 64 bits and uses up to seven distinct fields.
///
/// # Layout
///
/// Bits    | Description
/// ------- | -----------
/// 0..=4   | Operation     ┐
/// 5..=8   | Operand type  ├─ Dispatch key
/// 9..=11  | B memory kind │
/// 12..=14 | C memory kind ┘
/// 15      | Unused
/// 16..=31 | A field
/// 32..=47 | B field
/// 48..=63 | C field
///
/// - Operation: The kind of instruction, which determines how the other fields are interpreted.
/// - Operand type: The data type of the B/C fields.
/// - Memory kinds: Where the operands are stored and how they are interpreted.
/// - A field: Usually the destination register index.
/// - B and C fields: Operands, i.e. indices for registers and constants or encoded values.
/// - Dispatch key: An optimization tool for the VM. The key is a unique identifier for combined
///   fields that can be used for branchless dispatch.
#[derive(Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(C)]
pub struct Instruction(u64);

impl Instruction {
    pub fn no_op() -> Instruction {
        Instruction(0)
    }

    pub fn r#move(destination: u16, operand_type: OperandType, operand: Address) -> Instruction {
        Instruction::from(Move {
            destination,
            operand_type,
            operand,
            jump_distance: 0,
            jump_forward: true,
        })
    }

    pub fn move_with_jump(
        destination: u16,
        operand_type: OperandType,
        operand: Address,
        jump_distance: u16,
        jump_forward: bool,
    ) -> Instruction {
        Instruction::from(Move {
            destination,
            operand_type,
            operand,
            jump_distance,
            jump_forward,
        })
    }

    pub fn reference(destination: u16, source: u16, width: u16) -> Instruction {
        Instruction::from(Reference {
            destination,
            start_register: source,
            end_register: width,
        })
    }

    pub fn get(
        destination: u16,
        operand_type: OperandType,
        base_register: u16,
        index: Address,
    ) -> Instruction {
        Instruction::from(Get {
            destination,
            operand_type,
            base_register,
            index,
        })
    }

    pub fn set(
        base: u16,
        operand_type: OperandType,
        index: Address,
        source: Address,
    ) -> Instruction {
        Instruction::from(Set {
            base,
            operand_type,
            index,
            source,
        })
    }

    pub fn drop(start_register: u16, end_register: u16) -> Instruction {
        Instruction::from(Drop {
            start_register,
            end_register,
        })
    }

    pub fn add(
        destination: u16,
        operand_type: OperandType,
        left_address: Address,
        right_address: Address,
    ) -> Instruction {
        Instruction::from(Add {
            destination,
            operand_type,
            left_address,
            right_address,
        })
    }

    pub fn subtract(
        destination: u16,
        operand_type: OperandType,
        left_address: Address,
        right_address: Address,
    ) -> Instruction {
        Instruction::from(Subtract {
            destination,
            operand_type,
            left_address,
            right_address,
        })
    }

    pub fn multiply(
        destination: u16,
        operand_type: OperandType,
        left_address: Address,
        right_address: Address,
    ) -> Instruction {
        Instruction::from(Multiply {
            destination,
            operand_type,
            left_address,
            right_address,
        })
    }

    pub fn divide(
        destination: u16,
        operand_type: OperandType,
        left_address: Address,
        right_address: Address,
    ) -> Instruction {
        Instruction::from(Divide {
            destination,
            operand_type,
            left_address,
            right_address,
        })
    }

    pub fn modulo(
        destination: u16,
        operand_type: OperandType,
        left_address: Address,
        right_address: Address,
    ) -> Instruction {
        Instruction::from(Modulo {
            destination,
            operand_type,
            left_address,
            right_address,
        })
    }

    pub fn power(
        destination: u16,
        operand_type: OperandType,
        base_address: Address,
        exponent_address: Address,
    ) -> Instruction {
        Instruction::from(Exponent {
            destination,
            operand_type,
            base_address,
            exponent_address,
        })
    }

    pub fn equal(
        comparator: bool,
        operand_type: OperandType,
        left_address: Address,
        right_address: Address,
    ) -> Instruction {
        Instruction::from(Equal {
            comparator,
            operand_type,
            left_address,
            right_address,
        })
    }

    pub fn less(
        comparator: bool,
        operand_type: OperandType,
        left_address: Address,
        right_address: Address,
    ) -> Instruction {
        Instruction::from(Less {
            comparator,
            operand_type,
            left_address,
            right_address,
        })
    }

    pub fn less_equal(
        comparator: bool,
        operand_type: OperandType,
        left_address: Address,
        right_address: Address,
    ) -> Instruction {
        Instruction::from(LessEqual {
            comparator,
            operand_type,
            left_address,
            right_address,
        })
    }

    pub fn test(comparator: bool, operand: Address, jump_distance: u16) -> Instruction {
        Instruction::from(Test {
            comparator,
            operand,
            jump_distance,
        })
    }

    pub fn negate(destination: u16, operand_type: OperandType, operand: Address) -> Instruction {
        Instruction::from(Negate {
            destination,
            operand_type,
            operand,
        })
    }

    pub fn call(destination: u16, callee: Address, arguments_start: u16) -> Instruction {
        Instruction::from(Call {
            destination,
            arguments_start,
            callee,
        })
    }

    pub fn call_native(
        destination: u16,
        function: NativeFunction,
        arguments_start: u16,
    ) -> Instruction {
        Instruction::from(CallNative {
            destination,
            function,
            arguments_start,
        })
    }

    pub fn jump(offset: u16, is_positive: bool) -> Instruction {
        Instruction::from(Jump {
            offset,
            is_positive,
            drop_register_start: 0,
            drop_list_end: 0,
        })
    }

    pub fn jump_with_drops(
        offset: u16,
        is_positive: bool,
        drop_register_start: u16,
        drop_list_end: u16,
    ) -> Instruction {
        Instruction::from(Jump {
            offset,
            is_positive,
            drop_register_start,
            drop_list_end,
        })
    }

    pub fn r#return() -> Instruction {
        Instruction::from(Return)
    }

    pub fn operation(self) -> Operation {
        Operation((self.0 & 0x1F) as u8)
    }

    pub fn operand_type(self) -> OperandType {
        OperandType(((self.0 >> 5) & 0x0F) as u8)
    }

    pub fn b_memory(self) -> Memory {
        Memory(((self.0 >> 9) & 0x07) as u8)
    }

    pub fn c_memory(self) -> Memory {
        Memory(((self.0 >> 12) & 0x07) as u8)
    }

    pub fn a_field(self) -> u64 {
        (self.0 >> 16) & 0xFFFF
    }

    pub fn b_field(self) -> u64 {
        (self.0 >> 32) & 0xFFFF
    }

    pub fn c_field(self) -> u64 {
        (self.0 >> 48) & 0xFFFF
    }

    pub fn b_address(self) -> Address {
        Address {
            memory: self.b_memory(),
            index: self.b_field() as u16,
        }
    }

    pub fn c_address(self) -> Address {
        Address {
            memory: self.c_memory(),
            index: self.c_field() as u16,
        }
    }

    pub const fn dispatch_key(self) -> u64 {
        self.0 & 0x7FFF
    }

    pub fn is_coallescible_with_jump(self, forward: bool) -> bool {
        match self.operation() {
            Operation::DROP => true,
            Operation::MOVE => {
                let Move {
                    jump_distance,
                    jump_forward,
                    ..
                } = Move::from(self);

                jump_distance == 0 || (jump_forward == forward)
            }
            Operation::TEST => {
                let Test { jump_distance, .. } = Test::from(self);

                jump_distance == 0 && forward
            }
            _ => false,
        }
    }

    pub fn disassembly_info(self) -> String {
        let operation = self.operation();

        match operation {
            Operation::NO_OP => String::new(),
            Operation::MOVE => Move::from(self).to_string(),
            Operation::REFERENCE => Reference::from(self).to_string(),
            Operation::GET => Get::from(self).to_string(),
            Operation::SET => Set::from(self).to_string(),
            Operation::DROP => Drop::from(self).to_string(),
            Operation::ADD => Add::from(self).to_string(),
            Operation::SUBTRACT => Subtract::from(self).to_string(),
            Operation::MULTIPLY => Multiply::from(self).to_string(),
            Operation::DIVIDE => Divide::from(self).to_string(),
            Operation::MODULO => Modulo::from(self).to_string(),
            Operation::EXPONENT => Exponent::from(self).to_string(),
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
        write!(f, "{} {}", self.operation(), self.disassembly_info())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstructionBuilder {
    operation: Operation,
    operand_type: OperandType,
    b_memory: Memory,
    c_memory: Memory,
    a_field: u16,
    b_field: u16,
    c_field: u16,
}

impl InstructionBuilder {
    pub const fn new(operation: Operation) -> Self {
        Self {
            operation,
            operand_type: OperandType(0),
            b_memory: Memory(0),
            c_memory: Memory(0),
            a_field: 0,
            b_field: 0,
            c_field: 0,
        }
    }

    pub const fn b_memory(mut self, memory: Memory) -> Self {
        self.b_memory = memory;

        self
    }

    pub const fn c_memory(mut self, memory: Memory) -> Self {
        self.c_memory = memory;

        self
    }

    pub const fn operand_type(mut self, operand_type: OperandType) -> Self {
        self.operand_type = operand_type;

        self
    }

    pub const fn a_field(mut self, a_field: u16) -> Self {
        self.a_field = a_field;

        self
    }

    pub const fn b_field(mut self, b_field: u16) -> Self {
        self.b_field = b_field;

        self
    }

    pub const fn c_field(mut self, c_field: u16) -> Self {
        self.c_field = c_field;

        self
    }

    pub const fn b_address(mut self, operand: Address) -> Self {
        self.b_memory = operand.memory;
        self.b_field = operand.index;

        self
    }

    pub const fn c_address(mut self, operand: Address) -> Self {
        self.c_memory = operand.memory;
        self.c_field = operand.index;

        self
    }

    pub const fn build(self) -> Instruction {
        let mut bits = self.operation.0 as u64;

        bits |= (self.operand_type.0 as u64) << 5;
        bits |= (self.b_memory.0 as u64) << 9;
        bits |= (self.c_memory.0 as u64) << 12;
        bits |= (self.a_field as u64) << 16;
        bits |= (self.b_field as u64) << 32;
        bits |= (self.c_field as u64) << 48;

        Instruction(bits)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Memory(u8);

impl Memory {
    /// Indicates that the field is unused and should be ignored.
    pub const EMPTY: Memory = Memory(0);

    /// Represents the index of a VM register in the current stack frame.
    pub const REGISTER: Memory = Memory(1);

    /// Represents the index of a reference to another register.
    pub const REFERENCE: Memory = Memory(2);

    /// Represents an encoded value that is stored directly in the instruction.
    pub const ENCODED: Memory = Memory(3);

    /// Represents the index of a value in the constants table.
    pub const CONSTANT: Memory = Memory(4);
}

impl Display for Memory {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            Self::EMPTY => write!(f, "empty"),
            Self::REGISTER => write!(f, "reg"),
            Self::REFERENCE => write!(f, "ref"),
            Self::ENCODED => write!(f, "enc"),
            Self::CONSTANT => write!(f, "const"),
            _ => write!(f, "invalid"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Address {
    pub memory: Memory,
    pub index: u16,
}

impl Address {
    pub fn new(memory: Memory, index: u16) -> Self {
        Self { memory, index }
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}_{}", self.memory, self.index)
    }
}

#[cfg(test)]
mod tests {
    use crate::instruction::*;

    fn create_instruction() -> Instruction {
        Instruction::move_with_jump(
            42,
            OperandType::U_128,
            Address {
                memory: Memory::CONSTANT,
                index: 666,
            },
            777,
            true,
        )
    }

    #[test]
    fn decode_operation() {
        let instruction = create_instruction();

        assert_eq!(instruction.operation(), Operation::MOVE);
    }

    #[test]
    fn decode_operand_type() {
        let instruction = create_instruction();

        assert_eq!(instruction.operand_type(), OperandType::U_128);
    }

    #[test]
    fn decode_b_memory() {
        let instruction = create_instruction();

        assert_eq!(instruction.b_memory(), Memory::CONSTANT);
    }

    #[test]
    fn decode_c_memory() {
        let instruction = Instruction::add(
            42,
            OperandType::F_64,
            Address {
                memory: Memory::CONSTANT,
                index: 1,
            },
            Address {
                memory: Memory::CONSTANT,
                index: 2,
            },
        );

        assert_eq!(instruction.c_memory(), Memory::CONSTANT);
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
}
