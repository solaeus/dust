//! The Dust instruction set.
//!
//! Each instruction is 64 bits and uses up to seven distinct fields.
//!
//! # Layout
//!
//! Bits  | Description
//! ----- | -----------
//! 0-6   | Operation
//! 7     | Flag indicating if the B field is a constant
//! 8     | Flag indicating if the C field is a constant
//! 9     | D field (boolean)
//! 10-12 | B field type
//! 13-15 | C field type
//! 16-31 | A field (unsigned 16-bit integer)
//! 32-47 | B field (unsigned 16-bit integer)
//! 48-63 | C field (unsigned 16-bit integer)
//!
//! **Be careful when working with instructions directly**. When modifying an instruction's fields,
//! you may also need to modify its flags. It is usually best to remove instructions and insert new
//! ones in their place instead of mutating them.
//!
//! # Examples
//!
//! ## Creating Instructions
//!
//! For each operation, there are two ways to create an instruction:
//!
//! - Use the associated function on `Instruction`
//! - Use the corresponding struct and call `Instruction::from`
//!
//! Both produce the same result, but the first is more concise. The structs are more useful when
//! reading instructions, as shown below.
//!
//! ```
//! # use dust_lang::instruction::{Instruction, Move};
//! let move_1 = Instruction::r#move(42, 4);
//! let move_2 = Instruction::from(Move { from: 42, to: 4 });
//!
//! assert_eq!(move_1, move_2);
//! ```
//!
//! Use the [`Argument`][] type when creating instructions. In addition to being easy to read and
//! write, this ensures that the instruction has the correct flags to represent the arguments.
//!
//! ```
//! # use dust_lang::instruction::{Instruction, Add, Argument};
//! let add_1 = Instruction::add(
//!     0,
//!     Argument::Register(1),
//!     Argument::Constant(2)
//! );
//! let add_2 = Instruction::from(Add {
//!     destination: 0,
//!     left: Argument::Register(1),
//!     right: Argument::Constant(2),
//! });
//!
//! assert_eq!(add_1, add_2);
//! ```
//!
//! ## Reading Instructions
//!
//! To read an instruction, check its operation with [`Instruction::operation`], then convert the
//! instruction to the struct that corresponds to that operation. Like the example above, this
//! removes the burden of dealing with the options directly and automatically casts the A, B, C and
//! D fields as `u16`, `bool` or `Argument` values.
//!
//! ```
//! # use dust_lang::instruction::{Instruction, Add, Argument, Operation};
//! # let mystery_instruction = Instruction::add(
//! #     1,
//! #     Argument::Register(1),
//! #     Argument::Constant(2)
//! # );
//! // Let's read an instruction and see if it performs addition-assignment,
//! // like in one of the following examples:
//! //   - `a += 2`
//! //   - `a = a + 2`
//! //   - `a = 2 + a`
//!
//! let operation = mystery_instruction.operation();
//! let is_add_assign = match operation {
//!     Operation::Add => {
//!         let Add { destination, left, right } = Add::from(&mystery_instruction);
//!
//!         left == Argument::Register(destination)
//!         || right == Argument::Register(destination);
//!
//!     }
//!     _ => false,
//! };
//!
//! assert!(is_add_assign);
//! ```
mod add;
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
mod load_self;
mod modulo;
mod multiply;
mod negate;
mod not;
mod operation;
mod point;
mod r#return;
mod subtract;
mod test;
mod test_set;
mod type_code;

pub use add::Add;
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
pub use load_self::LoadSelf;
pub use modulo::Modulo;
pub use multiply::Multiply;
pub use negate::Negate;
pub use not::Not;
pub use operation::Operation;
pub use point::Point;
pub use r#return::Return;
pub use subtract::Subtract;
pub use test::Test;
pub use test_set::TestSet;

use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display, Formatter};
pub use type_code::TypeCode;

