//! An operation and its arguments for the Dust virtual machine.
//!
//! Each instruction is a 64-bit unsigned integer that is divided into five fields:
//! - Bits 0-8: The operation code.
//! - Bit 9: Boolean flag indicating whether the B argument is a constant.
//! - Bit 10: Boolean flag indicating whether the C argument is a constant.
//! - Bit 11: Boolean flag indicating whether the A argument is a local.
//! - Bit 12: Boolean flag indicating whether the B argument is a local.
//! - Bit 13: Boolean flag indicating whether the C argument is a local.
//! - Bits 17-32: The A argument,
//! - Bits 33-48: The B argument.
//! - Bits 49-63: The C argument.
//!
//! Be careful when working with instructions directly. When modifying an instruction, be sure to
//! account for the fact that setting the A, B, or C arguments to 0 will have no effect. It is
//! usually best to remove instructions and insert new ones in their place instead of mutating them.

use serde::{Deserialize, Serialize};

use crate::{Chunk, NativeFunction, Operation};

pub struct InstructionBuilder {
    operation: Operation,
    a: u16,
    b: u16,
    c: u16,
    b_is_constant: bool,
    c_is_constant: bool,
    a_is_local: bool,
    b_is_local: bool,
    c_is_local: bool,
}

impl InstructionBuilder {
    pub fn build(&self) -> Instruction {
        Instruction(
            (self.operation as u64)
                | ((self.b_is_constant as u64) << 9)
                | ((self.c_is_constant as u64) << 10)
                | ((self.a_is_local as u64) << 11)
                | ((self.b_is_local as u64) << 12)
                | ((self.c_is_local as u64) << 13)
                | ((self.a as u64) << 16)
                | ((self.b as u64) << 32)
                | ((self.c as u64) << 48),
        )
    }

    pub fn a(&mut self, a: u16) -> &mut Self {
        self.a = a;
        self
    }

    pub fn a_to_boolean(&mut self, a: bool) -> &mut Self {
        self.a = a as u16;
        self
    }

    pub fn b(&mut self, b: u16) -> &mut Self {
        self.b = b;
        self
    }

    pub fn b_to_boolean(&mut self, b: bool) -> &mut Self {
        self.b = b as u16;
        self
    }

    pub fn c(&mut self, c: u16) -> &mut Self {
        self.c = c;
        self
    }

    pub fn c_to_boolean(&mut self, c: bool) -> &mut Self {
        self.c = c as u16;
        self
    }

    pub fn b_is_constant(&mut self, b_is_constant: bool) -> &mut Self {
        self.b_is_constant = b_is_constant;
        self
    }

    pub fn c_is_constant(&mut self, c_is_constant: bool) -> &mut Self {
        self.c_is_constant = c_is_constant;
        self
    }

    pub fn a_is_local(&mut self, a_is_local: bool) -> &mut Self {
        self.a_is_local = a_is_local;
        self
    }

    pub fn b_is_local(&mut self, b_is_local: bool) -> &mut Self {
        self.b_is_local = b_is_local;
        self
    }

    pub fn c_is_local(&mut self, c_is_local: bool) -> &mut Self {
        self.c_is_local = c_is_local;
        self
    }
}

impl From<&Instruction> for InstructionBuilder {
    fn from(instruction: &Instruction) -> Self {
        InstructionBuilder {
            operation: instruction.operation(),
            a: instruction.a(),
            b: instruction.b(),
            c: instruction.c(),
            b_is_constant: instruction.b_is_constant(),
            c_is_constant: instruction.c_is_constant(),
            a_is_local: instruction.a_is_local(),
            b_is_local: instruction.b_is_local(),
            c_is_local: instruction.c_is_local(),
        }
    }
}

impl From<Instruction> for InstructionBuilder {
    fn from(instruction: Instruction) -> Self {
        InstructionBuilder {
            operation: instruction.operation(),
            a: instruction.a(),
            b: instruction.b(),
            c: instruction.c(),
            b_is_constant: instruction.b_is_constant(),
            c_is_constant: instruction.c_is_constant(),
            a_is_local: instruction.a_is_local(),
            b_is_local: instruction.b_is_local(),
            c_is_local: instruction.c_is_local(),
        }
    }
}

