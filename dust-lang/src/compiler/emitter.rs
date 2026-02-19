use std::collections::HashMap;

use rustc_hash::FxBuildHasher;
use smallvec::SmallVec;
use tracing::{debug, trace};

use crate::{
    compiler::error::{CompileError, InternalCompileError},
    constant_table::{ConstantId, ConstantTable},
    dust_type::DustType,
    instruction::{Address, Drop, Instruction, MemoryKind, Move, Operation, Test},
    native_function::NativeFunction,
    prototype::{Prototype, PrototypeId, PrototypeList},
    resolver::{
        Resolver,
        declaration_graph::{DeclarationId, DeclarationKind},
        scope_graph::ScopeId,
        type_graph::{TypeId, TypeNode},
    },
    small_type::SmallType,
    source::{Position, Source, Span},
    syntax::{Syntax, SyntaxError, SyntaxKind, SyntaxReader, SyntaxReaderIterator, SyntaxVisitor},
};

#[derive(Debug)]
pub struct Emitter<'a> {
    function: SyntaxReader<'a>,

    prototype_id: PrototypeId,

    source: &'a Source<'a>,

    syntax: &'a Syntax,

    constants: &'a mut ConstantTable,

    resolver: &'a mut Resolver,

    prototypes: &'a mut PrototypeList,

    /// Emitted bytecode instructions, filled during compilation.
    instructions: Vec<Instruction>,

    /// Local variables declared in the function.
    locals: HashMap<DeclarationId, Place, FxBuildHasher>,

    /// Concatenated list of arguments referenced by CALL instructions.
    call_arguments: Vec<Address>,

    /// Concatenated list of register indices that are referenced by DROP and JUMP instructions.
    drop_lists: Vec<u16>,

    /// Stack of register index lists that need to be dropped when exiting scopes.
    pending_drops: Vec<SmallVec<[u16; 8]>>,

    jump_placements: HashMap<u16, JumpPlacement>,

    jump_over_else_anchor_ids: Vec<u16>,

    current_scope_id: ScopeId,

    next_jump_id: u16,

    next_local_register: u16,

    next_temporary_register: u16,

    top_allocated_register: u16,

    top_emitted_register: u16,
}

impl<'a> Emitter<'a> {
    pub fn new(
        function: SyntaxReader<'a>,
        declaration_id: DeclarationId,
        starting_scope_id: ScopeId,
        prototype_id: PrototypeId,
        parameters: Option<SyntaxReaderIterator<'a>>,
        (source, syntax, constants, resolver, prototypes): (
            &'a Source,
            &'a Syntax,
            &'a mut ConstantTable,
            &'a mut Resolver,
            &'a mut PrototypeList,
        ),
    ) -> Result<Self, CompileError> {
        let parameter_count = parameters.as_ref().map_or(0, |parameters| parameters.len());
        let mut emitter = Self {
            function,
            prototype_id,
            source,
            syntax,
            constants,
            resolver,
            prototypes,
            instructions: Vec::new(),
            locals: HashMap::with_capacity_and_hasher(parameter_count + 1, FxBuildHasher),
            call_arguments: Vec::new(),
            drop_lists: Vec::new(),
            pending_drops: vec![SmallVec::new()],
            jump_placements: HashMap::new(),
            jump_over_else_anchor_ids: Vec::new(),
            current_scope_id: starting_scope_id,
            next_jump_id: 0,
            next_local_register: 0,
            next_temporary_register: 0,
            top_allocated_register: 0,
            top_emitted_register: 0,
        };

        emitter
            .locals
            .insert(declaration_id, Place::Prototype { prototype_id });

        if let Some(parameters) = parameters {
            let type_id = *emitter
                .resolver
                .declarations
                .get_declaration_type(&declaration_id)?;
            let type_node = *emitter.resolver.types.get_type(type_id)?;
            let TypeNode::Function {
                value_parameters, ..
            } = type_node
            else {
                return Err(CompileError::ExpectedFunctionType {
                    found: type_id,
                    position: function.position(),
                });
            };
            let value_parameter_types = emitter
                .resolver
                .types
                .get_type_members(value_parameters)?
                .iter()
                .copied()
                .collect::<SmallVec<[TypeId; 8]>>();

            for (parameter, expected_type) in parameters
                .into_iter()
                .zip(value_parameter_types.into_iter())
            {
                let parameter_id = *emitter.resolver.get_declaration_binding(&parameter.id)?;
                let register_size = emitter
                    .resolver
                    .get_register_size(expected_type, &parameter)?;
                let target = emitter.allocate_local_registers(register_size);

                emitter.locals.insert(parameter_id, Place::Target(target));
            }
        }

        Ok(emitter)
    }

    pub fn emit(mut self) -> Result<Prototype, CompileError> {
        let function_body = self.function.expect_right_child()?.expect_right_child()?;

        match function_body.kind() {
            SyntaxKind::BlockExpression => {
                let children = function_body.children();
                let last_index = children.len() - 1;

                for (index, child) in children.into_iter().enumerate() {
                    let child_emission = if index == last_index {
                        self.handle_implicit_return(child, None)?
                    } else if child.is_expression() {
                        self.visit_expression(child, None)?
                    } else if child.is_statement() {
                        let instructions = self.visit_statement(child)?;

                        Emission::Instructions(instructions)
                    } else {
                        self.visit_item(child)?;

                        continue;
                    };

                    self.handle_top_emission(child_emission, child)?;
                }
            }
            _ => {
                return Err(CompileError::Internal(
                    InternalCompileError::InvalidSyntaxNode(function_body.kind()),
                ));
            }
        }

        self.finish()
    }

    pub fn finish(mut self) -> Result<Prototype, CompileError> {
        // self.context.constants.finalize_string_pool();

        for JumpPlacement {
            index,
            distance,
            forward,
            coalesce,
        } in self.jump_placements.into_values()
        {
            if coalesce {
                let instruction = &mut self.instructions[index];

                match instruction.operation() {
                    Operation::DROP => {
                        let Drop {
                            drop_list_start,
                            drop_list_end,
                        } = Drop::from(&*instruction);

                        *instruction = Instruction::jump_with_drops(
                            distance,
                            forward,
                            drop_list_start,
                            drop_list_end,
                        )
                    }
                    Operation::MOVE => {
                        let Move {
                            destination,
                            operand,
                            jump_distance,
                            ..
                        } = Move::from(&*instruction);
                        let total_distance = jump_distance + distance;

                        *instruction = Instruction::move_with_jump(
                            destination,
                            operand,
                            total_distance,
                            forward,
                        );
                    }
                    Operation::TEST => {
                        let Test {
                            comparator,
                            operand,
                            jump_distance,
                            ..
                        } = Test::from(&*instruction);
                        let total_distance = jump_distance + distance;

                        *instruction = Instruction::test(operand, comparator, total_distance);
                    }
                    _ => {
                        return Err(CompileError::Internal(
                            InternalCompileError::InvalidJumpAnchorInstruction(
                                instruction.operation(),
                            ),
                        ));
                    }
                }
            } else {
                let jump_instruction = Instruction::jump(distance, forward);

                self.instructions[index] = jump_instruction;
            }
        }

        let declaration_id = self.resolver.get_declaration_binding(&self.function.id)?;
        let type_id = *self
            .resolver
            .declarations
            .get_declaration_type(declaration_id)?;
        let return_type = {
            let get_type = self
                .resolver
                .get_full_type(type_id, self.source)?
                .into_function_type()
                .map(|function_type| function_type.return_type);

            if let Some(function_type) = get_type {
                function_type
            } else {
                let function_declaration_id =
                    *self.resolver.get_declaration_binding(&self.function.id)?;
                let function_declaration = self
                    .resolver
                    .declarations
                    .get_declaration(function_declaration_id)?;
                let position = function_declaration.position.ok_or(CompileError::Internal(
                    InternalCompileError::MissingDeclarationPosition(function_declaration_id),
                ))?;

                return Err(CompileError::ExpectedFunctionType {
                    found: type_id,
                    position,
                });
            }
        };

        Ok(Prototype {
            instructions: self.instructions,
            call_arguments: self.call_arguments,
            drops: self.drop_lists,
            return_type,
            register_count: self.top_emitted_register,
        })
    }

    fn emit_instruction(&mut self, instruction: Instruction) {
        trace!("Emitting {} instruction", instruction.operation());

        self.top_emitted_register = self.top_emitted_register.max(self.top_allocated_register);

        self.instructions.push(instruction);
    }

    fn create_jump_id(&mut self) -> u16 {
        let anchor_id = self.next_jump_id;

        self.next_jump_id += 1;

        anchor_id
    }