use crate::NativeFunction;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InstructionFields {
    pub operation: Operation,
    pub a_field: u16,
    pub b_field: u16,
    pub c_field: u16,
    pub d_field: bool,
    pub b_is_constant: bool,
    pub c_is_constant: bool,
    pub b_type: TypeCode,
    pub c_type: TypeCode,
}

impl InstructionFields {
    pub fn build(self) -> Instruction {
        let bits = ((self.operation.0 as u64) << 57)
            | ((self.b_is_constant as u64) << 56)
            | ((self.c_is_constant as u64) << 55)
            | ((self.d_field as u64) << 54)
            | ((self.b_type.0 as u64) << 51)
            | ((self.c_type.0 as u64) << 48)
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
            d_field: instruction.d_field(),
            b_is_constant: instruction.b_is_constant(),
            c_is_constant: instruction.c_is_constant(),
            b_type: instruction.b_type(),
            c_type: instruction.c_type(),
        }
    }
}

impl Default for InstructionFields {
    fn default() -> Self {
        InstructionFields {
            operation: Operation::POINT,
            a_field: 0,
            b_field: 0,
            c_field: 0,
            d_field: false,
            b_is_constant: false,
            c_is_constant: false,
            b_type: TypeCode::BOOLEAN,
            c_type: TypeCode::BOOLEAN,
        }
    }
}

/// An instruction for the Dust virtual machine.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Instruction(u64);

impl Instruction {
    pub fn operation(&self) -> Operation {
        let first_7_bits = (self.0 >> 57) as u8;

        Operation(first_7_bits)
    }

    pub fn b_is_constant(&self) -> bool {
        (self.0 >> 56) & 1 != 0
    }

    pub fn c_is_constant(&self) -> bool {
        (self.0 >> 55) & 1 != 0
    }

    pub fn d_field(&self) -> bool {
        (self.0 >> 54) & 1 != 0
    }

    pub fn b_type(&self) -> TypeCode {
        let byte = ((self.0 >> 51) & 0b111) as u8;

        TypeCode(byte)
    }

    pub fn c_type(&self) -> TypeCode {
        let byte = ((self.0 >> 48) & 0b111) as u8;

        TypeCode(byte)
    }

    pub fn a_field(&self) -> u16 {
        ((self.0 >> 32) & 0xFFFF) as u16
    }

    pub fn b_field(&self) -> u16 {
        ((self.0 >> 16) & 0xFFFF) as u16
    }

