//! An operation and its arguments for the Dust virtual machine.
//!
//! Each instruction is a 64-bit unsigned integer that is divided into nine fields:
//!
//! Bit   | Description
//! ----- | -----------
//! 0-8   | The operation code.
//! 9     | Boolean flag indicating whether the B argument is a constant.
//! 10    | Boolean flag indicating whether the C argument is a constant.
//! 11    | Boolean flag indicating whether the A argument is a local.
//! 12    | Boolean flag indicating whether the B argument is a local.
//! 13    | Boolean flag indicating whether the C argument is a local.
//! 17-32 | The A argument,
//! 33-48 | The B argument.
//! 49-63 | The C argument.
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
mod define_local;
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
mod r#return;
mod set_local;
mod subtract;
mod test;
mod test_set;

use std::fmt::{self, Debug, Display, Formatter};

pub use add::Add;
pub use call::Call;
pub use call_native::CallNative;
pub use close::Close;
pub use define_local::DefineLocal;
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
pub use r#move::Move;
pub use r#return::Return;
pub use set_local::SetLocal;
pub use subtract::Subtract;
pub use test::Test;
pub use test_set::TestSet;

use serde::{Deserialize, Serialize};

use crate::{NativeFunction, Operation};

/// An operation and its arguments for the Dust virtual machine.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Instruction(u64);

impl Instruction {
    pub fn new(operation: Operation) -> Instruction {
        Instruction(operation as u64)
    }

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

