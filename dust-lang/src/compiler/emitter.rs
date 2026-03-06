use std::collections::HashMap;

use rustc_hash::FxBuildHasher;
use smallvec::{SmallVec, smallvec};
use tracing::{debug, trace};

use crate::{
    compiler::error::CompileError,
    constant_table::{ConstantId, ConstantTable},
    dust_error::{ErrorKind, InternalError},
    instruction::{
        CallArgument, Drop, Instruction, Jump, MemoryKind, Move, OperandType, Operation, Test,
    },
    native_function::NativeFunction,
    prototype::{Prototype, PrototypeId, PrototypeList},
    register::RegisterClass,
    resolver::{
        Resolver,
        declaration_graph::{DeclarationId, DeclarationKind},
        scope_graph::ScopeId,
        type_graph::{TypeId, TypeNode},
    },
    source::{Position, Source, Span},
    syntax::{Syntax, SyntaxKind, SyntaxReader, SyntaxReaderIterator, SyntaxVisitor},
};

#[derive(Debug)]
pub struct Emitter<'a> {
    function: SyntaxReader<'a>,

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
    call_arguments: Vec<CallArgument>,

    /// Concatenated list of register indices that are referenced by DROP and JUMP instructions.
    drop_lists: Vec<u16>,

    /// Stack of register index lists that need to be dropped when exiting scopes.
    pending_drops: Vec<SmallVec<[u16; 8]>>,

    jump_placements: HashMap<u16, JumpPlacement>,

    jump_over_else_anchor_ids: Vec<u16>,

    current_scope_id: ScopeId,

    next_jump_id: u16,

    integer_32_tracker: RegisterTracker,
    integer_64_tracker: RegisterTracker,
    float_64_tracker: RegisterTracker,
    pointer_tracker: RegisterTracker,
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
    ) -> Result<Self, ErrorKind> {
        let parameter_count = parameters.as_ref().map_or(0, |parameters| parameters.len());
        let mut emitter = Self {
            function,
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
            integer_32_tracker: RegisterTracker::default(),
            integer_64_tracker: RegisterTracker::default(),
            float_64_tracker: RegisterTracker::default(),
            pointer_tracker: RegisterTracker::default(),
        };

        emitter
            .locals
            .insert(declaration_id, Place::Prototype { id: prototype_id });

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
                return Err(ErrorKind::Compile(CompileError::ExpectedFunctionType {
                    found: type_id,
                    position: function.position(),
                }));
            };
            let value_parameter_types = emitter
                .resolver
                .types
                .get_type_members(value_parameters)?
                .iter()
                .copied()
                .collect::<SmallVec<[TypeId; 8]>>();

            for (parameter, expected_type_id) in parameters
                .into_iter()
                .zip(value_parameter_types.into_iter())
            {
                let parameter_id = *emitter.resolver.get_declaration_binding(&parameter.id)?;
                let allocations = emitter.allocate_registers(expected_type_id, false)?;

                emitter
                    .locals
                    .insert(parameter_id, Place::Registers(allocations));
            }
        }

        Ok(emitter)
    }

    pub fn emit(mut self) -> Result<Prototype, ErrorKind> {
        let function_body = self.function.binary_children()?.1.binary_children()?.1;

        match function_body.kind() {
            SyntaxKind::BlockExpression => {
                let children = function_body.children()?;
                let last_index = children.len().saturating_sub(1);

                for (index, child) in children.enumerate() {
                    let child_emission = if index == last_index {
                        self.handle_implicit_return(child, None)?
                    } else if child.is_expression() {
                        self.visit_expression(child, None)?
                    } else if child.is_statement() {
                        let instructions = self.visit_statement(child)?;

                        Emission::Instructions(instructions)
                    } else {
                        self.visit_item(child);

                        continue;
                    };

                    self.handle_top_emission(child_emission, child)?;
                }
            }
            _ => {
                return Err(ErrorKind::Compile(CompileError::Unimplemented {
                    syntax_kind: function_body.kind(),
                    position: function_body.position(),
                }));
            }
        }

        self.finish()
    }

    pub fn finish(mut self) -> Result<Prototype, ErrorKind> {
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
                    Operation::TEST => {
                        let Test {
                            comparator,
                            operand_memory,
                            operand_index,
                            jump_distance,
                            ..
                        } = Test::from(&*instruction);
                        let total_distance = jump_distance + distance;

                        *instruction = Instruction::test(
                            comparator,
                            operand_memory,
                            operand_index,
                            total_distance,
                        );
                    }
                    _ => {
                        unreachable!(
                            "Invalid jump anchor instruction: {:?}",
                            instruction.operation()
                        );
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
                let position = function_declaration
                    .syntax
                    .ok_or(ErrorKind::Internal(InternalError::MissingDeclaration(
                        function_declaration_id,
                    )))?
                    .0;

                return Err(ErrorKind::Compile(CompileError::ExpectedFunctionType {
                    found: type_id,
                    position,
                }));
            }
        };

        Ok(Prototype {
            instructions: self.instructions,
            call_arguments: self.call_arguments,
            drops: self.drop_lists,
            return_type,
            i32_register_count: self.integer_32_tracker.max,
            i64_register_count: self.integer_64_tracker.max,
            f64_register_count: self.float_64_tracker.max,
            pointer_register_count: self.pointer_tracker.max,
        })
    }

    fn emit_instruction(&mut self, instruction: Instruction) {
        trace!("Emitting {} instruction", instruction.operation());

        self.instructions.push(instruction);
    }

    fn create_jump_id(&mut self) -> u16 {
        let anchor_id = self.next_jump_id;

        self.next_jump_id += 1;

        anchor_id
    }

    fn allocate_registers(
        &mut self,
        type_id: TypeId,
        temporary: bool,
    ) -> Result<Allocations, ErrorKind> {
        fn collect_allocations(
            emitter: &mut Emitter,
            type_id: TypeId,
            temporary: bool,
            allocations: &mut SmallVec<[Allocation; 8]>,
        ) -> Result<(), ErrorKind> {
            let type_node = emitter.resolver.types.get_type(type_id)?;

            let (operand_type, tracker, width) = match type_node {
                TypeNode::Unit => return Ok(()),
                TypeNode::Boolean => (OperandType::BOOLEAN, &mut emitter.integer_32_tracker, 1),
                TypeNode::Character => (OperandType::CHARACTER, &mut emitter.integer_32_tracker, 1),
                TypeNode::String => (OperandType::STRING, &mut emitter.pointer_tracker, 1),
                TypeNode::U8 => (OperandType::U_8, &mut emitter.integer_32_tracker, 1),
                TypeNode::I8 => (OperandType::I_8, &mut emitter.integer_32_tracker, 1),
                TypeNode::U16 => (OperandType::U_16, &mut emitter.integer_32_tracker, 1),
                TypeNode::I16 => (OperandType::I_16, &mut emitter.integer_32_tracker, 1),
                TypeNode::U32 => (OperandType::U_32, &mut emitter.integer_32_tracker, 1),
                TypeNode::I32 => (OperandType::I_32, &mut emitter.integer_32_tracker, 1),
                TypeNode::U64 => (OperandType::U_64, &mut emitter.integer_64_tracker, 1),
                TypeNode::I64 => (OperandType::I_64, &mut emitter.integer_64_tracker, 1),
                TypeNode::U128 => (OperandType::U_128, &mut emitter.integer_64_tracker, 2),
                TypeNode::I128 => (OperandType::I_128, &mut emitter.integer_64_tracker, 2),
                TypeNode::F32 => (OperandType::F_32, &mut emitter.float_64_tracker, 1),
                TypeNode::F64 => (OperandType::F_64, &mut emitter.float_64_tracker, 1),
                TypeNode::List { .. } => (OperandType::LIST, &mut emitter.pointer_tracker, 1),
                TypeNode::Function { .. } => {
                    (OperandType::FUNCTION, &mut emitter.integer_32_tracker, 1)
                }
                TypeNode::Struct { declaration_id, .. } => {
                    let declaration = emitter
                        .resolver
                        .declarations
                        .get_declaration(*declaration_id)?;
                    let members = if let DeclarationKind::Type { members, .. } = declaration.kind {
                        emitter
                            .resolver
                            .declarations
                            .get_declaration_members(members)?
                            .into_iter()
                            .copied()
                            .collect::<SmallVec<[DeclarationId; 4]>>()
                    } else {
                        return Err(ErrorKind::Internal(InternalError::ExpectedTypeDeclaration(
                            *declaration_id,
                        )));
                    };

                    for member in members {
                        let member_type_id = *emitter
                            .resolver
                            .declarations
                            .get_declaration_type(&member)?;

                        collect_allocations(emitter, member_type_id, temporary, allocations)?;
                    }

                    return Ok(());
                }
                TypeNode::Enum { declaration_id, .. } => {
                    let type_id = emitter
                        .resolver
                        .declarations
                        .get_declaration_type(declaration_id)?;

                    collect_allocations(emitter, *type_id, temporary, allocations)?;

                    return Ok(());
                }
                TypeNode::Inferred { resolved, .. } => {
                    if let Some(resolved) = resolved {
                        collect_allocations(emitter, *resolved, temporary, allocations)?;

                        return Ok(());
                    } else {
                        return Err(ErrorKind::Compile(CompileError::CannotInferType {
                            type_id,
                            position: None,
                        }));
                    }
                }
            };

            let next_register_index = if temporary {
                let index = tracker.next_temporary;

                tracker.next_temporary += width;
                tracker.max = tracker.max.max(tracker.next_temporary);

                index
            } else {
                let index = tracker.next_local;

                tracker.next_local += width;
                tracker.max = tracker.max.max(tracker.next_local);

                index
            };

            allocations.push(Allocation {
                r#type: operand_type,
                index: next_register_index,
            });

            Ok(())
        }

        let mut allocations = SmallVec::new();

        collect_allocations(self, type_id, temporary, &mut allocations)?;

        Ok(Allocations {
            allocations,
            temporary: true,
        })
    }

    fn free_temporary_registers(&mut self, allocations: &Allocations) {
        let Allocations {
            allocations,
            temporary,
        } = allocations;

        debug_assert!(temporary, "Cannot free local registers");

        for allocation in allocations {
            let tracker = match allocation.r#type {
                OperandType::BOOLEAN
                | OperandType::CHARACTER
                | OperandType::U_8
                | OperandType::I_8
                | OperandType::U_16
                | OperandType::I_16
                | OperandType::U_32
                | OperandType::I_32
                | OperandType::FUNCTION => &mut self.integer_32_tracker,
                OperandType::U_64 | OperandType::I_64 | OperandType::U_128 | OperandType::I_128 => {
                    &mut self.integer_64_tracker
                }
                OperandType::F_32 | OperandType::F_64 => &mut self.float_64_tracker,
                _ => &mut self.pointer_tracker,
            };

            tracker.next_temporary = tracker.next_temporary.min(allocation.index);
        }
    }

    fn enter_child_scope(&mut self, child_scope_id: ScopeId) {
        self.current_scope_id = child_scope_id;

        self.pending_drops.push(SmallVec::new());
    }

    fn enter_parent_scope(
        &mut self,
        parent_scope_id: ScopeId,
        integer_32_tracker: RegisterTracker,
        integer_64_tracker: RegisterTracker,
        float_64_tracker: RegisterTracker,
        pointer_tracker: RegisterTracker,
    ) {
        self.current_scope_id = parent_scope_id;
        self.integer_32_tracker = integer_32_tracker;
        self.integer_64_tracker = integer_64_tracker;
        self.float_64_tracker = float_64_tracker;
        self.pointer_tracker = pointer_tracker;
    }

    fn add_drop(&mut self, register: u16) {
        self.pending_drops.last_mut().unwrap().push(register);
    }

    fn handle_drops(&mut self, instructions: &mut InstructionsEmission) {
        let start = self.drop_lists.len() as u16;
        let mut pending_drops_for_scope = self.pending_drops.pop().unwrap();

        for register in pending_drops_for_scope.drain(..) {
            self.drop_lists.push(register);
        }

        let end = self.drop_lists.len() as u16;

        if start == end {
            return;
        }

        if let Some((last_instruction, _)) = instructions.instructions.last_mut() {
            match last_instruction.operation() {
                Operation::DROP => {
                    let Drop {
                        drop_list_start,
                        drop_list_end,
                    } = Drop::from(&*last_instruction);

                    if drop_list_end == start {
                        *last_instruction = Instruction::drop(drop_list_start, end);
                    }
                }
                Operation::JUMP => {
                    let Jump {
                        offset,
                        is_positive,
                        drop_list_start: _,
                        drop_list_end,
                    } = Jump::from(&*last_instruction);

                    if drop_list_end == 0 {
                        *last_instruction =
                            Instruction::jump_with_drops(offset, is_positive, start, end);
                    }
                }
                _ => {
                    let drop_instruction = Instruction::drop(start, end);

                    instructions.push(drop_instruction);
                }
            }
        }
    }

    fn get_constant_index(&mut self, constant: ConstantEmission) -> u16 {
        match constant {
            ConstantEmission::Boolean(boolean) => boolean as u16,
            ConstantEmission::Character(character) => {
                self.constants.add_character(character).inner()
            }
            ConstantEmission::String {
                pool_start,
                pool_end,
            } => self
                .constants
                .add_pooled_string(pool_start, pool_end)
                .inner(),
            ConstantEmission::U8(integer) => integer as u16,
            ConstantEmission::I8(integer) => integer as u16,
            ConstantEmission::U16(integer) => integer as u16,
            ConstantEmission::I16(integer) => integer as u16,
            ConstantEmission::U32(integer) => self.constants.add_u32(integer).inner(),
            ConstantEmission::I32(integer) => self.constants.add_i32(integer).inner(),
            ConstantEmission::U64(integer) => self.constants.add_u64(integer).inner(),
            ConstantEmission::I64(integer) => self.constants.add_i64(integer).inner(),
            ConstantEmission::U128(integer) => self.constants.add_u128(integer).inner(),
            ConstantEmission::I128(integer) => self.constants.add_i128(integer).inner(),
            ConstantEmission::F32(float) => self.constants.add_f32(float).inner(),
            ConstantEmission::F64(float) => self.constants.add_f64(float).inner(),
        }
    }

    fn combine_constants(
        &mut self,
        operator: &SyntaxReader,
        left_constant: ConstantEmission,
        left_node: &SyntaxReader,
        right_constant: ConstantEmission,
        right_node: &SyntaxReader,
    ) -> Result<ConstantEmission, ErrorKind> {
        let check_for_division_by_zero = || {
            if matches!(
                right_constant,
                ConstantEmission::U8(0)
                    | ConstantEmission::I8(0)
                    | ConstantEmission::U16(0)
                    | ConstantEmission::I16(0)
                    | ConstantEmission::U32(0)
                    | ConstantEmission::I32(0)
                    | ConstantEmission::U64(0)
                    | ConstantEmission::I64(0)
                    | ConstantEmission::U128(0)
                    | ConstantEmission::I128(0)
                    | ConstantEmission::F32(0.0)
                    | ConstantEmission::F64(0.0)
            ) {
                Err(ErrorKind::Compile(CompileError::DivisionByZero {
                    position: Position::new(
                        left_node.file_id(),
                        Span::join(&left_node.span(), &right_node.span()),
                    ),
                }))
            } else {
                Ok(())
            }
        };
        let create_error = || {
            let left_type = left_constant.operand_type();
            let right_type = right_constant.operand_type();

            ErrorKind::Compile(CompileError::CannotApplyBinaryOperator {
                operator: operator.kind(),
                operand_position: operator.position(),
                left_type,
                left_position: left_node.position(),
                right_type,
                right_position: right_node.position(),
            })
        };

        match operator.kind() {
            SyntaxKind::AdditionExpression => left_constant
                .add(right_constant, self)?
                .ok_or_else(create_error),
            SyntaxKind::SubtractionExpression => left_constant
                .subtract(right_constant)
                .ok_or_else(create_error),
            SyntaxKind::MultiplicationExpression => left_constant
                .multiply(right_constant)
                .ok_or_else(create_error),
            SyntaxKind::DivisionExpression => {
                check_for_division_by_zero()?;

                left_constant
                    .divide(right_constant)
                    .ok_or_else(create_error)
            }
            SyntaxKind::ModuloExpression => {
                check_for_division_by_zero()?;

                left_constant
                    .modulo(right_constant)
                    .ok_or_else(create_error)
            }
            SyntaxKind::EqualExpression => left_constant
                .equal(right_constant, self)
                .ok_or_else(create_error),
            SyntaxKind::NotEqualExpression => left_constant
                .not_equal(right_constant, self)
                .ok_or_else(create_error),
            SyntaxKind::LessThanExpression => left_constant
                .less(right_constant, self)
                .ok_or_else(create_error),
            SyntaxKind::GreaterThanExpression => left_constant
                .greater(right_constant, self)
                .ok_or_else(create_error),
            SyntaxKind::LessThanOrEqualExpression => left_constant
                .less_equal(right_constant, self)
                .ok_or_else(create_error),
            SyntaxKind::GreaterThanOrEqualExpression => left_constant
                .greater_equal(right_constant, self)
                .ok_or_else(create_error),
            _ => unreachable!("Invalid binary operator: {:?}", operator.kind()),
        }
    }

    fn handle_top_emission(
        &mut self,
        emission: Emission,
        node: SyntaxReader,
    ) -> Result<(), ErrorKind> {
        match emission {
            Emission::Constant(constant) => {
                let type_id = *self.resolver.get_type_binding(&node.id)?;
                let registers = self.allocate_registers(type_id, true)?;

                for allocation in &registers.allocations {
                    let move_instruction = Instruction::r#move(
                        allocation.index,
                        allocation.r#type,
                        MemoryKind::CONSTANT,
                        self.get_constant_index(constant),
                    );

                    self.emit_instruction(move_instruction);
                }
            }
            Emission::Place(place) => {
                let type_id = *self.resolver.get_type_binding(&node.id)?;
                let registers = self.allocate_registers(type_id, true)?;
                let memory_kind = match place {
                    Place::Registers(_) => MemoryKind::REGISTER,
                    Place::Prototype { .. } | Place::Constant { .. } => MemoryKind::CONSTANT,
                };

                for allocation in &registers.allocations {
                    let move_instruction = Instruction::r#move(
                        allocation.index,
                        allocation.r#type,
                        memory_kind,
                        allocation.index,
                    );

                    self.emit_instruction(move_instruction);
                }
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
                return Err(ErrorKind::Compile(
                    CompileError::ExpectedNativeFunctionCall {
                        position: node.position(),
                    },
                ));
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
    ) -> Result<(MemoryKind, u16), ErrorKind> {
        match emission {
            Emission::Constant(constant) => {
                Ok((MemoryKind::CONSTANT, self.get_constant_index(constant)))
            }
            Emission::Place(place) => Ok(place.memory_and_index()),
            Emission::Instructions(operand_instructions) => {
                if let Some(registers) = &operand_instructions.registers
                    && registers.temporary
                {
                    self.free_temporary_registers(registers);
                }

                instructions.merge(operand_instructions);

                if let Some(registers) = &instructions.registers {
                    Ok((MemoryKind::REGISTER, registers.start_index()))
                } else {
                    Err(ErrorKind::Compile(CompileError::ExpectedValue {
                        node_kind: node.kind(),
                        position: node.position(),
                    }))
                }
            }
            Emission::NativeFunction(_) => Err(ErrorKind::Compile(
                CompileError::ExpectedNativeFunctionCall {
                    position: node.position(),
                },
            )),
            Emission::None => Err(ErrorKind::Compile(CompileError::ExpectedValue {
                node_kind: node.kind(),
                position: node.position(),
            })),
        }
    }

    fn handle_condition_emission(
        &mut self,
        instructions: &mut InstructionsEmission,
        emission: Emission,
        node: &SyntaxReader,
    ) -> Result<(), ErrorKind> {
        match emission {
            Emission::Constant(constant) => {
                let operand_index = self.get_constant_index(constant);
                let test_instruction =
                    Instruction::test(true, MemoryKind::CONSTANT, operand_index, 1);

                instructions.push(test_instruction);
            }
            Emission::Place(place) => {
                let (memory_kind, operand_index) = place.memory_and_index();
                let test_instruction = Instruction::test(true, memory_kind, operand_index, 1);

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

                            if let Some(target) = &condition_instructions.registers
                                && target.temporary
                            {
                                self.free_temporary_registers(target);
                            }
                        }
                        Operation::TEST => {
                            let first_move_instruction =
                                condition_instructions.instructions[length - 2].0;

                            let Move {
                                operand_memory,
                                operand_index,
                                ..
                            } = Move::from(&first_move_instruction);

                            let new_test_instruction =
                                Instruction::test(true, operand_memory, operand_index, 1);

                            condition_instructions.instructions.truncate(length - 2);
                            condition_instructions.push(new_test_instruction);

                            if let Some(target) = &condition_instructions.registers
                                && target.temporary
                            {
                                self.free_temporary_registers(target);
                            }
                        }
                        _ => {}
                    }
                }

                instructions.merge(condition_instructions);
            }
            _ => {
                let found = *self.resolver.get_type_binding(&node.id)?;

                return Err(ErrorKind::Compile(
                    CompileError::ExpectedBooleanExpression {
                        found,
                        node_kind: node.kind(),
                        position: node.position(),
                    },
                ));
            }
        }

        Ok(())
    }

    fn handle_branch_emission(
        &mut self,
        instructions_emission: &mut InstructionsEmission,
        emission: Emission,
        allocations: &Allocations,
        node: SyntaxReader,
    ) -> Result<(), ErrorKind> {
        match emission {
            Emission::Constant(constant) => {
                assert_eq!(allocations.allocations.len(), 1);

                for allocation in &allocations.allocations {
                    let move_instruction = Instruction::r#move(
                        allocation.index,
                        allocation.r#type,
                        MemoryKind::CONSTANT,
                        self.get_constant_index(constant),
                    );

                    instructions_emission.push(move_instruction);
                }
            }
            Emission::Place(place) => {
                assert_eq!(allocations.allocations.len(), 1);

                for allocation in &allocations.allocations {
                    let (memory_kind, operand_index) = place.memory_and_index();
                    let move_instruction = Instruction::r#move(
                        allocation.index,
                        allocation.r#type,
                        memory_kind,
                        operand_index,
                    );

                    instructions_emission.push(move_instruction);
                }
            }
            Emission::Instructions(branch_instructions) => {
                instructions_emission.merge(branch_instructions);
                instructions_emission.set_target(Some(allocations.clone()));
            }
            Emission::NativeFunction(_) => {
                return Err(ErrorKind::Compile(
                    CompileError::ExpectedNativeFunctionCall {
                        position: node.position(),
                    },
                ));
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
    ) -> Result<(), ErrorKind> {
        let type_id = *self.resolver.get_type_binding(&node.id)?;
        let return_instruction = match emission {
            Emission::Constant(constant) => Instruction::r#return(
                true,
                constant.operand_type(),
                MemoryKind::CONSTANT,
                self.get_constant_index(constant),
                1,
            ),
            Emission::Place(place) => {
                let operand_type = *self
                    .resolver
                    .get_operand_types(type_id)?
                    .first()
                    .ok_or_else(|| CompileError::ExpectedValue {
                        node_kind: node.kind(),
                        position: node.position(),
                    })?;
                let (memory_kind, operand_index) = place.memory_and_index();

                Instruction::r#return(true, operand_type, memory_kind, operand_index, 1)
            }
            Emission::Instructions(instructions) => {
                return_instructions.merge(instructions);

                if let Some(allocations) = &return_instructions.registers {
                    let operand_type = *self
                        .resolver
                        .get_operand_types(type_id)?
                        .first()
                        .ok_or_else(|| CompileError::ExpectedValue {
                            node_kind: node.kind(),
                            position: node.position(),
                        })?;

                    Instruction::r#return(
                        true,
                        operand_type,
                        MemoryKind::REGISTER,
                        allocations.start_index(),
                        allocations.allocations.len() as u16,
                    )
                } else {
                    Instruction::r#return(false, OperandType::default(), MemoryKind::REGISTER, 0, 0)
                }
            }
            Emission::None => {
                Instruction::r#return(false, OperandType::default(), MemoryKind::CONSTANT, 0, 0)
            }
            Emission::NativeFunction(_) => {
                return Err(ErrorKind::Compile(
                    CompileError::ExpectedNativeFunctionCall {
                        position: node.position(),
                    },
                ));
            }
        };

        return_instructions.push(return_instruction);

        Ok(())
    }

    fn handle_implicit_return(
        &mut self,
        node: SyntaxReader,
        target: Option<&Allocations>,
    ) -> Result<Emission, ErrorKind> {
        let mut return_emission = InstructionsEmission::new();

        if node.is_expression() {
            let expression_emission = self.visit_expression(node, target)?;

            self.handle_return_emission(&mut return_emission, expression_emission, node)?;
        } else {
            if node.is_item() {
                self.visit_item(node);
            } else {
                self.visit_statement(node);
            }

            let return_instruction =
                Instruction::r#return(false, OperandType::default(), MemoryKind::default(), 0, 0);

            return_emission.push(return_instruction);
        }

        Ok(Emission::Instructions(return_emission))
    }
}