impl From<&mut Instruction> for InstructionBuilder {
    fn from(instruction: &mut Instruction) -> Self {
        InstructionBuilder {
            operation: instruction.operation(),
            a: instruction.a(),
            b: instruction.b(),
            c: instruction.c(),
            b_is_constant: instruction.b_is_constant(),
            c_is_constant: instruction.c_is_constant(),
            a_is_local: instruction.a_is_local(),
            b_is_local: instruction.b_is_local(),
            c_is_local: instruction.c_is_local(),
        }
    }
}

/// An operation and its arguments for the Dust virtual machine.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Instruction(u64);

impl Instruction {
    pub fn new(operation: Operation) -> Instruction {
        Instruction(operation as u64)
    }

    pub fn builder(operation: Operation) -> InstructionBuilder {
        InstructionBuilder {
            operation,
            a: 0,
            b: 0,
            c: 0,
            b_is_constant: false,
            c_is_constant: false,
            a_is_local: false,
            b_is_local: false,
            c_is_local: false,
        }
    }

    pub fn operation(&self) -> Operation {
        Operation::from((self.0 & 0b11111111) as u8)
    }

    pub fn set_b_is_constant(&mut self) -> &mut Self {
        self.0 = (self.0 & !(1 << 9)) | ((true as u64) << 9);

        self
    }

    pub fn set_c_is_constant(&mut self) -> &mut Self {
        self.0 = (self.0 & !(1 << 10)) | ((true as u64) << 10);

        self
    }

    pub fn set_a_is_local(&mut self) -> &mut Self {
        self.0 = (self.0 & !(1 << 11)) | ((true as u64) << 11);

        self
    }

    pub fn set_b_is_local(&mut self) -> &mut Self {
        self.0 = (self.0 & !(1 << 12)) | ((true as u64) << 12);

        self
    }

    pub fn set_c_is_local(&mut self) -> &mut Self {
        self.0 = (self.0 & !(1 << 13)) | ((true as u64) << 13);

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

    pub fn r#move(to_register: u16, from_register: u16) -> Instruction {
        *Instruction::new(Operation::Move)
            .set_b(to_register)
            .set_c(from_register)
    }

    pub fn close(from_register: u16, to_register: u16) -> Instruction {
        *Instruction::new(Operation::Close)
            .set_b(from_register)
            .set_c(to_register)
    }

    pub fn load_boolean(to_register: u16, value: bool, skip: bool) -> Instruction {
        *Instruction::new(Operation::LoadBoolean)
            .set_a(to_register)
            .set_b_to_boolean(value)
            .set_c_to_boolean(skip)
    }

    pub fn load_constant(to_register: u16, constant_index: u16, skip: bool) -> Instruction {
        *Instruction::new(Operation::LoadConstant)
            .set_a(to_register)
            .set_b(constant_index)
            .set_c_to_boolean(skip)
    }

    pub fn load_list(to_register: u16, start_register: u16) -> Instruction {
        *Instruction::new(Operation::LoadList)
            .set_a(to_register)
            .set_b(start_register)
    }

    pub fn load_self(to_register: u16) -> Instruction {
        *Instruction::new(Operation::LoadSelf).set_a(to_register)
    }

    pub fn define_local(to_register: u16, local_index: u16, is_mutable: bool) -> Instruction {
        *Instruction::new(Operation::DefineLocal)
            .set_a(to_register)
            .set_b(local_index)
            .set_c_to_boolean(is_mutable)
    }

    pub fn get_local(to_register: u16, local_index: u16) -> Instruction {
        *Instruction::new(Operation::GetLocal)
            .set_a(to_register)
            .set_b(local_index)
    }

    pub fn set_local(from_register: u16, local_index: u16) -> Instruction {
        *Instruction::new(Operation::SetLocal)
            .set_a(from_register)
            .set_b(local_index)
    }

    pub fn add(to_register: u16, left_index: u16, right_index: u16) -> Instruction {
        *Instruction::new(Operation::Add)
            .set_a(to_register)
            .set_b(left_index)
            .set_c(right_index)
    }