    pub fn c_field(&self) -> u16 {
        (self.0 & 0xFFFF) as u16
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

    pub fn as_operand(&self) -> Operand {
        match self.operation() {
            Operation::POINT => {
                let Point { destination, to } = Point::from(*self);

                Operand::Register(destination, to.as_type())
            }
            Operation::LOAD_ENCODED => {
                let LoadEncoded {
                    destination,
                    value_type,
                    ..
                } = LoadEncoded::from(*self);

                Operand::Register(destination, value_type)
            }
            Operation::LOAD_CONSTANT => {
                let LoadConstant {
                    constant_type,
                    constant_index,
                    ..
                } = LoadConstant::from(*self);

                Operand::Constant(constant_index, constant_type)
            }
            Operation::LOAD_LIST => {
                let LoadList { destination, .. } = LoadList::from(*self);

                Operand::Register(destination, TypeCode::LIST)
            }
            Operation::LOAD_FUNCTION => {
                let LoadFunction { destination, .. } = LoadFunction::from(*self);

                Operand::Register(destination, TypeCode::FUNCTION)
            }
            Operation::LOAD_SELF => {
                let LoadSelf { destination, .. } = LoadSelf::from(*self);

                Operand::Register(destination, TypeCode::FUNCTION)
            }
            _ => todo!(),
        }
    }

    pub fn point(destination: u16, to: Operand) -> Instruction {
        Instruction::from(Point { destination, to })
    }

    pub fn close(from: u16, to: u16) -> Instruction {
        Instruction::from(Close { from, to })
    }

    pub fn load_encoded(
        destination: u16,
        value: u16,
        value_type: TypeCode,
        jump_next: bool,
    ) -> Instruction {
        Instruction::from(LoadEncoded {
            destination,
            value,
            value_type,
            jump_next,
        })
    }

    pub fn load_constant(
        destination: u16,
        constant_index: u16,
        constant_type: TypeCode,
        jump_next: bool,
    ) -> Instruction {
        Instruction::from(LoadConstant {
            destination,
            constant_index,
            constant_type,
            jump_next,
        })
    }

    pub fn load_function(destination: u16, prototype_index: u16, jump_next: bool) -> Instruction {
        Instruction::from(LoadFunction {
            destination,
            prototype_index,
            jump_next,
        })
    }

    pub fn load_list(destination: u16, start_register: u16, jump_next: bool) -> Instruction {
        Instruction::from(LoadList {
            destination,
            start_register,
            jump_next,
        })
    }

    pub fn load_self(destination: u16, jump_next: bool) -> Instruction {
        Instruction::from(LoadSelf {
            destination,
            jump_next,
        })
    }

    pub fn add(destination: u16, left: Operand, right: Operand) -> Instruction {
        Instruction::from(Add {
            destination,
            left,
            right,
        })
    }

    pub fn subtract(destination: u16, left: Operand, right: Operand) -> Instruction {
        Instruction::from(Subtract {
            destination,
            left,
            right,
        })
    }

    pub fn multiply(destination: u16, left: Operand, right: Operand) -> Instruction {
        Instruction::from(Multiply {
            destination,
            left,
            right,
        })
    }

    pub fn divide(destination: u16, left: Operand, right: Operand) -> Instruction {
        Instruction::from(Divide {
            destination,
            left,
            right,
        })
    }

    pub fn modulo(destination: u16, left: Operand, right: Operand) -> Instruction {
        Instruction::from(Modulo {
            destination,
            left,
            right,
        })
    }

    pub fn equal(comparator: bool, left: Operand, right: Operand) -> Instruction {
        Instruction::from(Equal {
            comparator,
            left,
            right,
        })
    }

    pub fn less(comparator: bool, left: Operand, right: Operand) -> Instruction {
        Instruction::from(Less {
            comparator,
            left,
            right,
        })
    }

    pub fn less_equal(comparator: bool, left: Operand, right: Operand) -> Instruction {
        Instruction::from(LessEqual {
            comparator,
            left,
            right,
        })
    }

    pub fn negate(destination: u16, argument: Operand, argument_type: TypeCode) -> Instruction {
        Instruction::from(Negate {
            destination,
            argument,
            argument_type,
        })
    }

    pub fn not(destination: u16, argument: Operand) -> Instruction {
        Instruction::from(Not {
            destination,
            argument,
        })
    }

    pub fn test(operand_register: u16, value: bool) -> Instruction {
        Instruction::from(Test {
            operand_register,
            test_value: value,
        })
    }

    pub fn test_set(destination: u16, argument: Operand, value: bool) -> Instruction {
        Instruction::from(TestSet {
            destination,
            argument,
            test_value: value,
        })
    }

    pub fn jump(offset: u16, is_positive: bool) -> Instruction {
        Instruction::from(Jump {
            offset,
            is_positive,
        })
    }

    pub fn call(
        destination: u16,
        function_register: u16,
        argument_count: u16,
        is_recursive: bool,
    ) -> Instruction {
        Instruction::from(Call {
            destination,
            function_register,
            argument_count,
            is_recursive,
        })
    }

    pub fn call_native(
        destination: u16,
        function: NativeFunction,
        argument_count: u16,
    ) -> Instruction {
        Instruction::from(CallNative {
            destination,
            function,
            argument_count,
        })
    }

    pub fn r#return(
        should_return_value: bool,
        return_register: u16,
        r#type: TypeCode,
    ) -> Instruction {
        Instruction::from(Return {
            should_return_value,
            return_register,
            r#type,
        })
    }

