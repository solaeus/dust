//! Instructions for the Dust virtual machine.
//!
//! Each instruction is 32 bits and uses up to seven distinct fields:
//!
//! Bit   | Description
//! ----- | -----------
//! 0-4   | Operation code
//! 5     | Flag indicating if the B argument is a constant
//! 6     | Flag indicating if the C argument is a constant
//! 7     | D field (boolean)
//! 8-15  | A field (unsigned 8-bit integer)
//! 16-23 | B field (unsigned 8-bit integer)
//! 24-31 | C field (unsigned 8-bit integer)
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
//! D fields as `u8`, `bool` or `Argument` values.
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
//! //  - `a += 2`
//! //  - `a = a + 2`
//! //  - `a = 2 + a`
//!
//! let operation = mystery_instruction.operation();
//!
//! match operation {
//!     Operation::Add => {
//!         let Add { destination, left, right } = Add::from(&mystery_instruction);
//!         let is_add_assign =
//!             left == Argument::Register(destination)
//!             || right == Argument::Register(destination);
//!
//!         assert!(is_add_assign);
//!     }
//!     _ => {} // Handle other operations...
//! }
//! ```
mod add;
mod call;
mod call_native;
mod close;
mod divide;
mod equal;
mod get_local;
mod jump;
mod less;
mod less_equal;
mod load_boolean;
mod load_constant;
mod load_list;
mod load_self;
mod modulo;
mod r#move;
mod multiply;
mod negate;
mod not;
mod operation;
mod options;
mod r#return;
mod set_local;
mod subtract;
mod test;
mod test_set;

pub use add::Add;
pub use call::Call;
pub use call_native::CallNative;
pub use close::Close;
pub use divide::Divide;
pub use equal::Equal;
pub use get_local::GetLocal;
pub use jump::Jump;
pub use less::Less;
pub use less_equal::LessEqual;
pub use load_boolean::LoadBoolean;
pub use load_constant::LoadConstant;
pub use load_list::LoadList;
pub use load_self::LoadSelf;
pub use modulo::Modulo;
pub use multiply::Multiply;
pub use negate::Negate;
pub use not::Not;
pub use operation::Operation;
pub use options::InstructionOptions;
pub use r#move::Move;
pub use r#return::Return;
pub use set_local::SetLocal;
pub use subtract::Subtract;
pub use test::Test;
pub use test_set::TestSet;

use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display, Formatter};

use crate::NativeFunction;

/// An operation and its arguments for the Dust virtual machine.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Instruction {
    pub metadata: u8,
    pub a: u8,
    pub b: u8,
    pub c: u8,
}

impl Instruction {
    pub fn operation(&self) -> Operation {
        let operation_bits = self.metadata & 0b0001_1111;

        Operation::from(operation_bits)
    }

    pub fn d(&self) -> bool {
        self.metadata & 0b1000_0000 != 0
    }

