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

    pub fn set_a(&mut self, a: u16) -> &mut Self {
        self.a = a;
        self
    }

    pub fn set_b(&mut self, b: u16) -> &mut Self {
        self.b = b;
        self
    }

    pub fn set_b_to_boolean(&mut self, b: bool) -> &mut Self {
        self.b = b as u16;
        self
    }

    pub fn set_c(&mut self, c: u16) -> &mut Self {
        self.c = c;
        self
    }

    pub fn set_c_to_boolean(&mut self, c: bool) -> &mut Self {
        self.c = c as u16;
        self
    }

    pub fn set_b_is_constant(&mut self, b_is_constant: bool) -> &mut Self {
        self.b_is_constant = b_is_constant;
        self
    }

    pub fn set_c_is_constant(&mut self, c_is_constant: bool) -> &mut Self {
        self.c_is_constant = c_is_constant;
        self
    }

    pub fn set_a_is_local(&mut self, a_is_local: bool) -> &mut Self {
        self.a_is_local = a_is_local;
        self
    }

    pub fn set_b_is_local(&mut self, b_is_local: bool) -> &mut Self {
        self.b_is_local = b_is_local;
        self
    }

    pub fn set_c_is_local(&mut self, c_is_local: bool) -> &mut Self {
        self.c_is_local = c_is_local;
        self
    }
}

/// An operation and its arguments for the Dust virtual machine.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Instruction(u64);

impl Instruction {
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