    fn allocate_temporary_registers(&mut self, count: u16) -> TargetRegister {
        let target = match count {
            0 => panic!("Cannot allocate zero registers"),
            1 => {
                let index = self.next_temporary_register;
                self.next_temporary_register += 1;

                trace!("Allocating temporary reg_{}", index);

                TargetRegister::Single {
                    index,
                    is_temporary: true,
                }
            }
            2.. => {
                let reference_index = self.next_temporary_register;
                self.next_temporary_register += count;

                trace!(
                    "Allocating temporaries reg_{}..=reg_{}",
                    reference_index,
                    reference_index + count
                );

                TargetRegister::Compound {
                    base_index: reference_index,
                    length: count,
                    is_temporary: true,
                }
            }
        };

        if self.next_temporary_register > self.top_allocated_register {
            self.top_allocated_register = self.next_temporary_register;
        }

        target
    }

    fn free_temporary_registers(&mut self, count: u16) {
        debug_assert!(self.next_temporary_register >= self.next_local_register + count);
        trace!(
            "Freeing temporary registers: {}",
            ((self.next_temporary_register - count)..(self.next_temporary_register))
                .map(|register| format!("reg_{register}"))
                .collect::<String>()
        );

        self.next_temporary_register -= count;

        if self.top_allocated_register > self.next_temporary_register {
            self.top_allocated_register = self.next_temporary_register;
        }

        if self.top_allocated_register < self.top_emitted_register {
            self.top_allocated_register = self.top_emitted_register;
        }
    }

    fn allocate_local_registers(&mut self, count: u16) -> TargetRegister {
        let target = match count {
            0 => panic!("Cannot allocate zero registers"),
            1 => {
                let index = self.next_local_register;
                self.next_local_register += 1;

                trace!("Allocating local reg_{}", index);

                TargetRegister::Single {
                    index,
                    is_temporary: false,
                }
            }
            2.. => {
                let reference_index = self.next_local_register;
                self.next_local_register += count;

                trace!(
                    "Allocating locals reg_{}..=reg_{}",
                    reference_index,
                    reference_index + count
                );

                TargetRegister::Compound {
                    base_index: reference_index,
                    length: count,
                    is_temporary: false,
                }
            }
        };

        self.next_temporary_register = self.next_temporary_register.max(self.next_local_register);

        if self.next_local_register > self.top_allocated_register {
            self.top_allocated_register = self.next_local_register;
        }
        if self.next_temporary_register > self.top_allocated_register {
            self.top_allocated_register = self.next_temporary_register;
        }

        target
    }

    fn enter_child_scope(&mut self, child_scope_id: ScopeId) {
        self.current_scope_id = child_scope_id;

        self.pending_drops.push(SmallVec::new());
    }

    fn enter_parent_scope(
        &mut self,
        parent_scope_id: ScopeId,
        next_local_register: u16,
        next_temporary_register: u16,
    ) {
        self.current_scope_id = parent_scope_id;
        self.next_local_register = next_local_register;
        self.next_temporary_register = next_temporary_register;
    }

    fn add_drop(&mut self, register: u16) {
        self.pending_drops.last_mut().unwrap().push(register);
    }

    fn handle_drops(&mut self, instructions: &mut InstructionsEmission) {
        let start = self.drop_lists.len() as u16;
        let mut pending_drops_for_scope = self.pending_drops.pop().unwrap();

        for register in pending_drops_for_scope.drain(..) {
            if let Some(target) = instructions.target
                && register == target.index()
            {
                continue;
            }

            self.drop_lists.push(register);
        }

        let end = self.drop_lists.len() as u16;

        if start == end {
            return;
        }

        if let Some((last_instruction, _)) = instructions.instructions.last_mut() {
            match last_instruction.operation() {
                Operation::DROP => {
                    if last_instruction.b_field() == start {
                        last_instruction.set_b_field(end);
                    }
                }
                Operation::JUMP => {
                    last_instruction.set_b_field(start);
                    last_instruction.set_c_field(end);
                }
                _ => {
                    let drop_instruction = Instruction::drop(start, end);

                    instructions.push(drop_instruction);
                }
            }
        }
    }

    fn get_constant_address(&mut self, constant: ConstantEmission) -> Address {
        let constant_id = match constant {
            ConstantEmission::Boolean(boolean) => return Address::encoded_boolean(boolean),
            ConstantEmission::Byte(byte) => return Address::encoded_byte(byte),
            ConstantEmission::Character(character) => self.constants.add_character(character),
            ConstantEmission::Float(float) => self.constants.add_float(float),
            ConstantEmission::Integer(integer) => self.constants.add_integer(integer),
            ConstantEmission::String {
                pool_start,
                pool_end,
            } => self.constants.add_pooled_string(pool_start, pool_end),
        };

        Address::constant(constant_id.0)
    }

