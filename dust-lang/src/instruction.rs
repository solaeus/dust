use crate::{Chunk, Operation, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Instruction(u32);

impl Instruction {
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
        instruction.set_b(if value { 1 } else { 0 });
        instruction.set_c(if skip { 1 } else { 0 });

        instruction
    }

    pub fn load_constant(to_register: u8, constant_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::LoadConstant as u32);

        instruction.set_a(to_register);
        instruction.set_b(constant_index);

        instruction
    }

    pub fn load_list(to_register: u8, start_register: u8, list_length: u8) -> Instruction {
        let mut instruction = Instruction(Operation::LoadList as u32);

        instruction.set_a(to_register);
        instruction.set_b(start_register);
        instruction.set_c(list_length);

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

        instruction.set_a(if comparison_boolean { 1 } else { 0 });
        instruction.set_b(left_index);
        instruction.set_c(right_index);

        instruction
    }

    pub fn less(comparison_boolean: bool, left_index: u8, right_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::Less as u32);

        instruction.set_a(if comparison_boolean { 1 } else { 0 });
        instruction.set_b(left_index);
        instruction.set_c(right_index);

        instruction
    }

    pub fn less_equal(comparison_boolean: bool, left_index: u8, right_index: u8) -> Instruction {
        let mut instruction = Instruction(Operation::LessEqual as u32);

        instruction.set_a(if comparison_boolean { 1 } else { 0 });
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

    pub fn jump(offset: u8, is_positive: bool) -> Instruction {
        let mut instruction = Instruction(Operation::Jump as u32);

        instruction.set_b(offset);
        instruction.set_c(if is_positive { 1 } else { 0 });

        instruction
    }

    pub fn r#return(from_register: u8, to_register: u8) -> Instruction {
        let mut instruction = Instruction(Operation::Return as u32);

        instruction.set_a(from_register);
        instruction.set_b(to_register);

        instruction
    }

    pub fn end(returns_value: bool) -> Instruction {
        let mut instruction = Instruction(Operation::End as u32);

        instruction.set_a_to_boolean(returns_value);

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

    pub fn set_a(&mut self, destination: u8) {
        self.0 |= (destination as u32) << 24;
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

    pub fn disassembly_info(&self, chunk: Option<&Chunk>) -> (Option<String>, Option<isize>) {
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
        let mut jump_offset = None;

        let info = match self.operation() {
            Operation::Move => Some(format!("R{} = R{}", self.a(), self.b())),
            Operation::Close => {
                let from_register = self.b();
                let to_register = self.c().saturating_sub(1);

                Some(format!("R{from_register}..=R{to_register}"))
            }
            Operation::LoadBoolean => {
                let to_register = self.a();
                let boolean = self.b_as_boolean();
                let jump = self.c_as_boolean();
                let info = if jump {
                    jump_offset = Some(1);

                    format!("R{to_register} = {boolean} && JUMP")
                } else {
                    format!("R{to_register} = {boolean}")
                };

                Some(info)
            }
            Operation::LoadConstant => {
                let constant_index = self.b();

                if let Some(chunk) = chunk {
                    match chunk.get_constant(constant_index, Span(0, 0)) {
                        Ok(value) => Some(format!("R{} = C{} {}", self.a(), constant_index, value)),
                        Err(error) => {
                            Some(format!("R{} = C{} {:?}", self.a(), constant_index, error))
                        }
                    }
                } else {
                    Some(format!("R{} = C{}", self.a(), constant_index))
                }
            }
            Operation::LoadList => {
                let destination = self.a();
                let first_index = self.b();
                let last_index = destination.saturating_sub(1);

                Some(format!(
                    "R{} = [R{}..=R{}]",
                    destination, first_index, last_index
                ))
            }
            Operation::DefineLocal => {
                let destination = self.a();
                let local_index = self.b();
                let identifier_display = if let Some(chunk) = chunk {
                    match chunk.get_identifier(local_index) {
                        Some(identifier) => identifier.to_string(),
                        None => "???".to_string(),
                    }
                } else {
                    "???".to_string()
                };
                let mutable_display = if self.c_as_boolean() { "mut" } else { "" };

                Some(format!(
                    "L{local_index} = R{destination} {mutable_display} {identifier_display}"
                ))
            }
            Operation::GetLocal => {
                let local_index = self.b();

                Some(format!("R{} = L{}", self.a(), local_index))
            }
            Operation::SetLocal => {
                let local_index = self.b();
                let identifier_display = if let Some(chunk) = chunk {
                    match chunk.get_identifier(local_index) {
                        Some(identifier) => identifier.to_string(),
                        None => "???".to_string(),
                    }
                } else {
                    "???".to_string()
                };

                Some(format!(
                    "L{} = R{} {}",
                    local_index,
                    self.a(),
                    identifier_display
                ))
            }
            Operation::Add => {
                let destination = self.a();
                let (first_argument, second_argument) = format_arguments();

                Some(format!(
                    "R{destination} = {first_argument} + {second_argument}",
                ))
            }
            Operation::Subtract => {
                let destination = self.a();
                let (first_argument, second_argument) = format_arguments();

                Some(format!(
                    "R{destination} = {first_argument} - {second_argument}",
                ))
            }
            Operation::Multiply => {
                let destination = self.a();
                let (first_argument, second_argument) = format_arguments();

                Some(format!(
                    "R{destination} = {first_argument} * {second_argument}",
                ))
            }
            Operation::Divide => {
                let destination = self.a();
                let (first_argument, second_argument) = format_arguments();

                Some(format!(
                    "R{destination} = {first_argument} / {second_argument}",
                ))
            }
            Operation::Modulo => {
                let destination = self.a();
                let (first_argument, second_argument) = format_arguments();

                Some(format!(
                    "R{destination} = {first_argument} % {second_argument}",
                ))
            }
            Operation::Test => {
                let destination = self.a();
                let test_value = self.c_as_boolean();

                jump_offset = Some(1);

                Some(format!("if R{destination} != {test_value} {{ JUMP }}",))
            }
            Operation::TestSet => {
                let destination = self.a();
                let argument = format!("R{}", self.b());
                let test_value = self.c_as_boolean();
                let bang = if test_value { "" } else { "!" };

                jump_offset = Some(1);

                Some(format!(
                    "if {bang}R{destination} {{ R{destination} = R{argument} }}",
                ))
            }
            Operation::Equal => {
                let comparison_symbol = if self.a_as_boolean() { "==" } else { "!=" };

                let (first_argument, second_argument) = format_arguments();
                jump_offset = Some(1);

                Some(format!(
                    "if {first_argument} {comparison_symbol} {second_argument} {{ JUMP }}",
                ))
            }
            Operation::Less => {
                let comparison_symbol = if self.a_as_boolean() { "<" } else { ">=" };
                let (first_argument, second_argument) = format_arguments();
                jump_offset = Some(1);

                Some(format!(
                    "if {first_argument} {comparison_symbol} {second_argument}",
                ))
            }
            Operation::LessEqual => {
                let comparison_symbol = if self.a_as_boolean() { "<=" } else { ">" };
                let (first_argument, second_argument) = format_arguments();
                jump_offset = Some(1);

                Some(format!(
                    "if {first_argument} {comparison_symbol} {second_argument}",
                ))
            }
            Operation::Negate => {
                let destination = self.a();
                let argument = if self.b_is_constant() {
                    format!("C{}", self.b())
                } else {
                    format!("R{}", self.b())
                };

                Some(format!("R{destination} = -{argument}"))
            }
            Operation::Not => {
                let destination = self.a();
                let argument = if self.b_is_constant() {
                    format!("C{}", self.b())
                } else {
                    format!("R{}", self.b())
                };

                Some(format!("R{destination} = !{argument}"))
            }
            Operation::Jump => {
                let offset = self.b() as isize;
                let is_positive = self.c_as_boolean();

                if is_positive {
                    jump_offset = Some(offset);
                } else {
                    jump_offset = Some(-offset);
                }

                None
            }
            Operation::Return => {
                let from_register = self.a();
                let to_register = self.b();

                Some(format!("R{from_register}..=R{to_register}"))
            }
            Operation::End => {
                let return_value = self.a_as_boolean();

                if return_value {
                    Some("return".to_string())
                } else {
                    Some("null".to_string())
                }
            }
        };

        (info, jump_offset)
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
        let mut instruction = Instruction::load_constant(0, 1);

        instruction.set_b_is_constant();
        instruction.set_c_is_constant();

        assert_eq!(instruction.operation(), Operation::LoadConstant);
        assert_eq!(instruction.a(), 0);
        assert_eq!(instruction.b(), 1);
        assert!(instruction.b_is_constant());
        assert!(instruction.b_is_constant());
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
    fn r#return() {
        let instruction = Instruction::r#return(4, 8);

        assert_eq!(instruction.operation(), Operation::Return);
        assert_eq!(instruction.a(), 4);
        assert_eq!(instruction.b(), 8);
    }

    #[test]
    fn end() {
        let instruction = Instruction::end(true);

        assert_eq!(instruction.operation(), Operation::End);
        assert!(instruction.a_as_boolean());
    }
}