    pub fn subtract(to_register: u16, left_index: u16, right_index: u16) -> Instruction {
        *Instruction::new(Operation::Subtract)
            .set_a(to_register)
            .set_b(left_index)
            .set_c(right_index)
    }

    pub fn multiply(to_register: u16, left_index: u16, right_index: u16) -> Instruction {
        *Instruction::new(Operation::Multiply)
            .set_a(to_register)
            .set_b(left_index)
            .set_c(right_index)
    }

    pub fn divide(to_register: u16, left_index: u16, right_index: u16) -> Instruction {
        *Instruction::new(Operation::Divide)
            .set_a(to_register)
            .set_b(left_index)
            .set_c(right_index)
    }

    pub fn modulo(to_register: u16, left_index: u16, right_index: u16) -> Instruction {
        *Instruction::new(Operation::Modulo)
            .set_a(to_register)
            .set_b(left_index)
            .set_c(right_index)
    }

    pub fn test(test_register: u16, test_value: bool) -> Instruction {
        *Instruction::new(Operation::Test)
            .set_b(test_register)
            .set_c_to_boolean(test_value)
    }

    pub fn test_set(to_register: u16, argument_index: u16, test_value: bool) -> Instruction {
        *Instruction::new(Operation::TestSet)
            .set_a(to_register)
            .set_b(argument_index)
            .set_c_to_boolean(test_value)
    }

    pub fn equal(comparison_boolean: bool, left_index: u16, right_index: u16) -> Instruction {
        *Instruction::new(Operation::Equal)
            .set_a_to_boolean(comparison_boolean)
            .set_b(left_index)
            .set_c(right_index)
    }

    pub fn less(comparison_boolean: bool, left_index: u16, right_index: u16) -> Instruction {
        *Instruction::new(Operation::Less)
            .set_a_to_boolean(comparison_boolean)
            .set_b(left_index)
            .set_c(right_index)
    }

    pub fn less_equal(comparison_boolean: bool, left_index: u16, right_index: u16) -> Instruction {
        *Instruction::new(Operation::LessEqual)
            .set_a_to_boolean(comparison_boolean)
            .set_b(left_index)
            .set_c(right_index)
    }

    pub fn negate(to_register: u16, from_index: u16) -> Instruction {
        *Instruction::new(Operation::Negate)
            .set_a(to_register)
            .set_b(from_index)
    }

    pub fn not(to_register: u16, from_index: u16) -> Instruction {
        *Instruction::new(Operation::Not)
            .set_a(to_register)
            .set_b(from_index)
    }

    pub fn jump(jump_offset: u16, is_positive: bool) -> Instruction {
        *Instruction::new(Operation::Jump)
            .set_b(jump_offset)
            .set_c_to_boolean(is_positive)
    }

    pub fn call(to_register: u16, function_register: u16, argument_count: u16) -> Instruction {
        *Instruction::new(Operation::Call)
            .set_a(to_register)
            .set_b(function_register)
            .set_c(argument_count)
    }

    pub fn call_native(
        to_register: u16,
        native_fn: NativeFunction,
        argument_count: u16,
    ) -> Instruction {
        *Instruction::new(Operation::CallNative)
            .set_a(to_register)
            .set_b(native_fn as u16)
            .set_c(argument_count)
    }

    pub fn r#return(should_return_value: bool) -> Instruction {
        *Instruction::new(Operation::Return).set_b_to_boolean(should_return_value)
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
            | Operation::Equal
            | Operation::Less
            | Operation::LessEqual
            | Operation::Negate
            | Operation::Not
            | Operation::Call => true,

            Operation::CallNative => {
                let function = NativeFunction::from(self.b());

                function.returns_value()
            }