    fn combine_constants(
        &mut self,
        operation: SyntaxKind,
        left: ConstantEmission,
        left_node: &SyntaxReader,
        right: ConstantEmission,
        right_node: &SyntaxReader,
    ) -> Result<ConstantEmission, CompileError> {
        let check_for_division_by_zero = || {
            if matches!(
                right,
                ConstantEmission::Byte(0)
                    | ConstantEmission::Integer(0)
                    | ConstantEmission::Float(0.0)
            ) {
                Err(CompileError::DivisionByZero {
                    position: Position::new(
                        left_node.file_id(),
                        Span::join(&left_node.span(), &right_node.span()),
                    ),
                })
            } else {
                Ok(())
            }
        };

        let combined = match (left, right) {
            (ConstantEmission::Boolean(left), ConstantEmission::Boolean(right)) => {
                match operation {
                    SyntaxKind::AndExpression => ConstantEmission::Boolean(left && right),
                    SyntaxKind::OrExpression => ConstantEmission::Boolean(left || right),
                    SyntaxKind::GreaterThanExpression => ConstantEmission::Boolean(left || right),
                    SyntaxKind::GreaterThanOrEqualExpression => {
                        ConstantEmission::Boolean(left >= right)
                    }
                    SyntaxKind::LessThanExpression => ConstantEmission::Boolean(left || right),
                    SyntaxKind::LessThanOrEqualExpression => {
                        ConstantEmission::Boolean(left <= right)
                    }
                    SyntaxKind::EqualExpression => ConstantEmission::Boolean(left == right),
                    SyntaxKind::NotEqualExpression => ConstantEmission::Boolean(left != right),
                    _ => {
                        return Err(CompileError::Internal(
                            InternalCompileError::InvalidSyntaxNode(operation),
                        ));
                    }
                }
            }
            (ConstantEmission::Byte(left), ConstantEmission::Byte(right)) => match operation {
                SyntaxKind::AdditionExpression => {
                    ConstantEmission::Byte(left.saturating_add(right))
                }
                SyntaxKind::SubtractionExpression => {
                    ConstantEmission::Byte(left.saturating_sub(right))
                }
                SyntaxKind::MultiplicationExpression => {
                    ConstantEmission::Byte(left.saturating_mul(right))
                }
                SyntaxKind::DivisionExpression => {
                    check_for_division_by_zero()?;

                    ConstantEmission::Byte(left.saturating_div(right))
                }
                SyntaxKind::ModuloExpression => {
                    check_for_division_by_zero()?;

                    ConstantEmission::Byte(left % right)
                }
                SyntaxKind::ExponentExpression => {
                    ConstantEmission::Byte(left.saturating_pow(right as u32))
                }
                SyntaxKind::GreaterThanExpression => ConstantEmission::Boolean(left > right),
                SyntaxKind::GreaterThanOrEqualExpression => {
                    ConstantEmission::Boolean(left >= right)
                }
                SyntaxKind::LessThanExpression => ConstantEmission::Boolean(left < right),
                SyntaxKind::LessThanOrEqualExpression => ConstantEmission::Boolean(left <= right),
                SyntaxKind::EqualExpression => ConstantEmission::Boolean(left == right),
                SyntaxKind::NotEqualExpression => ConstantEmission::Boolean(left != right),
                _ => {
                    return Err(CompileError::Internal(
                        InternalCompileError::InvalidSyntaxNode(operation),
                    ));
                }
            },
            (ConstantEmission::Float(left), ConstantEmission::Float(right)) => match operation {
                SyntaxKind::AdditionExpression => ConstantEmission::Float(left + right),
                SyntaxKind::SubtractionExpression => ConstantEmission::Float(left - right),
                SyntaxKind::MultiplicationExpression => ConstantEmission::Float(left * right),
                SyntaxKind::DivisionExpression => {
                    check_for_division_by_zero()?;

                    ConstantEmission::Float(left / right)
                }
                SyntaxKind::ModuloExpression => {
                    check_for_division_by_zero()?;

                    ConstantEmission::Float(left % right)
                }
                SyntaxKind::ExponentExpression => ConstantEmission::Float(left.powf(right)),
                SyntaxKind::GreaterThanExpression => ConstantEmission::Boolean(left > right),
                SyntaxKind::GreaterThanOrEqualExpression => {
                    ConstantEmission::Boolean(left >= right)
                }
                SyntaxKind::LessThanExpression => ConstantEmission::Boolean(left < right),
                SyntaxKind::LessThanOrEqualExpression => ConstantEmission::Boolean(left <= right),
                SyntaxKind::EqualExpression => ConstantEmission::Boolean(left == right),
                SyntaxKind::NotEqualExpression => ConstantEmission::Boolean(left != right),
                _ => {
                    return Err(CompileError::Internal(
                        InternalCompileError::InvalidSyntaxNode(operation),
                    ));
                }
            },
            (ConstantEmission::Integer(left), ConstantEmission::Integer(right)) => {
                match operation {
                    SyntaxKind::AdditionExpression => {
                        ConstantEmission::Integer(left.saturating_add(right))
                    }
                    SyntaxKind::SubtractionExpression => {
                        ConstantEmission::Integer(left.saturating_sub(right))
                    }
                    SyntaxKind::MultiplicationExpression => {
                        ConstantEmission::Integer(left.saturating_mul(right))
                    }
                    SyntaxKind::DivisionExpression => {
                        check_for_division_by_zero()?;

                        ConstantEmission::Integer(left.saturating_div(right))
                    }
                    SyntaxKind::ModuloExpression => {
                        check_for_division_by_zero()?;

                        ConstantEmission::Integer(left % right)
                    }
                    SyntaxKind::ExponentExpression => {
                        ConstantEmission::Integer(left.saturating_pow(right as u32))
                    }
                    SyntaxKind::GreaterThanExpression => ConstantEmission::Boolean(left > right),
                    SyntaxKind::GreaterThanOrEqualExpression => {
                        ConstantEmission::Boolean(left >= right)
                    }
                    SyntaxKind::LessThanExpression => ConstantEmission::Boolean(left < right),
                    SyntaxKind::LessThanOrEqualExpression => {
                        ConstantEmission::Boolean(left <= right)
                    }
                    SyntaxKind::EqualExpression => ConstantEmission::Boolean(left == right),
                    SyntaxKind::NotEqualExpression => ConstantEmission::Boolean(left != right),
                    _ => {
                        return Err(CompileError::Internal(
                            InternalCompileError::InvalidSyntaxNode(operation),
                        ));
                    }
                }
            }
            (ConstantEmission::Character(left), ConstantEmission::Character(right)) => {
                match operation {
                    SyntaxKind::AdditionExpression => {
                        let mut string = String::with_capacity(2);

                        string.push(left);
                        string.push(right);

                        let combined = self.constants.push_str_to_string_pool(&string);

                        ConstantEmission::String {
                            pool_start: combined.0,
                            pool_end: combined.1,
                        }
                    }
                    SyntaxKind::GreaterThanExpression => ConstantEmission::Boolean(left > right),
                    SyntaxKind::GreaterThanOrEqualExpression => {
                        ConstantEmission::Boolean(left >= right)
                    }
                    SyntaxKind::LessThanExpression => ConstantEmission::Boolean(left < right),
                    SyntaxKind::LessThanOrEqualExpression => {
                        ConstantEmission::Boolean(left <= right)
                    }
                    SyntaxKind::EqualExpression => ConstantEmission::Boolean(left == right),
                    SyntaxKind::NotEqualExpression => ConstantEmission::Boolean(left != right),
                    _ => {
                        return Err(CompileError::Internal(
                            InternalCompileError::InvalidSyntaxNode(operation),
                        ));
                    }
                }
            }
            (
                ConstantEmission::String {
                    pool_start: left_pool_start,
                    pool_end: left_pool_end,
                },
                ConstantEmission::String {
                    pool_start: right_pool_start,
                    pool_end: right_pool_end,
                },
            ) => {
                let left = self
                    .constants
                    .get_string_pool_range(left_pool_start as usize..left_pool_end as usize);
                let right = self
                    .constants
                    .get_string_pool_range(right_pool_start as usize..right_pool_end as usize);

                match operation {
                    SyntaxKind::AdditionExpression => {
                        if left_pool_end == right_pool_start {
                            return Ok(ConstantEmission::String {
                                pool_start: left_pool_start,
                                pool_end: right_pool_end,
                            });
                        }

                        let mut concetenated = String::with_capacity(left.len() + right.len());

                        concetenated.push_str(left);
                        concetenated.push_str(right);

                        let (pool_start, pool_end) =
                            self.constants.push_str_to_string_pool(&concetenated);

                        ConstantEmission::String {
                            pool_start,
                            pool_end,
                        }
                    }
                    SyntaxKind::GreaterThanExpression => ConstantEmission::Boolean(left > right),
                    SyntaxKind::GreaterThanOrEqualExpression => {
                        ConstantEmission::Boolean(left >= right)
                    }
                    SyntaxKind::LessThanExpression => ConstantEmission::Boolean(left < right),
                    SyntaxKind::LessThanOrEqualExpression => {
                        ConstantEmission::Boolean(left <= right)
                    }
                    SyntaxKind::EqualExpression => ConstantEmission::Boolean(left == right),
                    SyntaxKind::NotEqualExpression => ConstantEmission::Boolean(left != right),
                    _ => {
                        return Err(CompileError::Internal(
                            InternalCompileError::InvalidSyntaxNode(operation),
                        ));
                    }
                }
            }
            (
                ConstantEmission::Character(left),
                ConstantEmission::String {
                    pool_start,
                    pool_end,
                },
            ) => {
                let right = self
                    .constants
                    .get_string_pool_range(pool_start as usize..pool_end as usize);
                let mut concatenated = String::with_capacity(left.len_utf8() + right.len());

                concatenated.push(left);
                concatenated.push_str(right);

                let combined = match operation {
                    SyntaxKind::AdditionExpression => {
                        self.constants.push_str_to_string_pool(&concatenated)
                    }
                    _ => {
                        return Err(CompileError::Internal(
                            InternalCompileError::InvalidSyntaxNode(operation),
                        ));
                    }
                };

                ConstantEmission::String {
                    pool_start: combined.0,
                    pool_end: combined.1,
                }
            }
            (
                ConstantEmission::String {
                    pool_start,
                    pool_end,
                },
                ConstantEmission::Character(right),
            ) => {
                let left = self
                    .constants
                    .get_string_pool_range(pool_start as usize..pool_end as usize);
                let mut bytes = String::with_capacity(left.len() + right.len_utf8());

                bytes.push_str(left);
                bytes.push(right);

                let combined = match operation {
                    SyntaxKind::AdditionExpression => {
                        self.constants.push_str_to_string_pool(&bytes)
                    }
                    _ => {
                        return Err(CompileError::Internal(
                            InternalCompileError::InvalidSyntaxNode(operation),
                        ));
                    }
                };

                ConstantEmission::String {
                    pool_start: combined.0,
                    pool_end: combined.1,
                }
            }
            _ => {
                return Err(CompileError::ConstantTypeConflict {
                    expected: left_node.kind(),
                    found: right_node.kind(),
                    position: Position::new(
                        left_node.file_id(),
                        Span::join(&left_node.span(), &right_node.span()),
                    ),
                });
            }
        };

        Ok(combined)
    }

    fn handle_top_emission(
        &mut self,
        emission: Emission,
        node: SyntaxReader,
    ) -> Result<(), CompileError> {
        match emission {
            Emission::Constant(constant) => {
                let destination = self.allocate_temporary_registers(1);
                let address = self.get_constant_address(constant);
                let move_instruction = Instruction::r#move(destination.index(), address);

                self.emit_instruction(move_instruction);
            }
            Emission::Place(place) => {
                let destination = self.allocate_temporary_registers(1);
                let move_instruction = Instruction::r#move(destination.index(), place.address());

                self.emit_instruction(move_instruction);
            }
            Emission::Instructions(InstructionsEmission { instructions, .. }) => {
                for (instruction, mut jump_anchors) in instructions {
                    self.emit_instruction(instruction);

                    jump_anchors.sort();

                    for anchor in jump_anchors {
                        match anchor {
                            JumpAnchor::ForwardFromHere { id } => {
                                let coalesce = if instruction.is_coallescible_with_jump(true) {
                                    true
                                } else {
                                    self.emit_instruction(Instruction::no_op());

                                    false
                                };

                                self.jump_placements.insert(
                                    id,
                                    JumpPlacement {
                                        index: self.instructions.len() - 1,
                                        distance: 0,
                                        forward: true,
                                        coalesce,
                                    },
                                );
                            }
                            JumpAnchor::ForwardToNext { id } => {
                                let placement = self.jump_placements.get_mut(&id).unwrap();
                                let next_index = self.instructions.len();

                                placement.distance = (next_index - placement.index - 1) as u16;
                            }
                            JumpAnchor::LoopStartHere { forward_id } => {
                                let coalesce = if instruction.is_coallescible_with_jump(false) {
                                    true
                                } else {
                                    self.emit_instruction(Instruction::no_op());

                                    false
                                };

                                self.jump_placements.insert(
                                    forward_id,
                                    JumpPlacement {
                                        index: self.instructions.len() - 1,
                                        distance: 0,
                                        forward: true,
                                        coalesce,
                                    },
                                );
                            }
                            JumpAnchor::LoopEndOnNext {
                                forward_id,
                                backward_id,
                            } => {
                                let next_index = self.instructions.len();
                                let forward_index =
                                    self.jump_placements.get(&forward_id).unwrap().index;
                                let coalesce = if instruction.is_coallescible_with_jump(false) {
                                    true
                                } else {
                                    self.emit_instruction(Instruction::no_op());

                                    false
                                };

                                let forward_distance = if coalesce {
                                    (next_index - forward_index - 1) as u16
                                } else {
                                    (next_index - forward_index) as u16
                                };

                                self.jump_placements.get_mut(&forward_id).unwrap().distance =
                                    forward_distance;

                                let backward_distance =
                                    (self.instructions.len() - forward_index - 1) as u16;
                                let backward_placement = JumpPlacement {
                                    index: self.instructions.len() - 1,
                                    distance: backward_distance,
                                    forward: false,
                                    coalesce,
                                };

                                self.jump_placements.insert(backward_id, backward_placement);
                            }
                        }
                    }
                }
            }
            Emission::NativeFunction(_) => {
                return Err(CompileError::ExpectedNativeFunctionCall {
                    position: node.position(),
                });
            }
            Emission::None => {}
        }

        Ok(())
    }

