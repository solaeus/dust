//! An operation and its arguments for the Dust virtual machine.
//!
//! Each instruction is a 32-bit unsigned integer that is divided into five fields:
//! - Bits 0-6: The operation code.
//! - Bit 7: A flag indicating whether the B argument is a constant.
//! - Bit 8: A flag indicating whether the C argument is a constant.
//! - Bits 9-16: The A argument,
//! - Bits 17-24: The B argument.
//! - Bits 25-32: The C argument.
//!
//! Be careful when working with instructions directly. When modifying an instruction, be sure to
//! account for the fact that setting the A, B, or C arguments to 0 will have no effect. It is
//! usually best to remove instructions and insert new ones in their place instead of mutating them.

use serde::{Deserialize, Serialize};

use crate::{Chunk, NativeFunction, Operation, Type};

/// An operation and its arguments for the Dust virtual machine.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Instruction(u32);

impl Instruction {
    pub fn with_operation(operation: Operation) -> Instruction {
        Instruction(operation as u32)
    }

    pub fn r#move(to_register: u8, from_register: u8) -> Instruction {
        let mut instruction = Instruction(Operation::Move as u32);

        instruction.set_a(to_register);
        instruction.set_b(from_register);

        instruction
    }

    pub fn close(from_register: u8, to_register: u8) -> Instruction {
        let mut instruction = Instruction(Operation::Close as u32);

        instruction.set_b(from_register);
        instruction.set_c(to_register);

        instruction
    }

    pub fn load_boolean(to_register: u8, value: bool, skip: bool) -> Instruction {
        let mut instruction = Instruction(Operation::LoadBoolean as u32);

        instruction.set_a(to_register);
        instruction.set_b_to_boolean(value);
        instruction.set_c_to_boolean(skip);

        instruction
    }

    pub fn load_constant(to_register: u8, constant_index: u8, skip: bool) -> Instruction {
        let mut instruction = Instruction(Operation::LoadConstant as u32);

        instruction.set_a(to_register);
        instruction.set_b(constant_index);
        instruction.set_c_to_boolean(skip);

        instruction
    }

    pub fn load_list(to_register: u8, start_register: u8) -> Instruction {
        let mut instruction = Instruction(Operation::LoadList as u32);

        instruction.set_a(to_register);
        instruction.set_b(start_register);

        instruction
    }

    pub fn load_self(to_register: u8) -> Instruction {
        let mut instruction = Instruction(Operation::LoadSelf as u32);

        instruction.set_a(to_register);

        instruction
    }

    pub fn define_local(to_register: u8, local_index: u8, is_mutable: bool) -> Instruction {
        let mut instruction = Instruction(Operation::DefineLocal as u32);

        instruction.set_a(to_register);
        instruction.set_b(local_index);
        instruction.set_c(if is_mutable { 1 } else { 0 });

        instruction
    }

    pub fn get_local(to_register: u8, local_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::GetLocal as u32);

        instruction.set_a(to_register);
        instruction.set_b(local_index);

        instruction
    }

    pub fn set_local(from_register: u8, local_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::SetLocal as u32);

        instruction.set_a(from_register);
        instruction.set_b(local_index);

        instruction
    }

    pub fn add(to_register: u8, left_index: u8, right_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::Add as u32);

        instruction.set_a(to_register);
        instruction.set_b(left_index);
        instruction.set_c(right_index);

        instruction
    }

    pub fn subtract(to_register: u8, left_index: u8, right_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::Subtract as u32);

        instruction.set_a(to_register);
        instruction.set_b(left_index);
        instruction.set_c(right_index);

        instruction
    }

    pub fn multiply(to_register: u8, left_index: u8, right_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::Multiply as u32);

        instruction.set_a(to_register);
        instruction.set_b(left_index);
        instruction.set_c(right_index);

        instruction
    }

    pub fn divide(to_register: u8, left_index: u8, right_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::Divide as u32);

        instruction.set_a(to_register);
        instruction.set_b(left_index);
        instruction.set_c(right_index);

        instruction
    }

    pub fn modulo(to_register: u8, left_index: u8, right_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::Modulo as u32);

        instruction.set_a(to_register);
        instruction.set_b(left_index);
        instruction.set_c(right_index);

        instruction
    }

    pub fn test(to_register: u8, test_value: bool) -> Instruction {
        let mut instruction = Instruction(Operation::Test as u32);

        instruction.set_a(to_register);
        instruction.set_c_to_boolean(test_value);

        instruction
    }

    pub fn test_set(to_register: u8, argument_index: u8, test_value: bool) -> Instruction {
        let mut instruction = Instruction(Operation::TestSet as u32);

        instruction.set_a(to_register);
        instruction.set_b(argument_index);
        instruction.set_c_to_boolean(test_value);

        instruction
    }

    pub fn equal(comparison_boolean: bool, left_index: u8, right_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::Equal as u32);

        instruction.set_a_to_boolean(comparison_boolean);
        instruction.set_b(left_index);
        instruction.set_c(right_index);

        instruction
    }

    pub fn less(comparison_boolean: bool, left_index: u8, right_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::Less as u32);

        instruction.set_a_to_boolean(comparison_boolean);
        instruction.set_b(left_index);
        instruction.set_c(right_index);

        instruction
    }

    pub fn less_equal(comparison_boolean: bool, left_index: u8, right_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::LessEqual as u32);

        instruction.set_a_to_boolean(comparison_boolean);
        instruction.set_b(left_index);
        instruction.set_c(right_index);

        instruction
    }

    pub fn negate(to_register: u8, from_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::Negate as u32);

        instruction.set_a(to_register);
        instruction.set_b(from_index);

        instruction
    }

    pub fn not(to_register: u8, from_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::Not as u32);

        instruction.set_a(to_register);
        instruction.set_b(from_index);

        instruction
    }

    pub fn jump(jump_offset: u8, is_positive: bool) -> Instruction {
        let mut instruction = Instruction(Operation::Jump as u32);

        instruction.set_b(jump_offset);
        instruction.set_c_to_boolean(is_positive);

        instruction
    }

    pub fn call(to_register: u8, function_register: u8, argument_count: u8) -> Instruction {
        let mut instruction = Instruction(Operation::Call as u32);

        instruction.set_a(to_register);
        instruction.set_b(function_register);
        instruction.set_c(argument_count);

        instruction
    }

    pub fn call_native(
        to_register: u8,
        native_fn: NativeFunction,
        argument_count: u8,
    ) -> Instruction {
        let mut instruction = Instruction(Operation::CallNative as u32);
        let native_fn_byte = native_fn as u8;

        instruction.set_a(to_register);
        instruction.set_b(native_fn_byte);
        instruction.set_c(argument_count);

        instruction
    }

    pub fn r#return(should_return_value: bool) -> Instruction {
        let mut instruction = Instruction(Operation::Return as u32);

        instruction.set_b_to_boolean(should_return_value);

        instruction
    }

    pub fn operation(&self) -> Operation {
        Operation::from((self.0 & 0b0000_0000_0011_1111) as u8)
    }

    pub fn set_operation(&mut self, operation: Operation) {
        self.0 |= u8::from(operation) as u32;
    }

    pub fn data(&self) -> (Operation, u8, u8, u8, bool, bool) {
        (
            self.operation(),
            self.a(),
            self.b(),
            self.c(),
            self.b_is_constant(),
            self.c_is_constant(),
        )
    }

    pub fn a(&self) -> u8 {
        (self.0 >> 24) as u8
    }

    pub fn b(&self) -> u8 {
        (self.0 >> 16) as u8
    }

    pub fn c(&self) -> u8 {
        (self.0 >> 8) as u8
    }

    pub fn a_as_boolean(&self) -> bool {
        self.a() != 0
    }

    pub fn b_as_boolean(&self) -> bool {
        self.b() != 0
    }

    pub fn c_as_boolean(&self) -> bool {
        self.c() != 0
    }

    pub fn set_a_to_boolean(&mut self, boolean: bool) -> &mut Self {
        self.set_a(if boolean { 1 } else { 0 });

        self
    }

    pub fn set_b_to_boolean(&mut self, boolean: bool) -> &mut Self {
        self.set_b(if boolean { 1 } else { 0 });

        self
    }

    pub fn set_c_to_boolean(&mut self, boolean: bool) -> &mut Self {
        self.set_c(if boolean { 1 } else { 0 });

        self
    }

    pub fn set_a(&mut self, to_register: u8) {
        self.0 |= (to_register as u32) << 24;
    }

    pub fn set_b(&mut self, argument: u8) {
        self.0 |= (argument as u32) << 16;
    }

    pub fn set_c(&mut self, argument: u8) {
        self.0 |= (argument as u32) << 8;
    }

    pub fn b_is_constant(&self) -> bool {
        self.0 & 0b1000_0000 != 0
    }

    pub fn c_is_constant(&self) -> bool {
        self.0 & 0b0100_0000 != 0
    }

    pub fn set_b_is_constant(&mut self) -> &mut Self {
        self.0 |= 0b1000_0000;

        self
    }

    pub fn set_c_is_constant(&mut self) -> &mut Self {
        self.0 |= 0b0100_0000;

        self
    }

    pub fn returns_or_panics(&self) -> bool {
        match self.operation() {
            Operation::Return => true,
            Operation::CallNative => {
                let native_function = NativeFunction::from(self.b());

                matches!(native_function, NativeFunction::Panic)
            }
            _ => false,
        }
    }

    pub fn yields_value(&self) -> bool {
        match self.operation() {
            Operation::Add
            | Operation::Call
            | Operation::Divide
            | Operation::GetLocal
            | Operation::LoadBoolean
            | Operation::LoadConstant
            | Operation::LoadList
            | Operation::LoadSelf
            | Operation::Modulo
            | Operation::Multiply
            | Operation::Negate
            | Operation::Not
            | Operation::Subtract => true,
            Operation::CallNative => {
                let native_function = NativeFunction::from(self.b());

                *native_function.r#type().return_type != Type::None
            }
            _ => false,
        }
    }

    pub fn disassembly_info(&self, chunk: &Chunk) -> String {
        let format_arguments = || {
            let first_argument = if self.b_is_constant() {
                format!("C{}", self.b())
            } else {
                format!("R{}", self.b())
            };
            let second_argument = if self.c_is_constant() {
                format!("C{}", self.c())
            } else {
                format!("R{}", self.c())
            };

            (first_argument, second_argument)
        };

        match self.operation() {
            Operation::Move => format!("R{} = R{}", self.a(), self.b()),
            Operation::Close => {
                let from_register = self.b();
                let to_register = self.c().saturating_sub(1);

                format!("R{from_register}..=R{to_register}")
            }
            Operation::LoadBoolean => {
                let to_register = self.a();
                let boolean = self.b_as_boolean();
                let jump = self.c_as_boolean();

                if jump {
                    format!("R{to_register} = {boolean} && SKIP")
                } else {
                    format!("R{to_register} = {boolean}")
                }
            }
            Operation::LoadConstant => {
                let register_index = self.a();
                let constant_index = self.b();
                let jump = self.c_as_boolean();

                if jump {
                    format!("R{register_index} = C{constant_index} && SKIP")
                } else {
                    format!("R{register_index} = C{constant_index}")
                }
            }
            Operation::LoadList => {
                let to_register = self.a();
                let first_index = self.b();
                let last_index = self.c();

                format!("R{to_register} = [R{first_index}..=R{last_index}]",)
            }
            Operation::LoadSelf => {
                let to_register = self.a();
                let name = chunk
                    .name()
                    .map(|idenifier| idenifier.as_str())
                    .unwrap_or("self");

                format!("R{to_register} = {name}")
            }
            Operation::DefineLocal => {
                let to_register = self.a();
                let local_index = self.b();
                let identifier_display = match chunk.get_identifier(local_index) {
                    Some(identifier) => identifier.to_string(),
                    None => "???".to_string(),
                };
                let mutable_display = if self.c_as_boolean() { "mut" } else { "" };

                format!("R{to_register} = L{local_index} {mutable_display} {identifier_display}")
            }
            Operation::GetLocal => {
                let local_index = self.b();

                format!("R{} = L{}", self.a(), local_index)
            }
            Operation::SetLocal => {
                let local_index = self.b();
                let identifier_display = match chunk.get_identifier(local_index) {
                    Some(identifier) => identifier.to_string(),
                    None => "???".to_string(),
                };

                format!("L{} = R{} {}", local_index, self.a(), identifier_display)
            }
            Operation::Add => {
                let to_register = self.a();
                let (first_argument, second_argument) = format_arguments();

                format!("R{to_register} = {first_argument} + {second_argument}",)
            }
            Operation::Subtract => {
                let to_register = self.a();
                let (first_argument, second_argument) = format_arguments();

                format!("R{to_register} = {first_argument} - {second_argument}",)
            }
            Operation::Multiply => {
                let to_register = self.a();
                let (first_argument, second_argument) = format_arguments();

                format!("R{to_register} = {first_argument} * {second_argument}",)
            }
            Operation::Divide => {
                let to_register = self.a();
                let (first_argument, second_argument) = format_arguments();

                format!("R{to_register} = {first_argument} / {second_argument}",)
            }
            Operation::Modulo => {
                let to_register = self.a();
                let (first_argument, second_argument) = format_arguments();

                format!("R{to_register} = {first_argument} % {second_argument}",)
            }
            Operation::Test => {
                let to_register = self.a();
                let test_value = self.c_as_boolean();

                format!("if R{to_register} != {test_value} {{ SKIP }}")
            }
            Operation::TestSet => {
                let to_register = self.a();
                let argument = format!("R{}", self.b());
                let test_value = self.c_as_boolean();
                let bang = if test_value { "" } else { "!" };

                format!("if {bang}R{to_register} {{ R{to_register} = R{argument} }}",)
            }
            Operation::Equal => {
                let comparison_symbol = if self.a_as_boolean() { "==" } else { "!=" };

                let (first_argument, second_argument) = format_arguments();

                format!("if {first_argument} {comparison_symbol} {second_argument} {{ SKIP }}")
            }
            Operation::Less => {
                let comparison_symbol = if self.a_as_boolean() { "<" } else { ">=" };
                let (first_argument, second_argument) = format_arguments();

                format!("if {first_argument} {comparison_symbol} {second_argument} {{ SKIP }}")
            }
            Operation::LessEqual => {
                let comparison_symbol = if self.a_as_boolean() { "<=" } else { ">" };
                let (first_argument, second_argument) = format_arguments();

                format!("if {first_argument} {comparison_symbol} {second_argument} {{ SKIP }}")
            }
            Operation::Negate => {
                let to_register = self.a();
                let argument = if self.b_is_constant() {
                    format!("C{}", self.b())
                } else {
                    format!("R{}", self.b())
                };

                format!("R{to_register} = -{argument}")
            }
            Operation::Not => {
                let to_register = self.a();
                let argument = if self.b_is_constant() {
                    format!("C{}", self.b())
                } else {
                    format!("R{}", self.b())
                };

                format!("R{to_register} = !{argument}")
            }
            Operation::Jump => {
                let jump_distance = self.b();
                let is_positive = self.c_as_boolean();

                if is_positive {
                    format!("JUMP +{jump_distance}")
                } else {
                    format!("JUMP -{jump_distance}")
                }
            }
            Operation::Call => {
                let to_register = self.a();
                let function_register = self.b();
                let argument_count = self.c();

                let mut output = format!("R{to_register} = R{function_register}(");

                if argument_count != 0 {
                    let first_argument = function_register + 1;

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
                let to_register = self.a();
                let native_function = NativeFunction::from(self.b());
                let argument_count = self.c();
                let mut output = String::new();
                let native_function_name = native_function.as_str();

                if *native_function.r#type().return_type != Type::None {
                    output.push_str(&format!("R{} = {}(", to_register, native_function_name));
                } else {
                    output.push_str(&format!("{}(", native_function_name));
                }

                if argument_count != 0 {
                    let first_argument = to_register.saturating_sub(argument_count);

                    for (index, register) in (first_argument..to_register).enumerate() {
                        if index > 0 {
                            output.push_str(", ");
                        }

                        output.push_str(&format!("R{}", register));
                    }
                }

                output.push(')');

                output
            }
            Operation::Return => {
                let should_return_value = self.b_as_boolean();

                if should_return_value {
                    "->".to_string()
                } else {
                    "".to_string()
                }
            }
        }
    }
}