    pub fn r#move(from: u8, to: u8) -> Instruction {
        Instruction::from(Move { from, to })
    }

    pub fn close(from: u8, to: u8) -> Instruction {
        Instruction::from(Close { from, to })
    }

    pub fn load_boolean(destination: u8, value: bool, jump_next: bool) -> Instruction {
        Instruction::from(LoadBoolean {
            destination,
            value,
            jump_next,
        })
    }

    pub fn load_constant(destination: u8, constant_index: u8, jump_next: bool) -> Instruction {
        Instruction::from(LoadConstant {
            destination,
            constant_index,
            jump_next,
        })
    }

    pub fn load_list(destination: u8, start_register: u8) -> Instruction {
        Instruction::from(LoadList {
            destination,
            start_register,
        })
    }

    pub fn load_self(destination: u8) -> Instruction {
        Instruction::from(LoadSelf { destination })
    }

    pub fn get_local(destination: u8, local_index: u8) -> Instruction {
        Instruction::from(GetLocal {
            destination,
            local_index,
        })
    }

    pub fn set_local(register: u8, local_index: u8) -> Instruction {
        Instruction::from(SetLocal {
            local_index,
            register_index: register,
        })
    }

    pub fn add(destination: u8, left: Argument, right: Argument) -> Instruction {
        Instruction::from(Add {
            destination,
            left,
            right,
        })
    }

    pub fn subtract(destination: u8, left: Argument, right: Argument) -> Instruction {
        Instruction::from(Subtract {
            destination,
            left,
            right,
        })
    }

    pub fn multiply(destination: u8, left: Argument, right: Argument) -> Instruction {
        Instruction::from(Multiply {
            destination,
            left,
            right,
        })
    }

    pub fn divide(destination: u8, left: Argument, right: Argument) -> Instruction {
        Instruction::from(Divide {
            destination,
            left,
            right,
        })
    }

    pub fn modulo(destination: u8, left: Argument, right: Argument) -> Instruction {
        Instruction::from(Modulo {
            destination,
            left,
            right,
        })
    }

    pub fn test(argument: Argument, value: bool) -> Instruction {
        Instruction::from(Test {
            argument,
            test_value: value,
        })
    }

    pub fn test_set(destination: u8, argument: Argument, value: bool) -> Instruction {
        Instruction::from(TestSet {
            destination,
            argument,
            test_value: value,
        })
    }

    pub fn equal(destination: u8, value: bool, left: Argument, right: Argument) -> Instruction {
        Instruction::from(Equal {
            destination,
            value,
            left,
            right,
        })
    }

    pub fn less(destination: u8, value: bool, left: Argument, right: Argument) -> Instruction {
        Instruction::from(Less {
            destination,
            value,
            left,
            right,
        })
    }

    pub fn less_equal(
        destination: u8,
        value: bool,
        left: Argument,
        right: Argument,
    ) -> Instruction {
        Instruction::from(LessEqual {
            destination,
            value,
            left,
            right,
        })
    }

    pub fn negate(destination: u8, argument: Argument) -> Instruction {
        Instruction::from(Negate {
            destination,
            argument,
        })
    }

    pub fn not(destination: u8, argument: Argument) -> Instruction {
        Instruction::from(Not {
            destination,
            argument,
        })
    }

    pub fn jump(offset: u8, is_positive: bool) -> Instruction {
        Instruction::from(Jump {
            offset,
            is_positive,
        })
    }

    pub fn call(destination: u8, function: Argument, argument_count: u8) -> Instruction {
        Instruction::from(Call {
            destination,
            function,
            argument_count,
        })
    }

    pub fn call_native(
        destination: u8,
        function: NativeFunction,
        argument_count: u8,
    ) -> Instruction {
        Instruction::from(CallNative {
            destination,
            function,
            argument_count,
        })
    }

    pub fn r#return(should_return_value: bool) -> Instruction {
        Instruction::from(Return {
            should_return_value,
        })
    }

    pub fn as_argument(&self) -> Option<Argument> {
        match self.operation() {
            Operation::LoadConstant => Some(Argument::Constant(self.b)),
            Operation::LoadBoolean
            | Operation::LoadList
            | Operation::LoadSelf
            | Operation::GetLocal
            | Operation::Add
            | Operation::Subtract
            | Operation::Multiply
            | Operation::Divide
            | Operation::Modulo
            | Operation::Negate
            | Operation::Not
            | Operation::Call
            | Operation::CallNative => Some(Argument::Register(self.a)),
            _ => None,
        }
    }

    pub fn b_as_argument(&self) -> Argument {
        let b_is_constant = self.metadata & 0b0010_0000 != 0;

        if b_is_constant {
            Argument::Constant(self.b)
        } else {
            Argument::Register(self.b)
        }
    }

    pub fn b_and_c_as_arguments(&self) -> (Argument, Argument) {
        let b_is_constant = self.metadata & 0b0010_0000 != 0;
        let c_is_constant = self.metadata & 0b0100_0000 != 0;

        let left = if b_is_constant {
            Argument::Constant(self.b)
        } else {
            Argument::Register(self.b)
        };
        let right = if c_is_constant {
            Argument::Constant(self.c)
        } else {
            Argument::Register(self.c)
        };

        (left, right)
    }

    pub fn yields_value(&self) -> bool {
        match self.operation() {
            Operation::LoadBoolean
            | Operation::LoadConstant
            | Operation::LoadList
            | Operation::LoadMap
            | Operation::LoadSelf
            | Operation::GetLocal
            | Operation::Add
            | Operation::Subtract
            | Operation::Multiply
            | Operation::Divide
            | Operation::Modulo
            | Operation::Power
            | Operation::Negate
            | Operation::Not
            | Operation::Equal
            | Operation::Less
            | Operation::LessEqual
            | Operation::Call => true,
            Operation::CallNative => {
                let function = NativeFunction::from(self.b);

                function.returns_value()
            }
            Operation::Move
            | Operation::Close
            | Operation::SetLocal
            | Operation::Test
            | Operation::TestSet
            | Operation::Jump
            | Operation::Return => false,
        }
    }

    pub fn disassembly_info(&self) -> String {
        match self.operation() {
            Operation::Move => {
                let Move { from, to } = Move::from(self);

                format!("R{to} = R{from}")
            }
            Operation::Close => {
                let Close { from, to } = Close::from(self);

                format!("R{from}..R{to}")
            }
            Operation::LoadBoolean => {
                let LoadBoolean {
                    destination,
                    value,
                    jump_next,
                } = LoadBoolean::from(self);

                if jump_next {
                    format!("R{destination} = {value} && JUMP +1")
                } else {
                    format!("R{destination} = {value}")
                }
            }
            Operation::LoadConstant => {
                let LoadConstant {
                    destination,
                    constant_index,
                    jump_next,
                } = LoadConstant::from(self);

                if jump_next {
                    format!("R{destination} = C{constant_index} JUMP +1")
                } else {
                    format!("R{destination} = C{constant_index}")
                }
            }
            Operation::LoadList => {
                let LoadList {
                    destination,
                    start_register,
                } = LoadList::from(self);
                let end_register = destination.saturating_sub(1);

                format!("R{destination} = [R{start_register}..=R{end_register}]",)
            }
            Operation::LoadSelf => {
                let LoadSelf { destination } = LoadSelf::from(self);

                format!("R{destination} = self")
            }
            Operation::GetLocal => {
                let GetLocal {
                    destination,
                    local_index,
                } = GetLocal::from(self);

                format!("R{destination} = L{local_index}")
            }
            Operation::SetLocal => {
                let SetLocal {
                    register_index,
                    local_index,
                } = SetLocal::from(self);

                format!("L{local_index} = R{register_index}")
            }
            Operation::Add => {
                let Add {
                    destination,
                    left,
                    right,
                } = Add::from(self);

                format!("R{destination} = {left} + {right}")
            }
            Operation::Subtract => {
                let Subtract {
                    destination,
                    left,
                    right,
                } = Subtract::from(self);

                format!("R{destination} = {left} - {right}")
            }
            Operation::Multiply => {
                let Multiply {
                    destination,
                    left,
                    right,
                } = Multiply::from(self);

                format!("R{destination} = {left} * {right}")
            }
            Operation::Divide => {
                let Divide {
                    destination,
                    left,
                    right,
                } = Divide::from(self);

                format!("R{destination} = {left} / {right}")
            }
            Operation::Modulo => {
                let Modulo {
                    destination,
                    left,
                    right,
                } = Modulo::from(self);

                format!("R{destination} = {left} % {right}")
            }
            Operation::Test => {
                let Test {
                    argument,
                    test_value: value,
                } = Test::from(self);
                let bang = if value { "" } else { "!" };

                format!("if {bang}{argument} {{ JUMP +1 }}",)
            }
            Operation::TestSet => {
                let TestSet {
                    destination,
                    argument,
                    test_value: value,
                } = TestSet::from(self);
                let bang = if value { "" } else { "!" };

                format!("if {bang}{argument} {{ JUMP +1 }} else {{ R{destination} = {argument} }}")
            }
            Operation::Equal => {
                let Equal {
                    destination,
                    value,
                    left,
                    right,
                } = Equal::from(self);
                let comparison_symbol = if value { "==" } else { "!=" };

                format!("R{destination} = {left} {comparison_symbol} {right}")
            }
            Operation::Less => {
                let Less {
                    destination,
                    value,
                    left,
                    right,
                } = Less::from(self);
                let comparison_symbol = if value { "<" } else { ">=" };

                format!("R{destination} = {left} {comparison_symbol} {right}")
            }
            Operation::LessEqual => {
                let LessEqual {
                    destination,
                    value,
                    left,
                    right,
                } = LessEqual::from(self);
                let comparison_symbol = if value { "<=" } else { ">" };

                format!("R{destination} = {left} {comparison_symbol} {right}")
            }
            Operation::Negate => {
                let Negate {
                    destination,
                    argument,
                } = Negate::from(self);

                format!("R{destination} = -{argument}")
            }
            Operation::Not => {
                let Not {
                    destination,
                    argument,
                } = Not::from(self);

                format!("R{destination} = !{argument}")
            }
            Operation::Jump => {
                let Jump {
                    offset,
                    is_positive,
                } = Jump::from(self);

                if is_positive {
                    format!("JUMP +{offset}")
                } else {
                    format!("JUMP -{offset}")
                }
            }
            Operation::Call => {
                let Call {
                    destination,
                    function,
                    argument_count,
                } = Call::from(self);
                let arguments_start = destination.saturating_sub(argument_count);
                let arguments_end = arguments_start + argument_count;

                format!("R{destination} = {function}(R{arguments_start}..R{arguments_end})")
            }
            Operation::CallNative => {
                let CallNative {
                    destination,
                    function,
                    argument_count,
                } = CallNative::from(self);
                let arguments_start = destination.saturating_sub(argument_count);
                let arguments_end = arguments_start + argument_count;

                if function.returns_value() {
                    format!("R{destination} = {function}(R{arguments_start}..R{arguments_end})")
                } else {
                    format!("{function}(R{arguments_start}..R{arguments_end})")
                }
            }
            Operation::Return => {
                let Return {
                    should_return_value,
                } = Return::from(self);

                if should_return_value {
                    "RETURN".to_string()
                } else {
                    "".to_string()
                }
            }
            _ => {
                if cfg!(debug_assertions) {
                    panic!("Unknown operation {}", self.operation());
                } else {
                    "RETURN".to_string()
                }
            }
        }
    }
}

