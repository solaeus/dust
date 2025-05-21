//! The Dust instruction set.
//!
//! Each instruction is 64 bits and uses up to seven distinct fields.
//!
//! # Layout
//!
//! Bits  | Description
//! ----- | -----------
//! 0-4   | Operation
//! 5     | Flag indicating whether the A field is a register or memory index
//! 6-10  | Operand address kind (for the B field)
//! 11-15 | Operand address kind (for the C field)
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
//! Use the [`Destination`][], [`Address`][] and [`AddressKind`][] types when creating instructions.
//! In addition to being easy to read and write, this ensures that the instruction has the correct
//! flags to represent the operands.
//!
//! ```
//! # use dust_lang::instruction::{Add, Address, AddressKind, Destination, Instruction};
//! // Add the integers at M_INT_4 and M_INT_6 then store the result in M_INT_42.
//! let add_1 = Instruction::add(
//!     Destination::memory(42),
//!     Address::new(4, AddressKind::INTEGER_MEMORY),
//!     Address::new(6, AddressKind::INTEGER_MEMORY),
//! );
//! let add_2 = Instruction::from(Add {
//!     destination: Destination {
//!         index: 42,
//!         is_register: false,
//!     },
//!     left: Address {
//!         index: 4,
//!         kind: AddressKind::INTEGER_MEMORY,
//!     },
//!     right: Address {
//!         index: 6,
//!         kind: AddressKind::INTEGER_MEMORY,
//!     },
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
//! # use dust_lang::instruction::{Add, Address, AddressKind, Destination, Instruction, Operation};
//! # let mystery_instruction = Instruction::add(
//! #     Destination::memory(0),
//! #     Address::new(0, AddressKind::INTEGER_MEMORY),
//! #     Address::new(1, AddressKind::INTEGER_CONSTANT),
//! # );
//! // Let's read an instruction and see if it performs addition-assignment,
//! // like in one of the following examples:
//! //   - `a += 2`
//! //   - `a = a + 2`
//! //   - `a = 2 + a`
//! let operation = mystery_instruction.operation();
//! let is_add_assign = operation == Operation::ADD && {
//!     let Add { destination, left, right } = Add::from(&mystery_instruction);
//!
//!     destination.index == left.index || destination.index == right.index
//! };
//!
//! assert!(is_add_assign);
//! ```
mod add;
mod address;
mod call;
mod call_native;
mod close;
mod destination;
mod divide;
mod equal;
mod jump;
mod less;
mod less_equal;
mod load_constant;
mod load_encoded;
mod load_function;
mod load_list;
mod modulo;
mod r#move;
mod multiply;
mod negate;
mod not;
mod operation;
mod r#return;
mod subtract;
mod test;
mod test_set;

pub use add::Add;
pub use address::{Address, AddressKind};
pub use call::Call;
pub use call_native::CallNative;
pub use close::Close;
pub use destination::Destination;
pub use divide::Divide;
pub use equal::Equal;
pub use jump::Jump;
pub use less::Less;
pub use less_equal::LessEqual;
pub use load_constant::LoadConstant;
pub use load_encoded::LoadEncoded;
pub use load_function::LoadFunction;
pub use load_list::LoadList;
pub use modulo::Modulo;
pub use r#move::Move;
pub use multiply::Multiply;
pub use negate::Negate;
pub use not::Not;
pub use operation::Operation;
pub use r#return::Return;
pub use subtract::Subtract;
pub use test::Test;
pub use test_set::TestSet;

use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display, Formatter};