impl From<&Instruction> for u32 {
    fn from(instruction: &Instruction) -> Self {
        instruction.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r#move() {
        let mut instruction = Instruction::r#move(0, 1);

        instruction.set_b_is_constant();
        instruction.set_c_is_constant();

        assert_eq!(instruction.operation(), Operation::Move);
        assert_eq!(instruction.a(), 0);
        assert_eq!(instruction.b(), 1);
        assert!(instruction.b_is_constant());
        assert!(instruction.b_is_constant());
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
        let mut instruction = Instruction::load_constant(0, 1, true);

        instruction.set_b_is_constant();
        instruction.set_c_is_constant();

        assert_eq!(instruction.operation(), Operation::LoadConstant);
        assert_eq!(instruction.a(), 0);
        assert_eq!(instruction.b(), 1);
        assert!(instruction.b_is_constant());
        assert!(instruction.b_is_constant());
        assert!(instruction.c_as_boolean());
    }

    #[test]
    fn load_list() {
        let instruction = Instruction::load_list(0, 1);

        assert_eq!(instruction.operation(), Operation::LoadList);
        assert_eq!(instruction.a(), 0);
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
        let mut instruction = Instruction::define_local(0, 1, true);

        instruction.set_b_is_constant();

        assert_eq!(instruction.operation(), Operation::DefineLocal);
        assert_eq!(instruction.a(), 0);
        assert_eq!(instruction.b(), 1);
        assert_eq!(instruction.c(), true as u8);
        assert!(instruction.b_is_constant());
    }