    pub fn define_local(register: u16, local_index: u16, is_mutable: bool) -> Instruction {
        Instruction::from(DefineLocal {
            local_index,
            register,
            is_mutable,
        })
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
            register,
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

    pub fn equal(value: bool, left: Argument, right: Argument) -> Instruction {
        Instruction::from(Equal { value, left, right })
    }

    pub fn less(value: bool, left: Argument, right: Argument) -> Instruction {
        Instruction::from(Less { value, left, right })
    }

    pub fn less_equal(value: bool, left: Argument, right: Argument) -> Instruction {
        Instruction::from(LessEqual { value, left, right })
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
        match self.operation() {
            Operation::LoadConstant => Some(Argument::Constant(self.b())),
            Operation::GetLocal => Some(Argument::Local(self.b())),
            Operation::LoadBoolean
            | Operation::LoadList
            | Operation::LoadSelf
            | Operation::Add
            | Operation::Subtract
            | Operation::Multiply
            | Operation::Divide
            | Operation::Modulo
            | Operation::Negate
            | Operation::Not
            | Operation::Call
            | Operation::CallNative => Some(Argument::Register(self.a())),
            _ => None,
        }
    }

    pub fn b_as_argument(&self) -> Argument {
        if self.b_is_constant() {
            Argument::Constant(self.b())
        } else if self.b_is_local() {
            Argument::Local(self.b())
        } else {
            Argument::Register(self.b())
        }
    }

    pub fn b_and_c_as_arguments(&self) -> (Argument, Argument) {
        let left = if self.b_is_constant() {
            Argument::Constant(self.b())
        } else if self.b_is_local() {
            Argument::Local(self.b())
        } else {
            Argument::Register(self.b())
        };
        let right = if self.c_is_constant() {
            Argument::Constant(self.c())
        } else if self.c_is_local() {
            Argument::Local(self.c())
        } else {
            Argument::Register(self.c())
        };

        (left, right)
    }

    pub fn operation(&self) -> Operation {
        Operation::from((self.0 & 0b11111111) as u8)
    }

    pub fn set_b_is_constant(&mut self, boolean: bool) -> &mut Self {
        self.0 = (self.0 & !(1 << 9)) | ((boolean as u64) << 9);

        self
    }

    pub fn set_c_is_constant(&mut self, boolean: bool) -> &mut Self {
        self.0 = (self.0 & !(1 << 10)) | ((boolean as u64) << 10);

        self
    }

    pub fn set_a_is_local(&mut self, boolean: bool) -> &mut Self {
        self.0 = (self.0 & !(1 << 11)) | ((boolean as u64) << 11);

        self
    }

    pub fn set_b_is_local(&mut self, boolean: bool) -> &mut Self {
        self.0 = (self.0 & !(1 << 12)) | ((boolean as u64) << 12);

        self
    }

    pub fn set_c_is_local(&mut self, boolean: bool) -> &mut Self {
        self.0 = (self.0 & !(1 << 13)) | ((boolean as u64) << 13);

        self
    }

    pub fn a(&self) -> u16 {
        ((self.0 >> 16) & 0b1111111111111111) as u16
    }

    pub fn a_as_boolean(&self) -> bool {
        self.a() != 0
    }

    pub fn set_a(&mut self, a: u16) -> &mut Self {
        self.0 = (self.0 & !(0b1111111111111111 << 16)) | ((a as u64) << 16);

        self
    }

    pub fn set_a_to_boolean(&mut self, boolean: bool) -> &mut Self {
        self.0 = (self.0 & !(0b1111111111111111 << 16)) | ((boolean as u64) << 16);

        self
    }

    pub fn b(&self) -> u16 {
        ((self.0 >> 32) & 0b1111111111111111) as u16
    }

    pub fn b_as_boolean(&self) -> bool {
        self.b() != 0
    }

    pub fn set_b(&mut self, b: u16) -> &mut Self {
        self.0 = (self.0 & !(0b1111111111111111 << 32)) | ((b as u64) << 32);

        self
    }

    pub fn set_b_to_boolean(&mut self, boolean: bool) -> &mut Self {
        self.0 = (self.0 & !(0b1111111111111111 << 32)) | ((boolean as u64) << 32);

        self
    }

    pub fn c(&self) -> u16 {
        ((self.0 >> 48) & 0b1111111111111111) as u16
    }

    pub fn c_as_boolean(&self) -> bool {
        self.c() != 0
    }

    pub fn set_c(&mut self, c: u16) -> &mut Self {
        self.0 = (self.0 & !(0b1111111111111111 << 48)) | ((c as u64) << 48);

        self
    }

    pub fn set_c_to_boolean(&mut self, boolean: bool) -> &mut Self {
        self.0 = (self.0 & !(0b1111111111111111 << 48)) | ((boolean as u64) << 48);

        self
    }

    pub fn b_is_constant(&self) -> bool {
        (self.0 >> 9) & 1 == 1
    }

    pub fn c_is_constant(&self) -> bool {
        (self.0 >> 10) & 1 == 1
    }

    pub fn a_is_local(&self) -> bool {
        (self.0 >> 11) & 1 == 1
    }

    pub fn b_is_local(&self) -> bool {
        (self.0 >> 12) & 1 == 1
    }

    pub fn c_is_local(&self) -> bool {
        (self.0 >> 13) & 1 == 1
    }

    pub fn yields_value(&self) -> bool {
        match self.operation() {
            Operation::LoadBoolean
            | Operation::LoadConstant
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
            | Operation::Call => true,
            Operation::Move
            | Operation::Close
            | Operation::DefineLocal
            | Operation::SetLocal
            | Operation::Equal
            | Operation::Less
            | Operation::LessEqual
            | Operation::Test
            | Operation::TestSet
            | Operation::Jump
            | Operation::Return => false,
            Operation::CallNative => {
                let function = NativeFunction::from(self.b());

                function.returns_value()
            }
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
                    format!("{destination} = {value} && JUMP +1")
                } else {
                    format!("{destination} = {value}")
                }
            }
            Operation::LoadConstant => {
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
            Operation::LoadList => {
                let LoadList {
                    destination,
                    start_register,
                } = LoadList::from(self);
                let end_register = destination.index().saturating_sub(1);

                format!("{destination} = [R{start_register}..=R{end_register}]",)
            }
            Operation::LoadSelf => {
                let LoadSelf { destination } = LoadSelf::from(self);

                format!("{destination} = self")
            }
            Operation::DefineLocal => {
                let DefineLocal {
                    register,
                    local_index,
                    is_mutable,
                } = DefineLocal::from(self);

                if is_mutable {
                    format!("mut L{local_index} = R{register}")
                } else {
                    format!("L{local_index} = R{register}")
                }
            }
            Operation::GetLocal => {
                let GetLocal {
                    destination,
                    local_index,
                } = GetLocal::from(self);

                format!("{destination} = L{local_index}")
            }
            Operation::SetLocal => {
                let SetLocal {
                    register,
                    local_index,
                } = SetLocal::from(self);

                format!("L{local_index} = R{register}")
            }
            Operation::Add => {
                let Add {
                    destination,
                    left,
                    right,
                } = Add::from(self);

                format!("{destination} = {left} + {right}")
            }
            Operation::Subtract => {
                let Subtract {
                    destination,
                    left,
                    right,
                } = Subtract::from(self);

                format!("{destination} = {left} - {right}")
            }
            Operation::Multiply => {
                let Multiply {
                    destination,
                    left,
                    right,
                } = Multiply::from(self);

                format!("{destination} = {left} * {right}")
            }
            Operation::Divide => {
                let Divide {
                    destination,
                    left,
                    right,
                } = Divide::from(self);

                format!("{destination} = {left} / {right}")
            }
            Operation::Modulo => {
                let Modulo {
                    destination,
                    left,
                    right,
                } = Modulo::from(self);

                format!("{destination} = {left} % {right}")
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

                format!("if {bang}{argument} {{ JUMP +1 }} else {{ {destination} = {argument} }}")
            }
            Operation::Equal => {
                let Equal { value, left, right } = Equal::from(self);
                let comparison_symbol = if value { "==" } else { "!=" };

                format!("if {left} {comparison_symbol} {right} {{ JUMP +1 }}")
            }
            Operation::Less => {
                let Less { value, left, right } = Less::from(self);
                let comparison_symbol = if value { "<" } else { ">=" };

                format!("if {left} {comparison_symbol} {right} {{ JUMP +1 }}")
            }
            Operation::LessEqual => {
                let LessEqual { value, left, right } = LessEqual::from(self);
                let comparison_symbol = if value { "<=" } else { ">" };

                format!("if {left} {comparison_symbol} {right} {{ JUMP +1 }}")
            }
            Operation::Negate => {
                let Negate {
                    destination,
                    argument,
                } = Negate::from(self);

                format!("{destination} = -{argument}")
            }
            Operation::Not => {
                let Not {
                    destination,
                    argument,
                } = Not::from(self);

                format!("{destination} = !{argument}")
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
                let arguments_start = destination.index().saturating_sub(argument_count);
                let arguments_end = arguments_start + argument_count;

                format!("{destination} = {function}(R{arguments_start}..R{arguments_end})")
            }
            Operation::CallNative => {
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
        }
    }
}

impl From<&Instruction> for u64 {
    fn from(instruction: &Instruction) -> Self {
        instruction.0
    }
}

impl Debug for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.operation(), self.disassembly_info())
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
