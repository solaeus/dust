//! The Dust instruction set.
//!
//! Each instruction is 64 bits and uses up to seven distinct fields.
//!
//! # Layout
//!
//! Bits  | Description
//! ----- | -----------
//! 0-6   | Operation
//! 7     | Unused
//! 8     | Flag indicating if the B field is a constant
//! 9     | Flag indicating if the C field is a constant
//! 10    | D field (boolean)
//! 11-15 | Unused
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
mod add_byte;
mod add_char;
mod add_char_str;
mod add_float;
mod add_int;
mod add_str;
mod add_str_char;
mod call;
mod call_native;
mod close;
mod divide_byte;
mod divide_float;
mod divide_int;
mod equal_bool;
mod equal_byte;
mod equal_char;
mod equal_char_str;
mod equal_float;
mod equal_int;
mod equal_str;
mod equal_str_char;
mod get_local;
mod jump;
mod less_byte;
mod less_char;
mod less_equal_byte;
mod less_equal_char;
mod less_equal_float;
mod less_equal_int;
mod less_equal_str;
mod less_float;
mod less_int;
mod less_str;
mod load_boolean;
mod load_constant;
mod load_function;
mod load_list;
mod load_self;
mod modulo_byte;
mod modulo_float;
mod modulo_int;
mod multiply_byte;
mod multiply_float;
mod multiply_int;
mod negate_float;
mod negate_int;
mod not;
mod operation;
mod point;
mod r#return;
mod set_local;
mod subtract_byte;
mod subtract_float;
mod subtract_int;
mod test;
mod test_set;

pub use add_byte::AddByte;
pub use add_char::AddChar;
pub use add_char_str::AddCharStr;
pub use add_float::AddFloat;
pub use add_int::AddInt;
pub use add_str::AddStr;
pub use add_str_char::AddStrChar;
pub use call::Call;
pub use call_native::CallNative;
pub use close::Close;
pub use divide_byte::DivideByte;
pub use divide_float::DivideFloat;
pub use divide_int::DivideInt;
pub use equal_bool::EqualBool;
pub use equal_byte::EqualByte;
pub use equal_char::EqualChar;
pub use equal_char_str::EqualCharStr;
pub use equal_float::EqualFloat;
pub use equal_int::EqualInt;
pub use equal_str::EqualStr;
pub use equal_str_char::EqualStrChar;
pub use get_local::GetLocal;
pub use jump::Jump;
pub use less_byte::LessByte;
pub use less_char::LessChar;
pub use less_equal_byte::LessEqualByte;
pub use less_equal_char::LessEqualChar;
pub use less_equal_float::LessEqualFloat;
pub use less_equal_int::LessEqualInt;
pub use less_equal_str::LessEqualStr;
pub use less_float::LessFloat;
pub use less_int::LessInt;
pub use less_str::LessStr;
pub use load_boolean::LoadBoolean;
pub use load_constant::LoadConstant;
pub use load_function::LoadFunction;
pub use load_list::LoadList;
pub use load_self::LoadSelf;
pub use modulo_byte::ModuloByte;
pub use modulo_float::ModuloFloat;
pub use modulo_int::ModuloInt;
pub use multiply_byte::MultiplyByte;
pub use multiply_float::MultiplyFloat;
pub use multiply_int::MultiplyInt;
pub use negate_float::NegateFloat;
pub use negate_int::NegateInt;
pub use not::Not;
pub use operation::Operation;
pub use point::Point;
pub use r#return::Return;
pub use set_local::SetLocal;
pub use subtract_byte::SubtractByte;
pub use subtract_float::SubtractFloat;
pub use subtract_int::SubtractInt;
pub use test::Test;
pub use test_set::TestSet;

use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display, Formatter};

use crate::NativeFunction;

pub struct InstructionBuilder {
    pub operation: Operation,
    pub a_field: u16,
    pub b_field: u16,
    pub c_field: u16,
    pub d_field: bool,
    pub b_is_constant: bool,
    pub c_is_constant: bool,
}

impl InstructionBuilder {
    pub fn build(self) -> Instruction {
        let bits = self.operation.0 as u64
            | ((self.b_is_constant as u64) << 7)
            | ((self.c_is_constant as u64) << 8)
            | ((self.d_field as u64) << 9)
            | ((self.a_field as u64) << 31)
            | ((self.b_field as u64) << 47)
            | ((self.c_field as u64) << 63);

        Instruction(bits)
    }
}