impl Debug for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.operation(), self.disassembly_info())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Argument {
    Constant(u8),
    Register(u8),
}

impl Argument {
    pub fn index(&self) -> u8 {
        match self {
            Argument::Constant(index) => *index,
            Argument::Register(index) => *index,
        }
    }

    pub fn is_constant(&self) -> bool {
        matches!(self, Argument::Constant(_))
    }

    pub fn is_register(&self) -> bool {
        matches!(self, Argument::Register(_))
    }

    pub fn as_index_and_b_options(&self) -> (u8, InstructionOptions) {
        match self {
            Argument::Constant(index) => (*index, InstructionOptions::B_IS_CONSTANT),
            Argument::Register(index) => (*index, InstructionOptions::empty()),
        }
    }

    pub fn as_index_and_c_options(&self) -> (u8, InstructionOptions) {
        match self {
            Argument::Constant(index) => (*index, InstructionOptions::C_IS_CONSTANT),
            Argument::Register(index) => (*index, InstructionOptions::empty()),
        }
    }
}

impl Display for Argument {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Argument::Constant(index) => write!(f, "C{index}"),
            Argument::Register(index) => write!(f, "R{index}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem::offset_of;

    use super::*;

    #[test]
    fn instruction_is_4_bytes() {
        assert_eq!(size_of::<Instruction>(), 4);
    }

    #[test]
    fn instruction_layout() {
        assert_eq!(offset_of!(Instruction, metadata), 0);
        assert_eq!(offset_of!(Instruction, a), 1);
        assert_eq!(offset_of!(Instruction, b), 2);
        assert_eq!(offset_of!(Instruction, c), 3);
    }
}