    pub fn is_math(&self) -> bool {
        self.operation().is_math()
    }

    pub fn is_comparison(&self) -> bool {
        self.operation().is_comparison()
    }

    pub fn b_as_operand(&self) -> Operand {
        if self.b_is_constant() {
            Operand::Constant(self.b_field(), self.b_type())
        } else {
            Operand::Register(self.b_field(), self.b_type())
        }
    }

    pub fn b_and_c_as_operands(&self) -> (Operand, Operand) {
        let left = self.b_as_operand();
        let right = if self.c_is_constant() {
            Operand::Constant(self.c_field(), self.c_type())
        } else {
            Operand::Register(self.c_field(), self.c_type())
        };

        (left, right)
    }

    pub fn yields_value(&self) -> bool {
        match self.operation() {
            Operation::POINT
            | Operation::LOAD_ENCODED
            | Operation::LOAD_CONSTANT
            | Operation::LOAD_FUNCTION
            | Operation::LOAD_LIST
            | Operation::LOAD_SELF
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
            Operation::EQUAL
            | Operation::LESS
            | Operation::LESS_EQUAL
            | Operation::TEST
            | Operation::TEST_SET
            | Operation::JUMP
            | Operation::RETURN => false,
            _ => self.operation().panic_from_unknown_code(),
        }
    }