    #[test]
    fn add() {
        let mut instruction = Instruction::add(1, 1, 0);

        instruction.set_b_is_constant();

        assert_eq!(instruction.operation(), Operation::Add);
        assert_eq!(instruction.a(), 1);
        assert_eq!(instruction.b(), 1);
        assert_eq!(instruction.c(), 0);
        assert!(instruction.b_is_constant());
    }

    #[test]
    fn subtract() {
        let mut instruction = Instruction::subtract(0, 1, 2);

        instruction.set_b_is_constant();
        instruction.set_c_is_constant();

        assert_eq!(instruction.operation(), Operation::Subtract);
        assert_eq!(instruction.a(), 0);
        assert_eq!(instruction.b(), 1);
        assert_eq!(instruction.c(), 2);
        assert!(instruction.b_is_constant());
        assert!(instruction.b_is_constant());
    }

    #[test]
    fn multiply() {
        let mut instruction = Instruction::multiply(0, 1, 2);

        instruction.set_b_is_constant();
        instruction.set_c_is_constant();

        assert_eq!(instruction.operation(), Operation::Multiply);
        assert_eq!(instruction.a(), 0);
        assert_eq!(instruction.b(), 1);
        assert_eq!(instruction.c(), 2);
        assert!(instruction.b_is_constant());
        assert!(instruction.b_is_constant());
    }

