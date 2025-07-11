//! The Dust instruction set.
//!
//! Each instruction is 64 bits and uses up to eight distinct fields.
//!
//! # Layout
//!
//! Bits  | Description
//! ----- | -----------
//! 0-4   | Operation
//! 5-6   | Memory kind (for the A field)
//! 7-8   | Memory kind (for the B field)
//! 9-10  | Memory kind (for the C field)
//! 11-15 | Operand type info
//! 16-31 | A field (unsigned 16-bit integer), usually the destination index
//! 32-47 | B field (unsigned 16-bit integer), usually an operand index
//! 48-63 | C field (unsigned 16-bit integer), usually an operand index
//!
//! # Creating Instructions
//!
//! For each operation, there are two ways to create an instruction:
//!
//! - Use the associated function on `Instruction`
//! - Use the corresponding struct and call `Instruction::from`
//!
//! Both produce the same result, but the first is usually more concise. The structs are more useful
//! when reading instructions, as shown below.
//!
//! ```
//! # use dust_lang::instruction::{Add, Address, Instruction, MemoryKind, OperandType};
//! // Add the integers at int_h_4 and int_h_6 then store the result in int_h_42.
//! let add_1 = Instruction::add(
//!     Address::heap(42),
//!     Address::heap(4),
//!     Address::heap(6),
//!     OperandType::INTEGER,
//! );
//! let add_2 = Instruction::from(Add {
//!     destination: Address {
//!         index: 42,
//!         memory: MemoryKind::HEAP,
//!     },
//!     left: Address {
//!         index: 4,
//!         memory: MemoryKind::HEAP,
//!     },
//!     right: Address {
//!         index: 6,
//!         memory: MemoryKind::HEAP,
//!     },
//!     r#type: OperandType::INTEGER,
//! });
//!
//! assert_eq!(add_1, add_2);
//! ```
//!
//! # Reading Instructions
//!
//! To read an instruction, check its operation with [`Instruction::operation`], then convert the
//! instruction to the struct that corresponds to that operation. Like the example above, this
//! removes the burden of dealing with the instruction bit fields directly.
//!
//! ```
//! # use dust_lang::instruction::{Add, Address, Instruction, MemoryKind, OperandType, Operation};
//! # let mystery_instruction = Instruction::add(
//! #     Address::heap(0),
//! #     Address::heap(0),
//! #     Address::heap(1),
//! #     OperandType::INTEGER,
//! # );
//! // Let's read an instruction and see if it performs addition-assignment on integers, like in one
//! // of the following examples:
//! //   - `a += 2`
//! //   - `a = a + 2`
//! //   - `a = 2 + a`
//! let operation = mystery_instruction.operation();
//! let is_add_assign_integers = operation == Operation::ADD && {
//!     let Add { destination, left, right, r#type } = Add::from(&mystery_instruction);
//!
//!     r#type == OperandType::INTEGER
//!         && (destination == left || destination == right)
//! };
//!
//! assert!(is_add_assign_integers);
//! ```
mod add;
mod address;
mod call;
mod call_native;
mod divide;
mod equal;
mod jump;
mod less;
mod less_equal;
mod list;
mod load;
mod memory_kind;
mod modulo;
mod multiply;
mod negate;
mod operand_type;
mod operation;
mod r#return;
mod subtract;
mod test;

pub use add::Add;
pub use address::Address;
pub use call::Call;
pub use call_native::CallNative;
pub use divide::Divide;
pub use equal::Equal;
pub use jump::Jump;
pub use less::Less;
pub use less_equal::LessEqual;
pub use list::List;
pub use load::Load;
pub use memory_kind::MemoryKind;
pub use modulo::Modulo;
pub use multiply::Multiply;
pub use negate::Negate;
pub use operand_type::OperandType;
pub use operation::Operation;
pub use r#return::Return;
pub use subtract::Subtract;
pub use test::Test;

use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display, Formatter};

use crate::{NativeFunction, StrippedChunk};

/// An instruction for the Dust virtual machine.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Instruction(u64);