use crate::{NativeFunction, r#type::TypeKind};

/// An instruction for the Dust virtual machine.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Instruction(u64);

impl Instruction {
    pub fn destination(&self) -> Destination {
        Destination {
            index: self.a_field(),
            is_register: self.a_is_register(),
        }
    }

    pub fn destination_as_address(&self) -> Address {
        self.destination().as_address(self.operand_type())
    }

    pub fn b_address(&self) -> Address {
        Address {
            index: self.b_field(),
            kind: self.b_kind(),
        }
    }

    pub fn c_address(&self) -> Address {
        Address {
            index: self.c_field(),
            kind: self.c_kind(),
        }
    }

    pub fn operation(&self) -> Operation {
        let bits_0_to_4 = (self.0 & 0x1F) as u8;

        Operation(bits_0_to_4)
    }

    pub fn a_is_register(&self) -> bool {
        let bit_5 = self.0 & (1 << 5);

        bit_5 != 0
    }

    pub fn b_kind(&self) -> AddressKind {
        let bits_6_to_10 = (self.0 >> 6) & 0x1F;

        AddressKind(bits_6_to_10 as u8)
    }

    pub fn c_kind(&self) -> AddressKind {
        let bits_11_to_15 = (self.0 >> 11) & 0x1F;

        AddressKind(bits_11_to_15 as u8)
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

    pub fn set_a_field(&mut self, bits: u16) {
        let mut fields = InstructionFields::from(&*self);
        fields.a_field = bits;
        *self = fields.build();
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

    pub fn set_destination(&mut self, address: Address) {
        let mut fields = InstructionFields::from(&*self);
        fields.a_field = address.index;
        fields.a_is_register = address.is_register();
        *self = fields.build();
    }

    pub fn set_b_address(&mut self, address: Address) {
        let mut fields = InstructionFields::from(&*self);
        fields.b_field = address.index;
        fields.b_kind = address.kind;
        *self = fields.build();
    }

    pub fn set_c_address(&mut self, address: Address) {
        let mut fields = InstructionFields::from(&*self);
        fields.c_field = address.index;
        fields.c_kind = address.kind;
        *self = fields.build();
    }

    pub fn operand_type(&self) -> TypeKind {
        match self.operation() {
            Operation::NO_OP | Operation::CLOSE => TypeKind::None,
            Operation::LOAD_LIST => TypeKind::List,
            _ => self.b_kind().r#type(),
        }
    }

    pub fn as_address(&self) -> Address {
        match self.operation() {
            Operation::MOVE => {
                let Move { operand, .. } = Move::from(self);

                operand
            }
            Operation::LOAD_ENCODED => {
                let LoadEncoded {
                    destination,
                    r#type,
                    ..
                } = LoadEncoded::from(self);

                let kind = if destination.is_register {
                    match r#type {
                        AddressKind::BOOLEAN_MEMORY => AddressKind::BOOLEAN_REGISTER,
                        AddressKind::BYTE_MEMORY => AddressKind::BYTE_REGISTER,
                        _ => unreachable!(),
                    }
                } else {
                    r#type
                };

                Address {
                    index: destination.index,
                    kind,
                }
            }
            Operation::LOAD_CONSTANT => {
                let LoadConstant { constant, .. } = LoadConstant::from(self);

                constant
            }
            Operation::LOAD_LIST => {
                let LoadList { destination, .. } = LoadList::from(self);
                let kind = if destination.is_register {
                    AddressKind::LIST_REGISTER
                } else {
                    AddressKind::LIST_MEMORY
                };

                Address {
                    index: destination.index,
                    kind,
                }
            }
            Operation::LOAD_FUNCTION => {
                let LoadFunction { destination, .. } = LoadFunction::from(self);
                let kind = if destination.is_register {
                    AddressKind::FUNCTION_REGISTER
                } else {
                    AddressKind::FUNCTION_MEMORY
                };

                Address {
                    index: destination.index,
                    kind,
                }
            }
            Operation::ADD => {
                let Add {
                    destination, left, ..
                } = Add::from(self);
                let left_type = left.r#type();
                let destination_type = match left_type {
                    TypeKind::Character => TypeKind::String,
                    _ => left_type,
                };

                destination.as_address(destination_type)
            }
            Operation::SUBTRACT => {
                let Subtract {
                    destination, left, ..
                } = Subtract::from(self);

                destination.as_address(left.r#type())
            }
            Operation::MULTIPLY => {
                let Multiply {
                    destination, left, ..
                } = Multiply::from(self);

                destination.as_address(left.r#type())
            }
            Operation::DIVIDE => {
                let Divide {
                    destination, left, ..
                } = Divide::from(self);

                destination.as_address(left.r#type())
            }
            Operation::MODULO => {
                let Modulo {
                    destination, left, ..
                } = Modulo::from(self);

                destination.as_address(left.r#type())
            }
            Operation::NOT => {
                let Not { destination, .. } = Not::from(*self);

                destination.as_address(TypeKind::Boolean)
            }
            Operation::CALL => {
                let Call {
                    destination,
                    argument_list_index_and_return_type,
                    ..
                } = Call::from(self);
                let return_type = argument_list_index_and_return_type.kind.r#type();

                destination.as_address(return_type)
            }
            Operation::NO_OP => Address::default(),
            unsupported => todo!("Support {unsupported}"),
        }
    }

    pub fn no_op() -> Instruction {
        Instruction(Operation::NO_OP.0 as u64)
    }

    pub fn r#move(destination: Destination, to: Address) -> Instruction {
        Instruction::from(Move {
            destination,
            operand: to,
        })
    }

    pub fn close(from: Address, to: Address) -> Instruction {
        Instruction::from(Close { from, to })
    }

    pub fn load_encoded(
        destination: Destination,
        value: u16,
        r#type: AddressKind,
        jump_next: bool,
    ) -> Instruction {
        Instruction::from(LoadEncoded {
            destination,
            value,
            r#type,
            jump_next,
        })
    }

    pub fn load_constant(
        destination: Destination,
        constant: Address,
        jump_next: bool,
    ) -> Instruction {
        Instruction::from(LoadConstant {
            destination,
            constant,
            jump_next,
        })
    }

    pub fn load_function(
        destination: Destination,
        prototype: Address,
        jump_next: bool,
    ) -> Instruction {
        Instruction::from(LoadFunction {
            destination,
            prototype,
            jump_next,
        })
    }

    pub fn load_list(
        destination: Destination,
        start: Address,
        end: u16,
        jump_next: bool,
    ) -> Instruction {
        Instruction::from(LoadList {
            destination,
            start,
            end,
            jump_next,
        })
    }

    pub fn add(destination: Destination, left: Address, right: Address) -> Instruction {
        Instruction::from(Add {
            destination,
            left,
            right,
        })
    }

    pub fn subtract(destination: Destination, left: Address, right: Address) -> Instruction {
        Instruction::from(Subtract {
            destination,
            left,
            right,
        })
    }

    pub fn multiply(destination: Destination, left: Address, right: Address) -> Instruction {
        Instruction::from(Multiply {
            destination,
            left,
            right,
        })
    }

    pub fn divide(destination: Destination, left: Address, right: Address) -> Instruction {
        Instruction::from(Divide {
            destination,
            left,
            right,
        })
    }

    pub fn modulo(destination: Destination, left: Address, right: Address) -> Instruction {
        Instruction::from(Modulo {
            destination,
            left,
            right,
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

    pub fn negate(destination: Destination, operand: Address) -> Instruction {
        Instruction::from(Negate {
            destination,
            operand,
        })
    }

    pub fn not(destination: Destination, operand: Address) -> Instruction {
        Instruction::from(Not {
            destination,
            operand,
        })
    }

    pub fn test(operand: Address, comparator: bool) -> Instruction {
        Instruction::from(Test {
            operand,
            comparator,
        })
    }

    pub fn test_set(destination: Destination, operand: Address, value: bool) -> Instruction {
        Instruction::from(TestSet {
            destination,
            operand,
            comparator: value,
        })
    }

    pub fn jump(offset: u16, is_positive: bool) -> Instruction {
        Instruction::from(Jump {
            offset,
            is_positive,
        })
    }

    pub fn call(
        destination: Destination,
        function: Address,
        argument_list_index_and_return_type: Address,
    ) -> Instruction {
        Instruction::from(Call {
            destination,
            function,
            argument_list_index_and_return_type,
        })
    }

    pub fn call_native(
        destination: Destination,
        function: NativeFunction,
        argument_list_index: u16,
    ) -> Instruction {
        Instruction::from(CallNative {
            destination,
            function,
            argument_list_index,
        })
    }

    pub fn r#return(should_return_value: bool, return_address: Address) -> Instruction {
        Instruction::from(Return {
            should_return_value,
            return_value_address: return_address,
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
            Operation::MOVE
            | Operation::LOAD_ENCODED
            | Operation::LOAD_CONSTANT
            | Operation::LOAD_FUNCTION
            | Operation::LOAD_LIST
            | Operation::ADD
            | Operation::SUBTRACT
            | Operation::MULTIPLY
            | Operation::DIVIDE
            | Operation::MODULO
            | Operation::NEGATE
            | Operation::NOT
            | Operation::CALL => true,
            Operation::CALL_NATIVE => {
                let function = NativeFunction::from(self.b_field());

                function.returns_value()
            }
            Operation::CLOSE
            | Operation::EQUAL
            | Operation::LESS
            | Operation::LESS_EQUAL
            | Operation::TEST
            | Operation::TEST_SET
            | Operation::JUMP
            | Operation::RETURN
            | Operation::NO_OP => false,
            unknown => panic!("Unknown operation: {}", unknown.0),
        }
    }

    pub fn disassembly_info(&self) -> String {
        let operation = self.operation();

        match operation {
            Operation::MOVE => Move::from(self).to_string(),
            Operation::CLOSE => Close::from(self).to_string(),
            Operation::LOAD_ENCODED => LoadEncoded::from(self).to_string(),
            Operation::LOAD_CONSTANT => LoadConstant::from(self).to_string(),
            Operation::LOAD_FUNCTION => LoadFunction::from(self).to_string(),
            Operation::LOAD_LIST => LoadList::from(self).to_string(),
            Operation::ADD => Add::from(self).to_string(),
            Operation::SUBTRACT => Subtract::from(self).to_string(),
            Operation::MULTIPLY => Multiply::from(self).to_string(),
            Operation::DIVIDE => Divide::from(self).to_string(),
            Operation::MODULO => Modulo::from(self).to_string(),
            Operation::NEGATE => Negate::from(*self).to_string(),
            Operation::NOT => Not::from(*self).to_string(),
            Operation::EQUAL => Equal::from(self).to_string(),
            Operation::LESS => Less::from(self).to_string(),
            Operation::LESS_EQUAL => LessEqual::from(self).to_string(),
            Operation::TEST => Test::from(self).to_string(),
            Operation::TEST_SET => TestSet::from(*self).to_string(),
            Operation::CALL => Call::from(self).to_string(),
            Operation::CALL_NATIVE => CallNative::from(*self).to_string(),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstructionFields {
    pub operation: Operation,
    pub a_field: u16,
    pub b_field: u16,
    pub c_field: u16,
    pub a_is_register: bool,
    pub b_kind: AddressKind,
    pub c_kind: AddressKind,
}

impl InstructionFields {
    pub fn build(self) -> Instruction {
        let mut bits = 0_u64;

        bits |= self.operation.0 as u64;
        bits |= (self.a_is_register as u64) << 5;
        bits |= (self.b_kind.0 as u64) << 6;
        bits |= (self.c_kind.0 as u64) << 11;
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
            a_field: instruction.a_field(),
            b_field: instruction.b_field(),
            c_field: instruction.c_field(),
            a_is_register: instruction.a_is_register(),
            b_kind: instruction.b_kind(),
            c_kind: instruction.c_kind(),
        }
    }
}

impl Default for InstructionFields {
    fn default() -> Self {
        InstructionFields {
            operation: Operation::NO_OP,
            a_field: 0,
            b_field: 0,
            c_field: 0,
            a_is_register: false,
            b_kind: AddressKind(0),
            c_kind: AddressKind(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_operation() {
        let instruction = Instruction::add(
            Destination::memory(42),
            Address {
                index: 1,
                kind: AddressKind::CHARACTER_MEMORY,
            },
            Address {
                index: 2,
                kind: AddressKind::CHARACTER_MEMORY,
            },
        );

        assert_eq!(Operation::ADD, instruction.operation());
    }

    #[test]
    fn decode_a_is_register() {
        let instruction = Instruction::add(
            Destination::register(42),
            Address {
                index: 1,
                kind: AddressKind::CHARACTER_MEMORY,
            },
            Address {
                index: 2,
                kind: AddressKind::CHARACTER_MEMORY,
            },
        );

        assert!(instruction.a_is_register());
    }

    #[test]
    fn decode_b_kind() {
        let instruction = Instruction::add(
            Destination::register(42),
            Address {
                index: 1,
                kind: AddressKind::CHARACTER_MEMORY,
            },
            Address {
                index: 2,
                kind: AddressKind::CHARACTER_MEMORY,
            },
        );

        assert_eq!(AddressKind::CHARACTER_MEMORY, instruction.b_kind());
    }

    #[test]
    fn decode_c_kind() {
        let instruction = Instruction::add(
            Destination::register(42),
            Address {
                index: 1,
                kind: AddressKind::CHARACTER_MEMORY,
            },
            Address {
                index: 2,
                kind: AddressKind::STRING_MEMORY,
            },
        );

        assert_eq!(AddressKind::STRING_MEMORY, instruction.c_kind());
    }

    #[test]
    fn decode_a_field() {
        let instruction = Instruction::add(
            Destination::register(42),
            Address {
                index: 1,
                kind: AddressKind::CHARACTER_MEMORY,
            },
            Address {
                index: 2,
                kind: AddressKind::CHARACTER_MEMORY,
            },
        );

        assert_eq!(42, instruction.a_field());
    }

    #[test]
    fn decode_b_field() {
        let instruction = Instruction::add(
            Destination::register(42),
            Address {
                index: 1,
                kind: AddressKind::CHARACTER_MEMORY,
            },
            Address {
                index: 2,
                kind: AddressKind::CHARACTER_MEMORY,
            },
        );

        assert_eq!(1, instruction.b_field());
    }

    #[test]
    fn decode_c_field() {
        let instruction = Instruction::add(
            Destination::register(42),
            Address {
                index: 1,
                kind: AddressKind::CHARACTER_MEMORY,
            },
            Address {
                index: 2,
                kind: AddressKind::CHARACTER_MEMORY,
            },
        );

        assert_eq!(2, instruction.c_field());
    }
}