    pub fn disassembly_info(&self) -> String {
        let operation = self.operation();

        match operation {
            Operation::POINT => Point::from(*self).to_string(),
            Operation::LOAD_ENCODED => LoadEncoded::from(*self).to_string(),
            Operation::LOAD_CONSTANT => LoadConstant::from(*self).to_string(),
            Operation::LOAD_FUNCTION => LoadFunction::from(*self).to_string(),
            Operation::LOAD_LIST => LoadList::from(*self).to_string(),
            Operation::LOAD_SELF => LoadSelf::from(*self).to_string(),
            Operation::ADD => Add::from(*self).to_string(),
            Operation::SUBTRACT => Subtract::from(*self).to_string(),
            Operation::MULTIPLY => Multiply::from(*self).to_string(),
            Operation::DIVIDE => Divide::from(*self).to_string(),
            Operation::MODULO => Modulo::from(*self).to_string(),
            Operation::NEGATE => Negate::from(*self).to_string(),
            Operation::NOT => Not::from(*self).to_string(),
            Operation::EQUAL => Equal::from(*self).to_string(),
            Operation::LESS => Less::from(*self).to_string(),
            Operation::LESS_EQUAL => LessEqual::from(*self).to_string(),
            Operation::TEST => Test::from(*self).to_string(),
            Operation::TEST_SET => TestSet::from(*self).to_string(),
            Operation::CALL => Call::from(*self).to_string(),
            Operation::CALL_NATIVE => CallNative::from(*self).to_string(),
            Operation::JUMP => Jump::from(*self).to_string(),
            Operation::RETURN => Return::from(*self).to_string(),

            _ => operation.panic_from_unknown_code(),
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Operand {
    Constant(u16, TypeCode),
    Register(u16, TypeCode),
}

impl Operand {
    pub fn index(&self) -> u16 {
        match self {
            Operand::Constant(index, _) => *index,
            Operand::Register(index, _) => *index,
        }
    }

    pub fn is_constant(&self) -> bool {
        matches!(self, Operand::Constant(_, _))
    }

    pub fn is_register(&self) -> bool {
        matches!(self, Operand::Register(_, _))
    }

    pub fn as_index_and_constant_flag(&self) -> (u16, bool) {
        match self {
            Operand::Constant(index, _) => (*index, true),
            Operand::Register(index, _) => (*index, false),
        }
    }

    pub fn as_type(&self) -> TypeCode {
        match self {
            Operand::Constant(_, r#type) => *r#type,
            Operand::Register(_, r#type) => *r#type,
        }
    }
}

impl Display for Operand {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Operand::Constant(index, r#type) => match *r#type {
                TypeCode::BOOLEAN => write!(f, "C_{}", index),
                TypeCode::CHARACTER => write!(f, "C_{}", index),
                TypeCode::INTEGER => write!(f, "C_{}", index),
                TypeCode::FLOAT => write!(f, "C_{}", index),
                TypeCode::STRING => write!(f, "C_{}", index),
                TypeCode::LIST => write!(f, "C_{}", index),
                TypeCode::FUNCTION => write!(f, "C_{}", index),
                _ => panic!("Unknown type code: {}", r#type.0),
            },
            Operand::Register(index, r#type) => match *r#type {
                TypeCode::BOOLEAN => write!(f, "R_BOOL_{}", index),
                TypeCode::CHARACTER => write!(f, "R_CHAR_{}", index),
                TypeCode::INTEGER => write!(f, "R_INT_{}", index),
                TypeCode::FLOAT => write!(f, "R_FLOAT_{}", index),
                TypeCode::STRING => write!(f, "R_STR_{}", index),
                TypeCode::LIST => write!(f, "R_LIST_{}", index),
                TypeCode::FUNCTION => write!(f, "R_FN_{}", index),
                _ => panic!("Unknown type code: {}", r#type.0),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_operation() {
        let instruction = Instruction::add(
            42,
            Operand::Constant(4, TypeCode::STRING),
            Operand::Register(2, TypeCode::CHARACTER),
        );

        assert_eq!(instruction.operation(), Operation::ADD);
    }

    #[test]
    fn decode_a_field() {
        let instruction = Instruction::add(
            42,
            Operand::Constant(4, TypeCode::STRING),
            Operand::Register(2, TypeCode::CHARACTER),
        );

        assert_eq!(42, instruction.a_field());
    }

    #[test]
    fn decode_b_field() {
        let instruction = Instruction::add(
            42,
            Operand::Constant(4, TypeCode::STRING),
            Operand::Register(2, TypeCode::CHARACTER),
        );

        assert_eq!(4, instruction.b_field());
    }

    #[test]
    fn decode_c_field() {
        let instruction = Instruction::add(
            42,
            Operand::Constant(4, TypeCode::STRING),
            Operand::Register(2, TypeCode::CHARACTER),
        );

        assert_eq!(2, instruction.c_field());
    }

    #[test]
    fn decode_d_field() {
        let instruction = Instruction::add(
            42,
            Operand::Constant(4, TypeCode::STRING),
            Operand::Register(2, TypeCode::CHARACTER),
        );

        assert!(!instruction.d_field());
    }

    #[test]
    fn decode_b_is_constant() {
        let instruction = Instruction::add(
            42,
            Operand::Constant(4, TypeCode::STRING),
            Operand::Register(2, TypeCode::CHARACTER),
        );

        assert!(instruction.b_is_constant());
    }

    #[test]
    fn decode_c_is_constant() {
        let instruction = Instruction::add(
            42,
            Operand::Register(2, TypeCode::STRING),
            Operand::Constant(4, TypeCode::CHARACTER),
        );

        assert!(instruction.c_is_constant());
    }

    #[test]
    fn decode_b_type() {
        let instruction = Instruction::add(
            42,
            Operand::Constant(4, TypeCode::STRING),
            Operand::Register(2, TypeCode::CHARACTER),
        );

        assert_eq!(TypeCode::STRING, instruction.b_type());
    }

    #[test]
    fn decode_c_type() {
        let instruction = Instruction::add(
            42,
            Operand::Constant(4, TypeCode::STRING),
            Operand::Register(2, TypeCode::CHARACTER),
        );

        assert_eq!(TypeCode::CHARACTER, instruction.c_type());
    }
}
