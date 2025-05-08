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
//! Use the [`Operand`][] type when creating instructions. In addition to being easy to read and
//! write, this ensures that the instruction has the correct flags to represent the operands.
//!
//! ```
//! # use dust_lang::instruction::{Instruction, Add, Operand, TypeCode};
//! let add_1 = Instruction::add(
//!     0,
//!     Operand::Register(1, TypeCode::INTEGER),
//!     Operand::Constant(2, TypeCode::INTEGER)
//! );
//! let add_2 = Instruction::from(Add {
//!     destination: 0,
//!     left: Operand::Register(1, TypeCode::INTEGER),
//!     right: Operand::Constant(2, TypeCode::INTEGER),
//! });
//!
//! assert_eq!(add_1, add_2);
//! ```
//!
//! # Reading Instructions
//!
//! To read an instruction, check its operation with [`Instruction::operation`], then convert the
//! instruction to the struct that corresponds to that operation. Like the example above, this
//! removes the burden of dealing with the options directly and automatically casts the A, B, C and
//! D fields as `u16`, `bool` or `Operand` values.
//!
//! ```
//! # use dust_lang::instruction::{Instruction, Add, Operand, Operation, TypeCode};
//! # let mystery_instruction = Instruction::add(
//! #     1,
//! #     Operand::Register(1, TypeCode::INTEGER),
//! #     Operand::Constant(2, TypeCode::INTEGER)
//! # );
//! // Let's read an instruction and see if it performs addition-assignment,
//! // like in one of the following examples:
//! //   - `a += 2`
//! //   - `a = a + 2`
//! //   - `a = 2 + a`
//! let operation = mystery_instruction.operation();
//! let is_add_assign = match operation {
//!     Operation::ADD => {
//!         let Add { destination, left, right } = Add::from(&mystery_instruction);
//!
//!         left == Operand::Register(destination, TypeCode::INTEGER)
//!         || right == Operand::Register(destination, TypeCode::INTEGER)
//!
//!     }
//!     _ => false,
//! };
//!
//! assert!(is_add_assign);
//! ```
mod add;
mod address;
mod call;
mod call_native;
mod close;
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
mod type_code;

pub use add::Add;
pub use address::{Address, AddressKind};
pub use call::Call;
pub use call_native::CallNative;
pub use close::Close;
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
pub use type_code::TypeCode;

use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display, Formatter};