impl Default for InstructionBuilder {
    fn default() -> Self {
        InstructionBuilder {
            operation: Operation::POINT,
            a_field: 0,
            b_field: 0,
            c_field: 0,
            d_field: false,
            b_is_constant: false,
            c_is_constant: false,
        }
    }
}

/// An operation and its arguments for the Dust virtual machine.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Instruction(u64);

impl Instruction {
    pub fn operation(&self) -> Operation {
        let first_byte = (self.0 & 0b0111_1111) as u8;

        Operation(first_byte)
    }

    pub fn b_is_constant(&self) -> bool {
        (self.0 >> 8) & 1 == 0
    }

    pub fn c_is_constant(&self) -> bool {
        (self.0 >> 9) & 1 == 0
    }

    pub fn d_field(&self) -> bool {
        (self.0 >> 10) & 1 == 0
    }

    pub fn a_field(&self) -> u16 {
        ((self.0 >> 31) & 0xFFFF) as u16
    }

    pub fn b_field(&self) -> u16 {
        ((self.0 >> 47) & 0xFFFF) as u16
    }

    pub fn c_field(&self) -> u16 {
        (self.0 >> 48) as u16
    }

    pub fn set_a_field(&mut self, bits: u16) {
        self.0 = (bits as u64) << 31;
    }

    pub fn set_b_field(&mut self, bits: u16) {
        self.0 = (bits as u64) << 47;
    }

    pub fn set_c_field(&mut self, bits: u16) {
        self.0 = (bits as u64) << 63;
    }

    pub fn point(from: u16, to: u16) -> Instruction {
        Instruction::from(Point { from, to })
    }

    pub fn close(from: u16, to: u16) -> Instruction {
        Instruction::from(Close { from, to })
    }

    pub fn load_boolean(destination: u16, value: bool, jump_next: bool) -> Instruction {
        Instruction::from(LoadBoolean {
            destination,
            value,
            jump_next,
        })
    }