impl Instruction {
    pub fn destination(&self) -> Address {
        Address {
            index: self.a_field(),
            memory: self.a_memory_kind(),
        }
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

    pub fn operation(&self) -> Operation {
        let bits_0_to_4 = (self.0 & 0x1F) as u8;

        Operation(bits_0_to_4)
    }

    pub fn a_memory_kind(&self) -> MemoryKind {
        let bits_5_to_6 = (self.0 >> 5) & 0x3;

        MemoryKind(bits_5_to_6 as u8)
    }

    pub fn b_memory_kind(&self) -> MemoryKind {
        let bits_7_to_8 = (self.0 >> 7) & 0x3;

        MemoryKind(bits_7_to_8 as u8)
    }

    pub fn c_memory_kind(&self) -> MemoryKind {
        let bits_9_to_10 = (self.0 >> 9) & 0x3;

        MemoryKind(bits_9_to_10 as u8)
    }

    pub fn operand_type(&self) -> OperandType {
        let bits_11_to_15 = (self.0 >> 11) & 0x1F;

        OperandType(bits_11_to_15 as u8)
    }

    pub fn a_field(&self) -> usize {
        let bits_16_to_31 = (self.0 >> 16) & 0xFFFF;

        bits_16_to_31 as usize
    }

    pub fn b_field(&self) -> usize {
        let bits_32_to_47 = (self.0 >> 32) & 0xFFFF;

        bits_32_to_47 as usize
    }

    pub fn c_field(&self) -> usize {
        let bits_48_to_63 = (self.0 >> 48) & 0xFFFF;

        bits_48_to_63 as usize
    }

    pub fn set_a_field(&mut self, bits: usize) {
        let mut fields = InstructionFields::from(&*self);
        fields.a_field = bits;
        *self = fields.build();
    }

    pub fn set_b_field(&mut self, bits: usize) {
        let mut fields = InstructionFields::from(&*self);
        fields.b_field = bits;
        *self = fields.build();
    }

    pub fn set_destination(&mut self, address: Address) {
        let mut fields = InstructionFields::from(&*self);
        fields.a_field = address.index;
        fields.a_memory_kind = address.memory;
        *self = fields.build();
    }

    pub fn set_b_address(&mut self, address: Address) {
        let mut fields = InstructionFields::from(&*self);
        fields.b_field = address.index;
        fields.b_memory_kind = address.memory;
        *self = fields.build();
    }

    pub fn set_c_address(&mut self, address: Address) {
        let mut fields = InstructionFields::from(&*self);
        fields.c_field = address.index;
        fields.c_memory_kind = address.memory;
        *self = fields.build();
    }

    pub fn no_op() -> Instruction {
        Instruction(0)
    }

    pub fn load(
        destination: Address,
        operand: Address,
        r#type: OperandType,
        jump_next: bool,
    ) -> Instruction {
        Instruction::from(Load {
            destination,
            operand,
            r#type,
            jump_next,
        })
    }

    pub fn list(
        destination: Address,
        start: Address,
        end: Address,
        r#type: OperandType,
    ) -> Instruction {
        Instruction::from(List {
            destination,
            start,
            end,
            r#type,
        })
    }

    pub fn add(
        destination: Address,
        left: Address,
        right: Address,
        r#type: OperandType,
    ) -> Instruction {
        Instruction::from(Add {
            destination,
            left,
            right,
            r#type,
        })
    }

    pub fn subtract(
        destination: Address,
        left: Address,
        right: Address,
        r#type: OperandType,
    ) -> Instruction {
        Instruction::from(Subtract {
            destination,
            left,
            right,
            r#type,
        })
    }

    pub fn multiply(
        destination: Address,
        left: Address,
        right: Address,
        r#type: OperandType,
    ) -> Instruction {
        Instruction::from(Multiply {
            destination,
            left,
            right,
            r#type,
        })
    }

    pub fn divide(
        destination: Address,
        left: Address,
        right: Address,
        r#type: OperandType,
    ) -> Instruction {
        Instruction::from(Divide {
            destination,
            left,
            right,
            r#type,
        })
    }

    pub fn modulo(
        destination: Address,
        left: Address,
        right: Address,
        r#type: OperandType,
    ) -> Instruction {
        Instruction::from(Modulo {
            destination,
            left,
            right,
            r#type,
        })
    }

    pub fn equal(
        comparator: bool,
        left: Address,
        right: Address,
        r#type: OperandType,
    ) -> Instruction {
        Instruction::from(Equal {
            comparator,
            left,
            right,
            r#type,
        })
    }

    pub fn less(
        comparator: bool,
        left: Address,
        right: Address,
        r#type: OperandType,
    ) -> Instruction {
        Instruction::from(Less {
            comparator,
            left,
            right,
            r#type,
        })
    }

    pub fn less_equal(
        comparator: bool,
        left: Address,
        right: Address,
        r#type: OperandType,
    ) -> Instruction {
        Instruction::from(LessEqual {
            comparator,
            left,
            right,
            r#type,
        })
    }

    pub fn test(operand: Address, comparator: bool) -> Instruction {
        Instruction::from(Test {
            operand,
            comparator,
        })
    }

    pub fn negate(destination: Address, operand: Address, r#type: OperandType) -> Instruction {
        Instruction::from(Negate {
            destination,
            operand,
            r#type,
        })
    }

    pub fn jump(offset: usize, is_positive: bool) -> Instruction {
        Instruction::from(Jump {
            offset,
            is_positive,
        })
    }

    pub fn call(
        destination: Address,
        function: Address,
        argument_count: usize,
        return_type: OperandType,
    ) -> Instruction {
        Instruction::from(Call {
            destination,
            function,
            argument_count,
            return_type,
        })
    }

    pub fn call_native<C>(
        destination: Address,
        function: NativeFunction<C>,
        argument_count: usize,
    ) -> Instruction {
        Instruction::from(CallNative {
            destination,
            function,
            argument_count,
        })
    }

    pub fn r#return(
        should_return_value: bool,
        return_address: Address,
        r#type: OperandType,
    ) -> Instruction {
        Instruction::from(Return {
            should_return_value,
            return_value_address: return_address,
            r#type,
        })
    }