            Operation::Move
            | Operation::Close
            | Operation::DefineLocal
            | Operation::SetLocal
            | Operation::Test
            | Operation::TestSet
            | Operation::Jump
            | Operation::Return => true,
        }
    }

    pub fn disassembly_info(&self, chunk: &Chunk) -> String {
        let InstructionBuilder {
            operation,
            a,
            b,
            c,
            b_is_constant,
            c_is_constant,
            a_is_local,
            b_is_local,
            c_is_local,
        } = InstructionBuilder::from(self);
        let format_arguments = || {
            let first_argument = if b_is_constant {
                format!("C{}", b)
            } else {
                format!("R{}", b)
            };
            let second_argument = if c_is_constant {
                format!("C{}", c)
            } else {
                format!("R{}", c)
            };

            (first_argument, second_argument)
        };

        match operation {
            Operation::Move => format!("R{a} = R{b}"),
            Operation::Close => {
                format!("R{b}..R{c}")
            }
            Operation::LoadBoolean => {
                let boolean = b != 0;
                let jump = c != 0;

                if jump {
                    format!("R{a} = {boolean} && JUMP +1")
                } else {
                    format!("R{a} {boolean}")
                }
            }
            Operation::LoadConstant => {
                let jump = c != 0;

                if jump {
                    format!("R{a} = C{b} JUMP +1")
                } else {
                    format!("R{a} = C{b}")
                }
            }
            Operation::LoadList => {
                format!("R{a} = [R{b}..=R{c}]",)
            }
            Operation::LoadSelf => {
                let name = chunk
                    .name()
                    .map(|idenifier| idenifier.as_str())
                    .unwrap_or("self");

                format!("R{a} = {name}")
            }
            Operation::DefineLocal => {
                format!("L{b} = R{a}")
            }
            Operation::GetLocal => {
                format!("R{a} = L{b}")
            }
            Operation::SetLocal => {
                format!("L{b} = R{a}")
            }
            Operation::Add => {
                let (first_argument, second_argument) = format_arguments();

                format!("R{a} = {first_argument} + {second_argument}",)
            }
            Operation::Subtract => {
                let (first_argument, second_argument) = format_arguments();

                format!("R{a} = {first_argument} - {second_argument}",)
            }
            Operation::Multiply => {
                let (first_argument, second_argument) = format_arguments();

                format!("R{a} = {first_argument} * {second_argument}",)
            }
            Operation::Divide => {
                let (first_argument, second_argument) = format_arguments();

                format!("R{a} = {first_argument} / {second_argument}",)
            }
            Operation::Modulo => {
                let (first_argument, second_argument) = format_arguments();

                format!("R{a} = {first_argument} % {second_argument}",)
            }
            Operation::Test => {
                let test_register = if b_is_constant {
                    format!("C{b}")
                } else {
                    format!("R{b}")
                };
                let test_value = c != 0;
                let bang = if test_value { "" } else { "!" };

                format!("if {bang}{test_register} {{ JUMP +1 }}",)
            }
            Operation::TestSet => {
                let test_register = if b_is_constant {
                    format!("C{b}")
                } else {
                    format!("R{b}")
                };
                let test_value = c != 0;
                let bang = if test_value { "" } else { "!" };

                format!(
                    "if {bang}R{test_register} {{ JUMP +1 }} else {{ R{a} = R{test_register} }}"
                )
            }
            Operation::Equal => {
                let comparison_symbol = if a != 0 { "==" } else { "!=" };
                let (first_argument, second_argument) = format_arguments();

                format!("if {first_argument} {comparison_symbol} {second_argument} {{ JUMP +1 }}")
            }
            Operation::Less => {
                let comparison_symbol = if a != 0 { "<" } else { ">=" };
                let (first_argument, second_argument) = format_arguments();

                format!("if {first_argument} {comparison_symbol} {second_argument} {{ JUMP +1 }}")
            }
            Operation::LessEqual => {
                let comparison_symbol = if a != 0 { "<=" } else { ">" };
                let (first_argument, second_argument) = format_arguments();

                format!("if {first_argument} {comparison_symbol} {second_argument} {{ JUMP +1 }}")
            }
            Operation::Negate => {
                let argument = if b_is_constant {
                    format!("C{b}")
                } else {
                    format!("R{b}")
                };

                format!("R{a} = -{argument}")
            }
            Operation::Not => {
                let argument = if b_is_constant {
                    format!("C{b}")
                } else {
                    format!("R{b}")
                };

                format!("R{a} = !{argument}")
            }
            Operation::Jump => {
                let is_positive = c != 0;

                if is_positive {
                    format!("JUMP +{b}")
                } else {
                    format!("JUMP -{b}")
                }
            }
            Operation::Call => {
                let argument_count = c;

                let mut output = format!("R{a} = R{b}(");

                if argument_count != 0 {
                    let first_argument = b + 1;

                    for (index, register) in
                        (first_argument..first_argument + argument_count).enumerate()
                    {
                        if index > 0 {
                            output.push_str(", ");
                        }

                        output.push_str(&format!("R{}", register));
                    }
                }

                output.push(')');

                output
            }
            Operation::CallNative => {
                let native_function = NativeFunction::from(b);
                let argument_count = c;
                let mut output = String::new();
                let native_function_name = native_function.as_str();

                output.push_str(&format!("R{a} = {}(", native_function_name));

                if argument_count != 0 {
                    let first_argument = a.saturating_sub(argument_count);

                    for register in first_argument..a {
                        if register != first_argument {
                            output.push_str(", ");
                        }

                        output.push_str(&format!("R{}", register));
                    }
                }

                output.push(')');

                output
            }
            Operation::Return => {
                let should_return_value = b != 0;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder() {
        let instruction_from_builder = Instruction::builder(Operation::Add)
            .a(1)
            .b(2)
            .c(3)
            .b_is_constant(true)
            .c_is_constant(true)
            .a_is_local(true)
            .b_is_local(true)
            .c_is_local(true)
            .build();
        let instruction = *Instruction::add(1, 2, 3)
            .set_b_is_constant()
            .set_c_is_constant()
            .set_a_is_local()
            .set_b_is_local()
            .set_c_is_local();

        assert_eq!(instruction_from_builder, instruction);
    }

    #[test]
    fn r#move() {
        let instruction = Instruction::r#move(4, 1);

        assert_eq!(instruction.operation(), Operation::Move);
        assert_eq!(instruction.b(), 4);
        assert_eq!(instruction.c(), 1);
    }

    #[test]
    fn close() {
        let instruction = Instruction::close(1, 2);

        assert_eq!(instruction.operation(), Operation::Close);
        assert_eq!(instruction.b(), 1);
        assert_eq!(instruction.c(), 2);
    }

    #[test]
    fn load_boolean() {
        let instruction = Instruction::load_boolean(4, true, true);

        assert_eq!(instruction.operation(), Operation::LoadBoolean);
        assert_eq!(instruction.a(), 4);
        assert!(instruction.a_as_boolean());
        assert!(instruction.c_as_boolean());
    }

    #[test]
    fn load_constant() {
        let mut instruction = Instruction::load_constant(4, 1, true);

        instruction.set_b_is_constant();
        instruction.set_c_is_constant();

        assert_eq!(instruction.operation(), Operation::LoadConstant);
        assert_eq!(instruction.a(), 4);
        assert_eq!(instruction.b(), 1);
        assert!(instruction.b_is_constant());
        assert!(instruction.b_is_constant());
        assert!(instruction.c_as_boolean());
    }

    #[test]
    fn load_list() {
        let instruction = Instruction::load_list(4, 1);

        assert_eq!(instruction.operation(), Operation::LoadList);
        assert_eq!(instruction.a(), 4);
        assert_eq!(instruction.b(), 1);
    }

    #[test]
    fn load_self() {
        let instruction = Instruction::load_self(10);

        assert_eq!(instruction.operation(), Operation::LoadSelf);
        assert_eq!(instruction.a(), 10);
    }

    #[test]
    fn declare_local() {
        let instruction = *Instruction::define_local(4, 1, true).set_b_is_constant();

        assert_eq!(instruction.operation(), Operation::DefineLocal);
        assert_eq!(instruction.a(), 4);
        assert_eq!(instruction.b(), 1);
        assert!(instruction.c_as_boolean());
        assert!(instruction.b_is_constant());
    }

    #[test]
    fn add() {
        let instruction = *Instruction::add(1, 1, 4).set_b_is_constant();

        assert_eq!(instruction.operation(), Operation::Add);
        assert_eq!(instruction.a(), 1);
        assert_eq!(instruction.b(), 1);
        assert_eq!(instruction.c(), 4);
        assert!(instruction.b_is_constant());
    }

    #[test]
    fn subtract() {
        let mut instruction = Instruction::subtract(4, 1, 2);

        instruction.set_b_is_constant();
        instruction.set_c_is_constant();

        assert_eq!(instruction.operation(), Operation::Subtract);
        assert_eq!(instruction.a(), 4);
        assert_eq!(instruction.b(), 1);
        assert_eq!(instruction.c(), 2);
        assert!(instruction.b_is_constant());
        assert!(instruction.b_is_constant());
    }

    #[test]
    fn multiply() {
        let mut instruction = Instruction::multiply(4, 1, 2);

        instruction.set_b_is_constant();
        instruction.set_c_is_constant();

        assert_eq!(instruction.operation(), Operation::Multiply);
        assert_eq!(instruction.a(), 4);
        assert_eq!(instruction.b(), 1);
        assert_eq!(instruction.c(), 2);
        assert!(instruction.b_is_constant());
        assert!(instruction.b_is_constant());
    }

    #[test]
    fn divide() {
        let mut instruction = Instruction::divide(4, 1, 2);

        instruction.set_b_is_constant();
        instruction.set_c_is_constant();

        assert_eq!(instruction.operation(), Operation::Divide);
        assert_eq!(instruction.a(), 4);
        assert_eq!(instruction.b(), 1);
        assert_eq!(instruction.c(), 2);
        assert!(instruction.b_is_constant());
        assert!(instruction.b_is_constant());
    }

    #[test]
    fn test() {
        let instruction = Instruction::test(42, true);

        assert_eq!(instruction.operation(), Operation::Test);
        assert_eq!(instruction.b(), 42);
        assert!(instruction.c_as_boolean());
    }

    #[test]
    fn test_set() {
        let instruction = Instruction::test_set(4, 1, true);

        assert_eq!(instruction.operation(), Operation::TestSet);
        assert_eq!(instruction.a(), 4);
        assert_eq!(instruction.b(), 1);
        assert!(instruction.c_as_boolean());
    }

    #[test]
    fn equal() {
        let mut instruction = Instruction::equal(true, 1, 2);

        instruction.set_b_is_constant();
        instruction.set_c_is_constant();

        assert_eq!(instruction.operation(), Operation::Equal);
        assert!(instruction.a_as_boolean());
        assert_eq!(instruction.b(), 1);
        assert_eq!(instruction.c(), 2);
        assert!(instruction.b_is_constant());
        assert!(instruction.b_is_constant());
    }

    #[test]
    fn negate() {
        let mut instruction = Instruction::negate(4, 1);

        instruction.set_b_is_constant();
        instruction.set_c_is_constant();

        assert_eq!(instruction.operation(), Operation::Negate);
        assert_eq!(instruction.a(), 4);
        assert_eq!(instruction.b(), 1);
        assert!(instruction.b_is_constant());
        assert!(instruction.b_is_constant());
    }

    #[test]
    fn not() {
        let mut instruction = Instruction::not(4, 1);

        instruction.set_b_is_constant();
        instruction.set_c_is_constant();

        assert_eq!(instruction.operation(), Operation::Not);
        assert_eq!(instruction.a(), 4);
        assert_eq!(instruction.b(), 1);
        assert!(instruction.b_is_constant());
        assert!(instruction.b_is_constant());
    }

    #[test]
    fn jump() {
        let instruction = Instruction::jump(4, true);

        assert_eq!(instruction.operation(), Operation::Jump);

        assert_eq!(instruction.b(), 4);
        assert!(instruction.c_as_boolean());
    }

    #[test]
    fn call() {
        let instruction = Instruction::call(1, 3, 4);

        assert_eq!(instruction.operation(), Operation::Call);
        assert_eq!(instruction.a(), 1);
        assert_eq!(instruction.b(), 3);
        assert_eq!(instruction.c(), 4);
    }

    #[test]
    fn r#return() {
        let instruction = Instruction::r#return(true);

        assert_eq!(instruction.operation(), Operation::Return);
        assert!(instruction.b_as_boolean());
    }
}
