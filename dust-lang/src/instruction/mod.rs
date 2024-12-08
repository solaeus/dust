//! An operation and its arguments for the Dust virtual machine.
//!
//! Each instruction is 64 bits and holds ten distinct fields:
//!
//! Bit   | Description
//! ----- | -----------
//! 0-8   | Operation code
//! 9     | Flag for whether A is a local
//! 10    | Flag for whether B argument is a constant
//! 11    | Flag for whether C argument is a local
//! 12    | Flag for whether B argument is a constant
//! 13    | Flag for whether C argument is a local
//! 14    | D Argument
//! 15-16 | Unused
//! 17-32 | A argument
//! 33-48 | B argument
//! 49-63 | C argument
//!
//! Be careful when working with instructions directly. When modifying an instruction, be sure to
//! account for the fact that setting the A, B, or C arguments to 0 will have no effect. It is
//! usually best to remove instructions and insert new ones in their place instead of mutating them.
//!
//! For each operation, there are two ways to create an instruction:
//!
//! - Use the associated function on `Instruction`
//! - Use the corresponding struct and call `Instruction::from`
//!
//! # Examples
//!
//! ```
//! # use dust_lang::instruction::{Instruction, Move};
//! let move_1 = Instruction::r#move(42, 4);
//! let move_2 = Instruction::from(Move { from: 42, to: 4 });
//!
//! assert_eq!(move_1, move_2);
//! ```
//!
//! Use the `Destination` and `Argument` enums to create instructions. This is easier to read and
//! enforces consistency in how the `Instruction` methods are called.
//!
//! ```
//! # use dust_lang::instruction::{Instruction, Add, Destination, Argument};
//! let add_1 = Instruction::add(
//!     Destination::Register(0),
//!     Argument::Local(1),
//!     Argument::Constant(2)
//! );
//! let add_2 = Instruction::from(Add {
//!     destination: Destination::Register(0),
//!     left: Argument::Local(1),
//!     right: Argument::Constant(2),
//! });
//!
//! assert_eq!(add_1, add_2);
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
    operation: Operation,
    options: InstructionOptions,
    a: u16,
    b: u16,
    c: u16,
}

