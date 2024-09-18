use std::fmt::{self, Display, Formatter};

use crate::{Chunk, Operation, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Instruction(u32);

impl Instruction {
    pub fn r#move(to_register: u8, from_register: u8) -> Instruction {
        let mut instruction = Instruction(Operation::Move as u32);

        instruction.set_destination(to_register);
        instruction.set_first_argument(from_register);

        instruction
    }

    pub fn close(from_register: u8, to_register: u8) -> Instruction {
        let mut instruction = Instruction(Operation::Close as u32);

        instruction.set_first_argument(from_register);
        instruction.set_second_argument(to_register);

        instruction
    }

    pub fn load_boolean(to_register: u8, value: bool, skip: bool) -> Instruction {
        let mut instruction = Instruction(Operation::LoadBoolean as u32);

        instruction.set_destination(to_register);
        instruction.set_first_argument(if value { 1 } else { 0 });
        instruction.set_second_argument(if skip { 1 } else { 0 });

        instruction
    }

    pub fn load_constant(to_register: u8, constant_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::LoadConstant as u32);

        instruction.set_destination(to_register);
        instruction.set_first_argument(constant_index);

        instruction
    }

    pub fn load_list(to_register: u8, list_length: u8) -> Instruction {
        let mut instruction = Instruction(Operation::LoadList as u32);

        instruction.set_destination(to_register);
        instruction.set_first_argument(list_length);

        instruction
    }

    pub fn define_local(to_register: u8, variable_index: u8, is_mutable: bool) -> Instruction {
        let mut instruction = Instruction(Operation::DefineLocal as u32);

        instruction.set_destination(to_register);
        instruction.set_first_argument(variable_index);
        instruction.set_second_argument(if is_mutable { 1 } else { 0 });

        instruction
    }

    pub fn get_local(to_register: u8, variable_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::GetLocal as u32);

        instruction.set_destination(to_register);
        instruction.set_first_argument(variable_index);

        instruction
    }

    pub fn set_local(from_register: u8, variable_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::SetLocal as u32);

        instruction.set_destination(from_register);
        instruction.set_first_argument(variable_index);

        instruction
    }

    pub fn add(to_register: u8, left_index: u8, right_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::Add as u32);

        instruction.set_destination(to_register);
        instruction.set_first_argument(left_index);
        instruction.set_second_argument(right_index);

        instruction
    }

    pub fn subtract(to_register: u8, left_index: u8, right_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::Subtract as u32);

        instruction.set_destination(to_register);
        instruction.set_first_argument(left_index);
        instruction.set_second_argument(right_index);

        instruction
    }

    pub fn multiply(to_register: u8, left_index: u8, right_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::Multiply as u32);

        instruction.set_destination(to_register);
        instruction.set_first_argument(left_index);
        instruction.set_second_argument(right_index);

        instruction
    }

    pub fn divide(to_register: u8, left_index: u8, right_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::Divide as u32);

        instruction.set_destination(to_register);
        instruction.set_first_argument(left_index);
        instruction.set_second_argument(right_index);

        instruction
    }

    pub fn modulo(to_register: u8, left_index: u8, right_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::Modulo as u32);

        instruction.set_destination(to_register);
        instruction.set_first_argument(left_index);
        instruction.set_second_argument(right_index);

        instruction
    }

    pub fn and(to_register: u8, left_index: u8, right_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::And as u32);

        instruction.set_destination(to_register);
        instruction.set_first_argument(left_index);
        instruction.set_second_argument(right_index);

        instruction
    }

    pub fn or(to_register: u8, left_index: u8, right_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::Or as u32);

        instruction.set_destination(to_register);
        instruction.set_first_argument(left_index);
        instruction.set_second_argument(right_index);

        instruction
    }

    pub fn equal(comparison_boolean: bool, left_index: u8, right_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::Equal as u32);

        instruction.set_destination(if comparison_boolean { 1 } else { 0 });
        instruction.set_first_argument(left_index);
        instruction.set_second_argument(right_index);

        instruction
    }

    pub fn negate(to_register: u8, from_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::Negate as u32);

        instruction.set_destination(to_register);
        instruction.set_first_argument(from_index);

        instruction
    }

    pub fn not(to_register: u8, from_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::Not as u32);

        instruction.set_destination(to_register);
        instruction.set_first_argument(from_index);

        instruction
    }

    pub fn jump(offset: u8, is_positive: bool) -> Instruction {
        let mut instruction = Instruction(Operation::Jump as u32);

        instruction.set_first_argument(offset);
        instruction.set_second_argument(if is_positive { 1 } else { 0 });

        instruction
    }

    pub fn r#return() -> Instruction {
        Instruction(Operation::Return as u32)
    }

    pub fn operation(&self) -> Operation {
        Operation::from((self.0 & 0b0000_0000_0011_1111) as u8)
    }

    pub fn set_operation(&mut self, operation: Operation) {
        self.0 |= u8::from(operation) as u32;
    }

    pub fn destination(&self) -> u8 {
        (self.0 >> 24) as u8
    }

    pub fn destination_as_boolean(&self) -> bool {
        (self.0 >> 24) != 0
    }

    pub fn set_destination(&mut self, destination: u8) {
        self.0 &= 0x00FFFFFF;
        self.0 |= (destination as u32) << 24;
    }

    pub fn first_argument(&self) -> u8 {
        (self.0 >> 16) as u8
    }

    pub fn first_argument_is_constant(&self) -> bool {
        self.0 & 0b1000_0000 != 0
    }

    pub fn first_argument_as_boolean(&self) -> bool {
        self.first_argument() != 0
    }

    pub fn set_first_argument_to_constant(&mut self) -> &mut Self {
        self.0 |= 0b1000_0000;

        self
    }

    pub fn set_first_argument(&mut self, argument: u8) {
        self.0 |= (argument as u32) << 16;
    }

    pub fn second_argument(&self) -> u8 {
        (self.0 >> 8) as u8
    }

    pub fn second_argument_is_constant(&self) -> bool {
        self.0 & 0b0100_0000 != 0
    }

    pub fn second_argument_as_boolean(&self) -> bool {
        self.second_argument() != 0
    }

    pub fn set_second_argument_to_constant(&mut self) -> &mut Self {
        self.0 |= 0b0100_0000;

        self
    }

    pub fn set_second_argument(&mut self, argument: u8) {
        self.0 |= (argument as u32) << 8;
    }

    pub fn disassemble(&self, chunk: &Chunk) -> String {
        let mut disassembled = format!("{:16} ", self.operation().to_string());

        if let Some(info) = self.disassembly_info(Some(chunk)) {
            disassembled.push_str(&info);
        }

        disassembled
    }

    pub fn disassembly_info(&self, chunk: Option<&Chunk>) -> Option<String> {
        let format_arguments = || {
            let first_argument = if self.first_argument_is_constant() {
                format!("C({})", self.first_argument())
            } else {
                format!("R({})", self.first_argument())
            };
            let second_argument = if self.second_argument_is_constant() {
                format!("C({})", self.second_argument())
            } else {
                format!("R({})", self.second_argument())
            };

            (first_argument, second_argument)
        };

        let info = match self.operation() {
            Operation::Move => {
                format!("R({}) = R({})", self.destination(), self.first_argument())
            }
            Operation::Close => {
                let from_register = self.first_argument();
                let to_register = self.second_argument().saturating_sub(1);

                format!("R({from_register})..=R({to_register})")
            }
            Operation::LoadBoolean => {
                let to_register = self.destination();
                let boolean = self.first_argument_as_boolean();
                let skip_display = if self.second_argument_as_boolean() {
                    "IP++"
                } else {
                    ""
                };

                format!("R({to_register}) = {boolean} {skip_display}",)
            }
            Operation::LoadConstant => {
                let constant_index = self.first_argument();

                if let Some(chunk) = chunk {
                    match chunk.get_constant(constant_index, Span(0, 0)) {
                        Ok(value) => {
                            format!(
                                "R({}) = C({}) {}",
                                self.destination(),
                                constant_index,
                                value
                            )
                        }
                        Err(error) => format!(
                            "R({}) = C({}) {:?}",
                            self.destination(),
                            constant_index,
                            error
                        ),
                    }
                } else {
                    format!("R({}) = C({})", self.destination(), constant_index)
                }
            }
            Operation::LoadList => {
                let destination = self.destination();
                let first_index = destination - self.first_argument();
                let last_index = destination - 1;

                format!(
                    "R({}) = [R({})..=R({})]",
                    destination, first_index, last_index
                )
            }
            Operation::DefineLocal => {
                let local_index = self.first_argument();
                let identifier_display = if let Some(chunk) = chunk {
                    match chunk.get_identifier(local_index) {
                        Some(identifier) => identifier.to_string(),
                        None => "???".to_string(),
                    }
                } else {
                    "???".to_string()
                };

                format!(
                    "L({}) = R({}) {}",
                    local_index,
                    self.destination(),
                    identifier_display
                )
            }
            Operation::GetLocal => {
                let local_index = self.first_argument();

                format!("R({}) = L({})", self.destination(), local_index)
            }
            Operation::SetLocal => {
                let local_index = self.first_argument();
                let identifier_display = if let Some(chunk) = chunk {
                    match chunk.get_identifier(local_index) {
                        Some(identifier) => identifier.to_string(),
                        None => "???".to_string(),
                    }
                } else {
                    "???".to_string()
                };

                format!(
                    "L({}) = R({}) {}",
                    local_index,
                    self.destination(),
                    identifier_display
                )
            }
            Operation::Add => {
                let destination = self.destination();
                let (first_argument, second_argument) = format_arguments();

                format!("R({destination}) = {first_argument} + {second_argument}",)
            }
            Operation::Subtract => {
                let destination = self.destination();
                let (first_argument, second_argument) = format_arguments();

                format!("R({destination}) = {first_argument} - {second_argument}",)
            }
            Operation::Multiply => {
                let destination = self.destination();
                let (first_argument, second_argument) = format_arguments();

                format!("R({destination}) = {first_argument} * {second_argument}",)
            }
            Operation::Divide => {
                let destination = self.destination();
                let (first_argument, second_argument) = format_arguments();

                format!("R({destination}) = {first_argument} / {second_argument}",)
            }
            Operation::Modulo => {
                let destination = self.destination();
                let (first_argument, second_argument) = format_arguments();

                format!("R({destination}) = {first_argument} % {second_argument}",)
            }
            Operation::And => {
                let destination = self.destination();
                let (first_argument, second_argument) = format_arguments();

                format!("R({destination}) = {first_argument} && {second_argument}",)
            }
            Operation::Or => {
                let destination = self.destination();
                let (first_argument, second_argument) = format_arguments();

                format!("R({destination}) = {first_argument} || {second_argument}",)
            }
            Operation::Equal => {
                let comparison_symbol = if self.destination_as_boolean() {
                    "=="
                } else {
                    "!="
                };

                let (first_argument, second_argument) = format_arguments();

                format!("if {first_argument} {comparison_symbol} {second_argument} IP++",)
            }
            Operation::Less => {
                let comparison_symbol = if self.destination_as_boolean() {
                    "<"
                } else {
                    ">="
                };
                let (first_argument, second_argument) = format_arguments();

                format!("if {first_argument} {comparison_symbol} {second_argument} IP++",)
            }
            Operation::LessEqual => {
                let comparison_symbol = if self.destination_as_boolean() {
                    "<="
                } else {
                    ">"
                };
                let (first_argument, second_argument) = format_arguments();

                format!("if {first_argument} {comparison_symbol} {second_argument} IP++",)
            }
            Operation::Negate => {
                let destination = self.destination();
                let argument = if self.first_argument_is_constant() {
                    format!("C({})", self.first_argument())
                } else {
                    format!("R({})", self.first_argument())
                };

                format!("R({destination}) = -{argument}")
            }
            Operation::Not => {
                let destination = self.destination();
                let argument = if self.first_argument_is_constant() {
                    format!("C({})", self.first_argument())
                } else {
                    format!("R({})", self.first_argument())
                };

                format!("R({destination}) = !{argument}")
            }
            Operation::Jump => {
                let offset = self.first_argument();
                let positive = self.second_argument() != 0;

                if positive {
                    format!("IP += {}", offset)
                } else {
                    format!("IP -= {}", offset)
                }
            }
            Operation::Return => return None,
        };

        Some(info)
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some(info) = self.disassembly_info(None) {
            write!(f, "{} {}", self.operation(), info)
        } else {
            write!(f, "{}", self.operation())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r#move() {
        let mut instruction = Instruction::r#move(0, 1);

        instruction.set_first_argument_to_constant();
        instruction.set_second_argument_to_constant();

        assert_eq!(instruction.operation(), Operation::Move);
        assert_eq!(instruction.destination(), 0);
        assert_eq!(instruction.first_argument(), 1);
        assert!(instruction.first_argument_is_constant());
        assert!(instruction.second_argument_is_constant());
    }

    #[test]
    fn close() {
        let instruction = Instruction::close(1, 2);

        assert_eq!(instruction.operation(), Operation::Close);
        assert_eq!(instruction.first_argument(), 1);
        assert_eq!(instruction.second_argument(), 2);
    }

    #[test]
    fn load_boolean() {
        let instruction = Instruction::load_boolean(4, true, true);

        assert_eq!(instruction.operation(), Operation::LoadBoolean);
        assert_eq!(instruction.destination(), 4);
        assert!(instruction.first_argument_as_boolean());
        assert!(instruction.second_argument_as_boolean());
    }

    #[test]
    fn load_constant() {
        let mut instruction = Instruction::load_constant(0, 1);

        instruction.set_first_argument_to_constant();
        instruction.set_second_argument_to_constant();

        assert_eq!(instruction.operation(), Operation::LoadConstant);
        assert_eq!(instruction.destination(), 0);
        assert_eq!(instruction.first_argument(), 1);
        assert!(instruction.first_argument_is_constant());
        assert!(instruction.second_argument_is_constant());
    }

    #[test]
    fn declare_local() {
        let mut instruction = Instruction::define_local(0, 1, true);

        instruction.set_first_argument_to_constant();

        assert_eq!(instruction.operation(), Operation::DefineLocal);
        assert_eq!(instruction.destination(), 0);
        assert_eq!(instruction.first_argument(), 1);
        assert_eq!(instruction.second_argument(), true as u8);
        assert!(instruction.first_argument_is_constant());
    }

    #[test]
    fn add() {
        let mut instruction = Instruction::add(1, 1, 0);

        instruction.set_first_argument_to_constant();

        assert_eq!(instruction.operation(), Operation::Add);
        assert_eq!(instruction.destination(), 1);
        assert_eq!(instruction.first_argument(), 1);
        assert_eq!(instruction.second_argument(), 0);
        assert!(instruction.first_argument_is_constant());
    }

    #[test]
    fn subtract() {
        let mut instruction = Instruction::subtract(0, 1, 2);

        instruction.set_first_argument_to_constant();
        instruction.set_second_argument_to_constant();

        assert_eq!(instruction.operation(), Operation::Subtract);
        assert_eq!(instruction.destination(), 0);
        assert_eq!(instruction.first_argument(), 1);
        assert_eq!(instruction.second_argument(), 2);
        assert!(instruction.first_argument_is_constant());
        assert!(instruction.second_argument_is_constant());
    }

    #[test]
    fn multiply() {
        let mut instruction = Instruction::multiply(0, 1, 2);

        instruction.set_first_argument_to_constant();
        instruction.set_second_argument_to_constant();

        assert_eq!(instruction.operation(), Operation::Multiply);
        assert_eq!(instruction.destination(), 0);
        assert_eq!(instruction.first_argument(), 1);
        assert_eq!(instruction.second_argument(), 2);
        assert!(instruction.first_argument_is_constant());
        assert!(instruction.second_argument_is_constant());
    }

    #[test]
    fn divide() {
        let mut instruction = Instruction::divide(0, 1, 2);

        instruction.set_first_argument_to_constant();
        instruction.set_second_argument_to_constant();

        assert_eq!(instruction.operation(), Operation::Divide);
        assert_eq!(instruction.destination(), 0);
        assert_eq!(instruction.first_argument(), 1);
        assert_eq!(instruction.second_argument(), 2);
        assert!(instruction.first_argument_is_constant());
        assert!(instruction.second_argument_is_constant());
    }

    #[test]
    fn and() {
        let mut instruction = Instruction::and(0, 1, 2);

        instruction.set_first_argument_to_constant();
        instruction.set_second_argument_to_constant();

        assert_eq!(instruction.operation(), Operation::And);
        assert_eq!(instruction.destination(), 0);
        assert_eq!(instruction.first_argument(), 1);
        assert_eq!(instruction.second_argument(), 2);
        assert!(instruction.first_argument_is_constant());
        assert!(instruction.second_argument_is_constant());
    }

    #[test]
    fn or() {
        let mut instruction = Instruction::or(0, 1, 2);

        instruction.set_first_argument_to_constant();
        instruction.set_second_argument_to_constant();

        assert_eq!(instruction.operation(), Operation::Or);
        assert_eq!(instruction.destination(), 0);
        assert_eq!(instruction.first_argument(), 1);
        assert_eq!(instruction.second_argument(), 2);
        assert!(instruction.first_argument_is_constant());
        assert!(instruction.second_argument_is_constant());
    }

    #[test]
    fn equal() {
        let mut instruction = Instruction::equal(true, 1, 2);

        instruction.set_first_argument_to_constant();
        instruction.set_second_argument_to_constant();

        assert_eq!(instruction.operation(), Operation::Equal);
        assert!(instruction.destination_as_boolean());
        assert_eq!(instruction.first_argument(), 1);
        assert_eq!(instruction.second_argument(), 2);
        assert!(instruction.first_argument_is_constant());
        assert!(instruction.second_argument_is_constant());
    }

    #[test]
    fn negate() {
        let mut instruction = Instruction::negate(0, 1);

        instruction.set_first_argument_to_constant();
        instruction.set_second_argument_to_constant();

        assert_eq!(instruction.operation(), Operation::Negate);
        assert_eq!(instruction.destination(), 0);
        assert_eq!(instruction.first_argument(), 1);
        assert!(instruction.first_argument_is_constant());
        assert!(instruction.second_argument_is_constant());
    }

    #[test]
    fn not() {
        let mut instruction = Instruction::not(0, 1);

        instruction.set_first_argument_to_constant();
        instruction.set_second_argument_to_constant();

        assert_eq!(instruction.operation(), Operation::Not);
        assert_eq!(instruction.destination(), 0);
        assert_eq!(instruction.first_argument(), 1);
        assert!(instruction.first_argument_is_constant());
        assert!(instruction.second_argument_is_constant());
    }

    #[test]
    fn jump() {
        let mut instruction = Instruction::jump(4, true);

        instruction.set_first_argument_to_constant();
        instruction.set_second_argument_to_constant();

        assert_eq!(instruction.operation(), Operation::Jump);
        assert_eq!(instruction.first_argument(), 4);
        assert!(instruction.first_argument_as_boolean());
    }

    #[test]
    fn r#return() {
        let mut instruction = Instruction::r#return();

        instruction.set_first_argument_to_constant();
        instruction.set_second_argument_to_constant();

        assert_eq!(instruction.operation(), Operation::Return);
        assert!(instruction.first_argument_is_constant());
        assert!(instruction.second_argument_is_constant());
    }
}