    pub fn r#move(to_register: u16, from_register: u16) -> Instruction {
        Instruction::builder(Operation::Move)
            .set_a(to_register)
            .set_b(from_register)
            .build()
    }

    pub fn close(from_register: u16, to_register: u16) -> Instruction {
        Instruction::builder(Operation::Close)
            .set_b(from_register)
            .set_c(to_register)
            .build()
    }

    pub fn load_boolean(to_register: u16, value: bool, skip: bool) -> Instruction {
        Instruction::builder(Operation::LoadBoolean)
            .set_a(to_register)
            .set_b_to_boolean(value)
            .set_c_to_boolean(skip)
            .build()
    }

    pub fn load_constant(to_register: u16, constant_index: u16, skip: bool) -> Instruction {
        Instruction::builder(Operation::LoadConstant)
            .set_a(to_register)
            .set_b(constant_index)
            .set_c_to_boolean(skip)
            .build()
    }

    pub fn load_list(to_register: u16, start_register: u16) -> Instruction {
        Instruction::builder(Operation::LoadList)
            .set_a(to_register)
            .set_b(start_register)
            .build()
    }

    pub fn load_self(to_register: u16) -> Instruction {
        Instruction::builder(Operation::LoadSelf)
            .set_a(to_register)
            .build()
    }

    pub fn define_local(to_register: u16, local_index: u16, is_mutable: bool) -> Instruction {
        Instruction::builder(Operation::DefineLocal)
            .set_a(to_register as u16)
            .set_b(local_index as u16)
            .set_c_to_boolean(is_mutable)
            .build()
    }

    pub fn get_local(to_register: u16, local_index: u16) -> Instruction {
        Instruction::builder(Operation::GetLocal)
            .set_a(to_register)
            .set_b(local_index)
            .build()
    }

    pub fn set_local(from_register: u16, local_index: u16) -> Instruction {
        Instruction::builder(Operation::SetLocal)
            .set_a(from_register)
            .set_b(local_index)
            .build()
    }

    // pub fn add(to_register: u16, left_index: u16, right_index: u16) -> Instruction {
    //     Instruction::builder(Operation::Add)
    //         .set_a(to_register)
    //         .set_b(left_index)
    //         .set_c(right_index)
    //         .build()
    // }

    // pub fn subtract(to_register: u16, left_index: u16, right_index: u16) -> Instruction {
    //     let mut instruction = Instruction(Operation::Subtract as u32);

    //     instruction.set_a(to_register);
    //     instruction.set_b(left_index);
    //     instruction.set_c(right_index);

    //     instruction
    // }

    // pub fn multiply(to_register: u16, left_index: u16, right_index: u16) -> Instruction {
    //     let mut instruction = Instruction(Operation::Multiply as u32);

    //     instruction.set_a(to_register);
    //     instruction.set_b(left_index);
    //     instruction.set_c(right_index);

    //     instruction
    // }

    // pub fn divide(to_register: u16, left_index: u16, right_index: u16) -> Instruction {
    //     let mut instruction = Instruction(Operation::Divide as u32);

    //     instruction.set_a(to_register);
    //     instruction.set_b(left_index);
    //     instruction.set_c(right_index);

    //     instruction
    // }

    // pub fn modulo(to_register: u16, left_index: u16, right_index: u16) -> Instruction {
    //     let mut instruction = Instruction(Operation::Modulo as u32);

    //     instruction.set_a(to_register);
    //     instruction.set_b(left_index);
    //     instruction.set_c(right_index);

    //     instruction
    // }

    // pub fn test(test_register: u16, test_value: bool) -> Instruction {
    //     Instruction::builder(Operation::Test)
    //         .set_b(test_register)
    //         .set_c_to_boolean(test_value)
    //         .build()
    // }

    // pub fn test_set(to_register: u16, argument_index: u16, test_value: bool) -> Instruction {
    //     Instruction::builder(Operation::TestSet)
    //         .set_a(to_register)
    //         .set_b(argument_index)
    //         .set_c_to_boolean(test_value)
    //         .build()
    // }

    // pub fn equal(comparison_boolean: bool, left_index: u16, right_index: u16) -> Instruction {
    //     let mut instruction = Instruction(Operation::Equal as u32);

    //     instruction.set_a_to_boolean(comparison_boolean);
    //     instruction.set_b(left_index);
    //     instruction.set_c(right_index);

    //     instruction
    // }

    // pub fn less(comparison_boolean: bool, left_index: u16, right_index: u16) -> Instruction {
    //     let mut instruction = Instruction(Operation::Less as u32);

    //     instruction.set_a_to_boolean(comparison_boolean);
    //     instruction.set_b(left_index);
    //     instruction.set_c(right_index);

    //     instruction
    // }

    // pub fn less_equal(comparison_boolean: bool, left_index: u16, right_index: u16) -> Instruction {
    //     let mut instruction = Instruction(Operation::LessEqual as u32);

    //     instruction.set_a_to_boolean(comparison_boolean);
    //     instruction.set_b(left_index);
    //     instruction.set_c(right_index);

    //     instruction
    // }

    // pub fn negate(to_register: u16, from_index: u16) -> Instruction {
    //     let mut instruction = Instruction(Operation::Negate as u32);

    //     instruction.set_a(to_register);
    //     instruction.set_b(from_index);

    //     instruction
    // }

    // pub fn not(to_register: u16, from_index: u16) -> Instruction {
    //     let mut instruction = Instruction(Operation::Not as u32);

    //     instruction.set_a(to_register);
    //     instruction.set_b(from_index);

    //     instruction
    // }

    pub fn jump(jump_offset: u16, is_positive: bool) -> Instruction {
        Instruction::builder(Operation::Jump)
            .set_b(jump_offset)
            .set_c_to_boolean(is_positive)
            .build()
    }

    pub fn call(to_register: u16, function_register: u16, argument_count: u16) -> Instruction {
        Instruction::builder(Operation::Call)
            .set_a(to_register)
            .set_b(function_register)
            .set_c(argument_count)
            .build()
    }

    pub fn call_native(
        to_register: u16,
        native_fn: NativeFunction,
        argument_count: u16,
    ) -> Instruction {
        Instruction::builder(Operation::CallNative)
            .set_a(to_register)
            .set_b(native_fn as u16)
            .set_c(argument_count)
            .build()
    }

    pub fn r#return(should_return_value: bool) -> Instruction {
        Instruction::builder(Operation::Return)
            .set_b_to_boolean(should_return_value)
            .build()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r#move() {
        let mut instruction = Instruction::r#move(4, 1);

        instruction.set_b_is_constant();
        instruction.set_c_is_constant();

        assert_eq!(instruction.operation(), Operation::Move);
        assert_eq!(instruction.a(), 4);
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
        let mut instruction = Instruction::define_local(4, 1, true);

        instruction.set_b_is_constant();

        assert_eq!(instruction.operation(), Operation::DefineLocal);
        assert_eq!(instruction.a(), 4);
        assert_eq!(instruction.b(), 1);
        assert_eq!(instruction.c(), true as u16);
        assert!(instruction.b_is_constant());
    }

    #[test]
    fn add() {
        let mut instruction = Instruction::add(1, 1, 4);

        instruction.set_b_is_constant();

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