    pub fn load_constant(destination: u16, constant_index: u16, jump_next: bool) -> Instruction {
        Instruction::from(LoadConstant {
            destination,
            constant_index,
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

    pub fn get_local(destination: u16, local_index: u16) -> Instruction {
        Instruction::from(GetLocal {
            destination,
            local_index,
        })
    }

    pub fn set_local(register: u16, local_index: u16) -> Instruction {
        Instruction::from(SetLocal {
            local_index,
            register_index: register,
        })
    }

    pub fn add_int(destination: u16, left: Operand, right: Operand) -> Instruction {
        Instruction::from(AddInt {
            destination,
            left,
            right,
        })
    }

    pub fn add_float(destination: u16, left: Operand, right: Operand) -> Instruction {
        Instruction::from(AddFloat {
            destination,
            left,
            right,
        })
    }

    pub fn add_byte(destination: u16, left: Operand, right: Operand) -> Instruction {
        Instruction::from(AddByte {
            destination,
            left,
            right,
        })
    }

    pub fn add_str(destination: u16, left: Operand, right: Operand) -> Instruction {
        Instruction::from(AddStr {
            destination,
            left,
            right,
        })
    }

    pub fn add_char(destination: u16, left: Operand, right: Operand) -> Instruction {
        Instruction::from(AddChar {
            destination,
            left,
            right,
        })
    }

    pub fn add_str_char(destination: u16, left: Operand, right: Operand) -> Instruction {
        Instruction::from(AddStrChar {
            destination,
            left,
            right,
        })
    }

    pub fn add_char_str(destination: u16, left: Operand, right: Operand) -> Instruction {
        Instruction::from(AddCharStr {
            destination,
            left,
            right,
        })
    }

    pub fn subtract_int(destination: u16, left: Operand, right: Operand) -> Instruction {
        Instruction::from(SubtractInt {
            destination,
            left,
            right,
        })
    }

    pub fn subtract_float(destination: u16, left: Operand, right: Operand) -> Instruction {
        Instruction::from(SubtractFloat {
            destination,
            left,
            right,
        })
    }

    pub fn subtract_byte(destination: u16, left: Operand, right: Operand) -> Instruction {
        Instruction::from(SubtractByte {
            destination,
            left,
            right,
        })
    }

    pub fn multiply_int(destination: u16, left: Operand, right: Operand) -> Instruction {
        Instruction::from(MultiplyInt {
            destination,
            left,
            right,
        })
    }

    pub fn multiply_float(destination: u16, left: Operand, right: Operand) -> Instruction {
        Instruction::from(MultiplyFloat {
            destination,
            left,
            right,
        })
    }

    pub fn multiply_byte(destination: u16, left: Operand, right: Operand) -> Instruction {
        Instruction::from(MultiplyByte {
            destination,
            left,
            right,
        })
    }

    pub fn divide_int(destination: u16, left: Operand, right: Operand) -> Instruction {
        Instruction::from(DivideInt {
            destination,
            left,
            right,
        })
    }

    pub fn divide_float(destination: u16, left: Operand, right: Operand) -> Instruction {
        Instruction::from(DivideFloat {
            destination,
            left,
            right,
        })
    }

    pub fn divide_byte(destination: u16, left: Operand, right: Operand) -> Instruction {
        Instruction::from(DivideByte {
            destination,
            left,
            right,
        })
    }

    pub fn modulo_int(destination: u16, left: Operand, right: Operand) -> Instruction {
        Instruction::from(ModuloInt {
            destination,
            left,
            right,
        })
    }

    pub fn modulo_float(destination: u16, left: Operand, right: Operand) -> Instruction {
        Instruction::from(ModuloFloat {
            destination,
            left,
            right,
        })
    }

    pub fn modulo_byte(destination: u16, left: Operand, right: Operand) -> Instruction {
        Instruction::from(ModuloByte {
            destination,
            left,
            right,
        })
    }

    pub fn equal_int(comparator: bool, left: Operand, right: Operand) -> Instruction {
        Instruction::from(EqualInt {
            comparator,
            left,
            right,
        })
    }

    pub fn equal_float(comparator: bool, left: Operand, right: Operand) -> Instruction {
        Instruction::from(EqualFloat {
            comparator,
            left,
            right,
        })
    }

    pub fn equal_byte(comparator: bool, left: Operand, right: Operand) -> Instruction {
        Instruction::from(EqualByte {
            comparator,
            left,
            right,
        })
    }

    pub fn equal_str(comparator: bool, left: Operand, right: Operand) -> Instruction {
        Instruction::from(EqualStr {
            comparator,
            left,
            right,
        })
    }

    pub fn equal_char(comparator: bool, left: Operand, right: Operand) -> Instruction {
        Instruction::from(EqualChar {
            comparator,
            left,
            right,
        })
    }

    pub fn equal_str_char(comparator: bool, left: Operand, right: Operand) -> Instruction {
        Instruction::from(EqualStrChar {
            comparator,
            left,
            right,
        })
    }

    pub fn equal_char_str(comparator: bool, left: Operand, right: Operand) -> Instruction {
        Instruction::from(EqualCharStr {
            comparator,
            left,
            right,
        })
    }

    pub fn equal_bool(comparator: bool, left: Operand, right: Operand) -> Instruction {
        Instruction::from(EqualBool {
            comparator,
            left,
            right,
        })
    }

    pub fn less_int(comparator: bool, left: Operand, right: Operand) -> Instruction {
        Instruction::from(LessInt {
            comparator,
            left,
            right,
        })
    }

    pub fn less_float(comparator: bool, left: Operand, right: Operand) -> Instruction {
        Instruction::from(LessFloat {
            comparator,
            left,
            right,
        })
    }

    pub fn less_byte(comparator: bool, left: Operand, right: Operand) -> Instruction {
        Instruction::from(LessByte {
            comparator,
            left,
            right,
        })
    }

    pub fn less_str(comparator: bool, left: Operand, right: Operand) -> Instruction {
        Instruction::from(LessStr {
            comparator,
            left,
            right,
        })
    }

    pub fn less_char(comparator: bool, left: Operand, right: Operand) -> Instruction {
        Instruction::from(LessChar {
            comparator,
            left,
            right,
        })
    }

    pub fn less_equal_int(comparator: bool, left: Operand, right: Operand) -> Instruction {
        Instruction::from(LessEqualInt {
            comparator,
            left,
            right,
        })
    }

    pub fn less_equal_float(comparator: bool, left: Operand, right: Operand) -> Instruction {
        Instruction::from(LessEqualFloat {
            comparator,
            left,
            right,
        })
    }

    pub fn less_equal_byte(comparator: bool, left: Operand, right: Operand) -> Instruction {
        Instruction::from(LessEqualByte {
            comparator,
            left,
            right,
        })
    }

    pub fn less_equal_str(comparator: bool, left: Operand, right: Operand) -> Instruction {
        Instruction::from(LessEqualStr {
            comparator,
            left,
            right,
        })
    }

    pub fn less_equal_char(comparator: bool, left: Operand, right: Operand) -> Instruction {
        Instruction::from(LessEqualChar {
            comparator,
            left,
            right,
        })
    }

    pub fn negate_int(destination: u16, argument: Operand) -> Instruction {
        Instruction::from(NegateInt {
            destination,
            argument,
        })
    }

    pub fn negate_float(destination: u16, argument: Operand) -> Instruction {
        Instruction::from(NegateFloat {
            destination,
            argument,
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

    pub fn r#return(should_return_value: bool, return_register: u16) -> Instruction {
        Instruction::from(Return {
            should_return_value,
            return_register,
        })
    }

    pub fn is_math(&self) -> bool {
        self.operation().is_math()
    }

    pub fn is_comparison(&self) -> bool {
        self.operation().is_comparison()
    }

    pub fn as_argument(&self) -> Option<Operand> {
        match self.operation() {
            Operation::LOAD_CONSTANT => Some(Operand::Constant(self.b_field())),
            Operation::LOAD_BOOLEAN
            | Operation::LOAD_LIST
            | Operation::LOAD_SELF
            | Operation::GET_LOCAL
            | Operation::ADD_INT
            | Operation::ADD_FLOAT
            | Operation::ADD_BYTE
            | Operation::ADD_STR
            | Operation::ADD_CHAR
            | Operation::ADD_STR_CHAR
            | Operation::ADD_CHAR_STR
            | Operation::SUBTRACT_INT
            | Operation::SUBTRACT_FLOAT
            | Operation::SUBTRACT_BYTE
            | Operation::MULTIPLY_INT
            | Operation::MULTIPLY_FLOAT
            | Operation::MULTIPLY_BYTE
            | Operation::DIVIDE_INT
            | Operation::DIVIDE_FLOAT
            | Operation::DIVIDE_BYTE
            | Operation::MODULO_INT
            | Operation::MODULO_FLOAT
            | Operation::MODULO_BYTE
            | Operation::EQUAL_INT
            | Operation::EQUAL_FLOAT
            | Operation::EQUAL_BYTE
            | Operation::EQUAL_STR
            | Operation::EQUAL_CHAR
            | Operation::EQUAL_STR_CHAR
            | Operation::EQUAL_CHAR_STR
            | Operation::EQUAL_BOOL
            | Operation::LESS_INT
            | Operation::LESS_FLOAT
            | Operation::LESS_BYTE
            | Operation::LESS_STR
            | Operation::LESS_CHAR
            | Operation::LESS_EQUAL_INT
            | Operation::LESS_EQUAL_FLOAT
            | Operation::LESS_EQUAL_BYTE
            | Operation::LESS_EQUAL_STR
            | Operation::LESS_EQUAL_CHAR
            | Operation::NEGATE_INT
            | Operation::NEGATE_FLOAT
            | Operation::NOT
            | Operation::CALL => Some(Operand::Register(self.a_field())),
            Operation::CALL_NATIVE => {
                let function = NativeFunction::from(self.b_field());

                if function.returns_value() {
                    Some(Operand::Register(self.a_field()))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn b_as_argument(&self) -> Operand {
        if self.b_is_constant() {
            Operand::Constant(self.b_field())
        } else {
            Operand::Register(self.b_field())
        }
    }

    pub fn b_and_c_as_operands(&self) -> (Operand, Operand) {
        let left = if self.b_is_constant() {
            Operand::Constant(self.b_field())
        } else {
            Operand::Register(self.b_field())
        };
        let right = if self.c_is_constant() {
            Operand::Constant(self.c_field())
        } else {
            Operand::Register(self.c_field())
        };

        (left, right)
    }

    pub fn yields_value(&self) -> bool {
        match self.operation() {
            Operation::POINT
            | Operation::LOAD_BOOLEAN
            | Operation::LOAD_CONSTANT
            | Operation::LOAD_FUNCTION
            | Operation::LOAD_LIST
            | Operation::LOAD_SELF
            | Operation::GET_LOCAL
            | Operation::ADD_INT
            | Operation::ADD_FLOAT
            | Operation::ADD_BYTE
            | Operation::ADD_STR
            | Operation::ADD_CHAR
            | Operation::ADD_STR_CHAR
            | Operation::ADD_CHAR_STR
            | Operation::SUBTRACT_INT
            | Operation::SUBTRACT_FLOAT
            | Operation::SUBTRACT_BYTE
            | Operation::MULTIPLY_INT
            | Operation::MULTIPLY_FLOAT
            | Operation::MULTIPLY_BYTE
            | Operation::DIVIDE_INT
            | Operation::DIVIDE_FLOAT
            | Operation::DIVIDE_BYTE
            | Operation::MODULO_INT
            | Operation::MODULO_FLOAT
            | Operation::MODULO_BYTE
            | Operation::NEGATE_INT
            | Operation::NEGATE_FLOAT
            | Operation::NOT
            | Operation::CALL => true,
            Operation::CALL_NATIVE => {
                let function = NativeFunction::from(self.b_field());

                function.returns_value()
            }
            Operation::CLOSE
            | Operation::SET_LOCAL
            | Operation::EQUAL_INT
            | Operation::EQUAL_FLOAT
            | Operation::EQUAL_BYTE
            | Operation::EQUAL_STR
            | Operation::EQUAL_CHAR
            | Operation::EQUAL_STR_CHAR
            | Operation::EQUAL_CHAR_STR
            | Operation::EQUAL_BOOL
            | Operation::LESS_INT
            | Operation::LESS_FLOAT
            | Operation::LESS_BYTE
            | Operation::LESS_STR
            | Operation::LESS_CHAR
            | Operation::LESS_EQUAL_INT
            | Operation::LESS_EQUAL_FLOAT
            | Operation::LESS_EQUAL_BYTE
            | Operation::LESS_EQUAL_STR
            | Operation::LESS_EQUAL_CHAR
            | Operation::TEST
            | Operation::TEST_SET
            | Operation::JUMP
            | Operation::RETURN => false,
            _ => Operation::panic_from_unknown_code(self.operation().0),
        }
    }

    pub fn disassembly_info(&self) -> String {
        let operation = self.operation();

        match operation {
            Operation::POINT => Point::from(*self).to_string(),
            Operation::CLOSE => Close::from(*self).to_string(),
            Operation::LOAD_BOOLEAN => LoadBoolean::from(*self).to_string(),
            Operation::LOAD_CONSTANT => LoadConstant::from(*self).to_string(),
            Operation::LOAD_FUNCTION => LoadFunction::from(*self).to_string(),
            Operation::LOAD_LIST => LoadList::from(*self).to_string(),
            Operation::LOAD_SELF => LoadSelf::from(*self).to_string(),
            Operation::GET_LOCAL => GetLocal::from(*self).to_string(),
            Operation::SET_LOCAL => SetLocal::from(*self).to_string(),
            Operation::ADD_INT => AddInt::from(*self).to_string(),

            _ => Operation::panic_from_unknown_code(operation.0),
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
    Constant(u16),
    Register(u16),
}

impl Operand {
    pub fn index(&self) -> u16 {
        match self {
            Operand::Constant(index) => *index,
            Operand::Register(index) => *index,
        }
    }

    pub fn is_constant(&self) -> bool {
        matches!(self, Operand::Constant(_))
    }

    pub fn is_register(&self) -> bool {
        matches!(self, Operand::Register(_))
    }

    pub fn as_index_and_constant_flag(&self) -> (u16, bool) {
        match self {
            Operand::Constant(index) => (*index, true),
            Operand::Register(index) => (*index, false),
        }
    }
}

impl Display for Operand {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Operand::Constant(index) => write!(f, "C{index}"),
            Operand::Register(index) => write!(f, "R{index}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_operation() {
        let instruction = Instruction::add_int(42, Operand::Constant(4), Operand::Register(2));

        assert_eq!(instruction.operation(), Operation::ADD_INT);
    }

    #[test]
    fn decode_a_field() {
        let instruction = Instruction::add_int(42, Operand::Constant(4), Operand::Register(2));

        assert_eq!(42, instruction.a_field());
    }

    #[test]
    fn decode_b_field() {
        let instruction = Instruction::add_int(42, Operand::Constant(4), Operand::Register(2));

        assert_eq!(4, instruction.b_field());
    }

    #[test]
    fn decode_c_field() {
        let instruction = Instruction::add_int(42, Operand::Constant(4), Operand::Register(2));

        assert_eq!(2, instruction.c_field());
    }

    #[test]
    fn decode_d_field() {
        let instruction = Instruction::call(42, 4, 2, true);

        assert!(instruction.d_field());
    }

    #[test]
    fn decode_b_is_constant() {
        let instruction = Instruction::add_int(42, Operand::Constant(4), Operand::Register(2));

        assert!(instruction.b_is_constant());
    }

    #[test]
    fn decode_c_is_constant() {
        let instruction = Instruction::add_int(42, Operand::Register(2), Operand::Constant(4));

        assert!(instruction.c_is_constant());
    }
}
