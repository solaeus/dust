//! Virtual machine and errors
use std::{cmp::Ordering, mem::replace};

use crate::{
    parse, value::Primitive, AnnotatedError, Chunk, DustError, FunctionType, Instruction, Local,
    NativeFunction, NativeFunctionError, Operation, Span, Type, Value, ValueError,
};

pub fn run(source: &str) -> Result<Option<Value>, DustError> {
    let chunk = parse(source)?;
    let vm = Vm::new(chunk);

    vm.run()
        .map_err(|error| DustError::Runtime { error, source })
}

/// Dust virtual machine.
///
/// See the [module-level documentation](index.html) for more information.
#[derive(Debug, Eq, PartialEq)]
pub struct Vm {
    ip: usize,
    chunk: Chunk,
    stack: Vec<Register>,
    last_assigned_register: Option<u8>,
}

impl Vm {
    const STACK_LIMIT: usize = u16::MAX as usize;

    pub fn new(chunk: Chunk) -> Self {
        Self {
            ip: 0,
            chunk,
            stack: Vec::new(),
            last_assigned_register: None,
        }
    }

    pub fn run(mut self) -> Result<Option<Value>, VmError> {
        // DRY helper to get constant or register values for binary operations
        fn get_arguments(
            vm: &mut Vm,
            instruction: Instruction,
            position: Span,
        ) -> Result<(&Value, &Value), VmError> {
            let left = if instruction.b_is_constant() {
                vm.chunk.get_constant(instruction.b()).ok_or_else(|| {
                    VmError::ConstantIndexOutOfBounds {
                        index: instruction.b() as usize,
                        position,
                    }
                })?
            } else {
                vm.get_register(instruction.b(), position)?
            };
            let right = if instruction.c_is_constant() {
                vm.chunk.get_constant(instruction.c()).ok_or_else(|| {
                    VmError::ConstantIndexOutOfBounds {
                        index: instruction.c() as usize,
                        position,
                    }
                })?
            } else {
                vm.get_register(instruction.c(), position)?
            };

            Ok((left, right))
        }

        while let Ok((instruction, position)) = self.read(Span(0, 0)).copied() {
            log::info!(
                "{} | {} | {} | {}",
                self.ip - 1,
                position,
                instruction.operation(),
                instruction.disassembly_info(Some(&self.chunk))
            );

            match instruction.operation() {
                Operation::Move => {
                    let to_register = instruction.a();
                    let from_register = instruction.b();

                    self.set_pointer(to_register, from_register, position)?;
                }
                Operation::Close => {
                    let from_register = instruction.b();
                    let to_register = instruction.c();

                    if self.stack.len() < to_register as usize {
                        return Err(VmError::StackUnderflow { position });
                    }

                    for register_index in from_register..to_register {
                        self.stack[register_index as usize] = Register::Empty;
                    }
                }
                Operation::LoadBoolean => {
                    let to_register = instruction.a();
                    let boolean = instruction.b_as_boolean();
                    let jump = instruction.c_as_boolean();
                    let value = Value::boolean(boolean);

                    self.set_register(to_register, value, position)?;

                    if jump {
                        self.ip += 1;
                    }
                }
                Operation::LoadConstant => {
                    let to_register = instruction.a();
                    let from_constant = instruction.b();
                    let jump = instruction.c_as_boolean();

                    self.set_constant(to_register, from_constant, position)?;

                    if jump {
                        self.ip += 1
                    }
                }
                Operation::LoadList => {
                    let to_register = instruction.a();
                    let first_register = instruction.b();
                    let last_register = instruction.c();

                    let is_empty = to_register == first_register && first_register == last_register;
                    let item_type = if is_empty {
                        Type::Any
                    } else {
                        self.get_register(first_register, position)?.r#type()
                    };
                    let value = Value::list(first_register, last_register, item_type);

                    self.set_register(to_register, value, position)?;
                }
                Operation::LoadSelf => {
                    let to_register = instruction.a();
                    let value = Value::function(
                        self.chunk.clone(),
                        FunctionType {
                            type_parameters: None,
                            value_parameters: None,
                            return_type: None,
                        },
                    );

                    self.set_register(to_register, value, position)?;
                }
                Operation::DefineLocal => {
                    let from_register = instruction.a();
                    let to_local = instruction.b();

                    self.define_local(to_local, from_register, position)?;
                }
                Operation::GetLocal => {
                    let to_register = instruction.a();
                    let local_index = instruction.b();
                    let local = self.get_local(local_index, position)?;

                    self.set_pointer(to_register, local.register_index, position)?;
                }
                Operation::SetLocal => {
                    let register = instruction.a();
                    let local_index = instruction.b();
                    let local = self.get_local(local_index, position)?;

                    if !local.is_mutable {
                        return Err(VmError::CannotMutateImmutableLocal {
                            identifier: self
                                .chunk
                                .get_identifier(local.identifier_index)
                                .unwrap_or_else(|| "unknown".to_string()),
                            position,
                        });
                    }

                    self.define_local(local_index, register, position)?;
                }
                Operation::Add => {
                    let (left, right) = get_arguments(&mut self, instruction, position)?;
                    let sum = left
                        .add(right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.set_register(instruction.a(), sum, position)?;
                }
                Operation::Subtract => {
                    let (left, right) = get_arguments(&mut self, instruction, position)?;
                    let difference = left
                        .subtract(right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.set_register(instruction.a(), difference, position)?;
                }
                Operation::Multiply => {
                    let (left, right) = get_arguments(&mut self, instruction, position)?;
                    let product = left
                        .multiply(right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.set_register(instruction.a(), product, position)?;
                }
                Operation::Divide => {
                    let (left, right) = get_arguments(&mut self, instruction, position)?;
                    let quotient = left
                        .divide(right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.set_register(instruction.a(), quotient, position)?;
                }
                Operation::Modulo => {
                    let (left, right) = get_arguments(&mut self, instruction, position)?;
                    let remainder = left
                        .modulo(right)
                        .map_err(|error| VmError::Value { error, position })?;

                    self.set_register(instruction.a(), remainder, position)?;
                }
                Operation::Test => {
                    let register = instruction.a();
                    let test_value = instruction.c_as_boolean();
                    let value = self.get_register(register, position)?;
                    let boolean = if let Value::Primitive(Primitive::Boolean(boolean)) = value {
                        *boolean
                    } else {
                        return Err(VmError::ExpectedBoolean {
                            found: value.clone(),
                            position,
                        });
                    };

                    if boolean != test_value {
                        self.ip += 1;
                    }
                }
                Operation::TestSet => todo!(),
                Operation::Equal => {
                    debug_assert_eq!(
                        self.get_instruction(self.ip, position)?.0.operation(),
                        Operation::Jump
                    );

                    let (left, right) = get_arguments(&mut self, instruction, position)?;
                    let equal_result = left
                        .equal(right)
                        .map_err(|error| VmError::Value { error, position })?;
                    let boolean =
                        if let Value::Primitive(Primitive::Boolean(boolean)) = equal_result {
                            boolean
                        } else {
                            return Err(VmError::ExpectedBoolean {
                                found: equal_result.clone(),
                                position,
                            });
                        };
                    let compare_to = instruction.a_as_boolean();

                    if boolean == compare_to {
                        self.ip += 1;
                    } else {
                        let (jump, _) = self.get_instruction(self.ip, position)?;
                        let jump_distance = jump.a();
                        let is_positive = jump.b_as_boolean();
                        let new_ip = if is_positive {
                            self.ip + jump_distance as usize
                        } else {
                            self.ip - jump_distance as usize
                        };

                        self.ip = new_ip;
                    }
                }
                Operation::Less => {
                    debug_assert_eq!(
                        self.get_instruction(self.ip, position)?.0.operation(),
                        Operation::Jump
                    );

                    let (left, right) = get_arguments(&mut self, instruction, position)?;
                    let less_result = left
                        .less_than(right)
                        .map_err(|error| VmError::Value { error, position })?;
                    let boolean = if let Value::Primitive(Primitive::Boolean(boolean)) = less_result
                    {
                        boolean
                    } else {
                        return Err(VmError::ExpectedBoolean {
                            found: less_result.clone(),
                            position,
                        });
                    };
                    let compare_to = instruction.a_as_boolean();

                    if boolean == compare_to {
                        self.ip += 1;
                    } else {
                        let jump = self.get_instruction(self.ip, position)?.0;
                        let jump_distance = jump.a();
                        let is_positive = jump.b_as_boolean();
                        let new_ip = if is_positive {
                            self.ip + jump_distance as usize
                        } else {
                            self.ip - jump_distance as usize
                        };

                        self.ip = new_ip;
                    }
                }
                Operation::LessEqual => {
                    debug_assert_eq!(
                        self.get_instruction(self.ip, position)?.0.operation(),
                        Operation::Jump
                    );

                    let (left, right) = get_arguments(&mut self, instruction, position)?;
                    let less_or_equal_result = left
                        .less_than_or_equal(right)
                        .map_err(|error| VmError::Value { error, position })?;
                    let boolean = if let Value::Primitive(Primitive::Boolean(boolean)) =
                        less_or_equal_result
                    {
                        boolean
                    } else {
                        return Err(VmError::ExpectedBoolean {
                            found: less_or_equal_result.clone(),
                            position,
                        });
                    };
                    let compare_to = instruction.a_as_boolean();

                    if boolean == compare_to {
                        self.ip += 1;
                    } else {
                        let jump = self.get_instruction(self.ip, position)?.0;
                        let jump_distance = jump.a();
                        let is_positive = jump.b_as_boolean();
                        let new_ip = if is_positive {
                            self.ip + jump_distance as usize
                        } else {
                            self.ip - jump_distance as usize
                        };

                        self.ip = new_ip;
                    }
                }
                Operation::Negate => {
                    let value = if instruction.b_is_constant() {
                        self.chunk.get_constant(instruction.b()).ok_or_else(|| {
                            VmError::ConstantIndexOutOfBounds {
                                index: instruction.b() as usize,
                                position,
                            }
                        })?
                    } else {
                        self.get_register(instruction.b(), position)?
                    };
                    let negated = value
                        .negate()
                        .map_err(|error| VmError::Value { error, position })?;

                    self.set_register(instruction.a(), negated, position)?;
                }
                Operation::Not => {
                    let value = if instruction.b_is_constant() {
                        self.chunk.get_constant(instruction.b()).ok_or_else(|| {
                            VmError::ConstantIndexOutOfBounds {
                                index: instruction.b() as usize,
                                position,
                            }
                        })?
                    } else {
                        self.get_register(instruction.b(), position)?
                    };
                    let not = value
                        .not()
                        .map_err(|error| VmError::Value { error, position })?;

                    self.set_register(instruction.a(), not, position)?;
                }
                Operation::Jump => {
                    let jump_distance = instruction.b();
                    let is_positive = instruction.c_as_boolean();
                    let new_ip = if is_positive {
                        self.ip + jump_distance as usize
                    } else {
                        self.ip - jump_distance as usize - 1
                    };
                    self.ip = new_ip;
                }
                Operation::Call => {
                    let to_register = instruction.a();
                    let function_register = instruction.b();
                    let argument_count = instruction.c();
                    let value = self.get_register(function_register, position)?.clone();
                    let function = if let Value::Function(function) = value {
                        function
                    } else {
                        return Err(VmError::ExpectedFunction {
                            found: value,
                            position,
                        });
                    };
                    let mut function_vm = Vm::new(function.take_chunk());
                    let first_argument_index = function_register + 1;

                    for argument_index in
                        first_argument_index..first_argument_index + argument_count
                    {
                        let argument = match self.get_register(argument_index, position) {
                            Ok(value) => value.clone(),
                            Err(VmError::EmptyRegister { .. }) => continue,
                            Err(error) => return Err(error),
                        };
                        let top_of_stack = function_vm.stack.len() as u8;

                        function_vm.set_register(top_of_stack, argument, position)?;
                    }

                    let return_value = function_vm.run()?;

                    if let Some(value) = return_value {
                        self.set_register(to_register, value, position)?;
                    }
                }
                Operation::CallNative => {
                    let native_function = NativeFunction::from(instruction.b());
                    let return_value = native_function.call(instruction, &self, position)?;

                    if let Some(value) = return_value {
                        let to_register = instruction.a();

                        self.set_register(to_register, value, position)?;
                    }
                }
                Operation::Return => {
                    let should_return_value = instruction.b_as_boolean();

                    if !should_return_value {
                        return Ok(None);
                    }

                    if let Some(register) = self.last_assigned_register {
                        let value = self.empty_register(register, position)?;

                        return Ok(Some(value));
                    } else {
                        return Err(VmError::StackUnderflow { position });
                    }
                }
            }
        }

        Ok(None)
    }

    fn set_register(
        &mut self,
        to_register: u8,
        value: Value,
        position: Span,
    ) -> Result<(), VmError> {
        let length = self.stack.len();
        self.last_assigned_register = Some(to_register);
        let to_register = to_register as usize;

        if length == Self::STACK_LIMIT {
            return Err(VmError::StackOverflow { position });
        }

        match to_register.cmp(&length) {
            Ordering::Less => {
                log::trace!("Change R{to_register} to {value}");

                self.stack[to_register] = Register::Value(value);

                Ok(())
            }
            Ordering::Equal => {
                log::trace!("Set R{to_register} to {value}");

                self.stack.push(Register::Value(value));

                Ok(())
            }
            Ordering::Greater => {
                let difference = to_register - length;

                for index in 0..difference {
                    log::trace!("Set R{index} to empty");

                    self.stack.push(Register::Empty);
                }

                log::trace!("Set R{to_register} to {value}");

                self.stack.push(Register::Value(value));

                Ok(())
            }
        }
    }

    fn set_pointer(
        &mut self,
        to_register: u8,
        from_register: u8,
        position: Span,
    ) -> Result<(), VmError> {
        let length = self.stack.len();
        self.last_assigned_register = Some(to_register);
        let to_register = to_register as usize;

        if length == Self::STACK_LIMIT {
            return Err(VmError::StackOverflow { position });
        }

        match to_register.cmp(&length) {
            Ordering::Less => {
                log::trace!("Change R{to_register} to R{from_register}");

                self.stack[to_register] = Register::Pointer(from_register);

                Ok(())
            }
            Ordering::Equal => {
                log::trace!("Set R{to_register} to R{from_register}");

                self.stack.push(Register::Pointer(from_register));

                Ok(())
            }
            Ordering::Greater => {
                let difference = to_register - length;

                for index in 0..difference {
                    log::trace!("Set R{index} to empty");

                    self.stack.push(Register::Empty);
                }

                log::trace!("Set R{to_register} to R{from_register}");

                self.stack.push(Register::Pointer(from_register));

                Ok(())
            }
        }
    }

    fn set_constant(
        &mut self,
        to_register: u8,
        constant_index: u8,
        position: Span,
    ) -> Result<(), VmError> {
        let length = self.stack.len();
        self.last_assigned_register = Some(to_register);
        let to_register = to_register as usize;

        if length == Self::STACK_LIMIT {
            return Err(VmError::StackOverflow { position });
        }

        match to_register.cmp(&length) {
            Ordering::Less => {
                log::trace!("Change R{to_register} to C{constant_index}");

                self.stack[to_register] = Register::Constant(constant_index);

                Ok(())
            }
            Ordering::Equal => {
                log::trace!("Set R{to_register} to C{constant_index}");

                self.stack.push(Register::Constant(constant_index));

                Ok(())
            }
            Ordering::Greater => {
                let difference = to_register - length;

                for index in 0..difference {
                    log::trace!("Set R{index} to empty");

                    self.stack.push(Register::Empty);
                }

                log::trace!("Set R{to_register} to C{constant_index}");

                self.stack.push(Register::Constant(constant_index));

                Ok(())
            }
        }
    }

    pub fn get_register(&self, index: u8, position: Span) -> Result<&Value, VmError> {
        let index = index as usize;
        let register = self
            .stack
            .get(index)
            .ok_or_else(|| VmError::RegisterIndexOutOfBounds { index, position })?;

        match register {
            Register::Value(value) => Ok(value),
            Register::Pointer(register_index) => self.get_register(*register_index, position),
            Register::Constant(constant_index) => self
                .chunk
                .get_constant(*constant_index)
                .ok_or_else(|| VmError::ConstantIndexOutOfBounds {
                    index: *constant_index as usize,
                    position,
                }),
            Register::Empty => Err(VmError::EmptyRegister { index, position }),
        }
    }

    fn empty_register(mut self, index: u8, position: Span) -> Result<Value, VmError> {
        let index = index as usize;

        if index >= self.stack.len() {
            return Err(VmError::RegisterIndexOutOfBounds { index, position });
        }

        let register = replace(&mut self.stack[index], Register::Empty);

        match register {
            Register::Value(value) => Ok(value),
            Register::Pointer(register_index) => {
                let value = self.empty_register(register_index, position)?;

                Ok(value)
            }
            Register::Constant(constant_index) => {
                let value = self.chunk.take_constants().remove(constant_index as usize);

                Ok(value)
            }
            Register::Empty => Err(VmError::EmptyRegister { index, position }),
        }
    }

    fn read(&mut self, position: Span) -> Result<&(Instruction, Span), VmError> {
        if self.ip >= self.chunk.len() {
            self.ip = self.chunk.len() - 1;
        }

        let current = self.chunk.instructions().get(self.ip).ok_or_else(|| {
            VmError::InstructionIndexOutOfBounds {
                index: self.ip,
                position,
            }
        })?;

        self.ip += 1;

        Ok(current)
    }

    fn define_local(
        &mut self,
        local_index: u8,
        register_index: u8,
        position: Span,
    ) -> Result<(), VmError> {
        let local = self
            .chunk
            .locals_mut()
            .get_mut(local_index as usize)
            .ok_or_else(|| VmError::LocalIndexOutOfBounds {
                index: local_index as usize,
                position,
            })?;

        log::debug!("Define local L{}", local_index);

        local.register_index = register_index;

        Ok(())
    }

    fn get_local(&self, local_index: u8, position: Span) -> Result<&Local, VmError> {
        let local_index = local_index as usize;

        self.chunk
            .locals()
            .get(local_index)
            .ok_or_else(|| VmError::LocalIndexOutOfBounds {
                index: local_index,
                position,
            })
    }

    fn get_instruction(
        &self,
        index: usize,
        position: Span,
    ) -> Result<&(Instruction, Span), VmError> {
        self.chunk
            .instructions()
            .get(index)
            .ok_or_else(|| VmError::InstructionIndexOutOfBounds { index, position })
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Register {
    Empty,
    Value(Value),
    Pointer(u8),
    Constant(u8),
}

#[derive(Debug, Clone, PartialEq)]
pub enum VmError {
    CannotMutateImmutableLocal {
        identifier: String,
        position: Span,
    },
    EmptyRegister {
        index: usize,
        position: Span,
    },
    ExpectedBoolean {
        found: Value,
        position: Span,
    },
    ExpectedFunction {
        found: Value,
        position: Span,
    },
    ConstantIndexOutOfBounds {
        index: usize,
        position: Span,
    },
    RegisterIndexOutOfBounds {
        index: usize,
        position: Span,
    },
    InstructionIndexOutOfBounds {
        index: usize,
        position: Span,
    },
    LocalIndexOutOfBounds {
        index: usize,
        position: Span,
    },
    InvalidInstruction {
        instruction: Instruction,
        position: Span,
    },
    StackOverflow {
        position: Span,
    },
    StackUnderflow {
        position: Span,
    },
    UndefinedVariable {
        identifier: String,
        position: Span,
    },

    // Wrappers for foreign errors
    NativeFunction(NativeFunctionError),
    Value {
        error: ValueError,
        position: Span,
    },
}

impl AnnotatedError for VmError {
    fn title() -> &'static str {
        "Runtime Error"
    }

    fn description(&self) -> &'static str {
        match self {
            Self::CannotMutateImmutableLocal { .. } => "Cannot mutate immutable variable",
            Self::ConstantIndexOutOfBounds { .. } => "Constant index out of bounds",
            Self::EmptyRegister { .. } => "Empty register",
            Self::ExpectedBoolean { .. } => "Expected boolean",
            Self::ExpectedFunction { .. } => "Expected function",
            Self::RegisterIndexOutOfBounds { .. } => "Register index out of bounds",
            Self::InstructionIndexOutOfBounds { .. } => "Instruction index out of bounds",
            Self::InvalidInstruction { .. } => "Invalid instruction",
            Self::LocalIndexOutOfBounds { .. } => "Local index out of bounds",
            Self::StackOverflow { .. } => "Stack overflow",
            Self::StackUnderflow { .. } => "Stack underflow",
            Self::UndefinedVariable { .. } => "Undefined variable",
            Self::NativeFunction(error) => error.description(),
            Self::Value { .. } => "Value error",
        }
    }

    fn details(&self) -> Option<String> {
        match self {
            Self::ConstantIndexOutOfBounds { index, .. } => {
                Some(format!("Constant C{index} does not exist"))
            }
            Self::EmptyRegister { index, .. } => Some(format!("Register {index} is empty")),
            Self::ExpectedFunction { found, .. } => Some(format!("{found} is not a function")),
            Self::RegisterIndexOutOfBounds { index, .. } => {
                Some(format!("Register {index} does not exist"))
            }
            Self::InstructionIndexOutOfBounds { index, .. } => {
                Some(format!("Instruction {index} does not exist"))
            }
            Self::LocalIndexOutOfBounds { index, .. } => {
                Some(format!("Local L{index} does not exist"))
            }
            Self::UndefinedVariable { identifier, .. } => {
                Some(format!("{identifier} is not in scope"))
            }
            Self::NativeFunction(error) => error.details(),
            Self::Value { error, .. } => Some(error.to_string()),
            _ => None,
        }
    }

    fn position(&self) -> Span {
        match self {
            Self::CannotMutateImmutableLocal { position, .. } => *position,
            Self::ConstantIndexOutOfBounds { position, .. } => *position,
            Self::EmptyRegister { position, .. } => *position,
            Self::ExpectedBoolean { position, .. } => *position,
            Self::ExpectedFunction { position, .. } => *position,
            Self::RegisterIndexOutOfBounds { position, .. } => *position,
            Self::InstructionIndexOutOfBounds { position, .. } => *position,
            Self::InvalidInstruction { position, .. } => *position,
            Self::LocalIndexOutOfBounds { position, .. } => *position,
            Self::StackUnderflow { position } => *position,
            Self::StackOverflow { position } => *position,
            Self::UndefinedVariable { position, .. } => *position,
            Self::NativeFunction(error) => error.position(),
            Self::Value { position, .. } => *position,
        }
    }
}