    pub fn is_math(&self) -> bool {
        self.operation().is_math()
    }

    pub fn is_comparison(&self) -> bool {
        self.operation().is_comparison()
    }

    pub fn yields_value(&self) -> bool {
        match self.operation() {
            Operation::LOAD
            | Operation::LIST
            | Operation::ADD
            | Operation::SUBTRACT
            | Operation::MULTIPLY
            | Operation::DIVIDE
            | Operation::MODULO
            | Operation::NEGATE
            | Operation::CALL => true,
            Operation::CALL_NATIVE => {
                let function = NativeFunction::<StrippedChunk>::from_index(self.b_field());

                function.returns_value()
            }
            Operation::EQUAL
            | Operation::LESS
            | Operation::LESS_EQUAL
            | Operation::TEST
            | Operation::JUMP
            | Operation::RETURN
            | Operation::NO_OP => false,
            unknown => panic!("Unknown operation: {}", unknown.0),
        }
    }

    pub fn disassembly_info(self) -> String {
        let operation = self.operation();

        match operation {
            Operation::LOAD => Load::from(self).to_string(),
            Operation::LIST => List::from(self).to_string(),
            Operation::ADD => Add::from(self).to_string(),
            Operation::SUBTRACT => Subtract::from(self).to_string(),
            Operation::MULTIPLY => Multiply::from(self).to_string(),
            Operation::DIVIDE => Divide::from(self).to_string(),
            Operation::MODULO => Modulo::from(self).to_string(),
            Operation::NEGATE => Negate::from(self).to_string(),
            Operation::EQUAL => Equal::from(self).to_string(),
            Operation::LESS => Less::from(self).to_string(),
            Operation::LESS_EQUAL => LessEqual::from(self).to_string(),
            Operation::TEST => Test::from(self).to_string(),
            Operation::CALL => Call::from(self).to_string(),
            Operation::CALL_NATIVE => CallNative::<StrippedChunk>::from(self).to_string(),
            Operation::JUMP => Jump::from(self).to_string(),
            Operation::RETURN => Return::from(self).to_string(),
            Operation::NO_OP => String::new(),
            unknown => panic!("Unknown operation: {}", unknown.0),
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
    pub a_memory_kind: MemoryKind,
    pub b_memory_kind: MemoryKind,
    pub c_memory_kind: MemoryKind,
    pub operand_type: OperandType,
    pub a_field: usize,
    pub b_field: usize,
    pub c_field: usize,
}

impl InstructionFields {
    pub fn build(self) -> Instruction {
        let mut bits = 0_u64;

        bits |= self.operation.0 as u64;
        bits |= (self.a_memory_kind.0 as u64) << 5;
        bits |= (self.b_memory_kind.0 as u64) << 7;
        bits |= (self.c_memory_kind.0 as u64) << 9;
        bits |= (self.operand_type.0 as u64) << 11;
        bits |= (self.a_field as u64) << 16;
        bits |= (self.b_field as u64) << 32;
        bits |= (self.c_field as u64) << 48;

        Instruction(bits)
    }
}

impl From<&Instruction> for InstructionFields {
    fn from(instruction: &Instruction) -> Self {
        InstructionFields {
            operation: instruction.operation(),
            a_memory_kind: instruction.a_memory_kind(),
            b_memory_kind: instruction.b_memory_kind(),
            c_memory_kind: instruction.c_memory_kind(),
            operand_type: instruction.operand_type(),
            a_field: instruction.a_field(),
            b_field: instruction.b_field(),
            c_field: instruction.c_field(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_instruction() -> Instruction {
        Instruction::add(
            Address::register(42),
            Address::register(1),
            Address::cell(2),
            OperandType::INTEGER,
        )
    }

    #[test]
    fn decode_operation() {
        let instruction = create_instruction();

        assert_eq!(instruction.operation(), Operation::ADD);
    }

    #[test]
    fn decode_a_memory() {
        let instruction = create_instruction();

        assert_eq!(instruction.a_memory_kind(), MemoryKind::REGISTER);
    }

    #[test]
    fn decode_b_memory() {
        let instruction = create_instruction();

        assert_eq!(instruction.b_memory_kind(), MemoryKind::REGISTER);
    }

    #[test]
    fn decode_c_memory() {
        let instruction = create_instruction();

        assert_eq!(instruction.c_memory_kind(), MemoryKind::CELL);
    }

    #[test]
    fn decode_operand_type() {
        let instruction = create_instruction();

        assert_eq!(instruction.operand_type(), OperandType::INTEGER);
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
}