    fn handle_operand_emission(
        &mut self,
        instructions: &mut InstructionsEmission,
        emission: Emission,
        node: &SyntaxReader,
    ) -> Result<Address, CompileError> {
        match emission {
            Emission::Constant(constant) => Ok(self.get_constant_address(constant)),
            Emission::Place(place) => Ok(place.address()),
            Emission::Instructions(operand_instructions) => {
                let destination = operand_instructions
                    .target
                    .ok_or(CompileError::Syntax(SyntaxError::ExpectedExpression {
                        found: node.kind(),
                        position: node.position(),
                    }))?
                    .index();

                instructions.merge(operand_instructions);

                Ok(Address::register(destination))
            }
            Emission::NativeFunction(_) => Err(CompileError::ExpectedNativeFunctionCall {
                position: node.position(),
            }),
            Emission::None => Err(CompileError::Syntax(SyntaxError::ExpectedExpression {
                found: node.kind(),
                position: node.position(),
            })),
        }
    }

    fn handle_condition_emission(
        &mut self,
        instructions: &mut InstructionsEmission,
        emission: Emission,
        node: &SyntaxReader,
    ) -> Result<(), CompileError> {
        match emission {
            Emission::Constant(constant) => {
                let address = self.get_constant_address(constant);
                let test_instruction = Instruction::test(address, true, 1);

                instructions.push(test_instruction);
            }
            Emission::Place(place) => {
                let test_instruction = Instruction::test(place.address(), true, 1);

                instructions.push(test_instruction);
            }
            Emission::Instructions(mut condition_instructions) => {
                let length = condition_instructions.length();

                if condition_instructions.length() >= 3 {
                    let possible_condition_instruction =
                        &mut condition_instructions.instructions[length - 3].0;

                    match possible_condition_instruction.operation() {
                        Operation::LESS | Operation::LESS_EQUAL | Operation::EQUAL => {
                            condition_instructions.instructions.truncate(length - 2);

                            if let Some(target) = condition_instructions.target
                                && target.is_temporary()
                            {
                                self.free_temporary_registers(target.destination_count());
                            }
                        }
                        Operation::TEST => {
                            let first_move_instruction =
                                condition_instructions.instructions[length - 2].0;
                            let new_test_instruction =
                                Instruction::test(first_move_instruction.b_address(), false, 0);

                            condition_instructions.instructions.truncate(length - 2);
                            condition_instructions.push(new_test_instruction);

                            if let Some(target) = condition_instructions.target
                                && target.is_temporary()
                            {
                                self.free_temporary_registers(target.destination_count());
                            }
                        }
                        _ => {}
                    }
                }

                instructions.merge(condition_instructions);
            }
            _ => {
                let found = *self.resolver.get_type_binding(&node.id)?;

                return Err(CompileError::ExpectedBooleanExpression {
                    found,
                    node_kind: node.kind(),
                    position: node.position(),
                });
            }
        }

        Ok(())
    }

    fn handle_branch_emission(
        &mut self,
        instructions_emission: &mut InstructionsEmission,
        emission: Emission,
        destination_register: u16,
        node: SyntaxReader,
    ) -> Result<(), CompileError> {
        match emission {
            Emission::Constant(constant) => {
                let address = self.get_constant_address(constant);
                let move_instruction = Instruction::r#move(destination_register, address);

                instructions_emission.push(move_instruction);
            }
            Emission::Place(place) => {
                let move_instruction = Instruction::r#move(destination_register, place.address());

                instructions_emission.push(move_instruction);
            }
            Emission::Instructions(branch_instructions) => {
                instructions_emission.merge(branch_instructions);
            }
            Emission::NativeFunction(_) => {
                return Err(CompileError::ExpectedNativeFunctionCall {
                    position: node.position(),
                });
            }
            Emission::None => {}
        }

        Ok(())
    }

    fn handle_return_emission(
        &mut self,
        return_instructions: &mut InstructionsEmission,
        emission: Emission,
        node: SyntaxReader,
    ) -> Result<(), CompileError> {
        let type_id = *self.resolver.get_type_binding(&node.id)?;
        let address = match emission {
            Emission::Constant(constant) => self.get_constant_address(constant),
            Emission::Place(Place::Target(TargetRegister::Compound {
                base_index: base_register,
                ..
            })) => Address::register(base_register + 1),
            Emission::Place(place) => place.address(),
            Emission::Instructions(instructions) => {
                if let Some(target) = instructions.target {
                    return_instructions.merge(instructions);

                    match target {
                        TargetRegister::Single { index, .. } => Address::register(index),
                        TargetRegister::Compound {
                            base_index: base_register,
                            ..
                        } => Address::register(base_register + 1),
                    }
                } else {
                    debug_assert_eq!(type_id, TypeId::NONE);

                    return_instructions.merge(instructions);

                    Address::default()
                }
            }
            Emission::NativeFunction(_) => {
                return Err(CompileError::ExpectedNativeFunctionCall {
                    position: node.position(),
                });
            }
            Emission::None => Address::default(),
        };
        let return_instruction = Instruction::r#return(address);

        return_instructions.push(return_instruction);

        Ok(())
    }

    fn handle_implicit_return(
        &mut self,
        node: SyntaxReader,
        target: Option<TargetRegister>,
    ) -> Result<Emission, CompileError> {
        let mut return_emission = InstructionsEmission::new();

        if node.is_expression() {
            let expression_emission = self.visit_expression(node, target)?;

            self.handle_return_emission(&mut return_emission, expression_emission, node)?;
        } else {
            if node.is_item() {
                self.visit_item(node)?;
            } else {
                let instructions = self.visit_statement(node)?;

                return_emission.merge(instructions);
            }

            let return_instruction = Instruction::r#return(Address::default());

            return_emission.push(return_instruction);
        }

        Ok(Emission::Instructions(return_emission))
    }
}