impl Instruction {
    pub fn r#move(from: u16, to: u16) -> Instruction {
        Instruction::from(Move { from, to })
    }

    pub fn close(from: u16, to: u16) -> Instruction {
        Instruction::from(Close { from, to })
    }

    pub fn load_boolean(destination: Destination, value: bool, jump_next: bool) -> Instruction {
        Instruction::from(LoadBoolean {
            destination,
            value,
            jump_next,
        })
    }

    pub fn load_constant(
        destination: Destination,
        constant_index: u16,
        jump_next: bool,
    ) -> Instruction {
        Instruction::from(LoadConstant {
            destination,
            constant_index,
            jump_next,
        })
    }

    pub fn load_list(destination: Destination, start_register: u16) -> Instruction {
        Instruction::from(LoadList {
            destination,
            start_register,
        })
    }

    pub fn load_self(destination: Destination) -> Instruction {
        Instruction::from(LoadSelf { destination })
    }

    pub fn get_local(destination: Destination, local_index: u16) -> Instruction {
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

    pub fn add(destination: Destination, left: Argument, right: Argument) -> Instruction {
        Instruction::from(Add {
            destination,
            left,
            right,
        })
    }

    pub fn subtract(destination: Destination, left: Argument, right: Argument) -> Instruction {
        Instruction::from(Subtract {
            destination,
            left,
            right,
        })
    }

    pub fn multiply(destination: Destination, left: Argument, right: Argument) -> Instruction {
        Instruction::from(Multiply {
            destination,
            left,
            right,
        })
    }

    pub fn divide(destination: Destination, left: Argument, right: Argument) -> Instruction {
        Instruction::from(Divide {
            destination,
            left,
            right,
        })
    }

    pub fn modulo(destination: Destination, left: Argument, right: Argument) -> Instruction {
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

    pub fn test_set(destination: Destination, argument: Argument, value: bool) -> Instruction {
        Instruction::from(TestSet {
            destination,
            argument,
            test_value: value,
        })
    }

    pub fn equal(
        destination: Destination,
        value: bool,
        left: Argument,
        right: Argument,
    ) -> Instruction {
        Instruction::from(Equal {
            destination,
            value,
            left,
            right,
        })
    }

    pub fn less(
        destination: Destination,
        value: bool,
        left: Argument,
        right: Argument,
    ) -> Instruction {
        Instruction::from(Less {
            destination,
            value,
            left,
            right,
        })
    }

    pub fn less_equal(
        destination: Destination,
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

    pub fn negate(destination: Destination, argument: Argument) -> Instruction {
        Instruction::from(Negate {
            destination,
            argument,
        })
    }

    pub fn not(destination: Destination, argument: Argument) -> Instruction {
        Instruction::from(Not {
            destination,
            argument,
        })
    }

    pub fn jump(offset: u16, is_positive: bool) -> Instruction {
        Instruction::from(Jump {
            offset,
            is_positive,
        })
    }

    pub fn call(destination: Destination, function: Argument, argument_count: u16) -> Instruction {
        Instruction::from(Call {
            destination,
            function,
            argument_count,
        })
    }

    pub fn call_native(
        destination: Destination,
        function: NativeFunction,
        argument_count: u16,
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

    pub fn destination_as_argument(&self) -> Option<Argument> {
        match self.operation {
            Operation::LOAD_CONSTANT => Some(Argument::Constant(self.b)),
            Operation::GET_LOCAL => Some(Argument::Local(self.b)),
            Operation::LOAD_BOOLEAN
            | Operation::LOAD_LIST
            | Operation::LOAD_SELF
            | Operation::ADD
            | Operation::SUBTRACT
            | Operation::MULTIPLY
            | Operation::DIVIDE
            | Operation::MODULO
            | Operation::NEGATE
            | Operation::NOT
            | Operation::CALL
            | Operation::CALL_NATIVE => Some(Argument::Register(self.a)),
            _ => None,
        }
    }

    pub fn a_as_destination(&self) -> Destination {
        if self.options.a_is_local() {
            Destination::Local(self.a)
        } else {
            Destination::Register(self.a)
        }
    }

    pub fn b_as_argument(&self) -> Argument {
        if self.options.b_is_constant() {
            Argument::Constant(self.b)
        } else if self.options.b_is_local() {
            Argument::Local(self.b)
        } else {
            Argument::Register(self.b)
        }
    }

    pub fn b_and_c_as_arguments(&self) -> (Argument, Argument) {
        let left = if self.options.b_is_constant() {
            Argument::Constant(self.b)
        } else if self.options.b_is_local() {
            Argument::Local(self.b)
        } else {
            Argument::Register(self.b)
        };
        let right = if self.options.c_is_constant() {
            Argument::Constant(self.c)
        } else if self.options.c_is_local() {
            Argument::Local(self.c)
        } else {
            Argument::Register(self.c)
        };

        (left, right)
    }

    pub fn yields_value(&self) -> bool {
        match self.operation {
            Operation::LOAD_BOOLEAN
            | Operation::LOAD_CONSTANT
            | Operation::LOAD_LIST
            | Operation::LOAD_SELF
            | Operation::GET_LOCAL
            | Operation::ADD
            | Operation::SUBTRACT
            | Operation::MULTIPLY
            | Operation::DIVIDE
            | Operation::MODULO
            | Operation::NEGATE
            | Operation::NOT
            | Operation::CALL => true,
            Operation::MOVE
            | Operation::CLOSE
            | Operation::SET_LOCAL
            | Operation::EQUAL
            | Operation::LESS
            | Operation::LESS_EQUAL
            | Operation::TEST
            | Operation::TEST_SET
            | Operation::JUMP
            | Operation::RETURN => false,
            Operation::CALL_NATIVE => {
                let function = NativeFunction::from(self.b);

                function.returns_value()
            }
            _ => {
                if cfg!(debug_assertions) {
                    panic!("Unknown operation {}", self.operation);
                } else {
                    false
                }
            }
        }
    }

    pub fn disassembly_info(&self) -> String {
        match self.operation {
            Operation::MOVE => {
                let Move { from, to } = Move::from(self);

                format!("R{to} = R{from}")
            }
            Operation::CLOSE => {
                let Close { from, to } = Close::from(self);

                format!("R{from}..R{to}")
            }
            Operation::LOAD_BOOLEAN => {
                let LoadBoolean {
                    destination,
                    value,
                    jump_next,
                } = LoadBoolean::from(self);

                if jump_next {
                    format!("{destination} = {value} && JUMP +1")
                } else {
                    format!("{destination} = {value}")
                }
            }
            Operation::LOAD_CONSTANT => {
                let LoadConstant {
                    destination,
                    constant_index,
                    jump_next,
                } = LoadConstant::from(self);

                if jump_next {
                    format!("{destination} = C{constant_index} JUMP +1")
                } else {
                    format!("{destination} = C{constant_index}")
                }
            }
            Operation::LOAD_LIST => {
                let LoadList {
                    destination,
                    start_register,
                } = LoadList::from(self);
                let end_register = destination.index().saturating_sub(1);

                format!("{destination} = [R{start_register}..=R{end_register}]",)
            }
            Operation::LOAD_SELF => {
                let LoadSelf { destination } = LoadSelf::from(self);

                format!("{destination} = self")
            }
            Operation::GET_LOCAL => {
                let GetLocal {
                    destination,
                    local_index,
                } = GetLocal::from(self);

                format!("{destination} = L{local_index}")
            }
            Operation::SET_LOCAL => {
                let SetLocal {
                    register_index: register,
                    local_index,
                } = SetLocal::from(self);

                format!("L{local_index} = R{register}")
            }
            Operation::ADD => {
                let Add {
                    destination,
                    left,
                    right,
                } = Add::from(self);

                format!("{destination} = {left} + {right}")
            }
            Operation::SUBTRACT => {
                let Subtract {
                    destination,
                    left,
                    right,
                } = Subtract::from(self);

                format!("{destination} = {left} - {right}")
            }
            Operation::MULTIPLY => {
                let Multiply {
                    destination,
                    left,
                    right,
                } = Multiply::from(self);

                format!("{destination} = {left} * {right}")
            }
            Operation::DIVIDE => {
                let Divide {
                    destination,
                    left,
                    right,
                } = Divide::from(self);

                format!("{destination} = {left} / {right}")
            }
            Operation::MODULO => {
                let Modulo {
                    destination,
                    left,
                    right,
                } = Modulo::from(self);

                format!("{destination} = {left} % {right}")
            }
            Operation::TEST => {
                let Test {
                    argument,
                    test_value: value,
                } = Test::from(self);
                let bang = if value { "" } else { "!" };

                format!("if {bang}{argument} {{ JUMP +1 }}",)
            }
            Operation::TEST_SET => {
                let TestSet {
                    destination,
                    argument,
                    test_value: value,
                } = TestSet::from(self);
                let bang = if value { "" } else { "!" };

                format!("if {bang}{argument} {{ JUMP +1 }} else {{ {destination} = {argument} }}")
            }
            Operation::EQUAL => {
                let Equal {
                    destination,
                    value,
                    left,
                    right,
                } = Equal::from(self);
                let comparison_symbol = if value { "==" } else { "!=" };

                format!("{destination} = {left} {comparison_symbol} {right}")
            }
            Operation::LESS => {
                let Less {
                    destination,
                    value,
                    left,
                    right,
                } = Less::from(self);
                let comparison_symbol = if value { "<" } else { ">=" };

                format!("{destination} {left} {comparison_symbol} {right}")
            }
            Operation::LESS_EQUAL => {
                let LessEqual {
                    destination,
                    value,
                    left,
                    right,
                } = LessEqual::from(self);
                let comparison_symbol = if value { "<=" } else { ">" };

                format!("{destination} {left} {comparison_symbol} {right}")
            }
            Operation::NEGATE => {
                let Negate {
                    destination,
                    argument,
                } = Negate::from(self);

                format!("{destination} = -{argument}")
            }
            Operation::NOT => {
                let Not {
                    destination,
                    argument,
                } = Not::from(self);

                format!("{destination} = !{argument}")
            }
            Operation::JUMP => {
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
            Operation::CALL => {
                let Call {
                    destination,
                    function,
                    argument_count,
                } = Call::from(self);
                let arguments_start = destination.index().saturating_sub(argument_count);
                let arguments_end = arguments_start + argument_count;

                format!("{destination} = {function}(R{arguments_start}..R{arguments_end})")
            }
            Operation::CALL_NATIVE => {
                let CallNative {
                    destination,
                    function,
                    argument_count,
                } = CallNative::from(self);
                let arguments_start = destination.index().saturating_sub(argument_count);
                let arguments_end = arguments_start + argument_count;

                if function.returns_value() {
                    format!("{destination} = {function}(R{arguments_start}..R{arguments_end})")
                } else {
                    format!("{function}(R{arguments_start}..R{arguments_end})")
                }
            }
            Operation::RETURN => {
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
                    panic!("Unknown operation {}", self.operation);
                } else {
                    "RETURN".to_string()
                }
            }
        }
    }
}

impl Debug for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.operation, self.disassembly_info())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Destination {
    Local(u16),
    Register(u16),
}

impl Destination {
    pub fn index(&self) -> u16 {
        match self {
            Destination::Local(index) => *index,
            Destination::Register(index) => *index,
        }
    }

    pub fn is_local(&self) -> bool {
        matches!(self, Destination::Local(_))
    }

    pub fn is_register(&self) -> bool {
        matches!(self, Destination::Register(_))
    }

    pub fn as_index_and_a_options(&self) -> (u16, InstructionOptions) {
        match self {
            Destination::Local(index) => (*index, InstructionOptions::A_IS_LOCAL),
            Destination::Register(index) => (*index, InstructionOptions::empty()),
        }
    }
}

impl Display for Destination {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Destination::Local(index) => write!(f, "L{index}"),
            Destination::Register(index) => write!(f, "R{index}"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Argument {
    Constant(u16),
    Local(u16),
    Register(u16),
}

impl Argument {
    pub fn index(&self) -> u16 {
        match self {
            Argument::Constant(index) => *index,
            Argument::Local(index) => *index,
            Argument::Register(index) => *index,
        }
    }

    pub fn is_constant(&self) -> bool {
        matches!(self, Argument::Constant(_))
    }

    pub fn is_local(&self) -> bool {
        matches!(self, Argument::Local(_))
    }

    pub fn is_register(&self) -> bool {
        matches!(self, Argument::Register(_))
    }

    pub fn as_index_and_b_options(&self) -> (u16, InstructionOptions) {
        match self {
            Argument::Constant(index) => (*index, InstructionOptions::B_IS_CONSTANT),
            Argument::Local(index) => (*index, InstructionOptions::B_IS_LOCAL),
            Argument::Register(index) => (*index, InstructionOptions::empty()),
        }
    }

    pub fn as_index_and_c_options(&self) -> (u16, InstructionOptions) {
        match self {
            Argument::Constant(index) => (*index, InstructionOptions::C_IS_CONSTANT),
            Argument::Local(index) => (*index, InstructionOptions::C_IS_LOCAL),
            Argument::Register(index) => (*index, InstructionOptions::empty()),
        }
    }
}

impl Display for Argument {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Argument::Constant(index) => write!(f, "C{index}"),
            Argument::Local(index) => write!(f, "L{index}"),
            Argument::Register(index) => write!(f, "R{index}"),
        }
    }
}