    #[test]
    fn divide() {
        let mut instruction = Instruction::divide(0, 1, 2);

        instruction.set_b_is_constant();
        instruction.set_c_is_constant();

        assert_eq!(instruction.operation(), Operation::Divide);
        assert_eq!(instruction.a(), 0);
        assert_eq!(instruction.b(), 1);
        assert_eq!(instruction.c(), 2);
        assert!(instruction.b_is_constant());
        assert!(instruction.b_is_constant());
    }

    #[test]
    fn and() {
        let instruction = Instruction::test(4, true);

        assert_eq!(instruction.operation(), Operation::Test);
        assert_eq!(instruction.a(), 4);
        assert!(instruction.c_as_boolean());
    }

    #[test]
    fn or() {
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
        let mut instruction = Instruction::negate(0, 1);

        instruction.set_b_is_constant();
        instruction.set_c_is_constant();

        assert_eq!(instruction.operation(), Operation::Negate);
        assert_eq!(instruction.a(), 0);
        assert_eq!(instruction.b(), 1);
        assert!(instruction.b_is_constant());
        assert!(instruction.b_is_constant());
    }

    #[test]
    fn not() {
        let mut instruction = Instruction::not(0, 1);

        instruction.set_b_is_constant();
        instruction.set_c_is_constant();

        assert_eq!(instruction.operation(), Operation::Not);
        assert_eq!(instruction.a(), 0);
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