impl SyntaxVisitor for Emitter<'_> {
    type RootOutput = Emission;

    type ItemOutput = ();

    type StatementOutput = InstructionsEmission;

    type ExpressionInput = Option<TargetRegister>;

    type ExpressionOutput = Emission;

    type TypeOutput = ();

    type PathOutput = DeclarationId;

    fn visit_root(&mut self, node: SyntaxReader) -> Result<Self::RootOutput, CompileError> {
        debug!("Visting root");

        let children = node.children();
        let last_child = children.len() - 1;
        let mut final_emission = Emission::None;

        for (index, child) in children.into_iter().enumerate() {
            if index == last_child {
                final_emission = self.handle_implicit_return(child, None)?;
            } else if child.is_item() {
                self.visit_item(child)?;
            } else if child.is_statement() {
                let instructions = self.visit_statement(child)?;

                self.handle_top_emission(Emission::Instructions(instructions), child)?;
            } else {
                let emission = self.visit_expression(child, None)?;

                self.handle_top_emission(emission, child)?;
            }
        }

        Ok(final_emission)
    }

    fn visit_module_item(&mut self, _: SyntaxReader<'_>) -> Result<Self::ItemOutput, CompileError> {
        todo!()
    }

    fn visit_function_item(
        &mut self,
        node: SyntaxReader<'_>,
    ) -> Result<Self::ItemOutput, CompileError> {
        debug!("Visting function item");

        let function_expression = node.expect_right_child()?;

        let function_emission = self.visit_function_expression(function_expression, None)?;

        if let Emission::Place(Place::Prototype {
            prototype_id: prototype_index,
        }) = function_emission
        {
            let declaration_id = *self
                .resolver
                .get_declaration_binding(&function_expression.id)?;

            self.locals.insert(
                declaration_id,
                Place::Prototype {
                    prototype_id: prototype_index,
                },
            );
        }

        Ok(())
    }

    fn visit_use_item(&mut self, _: SyntaxReader<'_>) -> Result<Self::ItemOutput, CompileError> {
        todo!()
    }

    fn visit_struct_item(&mut self, _: SyntaxReader) -> Result<Self::ItemOutput, CompileError> {
        Ok(())
    }

    fn visit_expression_statement(
        &mut self,
        node: SyntaxReader<'_>,
    ) -> Result<Self::StatementOutput, CompileError> {
        debug!("Visting expression statement");

        let expression = node.expect_left_child()?;

        let expression_emission = self.visit_expression(expression, None)?;

        if let Emission::Instructions(mut instructions) = expression_emission {
            instructions.set_target(None);

            Ok(instructions)
        } else {
            Ok(InstructionsEmission::new())
        }
    }

    fn visit_let_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, CompileError> {
        debug!("Visting let statement");

        let mut children = node.children();
        let path = children.expect_next()?;
        let expression = children.expect_next()?;

        let type_id = *self.resolver.get_type_binding(&expression.id)?;

        let mut let_statement_instructions = InstructionsEmission::new();

        let register_size = self.resolver.get_register_size(type_id, &expression)?;
        let target = self.allocate_local_registers(register_size);
        let expression_emission = self.visit_expression(expression, Some(target))?;

        match expression_emission {
            Emission::Constant(constant) => {
                let address = self.get_constant_address(constant);
                let move_instruction = Instruction::r#move(target.index(), address);

                let_statement_instructions.push(move_instruction);
            }
            Emission::Place(place) => {
                let move_instruction = Instruction::r#move(target.index(), place.address());

                let_statement_instructions.push(move_instruction);
            }
            Emission::Instructions(expression_instructions) => {
                let_statement_instructions.merge(expression_instructions);
            }
            Emission::NativeFunction(_) => {
                return Err(CompileError::ExpectedNativeFunctionCall {
                    position: node.position(),
                });
            }
            Emission::None => {
                return Err(CompileError::ExpectedValue {
                    node_kind: expression.kind(),
                    position: expression.position(),
                });
            }
        };

        let declaration_id = *self.resolver.get_declaration_binding(&path.id)?;

        if type_id == TypeId::STRING {
            self.add_drop(target.index());
        }

        self.locals.insert(declaration_id, Place::Target(target));
        let_statement_instructions.set_target(None);

        Ok(let_statement_instructions)
    }

    fn visit_binary_assignment_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, CompileError> {
        debug!("Visting binary assignment statement");

        let emission = self.visit_math_binary_expression(node, None)?;
        let instructions = if let Emission::Instructions(mut instructions) = emission {
            instructions.set_target(None);

            instructions
        } else {
            InstructionsEmission::new()
        };

        Ok(instructions)
    }

    fn visit_reassignment_statement(
        &mut self,
        node: SyntaxReader<'_>,
    ) -> Result<Self::StatementOutput, CompileError> {
        debug!("Visting reassignment statement");

        let (path, expression_statement) = node.expect_binary_children()?;
        let expression = expression_statement.expect_left_child()?;

        let declaration_id = self.resolver.get_declaration_binding(&path.id)?;
        let target = self
            .locals
            .get(declaration_id)
            .ok_or_else(|| CompileError::OutOfScopeId {
                declaration_id: *declaration_id,
                usage_position: path.position(),
            })?
            .expect_target(&path)?;

        let mut reassignment_instructions = InstructionsEmission::new();
        let expression_emission = self.visit_expression(expression, Some(target))?;

        match expression_emission {
            Emission::Constant(constant) => {
                let address = self.get_constant_address(constant);
                let move_instruction = Instruction::r#move(target.index(), address);

                reassignment_instructions.push(move_instruction);
            }
            Emission::Place(place) => {
                let move_instruction = Instruction::r#move(target.index(), place.address());

                reassignment_instructions.push(move_instruction);
            }
            Emission::Instructions(instructions) => {
                reassignment_instructions.merge(instructions);
                reassignment_instructions.set_target(None);
            }
            Emission::NativeFunction(_) => {
                return Err(CompileError::ExpectedNativeFunctionCall {
                    position: node.position(),
                });
            }
            Emission::None => {
                return Err(CompileError::ExpectedValue {
                    node_kind: expression.kind(),
                    position: expression.position(),
                });
            }
        }

        Ok(reassignment_instructions)
    }

    fn visit_boolean_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting boolean expression");

        Ok(Emission::Constant(ConstantEmission::Boolean(
            node.payload().decode_boolean(),
        )))
    }

    fn visit_byte_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting byte expression");

        Ok(Emission::Constant(ConstantEmission::Byte(
            node.payload().decode_byte(),
        )))
    }

    fn visit_character_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting character expression");

        Ok(Emission::Constant(ConstantEmission::Character(
            node.payload().decode_character(),
        )))
    }

    fn visit_float_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting float expression");

        Ok(Emission::Constant(ConstantEmission::Float(
            node.payload().decode_float(),
        )))
    }

    fn visit_integer_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting integer expression");

        Ok(Emission::Constant(ConstantEmission::Integer(
            node.payload().decode_integer(),
        )))
    }

    fn visit_string_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting string expression");

        let bytes = self
            .source
            .get_file(node.file_id())
            .content_str(node.span().shrink(1));
        let (pool_start, pool_end) = self.constants.push_str_to_string_pool(bytes);

        self.resolver.add_type_binding(node.id, TypeId::STRING);

        Ok(Emission::Constant(ConstantEmission::String {
            pool_start,
            pool_end,
        }))
    }

    fn visit_list_expression(
        &mut self,
        node: SyntaxReader,
        target: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        fn handle_element_emission(
            emitter: &mut Emitter,
            instructions: &mut InstructionsEmission,
            element_emission: Emission,
            element_node: &SyntaxReader,
        ) -> Result<Address, CompileError> {
            match element_emission {
                Emission::Place(place) => Ok(place.address()),
                Emission::Constant(constant) => Ok(emitter.get_constant_address(constant)),
                Emission::Instructions(InstructionsEmission {
                    instructions: element_instructions,
                    target,
                    ..
                }) => {
                    let target = target.ok_or(CompileError::ExpectedValue {
                        node_kind: element_node.kind(),
                        position: element_node.position(),
                    })?;

                    instructions.instructions.extend(element_instructions);

                    Ok(Address::register(target.index()))
                }
                Emission::NativeFunction(_) => Err(CompileError::ExpectedNativeFunctionCall {
                    position: element_node.position(),
                }),
                Emission::None => Err(CompileError::ExpectedValue {
                    node_kind: element_node.kind(),
                    position: element_node.position(),
                }),
            }
        }
        debug!("Visting list expression");

        let elements = node.expect_multiple_children()?;
        let element_count_address =
            self.get_constant_address(ConstantEmission::Integer(elements.len() as i64));

        let target = target.unwrap_or_else(|| self.allocate_temporary_registers(1));
        let mut list_emission = {
            let mut emission = InstructionsEmission::with_capacity(elements.len());

            emission.push(Instruction::no_op()); // Placeholder for NEW_LIST

            emission
        };

        for (index, element) in elements.enumerate() {
            let element_emission = self.visit_expression(element, None)?;
            let element_address =
                handle_element_emission(self, &mut list_emission, element_emission, &element)?;
            let index_address = self.get_constant_address(ConstantEmission::Integer(index as i64));
            let set_list_instruction =
                Instruction::set_list(target.index(), element_address, index_address);

            list_emission.push(set_list_instruction);
        }

        let new_list_instruction = Instruction::new_list(target.index(), element_count_address);

        list_emission.instructions[0] = (new_list_instruction, Vec::new());

        list_emission.set_target(Some(target));

        Ok(Emission::Instructions(list_emission))
    }

    fn visit_index_expression(
        &mut self,
        node: SyntaxReader,
        target: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting index expression");

        let (list_expression, index_expression) = node.expect_binary_children()?;

        let left_emission = self.visit_expression(list_expression, None)?;
        let right_emission = self.visit_expression(index_expression, None)?;

        let mut index_emission = InstructionsEmission::new();

        let list_address =
            self.handle_operand_emission(&mut index_emission, left_emission, &list_expression)?;
        let index_address =
            self.handle_operand_emission(&mut index_emission, right_emission, &index_expression)?;

        let target = target.unwrap_or_else(|| self.allocate_temporary_registers(1));
        let get_list_instruction =
            Instruction::get_list(target.index(), list_address, index_address);

        index_emission.push(get_list_instruction);
        index_emission.set_target(Some(target));

        Ok(Emission::Instructions(index_emission))
    }

    fn visit_path_expression(
        &mut self,
        path_expression: SyntaxReader,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting path expression");

        let path = path_expression.expect_left_child()?;

        let declaration_id = self.visit_path(path)?;

        if let Some(local) = self.locals.get(&declaration_id) {
            return Ok(Emission::Place(*local));
        }

        let declaration = self.resolver.declarations.get_declaration(declaration_id)?;

        if let DeclarationKind::NativeFunction(function) = declaration.kind {
            Ok(Emission::NativeFunction(function))
        } else {
            Err(CompileError::OutOfScopeId {
                declaration_id,
                usage_position: path_expression.position(),
            })
        }
    }

    fn visit_struct_expression(
        &mut self,
        node: SyntaxReader,
        target: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting struct expression");

        fn flatten_leaf_operand_types(r#type: &DustType, out: &mut Vec<SmallType>) {
            match r#type {
                DustType::Struct(struct_type) => {
                    for (_, field_type) in &struct_type.fields {
                        flatten_leaf_operand_types(field_type, out);
                    }
                }
                DustType::None => {}
                other => out.push(other.as_operand_type()),
            }
        }

        let fields = node.expect_right_child()?.expect_multiple_children()?;

        let mut field_leaf_operand_types = Vec::with_capacity(fields.len());
        let mut total_leaf_count: u16 = 0;

        for field in fields {
            let field_expression = field.expect_right_child()?;
            let field_type_id = *self.resolver.get_type_binding(&field_expression.id)?;
            let field_full_type = self.resolver.get_full_type(field_type_id, self.source)?;

            let is_struct_field = matches!(field_full_type, DustType::Struct { .. });

            let mut leaf_types = Vec::new();

            flatten_leaf_operand_types(&field_full_type, &mut leaf_types);

            total_leaf_count += leaf_types.len() as u16;

            field_leaf_operand_types.push((is_struct_field, leaf_types));
        }

        let register_count = total_leaf_count.saturating_add(1).max(1);
        let target = if let Some(target) = target
            && target.destination_count() == register_count
        {
            target
        } else {
            self.allocate_temporary_registers(register_count)
        };

        let mut struct_emission = InstructionsEmission::new();
        let mut next_destination = target.index() + 1;

        for (field, (is_struct_field, leaf_types)) in
            fields.into_iter().zip(field_leaf_operand_types)
        {
            let field_expression = field.expect_right_child()?;
            let field_emission = self.visit_expression(field_expression, None)?;
            let field_address = self.handle_operand_emission(
                &mut struct_emission,
                field_emission,
                &field_expression,
            )?;

            if is_struct_field {
                if field_address.memory != MemoryKind::REGISTER {
                    todo!("Handle non-register struct field address");
                }

                for leaf_index in 0..leaf_types.len() {
                    let source = Address::register(field_address.index + 1 + leaf_index as u16);
                    let field_move_instruction = Instruction::r#move(next_destination, source);
                    struct_emission.push(field_move_instruction);
                    next_destination += 1;
                }
            } else {
                let field_move_instruction = Instruction::r#move(next_destination, field_address);
                struct_emission.push(field_move_instruction);
                next_destination += 1;
            }
        }

        if total_leaf_count > 0 {
            let struct_reference_instruction =
                Instruction::reference(target.index(), target.index() + 1, total_leaf_count);
            struct_emission.push(struct_reference_instruction);
        }

        struct_emission.set_target(Some(target));

        Ok(Emission::Instructions(struct_emission))
    }

    fn visit_block_expression(
        &mut self,
        node: SyntaxReader<'_>,
        target: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting block expression");

        let children = node.children();

        let block_scope_id = *self.resolver.get_scope_binding(&node.id)?;
        let parent_scope_id = self.current_scope_id;
        let parent_scope_next_local_register = self.next_local_register;
        let parent_scope_next_temporary_register = self.next_temporary_register;

        self.enter_child_scope(block_scope_id);

        let child_count = children.len();
        let mut block_emission = InstructionsEmission::new();

        for (index, child) in children.enumerate() {
            let is_last = index == child_count - 1;
            let child_emission = if child.is_item() {
                self.visit_item(child)?;

                continue;
            } else if child.is_statement() {
                let instructions = self.visit_statement(child)?;

                Emission::Instructions(instructions)
            } else {
                let expression_target = if is_last { target } else { None };

                self.visit_expression(child, expression_target)?
            };
            let child_target = child_emission.target();

            if is_last {
                match child_emission {
                    Emission::Constant(constant) => {
                        if block_emission.is_empty() {
                            self.enter_parent_scope(
                                parent_scope_id,
                                parent_scope_next_local_register,
                                parent_scope_next_temporary_register,
                            );
                            block_emission.set_target(child_target);
                            self.handle_drops(&mut block_emission);

                            return Ok(Emission::Constant(constant));
                        }

                        let target = target.unwrap_or_else(|| self.allocate_temporary_registers(1));
                        let address = self.get_constant_address(constant);
                        let move_instruction = Instruction::r#move(target.index(), address);

                        block_emission.push(move_instruction);
                        block_emission.set_target(Some(target));
                    }
                    Emission::Place(final_place) => {
                        if block_emission.is_empty() {
                            self.enter_parent_scope(
                                parent_scope_id,
                                parent_scope_next_local_register,
                                parent_scope_next_temporary_register,
                            );
                            block_emission.set_target(child_target);
                            self.handle_drops(&mut block_emission);

                            return Ok(Emission::Place(final_place));
                        }

                        if let Some(block_target) = target {
                            let move_instruction =
                                Instruction::r#move(block_target.index(), final_place.address());

                            block_emission.push(move_instruction);
                            block_emission.set_target(Some(block_target));
                        } else if final_place.address().memory == MemoryKind::REGISTER {
                            let target = final_place.expect_target(&child)?;

                            block_emission.set_target(Some(target));
                        } else {
                            let target = self.allocate_temporary_registers(1);
                            let move_instruction =
                                Instruction::r#move(target.index(), final_place.address());

                            block_emission.push(move_instruction);
                            block_emission.set_target(Some(target));
                        }
                    }
                    Emission::Instructions(instructions) => {
                        block_emission.merge(instructions);
                    }
                    Emission::NativeFunction(_) => {
                        return Err(CompileError::ExpectedNativeFunctionCall {
                            position: node.position(),
                        });
                    }
                    Emission::None => {}
                }
            } else if let Emission::Instructions(child_instructions) = child_emission {
                block_emission.merge(child_instructions);
            }
        }

        self.enter_parent_scope(
            parent_scope_id,
            parent_scope_next_local_register,
            parent_scope_next_temporary_register,
        );
        self.handle_drops(&mut block_emission);

        Ok(Emission::Instructions(block_emission))
    }

    fn visit_if_expression(
        &mut self,
        node: SyntaxReader<'_>,
        target: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting if expression");

        let mut children = node.expect_multiple_children()?;
        let condition = children.expect_next()?;
        let then_block = children.expect_next()?;
        let else_block = children.next();

        let mut if_emission = InstructionsEmission::new();

        let condition_emission = self.visit_expression(condition, None)?;

        self.handle_condition_emission(&mut if_emission, condition_emission, &condition)?;

        let target = target.unwrap_or_else(|| self.allocate_temporary_registers(1));
        let jump_over_then_id = self.create_jump_id();
        let start_else_anchor_count = self.jump_over_else_anchor_ids.len();

        if_emission.push_drop_anchor(JumpAnchor::ForwardFromHere {
            id: jump_over_then_id,
        });

        let then_emission = self.visit_block_expression(then_block, Some(target))?;

        self.handle_branch_emission(&mut if_emission, then_emission, target.index(), then_block)?;

        if_emission.push_drop_anchor(JumpAnchor::ForwardToNext {
            id: jump_over_then_id,
        });

        if let Some(else_expression) = else_block {
            let else_emission = self.visit_else_expression(else_expression, Some(target))?;
            let jump_over_else_id = self.create_jump_id();

            self.jump_over_else_anchor_ids.push(jump_over_else_id);

            if_emission.push_drop_anchor(JumpAnchor::ForwardFromHere {
                id: jump_over_else_id,
            });

            self.handle_branch_emission(
                &mut if_emission,
                else_emission,
                target.index(),
                else_expression,
            )?;

            if_emission.push_drop_anchor(JumpAnchor::ForwardToNext {
                id: jump_over_else_id,
            });
        }

        let end_else_anchor_count = self.jump_over_else_anchor_ids.len();

        for index in start_else_anchor_count..end_else_anchor_count {
            let jump_id = self.jump_over_else_anchor_ids[index];

            if_emission.push_drop_anchor(JumpAnchor::ForwardToNext { id: jump_id });
        }

        if_emission.set_target(Some(target));

        Ok(Emission::Instructions(if_emission))
    }

    fn visit_else_expression(
        &mut self,
        node: SyntaxReader,
        target: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting else expression");

        self.visit_block_expression(node.expect_left_child()?, target)
    }

    fn visit_math_binary_expression(
        &mut self,
        node: SyntaxReader,
        target: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting math binary expression");

        let (left_expression, right_expression) = node.expect_binary_children()?;

        let left_emission = self.visit_expression(left_expression, None)?;
        let right_emission = self.visit_expression(right_expression, None)?;

        if target.is_none()
            && let (Emission::Constant(left_value), Emission::Constant(right_value)) =
                (&left_emission, &right_emission)
        {
            let combined = self.combine_constants(
                node.kind(),
                *left_value,
                &left_expression,
                *right_value,
                &right_expression,
            )?;

            return Ok(Emission::Constant(combined));
        }

        let mut math_emission = InstructionsEmission::new();

        let left_target = left_emission.target();
        let left_address =
            self.handle_operand_emission(&mut math_emission, left_emission, &left_expression)?;
        let right_address =
            self.handle_operand_emission(&mut math_emission, right_emission, &right_expression)?;

        let math_expression_type = *self.resolver.get_type_binding(&node.id)?;
        let needs_drop = math_expression_type == TypeId::STRING;

        let math_instruction = match node.kind() {
            SyntaxKind::AdditionExpression => {
                let target = target.unwrap_or_else(|| self.allocate_temporary_registers(1));

                math_emission.set_target(Some(target));

                if needs_drop {
                    self.pending_drops.last_mut().unwrap().push(target.index());
                }

                Instruction::add(target.index(), left_address, right_address)
            }
            SyntaxKind::AdditionAssignmentStatement => {
                math_emission.set_target(left_target);

                Instruction::add(left_address.index, left_address, right_address)
            }
            SyntaxKind::SubtractionExpression => {
                let target = target.unwrap_or_else(|| self.allocate_temporary_registers(1));

                math_emission.set_target(Some(target));

                Instruction::subtract(target.index(), left_address, right_address)
            }
            SyntaxKind::SubtractionAssignmentStatement => {
                math_emission.set_target(left_target);

                Instruction::subtract(left_address.index, left_address, right_address)
            }
            SyntaxKind::MultiplicationExpression => {
                let target = target.unwrap_or_else(|| self.allocate_temporary_registers(1));

                math_emission.set_target(Some(target));

                Instruction::multiply(target.index(), left_address, right_address)
            }
            SyntaxKind::MultiplicationAssignmentStatement => {
                math_emission.set_target(left_target);

                Instruction::multiply(left_address.index, left_address, right_address)
            }
            SyntaxKind::DivisionExpression => {
                let target = target.unwrap_or_else(|| self.allocate_temporary_registers(1));

                math_emission.set_target(Some(target));

                Instruction::divide(target.index(), left_address, right_address)
            }
            SyntaxKind::DivisionAssignmentStatement => {
                math_emission.set_target(left_target);

                Instruction::divide(left_address.index, left_address, right_address)
            }
            SyntaxKind::ModuloExpression => {
                let target = target.unwrap_or_else(|| self.allocate_temporary_registers(1));

                math_emission.set_target(Some(target));

                Instruction::modulo(target.index(), left_address, right_address)
            }
            SyntaxKind::ModuloAssignmentStatement => {
                math_emission.set_target(left_target);

                Instruction::modulo(left_address.index, left_address, right_address)
            }
            SyntaxKind::ExponentExpression => {
                let target = target.unwrap_or_else(|| self.allocate_temporary_registers(1));

                math_emission.set_target(Some(target));

                Instruction::power(target.index(), left_address, right_address)
            }
            SyntaxKind::ExponentAssignmentStatement => {
                math_emission.set_target(left_target);

                Instruction::power(left_address.index, left_address, right_address)
            }
            _ => unreachable!("Expected binary expression, found {}", node.kind()),
        };

        math_emission.push(math_instruction);

        Ok(Emission::Instructions(math_emission))
    }

    fn visit_comparison_binary_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting comparison binary expression");

        let (left_expression, right_expression) = node.expect_binary_children()?;

        let left_emission = self.visit_expression(left_expression, None)?;
        let right_emission = self.visit_expression(right_expression, None)?;

        if let Emission::Constant(left_constant) = left_emission
            && let Emission::Constant(right_constant) = right_emission
        {
            let combined = self.combine_constants(
                node.kind(),
                left_constant,
                &left_expression,
                right_constant,
                &right_expression,
            )?;

            return Ok(Emission::Constant(combined));
        }

        let mut comparison_emission = InstructionsEmission::new();

        let left_address = self.handle_operand_emission(
            &mut comparison_emission,
            left_emission,
            &left_expression,
        )?;
        let right_address = self.handle_operand_emission(
            &mut comparison_emission,
            right_emission,
            &right_expression,
        )?;

        let target = input.unwrap_or_else(|| self.allocate_temporary_registers(1));
        let comparison_instruction = match node.kind() {
            SyntaxKind::EqualExpression => Instruction::equal(true, left_address, right_address),
            SyntaxKind::NotEqualExpression => {
                Instruction::equal(false, left_address, right_address)
            }
            SyntaxKind::LessThanExpression => Instruction::less(true, left_address, right_address),
            SyntaxKind::GreaterThanExpression => {
                Instruction::less_equal(false, left_address, right_address)
            }
            SyntaxKind::LessThanOrEqualExpression => {
                Instruction::less_equal(true, left_address, right_address)
            }
            SyntaxKind::GreaterThanOrEqualExpression => {
                Instruction::less(false, left_address, right_address)
            }
            _ => unreachable!("Expected comparison expression, found {}", node.kind()),
        };
        let load_false_instruction =
            Instruction::move_with_jump(target.index(), Address::encoded_boolean(false), 1, true);
        let load_true_instruction =
            Instruction::r#move(target.index(), Address::encoded_boolean(true));

        comparison_emission.push(comparison_instruction);
        comparison_emission.push(load_false_instruction);
        comparison_emission.push(load_true_instruction);
        comparison_emission.set_target(Some(target));

        Ok(Emission::Instructions(comparison_emission))
    }

    fn visit_logical_binary_expression(
        &mut self,
        node: SyntaxReader<'_>,
        target: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting logical binary expression");

        let (left_expression, right_expression) = node.expect_binary_children()?;

        let left_emission = self.visit_expression(left_expression, None)?;
        let right_emission = self.visit_expression(right_expression, None)?;

        if let Emission::Constant(left_constant) = left_emission
            && let Emission::Constant(right_constant) = right_emission
        {
            let combined = self.combine_constants(
                node.kind(),
                left_constant,
                &left_expression,
                right_constant,
                &right_expression,
            )?;

            return Ok(Emission::Constant(combined));
        }

        let mut logical_emission = InstructionsEmission::new();

        let left_address =
            self.handle_operand_emission(&mut logical_emission, left_emission, &left_expression)?;
        let right_address =
            self.handle_operand_emission(&mut logical_emission, right_emission, &right_expression)?;

        let target = target.unwrap_or_else(|| self.allocate_temporary_registers(1));

        let test_instruction = match node.kind() {
            SyntaxKind::AndExpression => Instruction::test(left_address, false, 1),
            SyntaxKind::OrExpression => Instruction::test(left_address, true, 1),
            _ => unreachable!("Expected logical expression, found {}", node.kind()),
        };
        let right_move_instruction =
            Instruction::move_with_jump(target.index(), right_address, 1, true);
        let left_move_instruction = Instruction::r#move(target.index(), left_address);

        logical_emission.push(test_instruction);
        logical_emission.push(right_move_instruction);
        logical_emission.push(left_move_instruction);
        logical_emission.set_target(Some(target));

        Ok(Emission::Instructions(logical_emission))
    }

    fn visit_unary_negation_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting unary negation expression");

        let expression = node.expect_left_child()?;

        let expression_emission = self.visit_expression(expression, None)?;

        if let Emission::Constant(constant) = expression_emission {
            let negated = match constant {
                ConstantEmission::Boolean(boolean) => ConstantEmission::Boolean(!boolean),
                ConstantEmission::Byte(byte) => ConstantEmission::Byte(!byte),
                ConstantEmission::Integer(integer) => ConstantEmission::Integer(-integer),
                ConstantEmission::Float(float) => ConstantEmission::Float(-float),
                _ => unreachable!(
                    "Expected constant suitable for negation, found {:?}",
                    constant
                ),
            };

            return Ok(Emission::Constant(negated));
        }

        let mut negation_emission = InstructionsEmission::new();

        let child_address =
            self.handle_operand_emission(&mut negation_emission, expression_emission, &expression)?;
        let target = input.unwrap_or_else(|| self.allocate_temporary_registers(1));
        let negate_instruction = Instruction::negate(target.index(), child_address);

        negation_emission.push(negate_instruction);
        negation_emission.set_target(Some(target));

        Ok(Emission::Instructions(negation_emission))
    }

    fn visit_while_expression(
        &mut self,
        node: SyntaxReader<'_>,
        _: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting while expression");

        let (condition, body) = node.expect_binary_children()?;

        let mut while_emission = InstructionsEmission::new();
        let condition_emission = self.visit_expression(condition, None)?;

        self.handle_condition_emission(&mut while_emission, condition_emission, &condition)?;

        let jump_forward_id = self.create_jump_id();
        let jump_backward_id = self.create_jump_id();

        while_emission.push_drop_anchor(JumpAnchor::LoopStartHere {
            forward_id: jump_forward_id,
        });

        let body_instructions = self.visit_expression_statement(body)?;

        while_emission.merge(body_instructions);
        while_emission.push_drop_anchor(JumpAnchor::LoopEndOnNext {
            forward_id: jump_forward_id,
            backward_id: jump_backward_id,
        });
        while_emission.set_target(None);

        Ok(Emission::Instructions(while_emission))
    }

    fn visit_function_expression(
        &mut self,
        node: SyntaxReader<'_>,
        _target: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting function expression");

        let declaration_id = *self.resolver.get_declaration_binding(&node.id)?;

        if let Some(prototype_index) = self
            .resolver
            .declarations
            .get_declaration_prototype(&declaration_id)
        {
            return Ok(Emission::Place(Place::Prototype {
                prototype_id: *prototype_index,
            }));
        }

        let (signature, body) = node.expect_binary_children()?;
        let parameters = signature.expect_left_child()?.expect_multiple_children()?;
        let function_scope_id = *self.resolver.get_scope_binding(&body.id)?;
        let prototype_id = self.prototypes.reserve_slot();

        self.resolver
            .declarations
            .set_declaration_prototype(declaration_id, prototype_id);

        let function_emitter = Emitter::new(
            node,
            declaration_id,
            function_scope_id,
            prototype_id,
            Some(parameters),
            (
                self.source,
                self.syntax,
                self.constants,
                self.resolver,
                self.prototypes,
            ),
        )?;
        let prototype = function_emitter.emit()?;

        self.prototypes.set_slot(prototype_id, prototype);

        Ok(Emission::Place(Place::Prototype { prototype_id }))
    }

    fn visit_call_expression(
        &mut self,
        node: SyntaxReader<'_>,
        target: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting call expression");

        let (callee, argument_list) = node.expect_binary_children()?;
        let arguments = argument_list.expect_multiple_children()?;

        let mut call_emission = InstructionsEmission::new();

        let arguments_start = self.call_arguments.len() as u16;
        let mut argument_count = 0u16;

        for argument in arguments {
            let argument_emission = self.visit_expression(argument, None)?;
            let argument_address =
                self.handle_operand_emission(&mut call_emission, argument_emission, &argument)?;

            self.call_arguments.push(argument_address);
            argument_count += 1;
        }

        let callee_emission = self.visit_expression(callee, None)?;
        let callee_address = match callee_emission {
            Emission::Place(place) => place.address(),
            Emission::NativeFunction(native_function) => {
                let destination_register = target.map(|target| target.index()).unwrap_or_default();
                let call_native_instruction =
                    Instruction::call_native(destination_register, native_function, 0);

                call_emission.push(call_native_instruction);

                return Ok(Emission::Instructions(call_emission));
            }
            _ => {
                return Err(CompileError::ExpectedFunction {
                    node_kind: callee.kind(),
                    position: callee.position(),
                });
            }
        };

        let return_type_id = *self.resolver.get_type_binding(&node.id)?;
        let return_operand_type = self.resolver.get_operand_type(return_type_id, &node)?;

        let register_count = self.resolver.get_register_size(return_type_id, &node)?;
        let target = if let Some(target) = target
            && target.destination_count() == register_count
        {
            Some(target)
        } else if return_operand_type != SmallType::NONE {
            Some(self.allocate_temporary_registers(register_count))
        } else {
            None
        };

        let call_instruction = Instruction::call(
            target.map(|target| target.index()),
            callee_address,
            arguments_start,
            argument_count,
        );

        call_emission.push(call_instruction);

        if return_operand_type != SmallType::NONE {
            call_emission.set_target(target);
        }

        Ok(Emission::Instructions(call_emission))
    }

    fn visit_type(&mut self, _: SyntaxReader) -> Result<Self::TypeOutput, CompileError> {
        Ok(())
    }

    fn visit_path(&mut self, path: SyntaxReader) -> Result<Self::PathOutput, CompileError> {
        debug!("Visting path");
        debug_assert_eq!(path.kind(), SyntaxKind::Path);

        self.resolver.get_declaration_binding(&path.id).copied()
    }
}