use crate::NativeFunction;

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
        let first_5_bits = (self.0 & 0b11111) as u8;

        Operation(first_5_bits)
    }

    pub fn a_is_register(&self) -> bool {
        let sixth_bit = (self.0 >> 5) & 1;

        sixth_bit != 0
    }

    pub fn b_kind(&self) -> AddressKind {
        let bits_6_to_10 = (self.0 >> 5) & 0x1F;

        AddressKind(bits_6_to_10 as u8)
    }

    pub fn c_kind(&self) -> AddressKind {
        let bits_11_to_15 = (self.0 >> 10) & 0x1F;

        AddressKind(bits_11_to_15 as u8)
    }

    pub fn a_field(&self) -> u16 {
        let bits_16_to_31 = (self.0 >> 32) & 0xFFFF;

        bits_16_to_31 as u16
    }

    pub fn b_field(&self) -> u16 {
        let bits_32_to_47 = (self.0 >> 16) & 0xFFFF;

        bits_32_to_47 as u16
    }

    pub fn c_field(&self) -> u16 {
        let bits_48_to_63 = self.0 & 0xFFFF;

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

    pub fn as_address(&self) -> Address {
        match self.operation() {
            Operation::MOVE => {
                let Move { operand, .. } = Move::from(self);

                operand
            }
            Operation::LOAD_ENCODED => {
                let LoadEncoded {
                    destination, value, ..
                } = LoadEncoded::from(*self);

                Address {
                    index: destination.index,
                    kind: value.kind,
                }
            }
            Operation::LOAD_CONSTANT => {
                let LoadConstant { constant, .. } = LoadConstant::from(*self);

                constant
            }
            Operation::LOAD_LIST => {
                let LoadList { destination, .. } = LoadList::from(*self);
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
                let LoadFunction { destination, .. } = LoadFunction::from(*self);
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
                let left_type = left.as_type_code();
                let destination_type = match left_type {
                    TypeCode::CHARACTER => TypeCode::STRING,
                    _ => left_type,
                };

                destination.as_address(destination_type)
            }
            Operation::SUBTRACT => {
                let Subtract {
                    destination, left, ..
                } = Subtract::from(*self);

                destination.as_address(left.as_type_code())
            }
            Operation::MULTIPLY => {
                let Multiply {
                    destination, left, ..
                } = Multiply::from(*self);

                destination.as_address(left.as_type_code())
            }
            Operation::DIVIDE => {
                let Divide {
                    destination, left, ..
                } = Divide::from(*self);

                destination.as_address(left.as_type_code())
            }
            Operation::MODULO => {
                let Modulo {
                    destination, left, ..
                } = Modulo::from(*self);

                destination.as_address(left.as_type_code())
            }
            Operation::NOT => {
                let Not { destination, .. } = Not::from(*self);

                destination.as_address(TypeCode::BOOLEAN)
            }
            Operation::CALL => {
                let Call {
                    destination,
                    return_type,
                    ..
                } = Call::from(*self);

                destination.as_address(return_type)
            }
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

    pub fn load_encoded(destination: Destination, value: Address, jump_next: bool) -> Instruction {
        Instruction::from(LoadEncoded {
            destination,
            value,
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

    pub fn test(operand_register: u16, value: bool) -> Instruction {
        Instruction::from(Test {
            operand_register,
            comparator: value,
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
        argument_list_index: u16,
        return_type: TypeCode,
    ) -> Instruction {
        Instruction::from(Call {
            destination,
            function,
            argument_list_index,
            return_type,
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

    pub fn r#return(should_return_value: bool, return_value: Address) -> Instruction {
        Instruction::from(Return {
            should_return_value,
            return_value,
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
            | Operation::RETURN => false,
            unknown => panic!("Unknown operation: {}", unknown.0),
        }
    }

    pub fn disassembly_info(&self) -> String {
        let operation = self.operation();

        match operation {
            Operation::MOVE => Move::from(self).to_string(),
            Operation::CLOSE => Close::from(self).to_string(),
            Operation::LOAD_ENCODED => LoadEncoded::from(*self).to_string(),
            Operation::LOAD_CONSTANT => LoadConstant::from(*self).to_string(),
            Operation::LOAD_FUNCTION => LoadFunction::from(*self).to_string(),
            Operation::LOAD_LIST => LoadList::from(*self).to_string(),
            Operation::ADD => Add::from(self).to_string(),
            Operation::SUBTRACT => Subtract::from(*self).to_string(),
            Operation::MULTIPLY => Multiply::from(*self).to_string(),
            Operation::DIVIDE => Divide::from(*self).to_string(),
            Operation::MODULO => Modulo::from(*self).to_string(),
            Operation::NEGATE => Negate::from(*self).to_string(),
            Operation::NOT => Not::from(*self).to_string(),
            Operation::EQUAL => Equal::from(*self).to_string(),
            Operation::LESS => Less::from(*self).to_string(),
            Operation::LESS_EQUAL => LessEqual::from(*self).to_string(),
            Operation::TEST => Test::from(self).to_string(),
            Operation::TEST_SET => TestSet::from(*self).to_string(),
            Operation::CALL => Call::from(*self).to_string(),
            Operation::CALL_NATIVE => CallNative::from(*self).to_string(),
            Operation::JUMP => Jump::from(self).to_string(),
            Operation::RETURN => Return::from(*self).to_string(),
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
        write!(f, "{} | {}", self.operation(), self.disassembly_info())
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
        let bits = ((self.operation.0 as u64) << 59)
            | ((self.a_is_register as u64) << 54)
            | ((self.b_kind.0 as u64) << 53)
            | ((self.c_kind.0 as u64) << 49)
            | ((self.a_field as u64) << 32)
            | ((self.b_field as u64) << 16)
            | (self.c_field as u64);

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

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Destination {
    pub index: u16,
    pub is_register: bool,
}

impl Destination {
    pub fn memory(index: u16) -> Destination {
        Destination {
            index,
            is_register: false,
        }
    }

    pub fn register(index: u16) -> Destination {
        Destination {
            index,
            is_register: true,
        }
    }

    pub fn as_address(&self, destination_type: TypeCode) -> Address {
        let kind = match (destination_type, self.is_register) {
            (TypeCode::BOOLEAN, true) => AddressKind::BOOLEAN_REGISTER,
            (TypeCode::BOOLEAN, false) => AddressKind::BOOLEAN_MEMORY,
            (TypeCode::BYTE, true) => AddressKind::BYTE_REGISTER,
            (TypeCode::BYTE, false) => AddressKind::BYTE_MEMORY,
            (TypeCode::CHARACTER, true) => AddressKind::CHARACTER_REGISTER,
            (TypeCode::CHARACTER, false) => AddressKind::CHARACTER_MEMORY,
            (TypeCode::FLOAT, true) => AddressKind::FLOAT_REGISTER,
            (TypeCode::FLOAT, false) => AddressKind::FLOAT_MEMORY,
            (TypeCode::INTEGER, true) => AddressKind::INTEGER_REGISTER,
            (TypeCode::INTEGER, false) => AddressKind::INTEGER_MEMORY,
            (TypeCode::STRING, true) => AddressKind::STRING_REGISTER,
            (TypeCode::STRING, false) => AddressKind::STRING_MEMORY,
            (TypeCode::LIST, true) => AddressKind::LIST_REGISTER,
            (TypeCode::LIST, false) => AddressKind::LIST_MEMORY,
            (TypeCode::FUNCTION, true) => AddressKind::FUNCTION_REGISTER,
            (TypeCode::FUNCTION, false) => AddressKind::FUNCTION_MEMORY,
            (_, _) => unreachable!(),
        };

        Address {
            index: self.index,
            kind,
        }
    }
}

impl Destination {
    pub fn display(&self, f: &mut Formatter, destination_type: TypeCode) -> fmt::Result {
        if self.is_register {
            write!(f, "R_")?;
        } else {
            write!(f, "M_")?;
        }

        match destination_type {
            TypeCode::BOOLEAN => write!(f, "BOOL_")?,
            TypeCode::BYTE => write!(f, "BYTE_")?,
            TypeCode::CHARACTER => write!(f, "CHAR_")?,
            TypeCode::FLOAT => write!(f, "FLOAT_")?,
            TypeCode::INTEGER => write!(f, "INT_")?,
            TypeCode::STRING => write!(f, "STR_")?,
            TypeCode::LIST => write!(f, "LIST_")?,
            TypeCode::FUNCTION => write!(f, "FN_")?,
            unknown => unknown.unknown_write(f)?,
        }

        write!(f, "{}", self.index)
    }
}