impl SyntaxVisitor for Emitter<'_> {
    type RootOutput = Emission;
    type StatementOutput = InstructionsEmission;
    type ExpressionInput = Allocations;
    type ExpressionOutput = Emission;
    type TypeOutput = ();
    type PathInput = ();
    type PathOutput = DeclarationId;

    fn visit_root(&mut self, node: SyntaxReader) -> Result<Self::RootOutput, ErrorKind> {
        debug!("Visting root");

        let children = node.children()?;
        let last_child = children.len() - 1;
        let mut final_emission = Emission::None;

        for (index, child) in children.enumerate() {
            if index == last_child {
                final_emission = self.handle_implicit_return(child, None)?;
            } else if child.is_item() {
                self.visit_item(child);
            } else if child.is_statement() {
                self.visit_statement(child);
            } else {
                let emission = self.visit_expression(child, None)?;

                self.handle_top_emission(emission, child)?;
            }
        }

        Ok(final_emission)
    }

    fn visit_module_item(&mut self, _: SyntaxReader<'_>) -> Result<(), ErrorKind> {
        todo!()
    }

    fn visit_function_item(&mut self, node: SyntaxReader<'_>) -> Result<(), ErrorKind> {
        debug!("Visting function item");

        let (_, function_expression) = node.binary_children()?;

        let function_emission = self.visit_function_expression(function_expression, None)?;

        if let Emission::Place(Place::Prototype {
            id: prototype_index,
        }) = function_emission
        {
            let declaration_id = *self
                .resolver
                .get_declaration_binding(&function_expression.id)?;

            self.locals.insert(
                declaration_id,
                Place::Prototype {
                    id: prototype_index,
                },
            );
        }

        Ok(())
    }

    fn visit_use_item(&mut self, _: SyntaxReader<'_>) -> Result<(), ErrorKind> {
        todo!()
    }

    fn visit_struct_item(&mut self, _: SyntaxReader) -> Result<(), ErrorKind> {
        Ok(())
    }

    fn visit_enum_item(&mut self, _: SyntaxReader) -> Result<(), ErrorKind> {
        Ok(())
    }

    fn visit_expression_statement(
        &mut self,
        node: SyntaxReader<'_>,
    ) -> Result<Self::StatementOutput, ErrorKind> {
        debug!("Visting expression statement");

        let expression = node.child()?;

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
    ) -> Result<Self::StatementOutput, ErrorKind> {
        debug!("Visting let statement");

        let mut children = node.children()?;
        let path = children.expect_next()?;
        let expression = children.expect_next()?;

        let type_id = *self.resolver.get_type_binding(&expression.id)?;

        let mut let_statement_instructions = InstructionsEmission::new();

        let registers = self.allocate_registers(type_id, false)?;
        let expression_emission = self.visit_expression(expression, Some(&registers))?;

        match expression_emission {
            Emission::Constant(constant) => {
                for allocation in &registers.allocations {
                    let move_instruction = Instruction::r#move(
                        allocation.index,
                        allocation.r#type,
                        MemoryKind::CONSTANT,
                        self.get_constant_index(constant),
                    );

                    let_statement_instructions.push(move_instruction);

                    if allocation.r#type == OperandType::STRING {
                        self.add_drop(allocation.index);
                    }
                }
            }
            Emission::Place(place) => {
                for allocation in &registers.allocations {
                    let (memory_kind, operand_index) = place.memory_and_index();
                    let move_instruction = Instruction::r#move(
                        allocation.index,
                        allocation.r#type,
                        memory_kind,
                        operand_index,
                    );

                    let_statement_instructions.push(move_instruction);

                    if allocation.r#type == OperandType::STRING {
                        self.add_drop(allocation.index);
                    }
                }
            }
            Emission::Instructions(expression_instructions) => {
                let_statement_instructions.merge(expression_instructions);
            }
            Emission::NativeFunction(_) => {
                return Err(ErrorKind::Compile(
                    CompileError::ExpectedNativeFunctionCall {
                        position: node.position(),
                    },
                ));
            }
            Emission::None => {
                return Err(ErrorKind::Compile(CompileError::ExpectedValue {
                    node_kind: expression.kind(),
                    position: expression.position(),
                }));
            }
        };

        let declaration_id = *self.resolver.get_declaration_binding(&path.id)?;

        self.locals
            .insert(declaration_id, Place::Registers(registers));
        let_statement_instructions.set_target(None);

        Ok(let_statement_instructions)
    }

    fn visit_binary_assignment_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, ErrorKind> {
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
    ) -> Result<Self::StatementOutput, ErrorKind> {
        debug!("Visting reassignment statement");

        let (path, expression_statement) = node.binary_children()?;
        let expression = expression_statement.child()?;

        let declaration_id = self.resolver.get_declaration_binding(&path.id)?;
        let local = self.locals.get(declaration_id).ok_or_else(|| {
            ErrorKind::Compile(CompileError::OutOfScopeId {
                declaration_id: *declaration_id,
                usage_position: path.position(),
            })
        })?;

        let registers = local.expect_register(&path)?.clone();

        let mut reassignment_instructions = InstructionsEmission::new();
        let expression_emission = self.visit_expression(expression, Some(&registers))?;

        match expression_emission {
            Emission::Constant(constant) => {
                for allocation in &registers.allocations {
                    let move_instruction = Instruction::r#move(
                        allocation.index,
                        allocation.r#type,
                        MemoryKind::CONSTANT,
                        self.get_constant_index(constant),
                    );

                    reassignment_instructions.push(move_instruction);

                    if allocation.r#type == OperandType::STRING {
                        self.add_drop(allocation.index);
                    }
                }
            }
            Emission::Place(place) => {
                for allocation in &registers.allocations {
                    let (memory_kind, operand_index) = place.memory_and_index();
                    let move_instruction = Instruction::r#move(
                        allocation.index,
                        allocation.r#type,
                        memory_kind,
                        operand_index,
                    );

                    reassignment_instructions.push(move_instruction);

                    if allocation.r#type == OperandType::STRING {
                        self.add_drop(allocation.index);
                    }
                }
            }
            Emission::Instructions(instructions) => {
                reassignment_instructions.merge(instructions);
                reassignment_instructions.set_target(None);
            }
            Emission::NativeFunction(_) => {
                return Err(ErrorKind::Compile(
                    CompileError::ExpectedNativeFunctionCall {
                        position: node.position(),
                    },
                ));
            }
            Emission::None => {
                return Err(ErrorKind::Compile(CompileError::ExpectedValue {
                    node_kind: expression.kind(),
                    position: expression.position(),
                }));
            }
        }

        Ok(reassignment_instructions)
    }

    fn visit_boolean_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<&Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting boolean expression");

        Ok(Emission::Constant(ConstantEmission::Boolean(
            node.payload().decode_boolean(),
        )))
    }

    fn visit_byte_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<&Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting byte expression");

        Ok(Emission::Constant(ConstantEmission::U8(
            node.payload().decode_byte(),
        )))
    }

    fn visit_character_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<&Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting character expression");

        Ok(Emission::Constant(ConstantEmission::Character(
            node.payload().decode_character(),
        )))
    }

    fn visit_float_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<&Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting float expression");

        Ok(Emission::Constant(ConstantEmission::F64(
            node.payload().decode_float(),
        )))
    }

    fn visit_integer_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<&Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting integer expression");

        Ok(Emission::Constant(ConstantEmission::I64(
            node.payload().decode_integer(),
        )))
    }

    fn visit_string_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<&Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting string expression");

        let bytes = self
            .source
            .get_file(node.file_id())?
            .content_str(node.span().shrink(1))?;
        let (pool_start, pool_end) = self.constants.push_str_to_string_pool(bytes)?;

        self.resolver.add_type_binding(node.id, TypeId::STRING);

        Ok(Emission::Constant(ConstantEmission::String {
            pool_start,
            pool_end,
        }))
    }

    fn visit_list_expression(
        &mut self,
        node: SyntaxReader,
        target: Option<&Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        fn handle_element_emission(
            emitter: &mut Emitter,
            destination_list: u16,
            element_type: OperandType,
            element_index: u16,
            instructions: &mut InstructionsEmission,
            element_emission: Emission,
            element_node: &SyntaxReader,
        ) -> Result<(), ErrorKind> {
            let index = emitter.constants.add_u64(element_index as u64);

            match element_emission {
                Emission::Place(Place::Registers(registers)) => {
                    for allocation in &registers.allocations {
                        let set_list_instruction = Instruction::set_list(
                            destination_list,
                            element_type,
                            MemoryKind::REGISTER,
                            allocation.index,
                            MemoryKind::CONSTANT,
                            index.inner(),
                        );

                        instructions.push(set_list_instruction);
                    }

                    Ok(())
                }
                Emission::Place(place) => {
                    let set_list_instruction = Instruction::set_list(
                        destination_list,
                        element_type,
                        MemoryKind::CONSTANT,
                        place.index(),
                        MemoryKind::CONSTANT,
                        index.inner(),
                    );

                    instructions.push(set_list_instruction);

                    Ok(())
                }
                Emission::Constant(constant) => {
                    let operand = emitter.get_constant_index(constant);
                    let set_list_instruction = Instruction::set_list(
                        destination_list,
                        element_type,
                        MemoryKind::CONSTANT,
                        operand,
                        MemoryKind::CONSTANT,
                        index.inner(),
                    );

                    instructions.push(set_list_instruction);

                    Ok(())
                }
                Emission::Instructions(InstructionsEmission {
                    instructions: element_instructions,
                    registers: target,
                    ..
                }) => {
                    let target = target.ok_or(ErrorKind::Compile(CompileError::ExpectedValue {
                        node_kind: element_node.kind(),
                        position: element_node.position(),
                    }))?;

                    instructions.instructions.extend(element_instructions);

                    Ok(Address::register(target.start_index))
                }
                Emission::NativeFunction(_) => Err(ErrorKind::Compile(
                    CompileError::ExpectedNativeFunctionCall {
                        position: element_node.position(),
                    },
                )),
                Emission::None => Err(ErrorKind::Compile(CompileError::ExpectedValue {
                    node_kind: element_node.kind(),
                    position: element_node.position(),
                })),
            }
        }

        debug!("Visting list expression");

        let elements = node.children()?;
        let element_count_address =
            self.get_constant_index(ConstantEmission::U64(elements.len() as u64));

        let target = if let Some(target) = target {
            target.clone()
        } else {
            let type_id = *self.resolver.get_type_binding(&node.id)?;
            let register_classes = self.resolver.get_field_types(type_id, &node)?;

            self.allocate_temporary_registers(register_classes)
        };
        let mut list_emission = {
            let mut emission = InstructionsEmission::with_capacity(elements.len());

            emission.push(Instruction::no_op()); // Placeholder for NEW_LIST

            emission
        };

        for (index, element) in elements.enumerate() {
            let element_emission = self.visit_expression(element, None)?;
            let element_address =
                handle_element_emission(self, &mut list_emission, element_emission, &element)?;
            let index_address = self.get_constant_index(ConstantEmission::U64(index as u64));
            let set_list_instruction =
                Instruction::set_list(target.start_index, element_address, index_address);

            list_emission.push(set_list_instruction);
        }

        let new_list_instruction = Instruction::new_list(target.start_index, element_count_address);

        list_emission.instructions[0] = (new_list_instruction, Vec::new());

        list_emission.set_target(Some(target));

        Ok(Emission::Instructions(list_emission))
    }

    fn visit_index_expression(
        &mut self,
        node: SyntaxReader,
        target: Option<&Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting index expression");

        let (list_expression, index_expression) = node.binary_children()?;

        let left_emission = self.visit_expression(list_expression, None)?;
        let right_emission = self.visit_expression(index_expression, None)?;

        let mut index_emission = InstructionsEmission::new();

        let list_address =
            self.handle_operand_emission(&mut index_emission, left_emission, &list_expression)?;
        let index_address =
            self.handle_operand_emission(&mut index_emission, right_emission, &index_expression)?;

        let target = if let Some(target) = target {
            target.clone()
        } else {
            let type_id = *self.resolver.get_type_binding(&node.id)?;
            let register_classes = self.resolver.get_field_types(type_id, &node)?;

            self.allocate_temporary_registers(register_classes)
        };
        let get_list_instruction =
            Instruction::get_list(target.start_index, list_address, index_address);

        index_emission.push(get_list_instruction);
        index_emission.set_target(Some(target));

        Ok(Emission::Instructions(index_emission))
    }

    fn visit_path_expression(
        &mut self,
        path_expression: SyntaxReader,
        _: Option<&Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting path expression");

        let path = path_expression.child()?;

        let declaration_id = self.visit_path(path, ())?;

        if let Some(local) = self.locals.get(&declaration_id) {
            return Ok(Emission::Place(local.clone()));
        }

        let declaration = self.resolver.declarations.get_declaration(declaration_id)?;

        if let DeclarationKind::NativeFunction(function) = declaration.kind {
            Ok(Emission::NativeFunction(function))
        } else {
            Err(ErrorKind::Compile(CompileError::OutOfScopeId {
                declaration_id,
                usage_position: path_expression.position(),
            }))
        }
    }

    fn visit_struct_expression(
        &mut self,
        node: SyntaxReader,
        target: Option<&Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting struct expression");

        let (struct_name, struct_fields) = node.binary_children()?;

        let target = if let Some(target) = target {
            target.clone()
        } else {
            let type_id = *self.resolver.get_type_binding(&struct_name.id)?;
            let register_classes = self.resolver.get_field_types(type_id, &node)?;

            self.allocate_temporary_registers(register_classes)
        };

        let mut struct_emission = InstructionsEmission::new();
        let mut next_destination = target.start_index + 1;

        for [_, field_expression] in struct_fields.children()?.array_chunks::<2>() {
            let field_emission = self.visit_expression(field_expression, None)?;
            let operand_type = self
                .resolver
                .get_type_binding(&field_expression.id)
                .and_then(|type_id| self.resolver.get_small_type(*type_id, &field_expression))?;
            let field_address = self.handle_operand_emission(
                &mut struct_emission,
                field_emission,
                &field_expression,
            )?;

            let field_move_instruction =
                Instruction::r#move(next_destination, operand_type, field_address, 0);
            next_destination += 1;

            struct_emission.push(field_move_instruction);
        }

        struct_emission.set_target(Some(target));

        Ok(Emission::Instructions(struct_emission))
    }

    fn visit_block_expression(
        &mut self,
        node: SyntaxReader<'_>,
        target: Option<&Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting block expression");

        let children = node.children()?;

        let block_scope_id = *self.resolver.get_scope_binding(&node.id)?;
        let parent_scope_id = self.current_scope_id;
        let parent_scope_integer_32_tracker = self.integer_32_tracker;
        let parent_scope_integer_64_tracker = self.integer_64_tracker;
        let parent_scope_float_64_tracker = self.float_64_tracker;
        let parent_scope_pointer_tracker = self.pointer_tracker;

        self.enter_child_scope(block_scope_id);

        let child_count = children.len();
        let mut block_emission = InstructionsEmission::new();

        for (index, child) in children.enumerate() {
            let is_last = index == child_count - 1;

            if child.is_item() {
                self.visit_item(child);

                continue;
            } else if child.is_statement() {
                self.visit_statement(child);

                continue;
            } else if !is_last {
                let expression_emission = self.visit_expression(child, None)?;

                if let Emission::Instructions(expression_instructions) = expression_emission {
                    block_emission.merge(expression_instructions);
                }
            } else {
                let last_emission = self.visit_expression(child, target)?;
                let target = last_emission.target().cloned();

                match last_emission {
                    Emission::Constant(constant) => {
                        if block_emission.is_empty() {
                            block_emission.set_target(target);
                            self.handle_drops(&mut block_emission);
                            self.enter_parent_scope(
                                parent_scope_id,
                                parent_scope_integer_32_tracker,
                                parent_scope_integer_64_tracker,
                                parent_scope_float_64_tracker,
                                parent_scope_pointer_tracker,
                            );

                            return Ok(Emission::Constant(constant));
                        }

                        let target = if let Some(target) = target {
                            target
                        } else {
                            let type_id = *self.resolver.get_type_binding(&child.id)?;
                            let register_classes =
                                self.resolver.get_field_types(type_id, &child)?;

                            self.allocate_temporary_registers(register_classes)
                        };
                        let operand_type = constant.operand_type();
                        let operand = self.get_constant_index(constant);
                        let move_instruction =
                            Instruction::r#move(target.start_index, operand_type, operand, 0);

                        block_emission.push(move_instruction);
                        block_emission.set_target(Some(target));
                    }
                    Emission::Place(final_place) => {
                        if block_emission.is_empty() {
                            block_emission.set_target(target);
                            self.handle_drops(&mut block_emission);
                            self.enter_parent_scope(
                                parent_scope_id,
                                parent_scope_integer_32_tracker,
                                parent_scope_integer_64_tracker,
                                parent_scope_float_64_tracker,
                                parent_scope_pointer_tracker,
                            );

                            return Ok(Emission::Place(final_place));
                        }

                        let operand_type = self
                            .resolver
                            .get_type_binding(&child.id)
                            .and_then(|type_id| self.resolver.get_small_type(*type_id, &child))?;
                        let (operand, size) = final_place.address_and_register_size();

                        if let Some(block_target) = target {
                            let move_instruction = Instruction::r#move(
                                block_target.start_index,
                                operand_type,
                                operand,
                                size,
                            );

                            block_emission.push(move_instruction);
                            block_emission.set_target(Some(block_target.clone()));
                        } else if operand.memory == MemoryKind::REGISTER {
                            let target = final_place.expect_register(&child)?.clone();

                            block_emission.set_target(Some(target));
                        } else {
                            let target = if let Some(target) = target {
                                target
                            } else {
                                let type_id = *self.resolver.get_type_binding(&child.id)?;
                                let register_classes =
                                    self.resolver.get_field_types(type_id, &child)?;

                                self.allocate_temporary_registers(register_classes)
                            };
                            let move_instruction = Instruction::r#move(
                                target.start_index,
                                operand_type,
                                operand,
                                size,
                            );

                            block_emission.push(move_instruction);
                            block_emission.set_target(Some(target));
                        }
                    }
                    Emission::Instructions(instructions) => {
                        block_emission.merge(instructions);
                    }
                    Emission::NativeFunction(_) => {
                        return Err(ErrorKind::Compile(
                            CompileError::ExpectedNativeFunctionCall {
                                position: node.position(),
                            },
                        ));
                    }
                    Emission::None => {}
                }

                break;
            };
        }

        self.enter_parent_scope(
            parent_scope_id,
            parent_scope_integer_32_tracker,
            parent_scope_integer_64_tracker,
            parent_scope_float_64_tracker,
            parent_scope_pointer_tracker,
        );
        self.handle_drops(&mut block_emission);

        Ok(Emission::Instructions(block_emission))
    }

    fn visit_if_expression(
        &mut self,
        node: SyntaxReader<'_>,
        target: Option<&Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting if expression");

        let mut children = node.children()?;
        let condition = children.expect_next()?;
        let then_block = children.expect_next()?;
        let else_block = children.next();

        let mut if_emission = InstructionsEmission::new();

        let condition_emission = self.visit_expression(condition, None)?;

        self.handle_condition_emission(&mut if_emission, condition_emission, &condition)?;

        let target = if let Some(target) = target {
            target.clone()
        } else {
            let type_id = *self.resolver.get_type_binding(&node.id)?;
            let register_classes = self.resolver.get_field_types(type_id, &node)?;

            self.allocate_temporary_registers(register_classes)
        };
        let jump_over_then_id = self.create_jump_id();
        let start_else_anchor_count = self.jump_over_else_anchor_ids.len();

        if_emission.push_drop_anchor(JumpAnchor::ForwardFromHere {
            id: jump_over_then_id,
        });

        let then_emission = self.visit_block_expression(then_block, Some(&target))?;

        self.handle_branch_emission(&mut if_emission, then_emission, &target, then_block)?;

        if_emission.push_drop_anchor(JumpAnchor::ForwardToNext {
            id: jump_over_then_id,
        });

        if let Some(else_block) = else_block {
            let else_emission = self.visit_block_expression(else_block, Some(&target))?;
            let jump_over_else_id = self.create_jump_id();

            self.jump_over_else_anchor_ids.push(jump_over_else_id);

            if_emission.push_drop_anchor(JumpAnchor::ForwardFromHere {
                id: jump_over_else_id,
            });

            self.handle_branch_emission(&mut if_emission, else_emission, &target, else_block)?;

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

    fn visit_math_binary_expression(
        &mut self,
        node: SyntaxReader,
        target: Option<&Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting math binary expression");

        let (left_expression, right_expression) = node.binary_children()?;

        let left_emission = self.visit_expression(left_expression, None)?;
        let right_emission = self.visit_expression(right_expression, None)?;

        if target.is_none()
            && let (Emission::Constant(left_value), Emission::Constant(right_value)) =
                (&left_emission, &right_emission)
        {
            let combined = self.combine_constants(
                &node,
                *left_value,
                &left_expression,
                *right_value,
                &right_expression,
            )?;

            return Ok(Emission::Constant(combined));
        }

        let mut math_emission = InstructionsEmission::new();

        let left_target = left_emission.target().cloned();
        let left_address =
            self.handle_operand_emission(&mut math_emission, left_emission, &left_expression)?;
        let right_address =
            self.handle_operand_emission(&mut math_emission, right_emission, &right_expression)?;

        let type_id = *self.resolver.get_type_binding(&node.id)?;
        let needs_drop = type_id == TypeId::STRING;
        let mut get_target = || -> Result<Allocations, ErrorKind> {
            if let Some(target) = target {
                Ok(target.clone())
            } else {
                let register_classes = self.resolver.get_field_types(type_id, &node)?;

                Ok(self.allocate_temporary_registers(register_classes))
            }
        };

        let math_instruction = match node.kind() {
            SyntaxKind::AdditionExpression => {
                let target = get_target()?;
                let destination = target.start_index;

                math_emission.set_target(Some(target));

                if needs_drop {
                    self.pending_drops.last_mut().unwrap().push(destination);
                }

                Instruction::add(destination, left_address, right_address)
            }
            SyntaxKind::AdditionAssignmentStatement => {
                math_emission.set_target(left_target);

                Instruction::add(left_address.index, left_address, right_address)
            }
            SyntaxKind::SubtractionExpression => {
                let target = get_target()?;
                let destination = target.start_index;

                math_emission.set_target(Some(target));

                Instruction::subtract(destination, left_address, right_address)
            }
            SyntaxKind::SubtractionAssignmentStatement => {
                math_emission.set_target(left_target);

                Instruction::subtract(left_address.index, left_address, right_address)
            }
            SyntaxKind::MultiplicationExpression => {
                let target = get_target()?;
                let destination = target.start_index;

                math_emission.set_target(Some(target));

                Instruction::multiply(destination, left_address, right_address)
            }
            SyntaxKind::MultiplicationAssignmentStatement => {
                math_emission.set_target(left_target);

                Instruction::multiply(left_address.index, left_address, right_address)
            }
            SyntaxKind::DivisionExpression => {
                let target = get_target()?;
                let destination = target.start_index;

                math_emission.set_target(Some(target));

                Instruction::divide(destination, left_address, right_address)
            }
            SyntaxKind::DivisionAssignmentStatement => {
                math_emission.set_target(left_target);

                Instruction::divide(left_address.index, left_address, right_address)
            }
            SyntaxKind::ModuloExpression => {
                let target = get_target()?;
                let destination = target.start_index;

                math_emission.set_target(Some(target));

                Instruction::modulo(destination, left_address, right_address)
            }
            SyntaxKind::ModuloAssignmentStatement => {
                math_emission.set_target(left_target);

                Instruction::modulo(left_address.index, left_address, right_address)
            }
            SyntaxKind::ExponentExpression => {
                let target = get_target()?;
                let destination = target.start_index;

                math_emission.set_target(Some(target));

                Instruction::power(destination, left_address, right_address)
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
        input: Option<&Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting comparison binary expression");

        let (left_expression, right_expression) = node.binary_children()?;

        let left_emission = self.visit_expression(left_expression, None)?;
        let right_emission = self.visit_expression(right_expression, None)?;

        if let Emission::Constant(left_constant) = left_emission
            && let Emission::Constant(right_constant) = right_emission
        {
            let combined = self.combine_constants(
                &node,
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

        let target = if let Some(target) = input {
            target.clone()
        } else {
            let register_classes = self.resolver.get_field_types(TypeId::BOOLEAN, &node)?;

            self.allocate_temporary_registers(register_classes)
        };
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
        let load_false_instruction = Instruction::r#move(
            target.start_index,
            OperandType::BOOLEAN,
            Address::encoded_boolean(false),
            1,
        );
        let load_true_instruction = Instruction::r#move(
            target.start_index,
            OperandType::BOOLEAN,
            Address::encoded_boolean(false),
            0,
        );

        comparison_emission.push(comparison_instruction);
        comparison_emission.push(load_false_instruction);
        comparison_emission.push(load_true_instruction);
        comparison_emission.set_target(Some(target));

        Ok(Emission::Instructions(comparison_emission))
    }

    fn visit_logical_binary_expression(
        &mut self,
        node: SyntaxReader<'_>,
        target: Option<&Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting logical binary expression");

        let (left_expression, right_expression) = node.binary_children()?;

        let left_emission = self.visit_expression(left_expression, None)?;
        let right_emission = self.visit_expression(right_expression, None)?;

        if let Emission::Constant(left_constant) = left_emission
            && let Emission::Constant(right_constant) = right_emission
        {
            let combined = self.combine_constants(
                &node,
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

        let target = if let Some(target) = target {
            target.clone()
        } else {
            let register_classes = self.resolver.get_field_types(TypeId::BOOLEAN, &node)?;

            self.allocate_temporary_registers(register_classes)
        };

        let test_instruction = match node.kind() {
            SyntaxKind::AndExpression => Instruction::test(left_address, false, 1),
            SyntaxKind::OrExpression => Instruction::test(left_address, true, 1),
            _ => unreachable!("Expected logical expression, found {}", node.kind()),
        };
        let right_move_instruction =
            Instruction::r#move(target.start_index, OperandType::BOOLEAN, right_address, 1);
        let left_move_instruction =
            Instruction::r#move(target.start_index, OperandType::BOOLEAN, left_address, 0);

        logical_emission.push(test_instruction);
        logical_emission.push(right_move_instruction);
        logical_emission.push(left_move_instruction);
        logical_emission.set_target(Some(target));

        Ok(Emission::Instructions(logical_emission))
    }

    fn visit_unary_negation_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<&Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting unary negation expression");

        let expression = node.child()?;

        let expression_emission = self.visit_expression(expression, None)?;

        if let Emission::Constant(constant) = expression_emission {
            let negated = constant.negate().ok_or_else(|| {
                ErrorKind::Compile(CompileError::CannotApplyUnaryOperator {
                    operator: node.kind(),
                    operand_type: constant.operand_type(),
                    operand_position: expression.position(),
                })
            })?;

            return Ok(Emission::Constant(negated));
        }

        let mut negation_emission = InstructionsEmission::new();

        let child_address =
            self.handle_operand_emission(&mut negation_emission, expression_emission, &expression)?;
        let target = if let Some(target) = input {
            target.clone()
        } else {
            let type_id = *self.resolver.get_type_binding(&node.id)?;
            let register_classes = self.resolver.get_field_types(type_id, &node)?;

            self.allocate_temporary_registers(register_classes)
        };
        let negate_instruction = Instruction::negate(target.start_index, child_address);

        negation_emission.push(negate_instruction);
        negation_emission.set_target(Some(target));

        Ok(Emission::Instructions(negation_emission))
    }

    fn visit_while_expression(
        &mut self,
        node: SyntaxReader<'_>,
        _: Option<&Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting while expression");

        let (condition, body) = node.binary_children()?;

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
        _target: Option<&Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting function expression");

        let declaration_id = *self.resolver.get_declaration_binding(&node.id)?;

        if let Some(prototype_index) = self
            .resolver
            .declarations
            .get_declaration_prototype(&declaration_id)
        {
            return Ok(Emission::Place(Place::Prototype {
                id: *prototype_index,
            }));
        }

        let (signature, body) = node.binary_children()?;
        let (parameters, return_type) = signature.binary_children()?;
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
            Some(parameters.children()?),
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

        Ok(Emission::Place(Place::Prototype { id: prototype_id }))
    }

    fn visit_call_expression(
        &mut self,
        node: SyntaxReader<'_>,
        target: Option<&Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting call expression");

        let (callee, argument_list) = node.binary_children()?;
        let arguments = argument_list.children()?;

        let mut call_emission = InstructionsEmission::new();

        let arguments_start = self.call_arguments.len() as u16;
        let mut argument_count = 0u16;

        for argument in arguments {
            let argument_emission = self.visit_expression(argument, None)?;
            let argument_address =
                self.handle_operand_emission(&mut call_emission, argument_emission, &argument)?;

            self.call_arguments.push(CallArgument {
                index: argument_address.index,
                memory: argument_address.memory,
                r#type: OperandType::UNIT,
            });
            argument_count += 1;
        }

        let callee_emission = self.visit_expression(callee, None)?;
        let callee_address =
            self.handle_operand_emission(&mut call_emission, callee_emission, &callee)?;

        let return_type_id = *self.resolver.get_type_binding(&node.id)?;
        let return_operand_type = self.resolver.get_small_type(return_type_id, &node)?;

        let register_count = self.resolver.get_field_types(return_type_id, &node)?;
        let target = if let Some(target) = target {
            Some(target.clone())
        } else if return_operand_type != OperandType::UNIT {
            Some(self.allocate_temporary_registers(register_count))
        } else {
            None
        };
        let destination = target.as_ref().map(|target| target.start_index);

        let call_instruction =
            Instruction::call(destination, callee_address, arguments_start, argument_count);

        call_emission.push(call_instruction);

        if return_operand_type != OperandType::UNIT {
            call_emission.set_target(target);
        }

        Ok(Emission::Instructions(call_emission))
    }

    fn visit_type(&mut self, _: SyntaxReader) -> Result<Self::TypeOutput, ErrorKind> {
        Ok(())
    }

    fn visit_path(
        &mut self,
        path: SyntaxReader,
        _: Self::PathInput,
    ) -> Result<Self::PathOutput, ErrorKind> {
        debug!("Visting path");
        debug_assert_eq!(path.kind(), SyntaxKind::Path);

        self.resolver.get_declaration_binding(&path.id).copied()
    }

    fn visit_simple_path(
        &mut self,
        node: SyntaxReader,
        _: Self::PathInput,
    ) -> Result<Self::PathOutput, ErrorKind> {
        debug!("Visting simple path");
        debug_assert_eq!(node.kind(), SyntaxKind::SimplePath);

        self.resolver.get_declaration_binding(&node.id).copied()
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
    fn target(&self) -> Option<&Allocations> {
        match self {
            Emission::Place(Place::Registers(target)) => Some(target),
            Emission::Instructions(emission) => emission.registers.as_ref(),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct InstructionsEmission {
    instructions: Vec<(Instruction, Vec<JumpAnchor>)>,
    registers: Option<Allocations>,
}

impl InstructionsEmission {
    fn new() -> Self {
        Self {
            instructions: Vec::new(),
            registers: None,
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        Self {
            instructions: Vec::with_capacity(capacity),
            registers: None,
        }
    }

    fn _with_instruction(instruction: Instruction) -> Self {
        Self {
            instructions: vec![(instruction, Vec::new())],
            registers: None,
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

    fn set_target(&mut self, target: Option<Allocations>) {
        self.registers = target;
    }

    fn push_drop_anchor(&mut self, anchor: JumpAnchor) {
        if let Some((_, anchors)) = self.instructions.last_mut() {
            anchors.push(anchor);
        }
    }

    fn merge(&mut self, other: InstructionsEmission) {
        self.instructions.extend(other.instructions);
        self.registers = other.registers;
    }
}

#[derive(Clone, Debug)]
pub enum Place {
    Constant { id: ConstantId },
    Prototype { id: PrototypeId },
    Registers(Allocations),
}

impl Place {
    fn expect_register(&self, node: &SyntaxReader) -> Result<&Allocations, ErrorKind> {
        match self {
            Place::Registers(target) => Ok(target),
            _ => Err(ErrorKind::Compile(CompileError::CannotMutate {
                position: node.position(),
            })),
        }
    }

    fn memory_and_index(&self) -> (MemoryKind, u16) {
        match self {
            Place::Constant { id } => (MemoryKind::CONSTANT, id.inner()),
            Place::Prototype { id } => (MemoryKind::CONSTANT, id.inner()),
            Place::Registers(target) => (MemoryKind::REGISTER, target.allocations[0].index),
        }
    }

    fn index(&self) -> u16 {
        match self {
            Place::Constant { id } => id.inner(),
            Place::Prototype { id } => id.inner(),
            Place::Registers(target) => target.allocations[0].index,
        }
    }

    fn register_count(&self) -> u16 {
        match self {
            Place::Constant { .. } => 0,
            Place::Prototype { .. } => 0,
            Place::Registers(target) => target.allocations.len() as u16,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Allocation {
    r#type: OperandType,
    index: u16,
}

#[derive(Clone, Debug)]
pub struct Allocations {
    allocations: SmallVec<[Allocation; 8]>,
    temporary: bool,
}

impl Allocations {
    fn start_index(&self) -> u16 {
        self.allocations
            .first()
            .map(|allocation| allocation.index)
            .unwrap_or_default()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ConstantEmission {
    Boolean(bool),
    Character(char),
    String { pool_start: u32, pool_end: u32 },
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    U128(u128),
    I128(i128),
    F32(f32),
    F64(f64),
}

impl ConstantEmission {
    fn operand_type(&self) -> OperandType {
        match self {
            ConstantEmission::Boolean(_) => OperandType::BOOLEAN,
            ConstantEmission::Character(_) => OperandType::CHARACTER,
            ConstantEmission::String { .. } => OperandType::STRING,
            ConstantEmission::U8(_) => OperandType::U_8,
            ConstantEmission::I8(_) => OperandType::I_8,
            ConstantEmission::U16(_) => OperandType::U_16,
            ConstantEmission::I16(_) => OperandType::I_16,
            ConstantEmission::U32(_) => OperandType::U_32,
            ConstantEmission::I32(_) => OperandType::I_32,
            ConstantEmission::U64(_) => OperandType::U_64,
            ConstantEmission::I64(_) => OperandType::I_64,
            ConstantEmission::U128(_) => OperandType::U_128,
            ConstantEmission::I128(_) => OperandType::I_128,
            ConstantEmission::F32(_) => OperandType::F_32,
            ConstantEmission::F64(_) => OperandType::F_64,
        }
    }

    fn add(self, other: Self, emitter: &mut Emitter) -> Result<Option<Self>, ErrorKind> {
        let sum = match (self, other) {
            (ConstantEmission::U8(left), ConstantEmission::U8(right)) => {
                ConstantEmission::U8(left + right)
            }
            (ConstantEmission::I8(left), ConstantEmission::I8(right)) => {
                ConstantEmission::I8(left + right)
            }
            (ConstantEmission::U16(left), ConstantEmission::U16(right)) => {
                ConstantEmission::U16(left + right)
            }
            (ConstantEmission::I16(left), ConstantEmission::I16(right)) => {
                ConstantEmission::I16(left + right)
            }
            (ConstantEmission::U32(left), ConstantEmission::U32(right)) => {
                ConstantEmission::U32(left + right)
            }
            (ConstantEmission::I32(left), ConstantEmission::I32(right)) => {
                ConstantEmission::I32(left + right)
            }
            (ConstantEmission::U64(left), ConstantEmission::U64(right)) => {
                ConstantEmission::U64(left + right)
            }
            (ConstantEmission::I64(left), ConstantEmission::I64(right)) => {
                ConstantEmission::I64(left + right)
            }
            (ConstantEmission::U128(left), ConstantEmission::U128(right)) => {
                ConstantEmission::U128(left + right)
            }
            (ConstantEmission::I128(left), ConstantEmission::I128(right)) => {
                ConstantEmission::I128(left + right)
            }
            (ConstantEmission::F32(left), ConstantEmission::F32(right)) => {
                ConstantEmission::F32(left + right)
            }
            (ConstantEmission::F64(left), ConstantEmission::F64(right)) => {
                ConstantEmission::F64(left + right)
            }
            (
                ConstantEmission::String {
                    pool_start: left_start,
                    pool_end: left_end,
                },
                ConstantEmission::String {
                    pool_start: right_start,
                    pool_end: right_end,
                },
            ) => {
                let left = emitter
                    .constants
                    .get_string_pool_range(left_start as usize..left_end as usize);
                let right = emitter
                    .constants
                    .get_string_pool_range(right_start as usize..right_end as usize);

                let mut combined = String::with_capacity(left.len() + right.len());

                combined.push_str(left);
                combined.push_str(right);

                let (combined_pool_start, combined_pool_end) =
                    emitter.constants.push_str_to_string_pool(&combined)?;

                ConstantEmission::String {
                    pool_start: combined_pool_start,
                    pool_end: combined_pool_end,
                }
            }
            _ => return Ok(None),
        };

        Ok(Some(sum))
    }

    fn subtract(self, other: Self) -> Option<Self> {
        match (self, other) {
            (ConstantEmission::U8(left), ConstantEmission::U8(right)) => {
                Some(ConstantEmission::U8(left - right))
            }
            (ConstantEmission::I8(left), ConstantEmission::I8(right)) => {
                Some(ConstantEmission::I8(left - right))
            }
            (ConstantEmission::U16(left), ConstantEmission::U16(right)) => {
                Some(ConstantEmission::U16(left - right))
            }
            (ConstantEmission::I16(left), ConstantEmission::I16(right)) => {
                Some(ConstantEmission::I16(left - right))
            }
            (ConstantEmission::U32(left), ConstantEmission::U32(right)) => {
                Some(ConstantEmission::U32(left - right))
            }
            (ConstantEmission::I32(left), ConstantEmission::I32(right)) => {
                Some(ConstantEmission::I32(left - right))
            }
            (ConstantEmission::U64(left), ConstantEmission::U64(right)) => {
                Some(ConstantEmission::U64(left - right))
            }
            (ConstantEmission::I64(left), ConstantEmission::I64(right)) => {
                Some(ConstantEmission::I64(left - right))
            }
            (ConstantEmission::U128(left), ConstantEmission::U128(right)) => {
                Some(ConstantEmission::U128(left - right))
            }
            (ConstantEmission::I128(left), ConstantEmission::I128(right)) => {
                Some(ConstantEmission::I128(left - right))
            }
            (ConstantEmission::F32(left), ConstantEmission::F32(right)) => {
                Some(ConstantEmission::F32(left - right))
            }
            (ConstantEmission::F64(left), ConstantEmission::F64(right)) => {
                Some(ConstantEmission::F64(left - right))
            }
            _ => None,
        }
    }

    fn multiply(self, other: Self) -> Option<Self> {
        match (self, other) {
            (ConstantEmission::U8(left), ConstantEmission::U8(right)) => {
                Some(ConstantEmission::U8(left * right))
            }
            (ConstantEmission::I8(left), ConstantEmission::I8(right)) => {
                Some(ConstantEmission::I8(left * right))
            }
            (ConstantEmission::U16(left), ConstantEmission::U16(right)) => {
                Some(ConstantEmission::U16(left * right))
            }
            (ConstantEmission::I16(left), ConstantEmission::I16(right)) => {
                Some(ConstantEmission::I16(left * right))
            }
            (ConstantEmission::U32(left), ConstantEmission::U32(right)) => {
                Some(ConstantEmission::U32(left * right))
            }
            (ConstantEmission::I32(left), ConstantEmission::I32(right)) => {
                Some(ConstantEmission::I32(left * right))
            }
            (ConstantEmission::U64(left), ConstantEmission::U64(right)) => {
                Some(ConstantEmission::U64(left * right))
            }
            (ConstantEmission::I64(left), ConstantEmission::I64(right)) => {
                Some(ConstantEmission::I64(left * right))
            }
            (ConstantEmission::U128(left), ConstantEmission::U128(right)) => {
                Some(ConstantEmission::U128(left * right))
            }
            (ConstantEmission::I128(left), ConstantEmission::I128(right)) => {
                Some(ConstantEmission::I128(left * right))
            }
            (ConstantEmission::F32(left), ConstantEmission::F32(right)) => {
                Some(ConstantEmission::F32(left * right))
            }
            (ConstantEmission::F64(left), ConstantEmission::F64(right)) => {
                Some(ConstantEmission::F64(left * right))
            }
            _ => None,
        }
    }

    fn divide(self, other: Self) -> Option<Self> {
        match (self, other) {
            (ConstantEmission::U8(left), ConstantEmission::U8(right)) => {
                Some(ConstantEmission::U8(left / right))
            }
            (ConstantEmission::I8(left), ConstantEmission::I8(right)) => {
                Some(ConstantEmission::I8(left / right))
            }
            (ConstantEmission::U16(left), ConstantEmission::U16(right)) => {
                Some(ConstantEmission::U16(left / right))
            }
            (ConstantEmission::I16(left), ConstantEmission::I16(right)) => {
                Some(ConstantEmission::I16(left / right))
            }
            (ConstantEmission::U32(left), ConstantEmission::U32(right)) => {
                Some(ConstantEmission::U32(left / right))
            }
            (ConstantEmission::I32(left), ConstantEmission::I32(right)) => {
                Some(ConstantEmission::I32(left / right))
            }
            (ConstantEmission::U64(left), ConstantEmission::U64(right)) => {
                Some(ConstantEmission::U64(left / right))
            }
            (ConstantEmission::I64(left), ConstantEmission::I64(right)) => {
                Some(ConstantEmission::I64(left / right))
            }
            (ConstantEmission::U128(left), ConstantEmission::U128(right)) => {
                Some(ConstantEmission::U128(left / right))
            }
            (ConstantEmission::I128(left), ConstantEmission::I128(right)) => {
                Some(ConstantEmission::I128(left / right))
            }
            (ConstantEmission::F32(left), ConstantEmission::F32(right)) => {
                Some(ConstantEmission::F32(left / right))
            }
            (ConstantEmission::F64(left), ConstantEmission::F64(right)) => {
                Some(ConstantEmission::F64(left / right))
            }
            _ => None,
        }
    }

    fn modulo(self, other: Self) -> Option<Self> {
        match (self, other) {
            (ConstantEmission::U8(left), ConstantEmission::U8(right)) => {
                Some(ConstantEmission::U8(left % right))
            }
            (ConstantEmission::I8(left), ConstantEmission::I8(right)) => {
                Some(ConstantEmission::I8(left % right))
            }
            (ConstantEmission::U16(left), ConstantEmission::U16(right)) => {
                Some(ConstantEmission::U16(left % right))
            }
            (ConstantEmission::I16(left), ConstantEmission::I16(right)) => {
                Some(ConstantEmission::I16(left % right))
            }
            (ConstantEmission::U32(left), ConstantEmission::U32(right)) => {
                Some(ConstantEmission::U32(left % right))
            }
            (ConstantEmission::I32(left), ConstantEmission::I32(right)) => {
                Some(ConstantEmission::I32(left % right))
            }
            (ConstantEmission::U64(left), ConstantEmission::U64(right)) => {
                Some(ConstantEmission::U64(left % right))
            }
            (ConstantEmission::I64(left), ConstantEmission::I64(right)) => {
                Some(ConstantEmission::I64(left % right))
            }
            (ConstantEmission::U128(left), ConstantEmission::U128(right)) => {
                Some(ConstantEmission::U128(left % right))
            }
            (ConstantEmission::I128(left), ConstantEmission::I128(right)) => {
                Some(ConstantEmission::I128(left % right))
            }
            (ConstantEmission::F32(left), ConstantEmission::F32(right)) => {
                Some(ConstantEmission::F32(left % right))
            }
            (ConstantEmission::F64(left), ConstantEmission::F64(right)) => {
                Some(ConstantEmission::F64(left % right))
            }
            _ => None,
        }
    }

    fn power(self, other: Self) -> Option<Self> {
        match (self, other) {
            (ConstantEmission::U8(left), ConstantEmission::U8(right)) => {
                Some(ConstantEmission::U8(left.pow(right as u32)))
            }
            (ConstantEmission::I8(left), ConstantEmission::I8(right)) => {
                Some(ConstantEmission::I8(left.pow(right as u32)))
            }
            (ConstantEmission::U16(left), ConstantEmission::U16(right)) => {
                Some(ConstantEmission::U16(left.pow(right as u32)))
            }
            (ConstantEmission::I16(left), ConstantEmission::I16(right)) => {
                Some(ConstantEmission::I16(left.pow(right as u32)))
            }
            (ConstantEmission::U32(left), ConstantEmission::U32(right)) => {
                Some(ConstantEmission::U32(left.pow(right)))
            }
            (ConstantEmission::I32(left), ConstantEmission::I32(right)) => {
                Some(ConstantEmission::I32(left.pow(right as u32)))
            }
            (ConstantEmission::U64(left), ConstantEmission::U32(right)) => {
                Some(ConstantEmission::U64(left.pow(right)))
            }
            (ConstantEmission::I64(left), ConstantEmission::I64(right)) => {
                Some(ConstantEmission::I64(left.pow(right as u32)))
            }
            (ConstantEmission::U128(left), ConstantEmission::U128(right)) => {
                Some(ConstantEmission::U128(left.pow(right as u32)))
            }
            (ConstantEmission::I128(left), ConstantEmission::I128(right)) => {
                Some(ConstantEmission::I128(left.pow(right as u32)))
            }
            (ConstantEmission::F32(left), ConstantEmission::F32(right)) => {
                Some(ConstantEmission::F32(left.powf(right)))
            }
            (ConstantEmission::F64(left), ConstantEmission::F64(right)) => {
                Some(ConstantEmission::F64(left.powf(right)))
            }
            _ => None,
        }
    }

    fn equal(self, other: Self, emitter: &Emitter) -> Option<Self> {
        match (self, other) {
            (ConstantEmission::Boolean(left), ConstantEmission::Boolean(right)) => {
                Some(ConstantEmission::Boolean(left == right))
            }
            (ConstantEmission::Character(left), ConstantEmission::Character(right)) => {
                Some(ConstantEmission::Boolean(left == right))
            }
            (
                ConstantEmission::String {
                    pool_start: left_start,
                    pool_end: left_end,
                },
                ConstantEmission::String {
                    pool_start: right_start,
                    pool_end: right_end,
                },
            ) => {
                let left = emitter
                    .constants
                    .get_string_pool_range(left_start as usize..left_end as usize);
                let right = emitter
                    .constants
                    .get_string_pool_range(right_start as usize..right_end as usize);

                Some(ConstantEmission::Boolean(left == right))
            }
            (ConstantEmission::U8(left), ConstantEmission::U8(right)) => {
                Some(ConstantEmission::Boolean(left == right))
            }
            (ConstantEmission::I8(left), ConstantEmission::I8(right)) => {
                Some(ConstantEmission::Boolean(left == right))
            }
            (ConstantEmission::U16(left), ConstantEmission::U16(right)) => {
                Some(ConstantEmission::Boolean(left == right))
            }
            (ConstantEmission::I16(left), ConstantEmission::I16(right)) => {
                Some(ConstantEmission::Boolean(left == right))
            }
            (ConstantEmission::U32(left), ConstantEmission::U32(right)) => {
                Some(ConstantEmission::Boolean(left == right))
            }
            (ConstantEmission::I32(left), ConstantEmission::I32(right)) => {
                Some(ConstantEmission::Boolean(left == right))
            }
            (ConstantEmission::U64(left), ConstantEmission::U64(right)) => {
                Some(ConstantEmission::Boolean(left == right))
            }
            (ConstantEmission::I64(left), ConstantEmission::I64(right)) => {
                Some(ConstantEmission::Boolean(left == right))
            }
            (ConstantEmission::U128(left), ConstantEmission::U128(right)) => {
                Some(ConstantEmission::Boolean(left == right))
            }
            (ConstantEmission::I128(left), ConstantEmission::I128(right)) => {
                Some(ConstantEmission::Boolean(left == right))
            }
            (ConstantEmission::F32(left), ConstantEmission::F32(right)) => {
                Some(ConstantEmission::Boolean(left == right))
            }
            (ConstantEmission::F64(left), ConstantEmission::F64(right)) => {
                Some(ConstantEmission::Boolean(left == right))
            }
            _ => None,
        }
    }

    fn not_equal(self, other: Self, emitter: &Emitter) -> Option<Self> {
        self.equal(other, emitter).map(|equality| match equality {
            ConstantEmission::Boolean(value) => ConstantEmission::Boolean(!value),
            _ => unreachable!("Expected boolean constant from equality comparison"),
        })
    }

    fn less(self, other: Self, emitter: &Emitter) -> Option<Self> {
        match (self, other) {
            (ConstantEmission::Character(left), ConstantEmission::Character(right)) => {
                Some(ConstantEmission::Boolean(left < right))
            }
            (
                ConstantEmission::String {
                    pool_start: left_start,
                    pool_end: left_end,
                },
                ConstantEmission::String {
                    pool_start: right_start,
                    pool_end: right_end,
                },
            ) => {
                let left = emitter
                    .constants
                    .get_string_pool_range(left_start as usize..left_end as usize);
                let right = emitter
                    .constants
                    .get_string_pool_range(right_start as usize..right_end as usize);

                Some(ConstantEmission::Boolean(left < right))
            }
            (ConstantEmission::U8(left), ConstantEmission::U8(right)) => {
                Some(ConstantEmission::Boolean(left < right))
            }
            (ConstantEmission::I8(left), ConstantEmission::I8(right)) => {
                Some(ConstantEmission::Boolean(left < right))
            }
            (ConstantEmission::U16(left), ConstantEmission::U16(right)) => {
                Some(ConstantEmission::Boolean(left < right))
            }
            (ConstantEmission::I16(left), ConstantEmission::I16(right)) => {
                Some(ConstantEmission::Boolean(left < right))
            }
            (ConstantEmission::U32(left), ConstantEmission::U32(right)) => {
                Some(ConstantEmission::Boolean(left < right))
            }
            (ConstantEmission::I32(left), ConstantEmission::I32(right)) => {
                Some(ConstantEmission::Boolean(left < right))
            }
            (ConstantEmission::U64(left), ConstantEmission::U64(right)) => {
                Some(ConstantEmission::Boolean(left < right))
            }
            (ConstantEmission::I64(left), ConstantEmission::I64(right)) => {
                Some(ConstantEmission::Boolean(left < right))
            }
            (ConstantEmission::U128(left), ConstantEmission::U128(right)) => {
                Some(ConstantEmission::Boolean(left < right))
            }
            (ConstantEmission::I128(left), ConstantEmission::I128(right)) => {
                Some(ConstantEmission::Boolean(left < right))
            }
            (ConstantEmission::F32(left), ConstantEmission::F32(right)) => {
                Some(ConstantEmission::Boolean(left < right))
            }
            (ConstantEmission::F64(left), ConstantEmission::F64(right)) => {
                Some(ConstantEmission::Boolean(left < right))
            }
            _ => None,
        }
    }

    fn greater(self, other: Self, emitter: &Emitter) -> Option<Self> {
        self.less_equal(other, emitter).map(|less| match less {
            ConstantEmission::Boolean(value) => ConstantEmission::Boolean(!value),
            _ => unreachable!("Expected boolean constant from less comparison"),
        })
    }

    fn less_equal(self, other: Self, emitter: &Emitter) -> Option<Self> {
        match (self, other) {
            (ConstantEmission::Character(left), ConstantEmission::Character(right)) => {
                Some(ConstantEmission::Boolean(left <= right))
            }
            (
                ConstantEmission::String {
                    pool_start: left_start,
                    pool_end: left_end,
                },
                ConstantEmission::String {
                    pool_start: right_start,
                    pool_end: right_end,
                },
            ) => {
                let left = emitter
                    .constants
                    .get_string_pool_range(left_start as usize..left_end as usize);
                let right = emitter
                    .constants
                    .get_string_pool_range(right_start as usize..right_end as usize);

                Some(ConstantEmission::Boolean(left <= right))
            }
            (ConstantEmission::U8(left), ConstantEmission::U8(right)) => {
                Some(ConstantEmission::Boolean(left <= right))
            }
            (ConstantEmission::I8(left), ConstantEmission::I8(right)) => {
                Some(ConstantEmission::Boolean(left <= right))
            }
            (ConstantEmission::U16(left), ConstantEmission::U16(right)) => {
                Some(ConstantEmission::Boolean(left <= right))
            }
            (ConstantEmission::I16(left), ConstantEmission::I16(right)) => {
                Some(ConstantEmission::Boolean(left <= right))
            }
            (ConstantEmission::U32(left), ConstantEmission::U32(right)) => {
                Some(ConstantEmission::Boolean(left <= right))
            }
            (ConstantEmission::I32(left), ConstantEmission::I32(right)) => {
                Some(ConstantEmission::Boolean(left <= right))
            }
            (ConstantEmission::U64(left), ConstantEmission::U64(right)) => {
                Some(ConstantEmission::Boolean(left <= right))
            }
            (ConstantEmission::I64(left), ConstantEmission::I64(right)) => {
                Some(ConstantEmission::Boolean(left <= right))
            }
            (ConstantEmission::U128(left), ConstantEmission::U128(right)) => {
                Some(ConstantEmission::Boolean(left <= right))
            }
            (ConstantEmission::I128(left), ConstantEmission::I128(right)) => {
                Some(ConstantEmission::Boolean(left <= right))
            }
            (ConstantEmission::F32(left), ConstantEmission::F32(right)) => {
                Some(ConstantEmission::Boolean(left <= right))
            }
            (ConstantEmission::F64(left), ConstantEmission::F64(right)) => {
                Some(ConstantEmission::Boolean(left <= right))
            }
            _ => None,
        }
    }

    fn greater_equal(self, other: Self, emitter: &Emitter) -> Option<Self> {
        self.less(other, emitter).map(|less| match less {
            ConstantEmission::Boolean(value) => ConstantEmission::Boolean(!value),
            _ => unreachable!("Expected boolean constant from less comparison"),
        })
    }

    fn negate(self) -> Option<Self> {
        match self {
            ConstantEmission::Boolean(value) => Some(ConstantEmission::Boolean(!value)),
            ConstantEmission::U8(value) => Some(ConstantEmission::U8(!value)),
            ConstantEmission::I8(value) => Some(ConstantEmission::I8(!value)),
            ConstantEmission::U16(value) => Some(ConstantEmission::U16(!value)),
            ConstantEmission::I16(value) => Some(ConstantEmission::I16(!value)),
            ConstantEmission::U32(value) => Some(ConstantEmission::U32(!value)),
            ConstantEmission::I32(value) => Some(ConstantEmission::I32(!value)),
            ConstantEmission::U64(value) => Some(ConstantEmission::U64(!value)),
            ConstantEmission::I64(value) => Some(ConstantEmission::I64(!value)),
            ConstantEmission::U128(value) => Some(ConstantEmission::U128(!value)),
            ConstantEmission::I128(value) => Some(ConstantEmission::I128(!value)),
            _ => None,
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

#[derive(Clone, Copy, Debug, Default)]
struct RegisterTracker {
    next_local: u16,
    next_temporary: u16,
    max: u16,
}

impl RegisterTracker {
    fn next_local(&mut self) -> u16 {
        let next = self.next_local;
        self.next_local + 1;
        self.max = self.max.max(self.next_local);

        next
    }

    fn next_temporary(&mut self) -> u16 {
        let next = self.next_temporary;
        self.next_temporary += 1;
        self.max = self.max.max(self.next_temporary);

        next
    }
}