#[derive(Clone, Debug)]
pub enum Emission {
    Instructions(InstructionsEmission),
    Constant(ConstantEmission),
    NativeFunction(NativeFunction),
    Place(Place),
    None,
}

impl Emission {
    fn target(&self) -> Option<TargetRegister> {
        match self {
            Emission::Place(Place::Target(target)) => Some(*target),
            Emission::Instructions(emission) => emission.target,
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct InstructionsEmission {
    instructions: Vec<(Instruction, Vec<JumpAnchor>)>,
    target: Option<TargetRegister>,
}

impl InstructionsEmission {
    fn new() -> Self {
        Self {
            instructions: Vec::new(),
            target: None,
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        Self {
            instructions: Vec::with_capacity(capacity),
            target: None,
        }
    }

    fn _with_instruction(instruction: Instruction) -> Self {
        Self {
            instructions: vec![(instruction, Vec::new())],
            target: None,
        }
    }

    fn length(&self) -> usize {
        self.instructions.len()
    }

    fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }

    fn push(&mut self, instruction: Instruction) {
        self.instructions.push((instruction, Vec::new()));
    }

    fn set_target(&mut self, target: Option<TargetRegister>) {
        self.target = target;
    }

    fn push_drop_anchor(&mut self, anchor: JumpAnchor) {
        if let Some((_, anchors)) = self.instructions.last_mut() {
            anchors.push(anchor);
        }
    }

    fn merge(&mut self, other: InstructionsEmission) {
        self.instructions.extend(other.instructions);
        self.target = other.target;
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Place {
    Constant { id: ConstantId },
    Prototype { prototype_id: PrototypeId },
    Target(TargetRegister),
}

impl Place {
    fn expect_target(self, node: &SyntaxReader) -> Result<TargetRegister, CompileError> {
        match self {
            Place::Target(target) => Ok(target),
            _ => Err(CompileError::CannotMutate {
                position: node.position(),
            }),
        }
    }

    fn address(&self) -> Address {
        match self {
            Place::Constant { id } => Address::constant(id.0),
            Place::Prototype { prototype_id } => Address::constant(prototype_id.index()),
            Place::Target(target) => target.address(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TargetRegister {
    Single {
        index: u16,
        is_temporary: bool,
    },
    Compound {
        base_index: u16,
        length: u16,
        is_temporary: bool,
    },
}

impl TargetRegister {
    fn address(&self) -> Address {
        match self {
            TargetRegister::Single { index, .. } => Address::register(*index),
            TargetRegister::Compound {
                base_index: base_register,
                ..
            } => Address::register(*base_register),
        }
    }

    fn index(&self) -> u16 {
        match self {
            TargetRegister::Single { index, .. } => *index,
            TargetRegister::Compound {
                base_index: base_register,
                ..
            } => *base_register,
        }
    }

    fn is_temporary(&self) -> bool {
        match self {
            TargetRegister::Single { is_temporary, .. }
            | TargetRegister::Compound { is_temporary, .. } => *is_temporary,
        }
    }

    fn destination_count(&self) -> u16 {
        match self {
            TargetRegister::Single { .. } => 1,
            TargetRegister::Compound { length: count, .. } => *count,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ConstantEmission {
    Boolean(bool),
    Byte(u8),
    Character(char),
    Float(f64),
    Integer(i64),
    String { pool_start: u32, pool_end: u32 },
}

impl ConstantEmission {
    fn operand_type(&self) -> SmallType {
        match self {
            ConstantEmission::Boolean(_) => SmallType::BOOLEAN,
            ConstantEmission::Byte(_) => SmallType::BYTE,
            ConstantEmission::Character(_) => SmallType::CHARACTER,
            ConstantEmission::Float(_) => SmallType::FLOAT,
            ConstantEmission::Integer(_) => SmallType::INTEGER,
            ConstantEmission::String { .. } => SmallType::STRING,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum JumpAnchor {
    ForwardFromHere { id: u16 },
    LoopStartHere { forward_id: u16 },
    ForwardToNext { id: u16 },
    LoopEndOnNext { forward_id: u16, backward_id: u16 },
}

#[derive(Clone, Copy, Debug)]
struct JumpPlacement {
    index: usize,
    distance: u16,
    forward: bool,
    coalesce: bool,
}
