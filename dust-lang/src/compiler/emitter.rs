use std::collections::HashMap;

use lexical_core::{
    ParseFloatOptions, ParseIntegerOptions, format::RUST_LITERAL, parse_with_options,
};
use rustc_hash::FxBuildHasher;
use smallvec::SmallVec;
use tracing::{debug, trace};

use crate::{
    compiler::error::CompileError,
    constant_list::ConstantListBuilder,
    error::{ErrorKind, InternalError},
    instruction::{Drop, Instruction, Jump, MemoryKind, Move, OperandType, Operation, Test},
    native_function::NativeFunction,
    prototype::{Prototype, PrototypeId, PrototypeList},
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

    constants: &'a mut ConstantListBuilder,

    resolver: &'a mut Resolver,

    prototypes: &'a mut PrototypeList,

    argument_count: u16,

    return_count: u16,

    /// Emitted bytecode instructions, filled during compilation.
    instructions: Vec<Instruction>,

    /// Local variables declared in the function.
    locals: HashMap<DeclarationId, Place, FxBuildHasher>,

    /// Concatenated list of register indices that are referenced by DROP and JUMP instructions.
    drop_lists: Vec<u16>,

    /// Stack of register index lists that need to be dropped when exiting scopes.
    pending_drops: Vec<SmallVec<[u16; 8]>>,

    register_tracker: RegisterTracker,

    jump_placements: HashMap<u16, JumpPlacement>,

    jump_over_else_anchor_ids: Vec<u16>,

    current_scope_id: ScopeId,

    next_jump_id: u16,
}

impl<'a> Emitter<'a> {
    pub fn new(
        function: SyntaxReader<'a>,
        declaration_id: Option<DeclarationId>,
        starting_scope_id: ScopeId,
        prototype_id: PrototypeId,
        arguments: Option<SyntaxReaderIterator<'a>>,
        (source, syntax, constants, resolver, prototypes): (
            &'a Source,
            &'a Syntax,
            &'a mut ConstantListBuilder,
            &'a mut Resolver,
            &'a mut PrototypeList,
        ),
    ) -> Result<Self, ErrorKind> {
        let mut locals = HashMap::default();

        if let Some(declaration_id) = declaration_id {
            locals.insert(
                declaration_id,
                Place::Constant {
                    operand_type: OperandType::FUNCTION,
                    index: prototype_id.inner(),
                },
            );
        }

        let type_id = *resolver.get_type_binding(&function.id)?;
        let type_node = *resolver.types.get_type(type_id)?;
        let TypeNode::Function {
            value_parameters,
            return_type_id,
            ..
        } = type_node
        else {
            return Err(ErrorKind::Compile(CompileError::ExpectedFunctionType {
                found: type_id,
                position: function.position(),
            }));
        };
        let argument_count = arguments.as_ref().map_or(0, |arguments| arguments.len()) as u16;
        let return_count = resolver.get_register_size(return_type_id, &function)? as u16;

        let mut emitter = Self {
            function,
            source,
            syntax,
            constants,
            resolver,
            prototypes,
            argument_count,
            return_count,
            instructions: Vec::new(),
            locals,
            drop_lists: Vec::new(),
            pending_drops: Vec::new(),
            register_tracker: RegisterTracker::new(argument_count, return_count),
            jump_placements: HashMap::default(),
            jump_over_else_anchor_ids: Vec::new(),
            current_scope_id: starting_scope_id,
            next_jump_id: 0,
        };

        if let Some(arguments) = arguments {
            let argument_types = emitter
                .resolver
                .types
                .get_type_members(value_parameters)?
                .iter()
                .copied()
                .collect::<SmallVec<[TypeId; 8]>>();

            for (argument, expected_type_id) in arguments.into_iter().zip(argument_types) {
                let argument_id = *emitter.resolver.get_declaration_binding(&argument.id)?;
                let allocations = emitter.allocate_registers(expected_type_id, false, &argument)?;

                emitter
                    .locals
                    .insert(argument_id, Place::Register(allocations));
            }
        }

        Ok(emitter)
    }

    pub fn emit(mut self) -> Result<Prototype, ErrorKind> {
        let (_, function_body) = self.function.binary_children()?;
        let children = function_body.children()?;

        let last_index = children.len().saturating_sub(1);

        for (index, child) in children.enumerate() {
            if child.is_statement() {
                if let Some(instructions) = self.visit_statement(child)? {
                    self.handle_top_emission(Emission::Instructions(instructions), child)?;
                }

                continue;
            }

            let child_emission = if index == last_index {
                self.handle_implicit_return(child, None)?
            } else {
                self.visit_expression(child, None)?
            };

            self.handle_top_emission(child_emission, child)?;
        }

        self.finish()
    }

    pub fn finish(mut self) -> Result<Prototype, ErrorKind> {
        for jump in self.jump_placements.into_values() {
            let JumpPlacement {
                index,
                distance,
                forward,
                coalesce,
            } = jump;

            if !coalesce {
                let jump_instruction = Instruction::jump(distance, forward);

                self.instructions[index] = jump_instruction;

                continue;
            }

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
        }

        let declaration_id = self.resolver.get_declaration_binding(&self.function.id)?;
        let type_id = *self
            .resolver
            .declarations
            .get_declaration_type(declaration_id)?;
        let return_type = self
            .resolver
            .get_full_type(type_id, self.source)?
            .into_function_type()
            .map(|function_type| function_type.return_type)
            .ok_or_else(|| {
                ErrorKind::Compile(CompileError::ExpectedFunctionType {
                    found: type_id,
                    position: self.function.position(),
                })
            })?;

        Ok(Prototype {
            instructions: self.instructions,
            drops: self.drop_lists,
            return_type,
            register_count: self.register_tracker.max,
            argument_count: self.argument_count,
            return_count: self.return_count,
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
        node: &SyntaxReader,
    ) -> Result<RegisterAllocation, ErrorKind> {
        fn collect_registers(
            emitter: &mut Emitter,
            type_id: TypeId,
            temporary: bool,
            registers: &mut SmallVec<[Register; 8]>,
        ) -> Result<(), ErrorKind> {
            let type_node = emitter.resolver.types.get_type(type_id)?;

            let (operand_type, width) = match type_node {
                TypeNode::Unit => return Ok(()),
                TypeNode::Boolean => (OperandType::BOOLEAN, RegisterWidth::Single),
                TypeNode::Character => (OperandType::CHARACTER, RegisterWidth::Single),
                TypeNode::String => (OperandType::STRING, RegisterWidth::Double),
                TypeNode::U8 => (OperandType::U_8, RegisterWidth::Single),
                TypeNode::I8 => (OperandType::I_8, RegisterWidth::Single),
                TypeNode::U16 => (OperandType::U_16, RegisterWidth::Single),
                TypeNode::I16 => (OperandType::I_16, RegisterWidth::Single),
                TypeNode::U32 => (OperandType::U_32, RegisterWidth::Single),
                TypeNode::I32 => (OperandType::I_32, RegisterWidth::Single),
                TypeNode::U64 => (OperandType::U_64, RegisterWidth::Double),
                TypeNode::I64 => (OperandType::I_64, RegisterWidth::Double),
                TypeNode::U128 => (OperandType::U_128, RegisterWidth::Quad),
                TypeNode::I128 => (OperandType::I_128, RegisterWidth::Quad),
                TypeNode::F32 => (OperandType::F_32, RegisterWidth::Single),
                TypeNode::F64 => (OperandType::F_64, RegisterWidth::Double),
                TypeNode::List { .. } => (OperandType::LIST, RegisterWidth::Double),
                TypeNode::Function { .. } => (OperandType::FUNCTION, RegisterWidth::Single),
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
                            .iter()
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

                        collect_registers(emitter, member_type_id, temporary, registers)?;
                    }

                    return Ok(());
                }
                TypeNode::Enum { declaration_id, .. } => {
                    let type_id = emitter
                        .resolver
                        .declarations
                        .get_declaration_type(declaration_id)?;

                    collect_registers(emitter, *type_id, temporary, registers)?;

                    return Ok(());
                }
                TypeNode::Inferred { resolved, .. } => {
                    if let Some(resolved) = resolved {
                        collect_registers(emitter, *resolved, temporary, registers)?;

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
                emitter.register_tracker.allocate_next_temporary(width)
            } else {
                emitter.register_tracker.allocate_next_local(width)
            };

            registers.push(Register {
                operand_type,
                index: next_register_index,
            });

            Ok(())
        }

        let mut allocations = SmallVec::new();

        collect_registers(self, type_id, temporary, &mut allocations)?;

        match allocations.len() {
            0 => Err(ErrorKind::Compile(CompileError::ExpectedValue {
                node_kind: node.kind(),
                position: node.position(),
            })),
            1 => Ok(RegisterAllocation::Single {
                register: allocations[0],
                temporary,
            }),
            _ => Ok(RegisterAllocation::Multiple {
                registers: allocations,
                temporary,
            }),
        }
    }

    fn free_temporary_registers(&mut self, registers: &RegisterAllocation) {
        debug_assert!(
            registers.temporary(),
            "Cannot free local registers mid-scope"
        );

        self.register_tracker.free_temporary(registers);
    }

    fn enter_child_scope(&mut self, child_scope_id: ScopeId) {
        self.current_scope_id = child_scope_id;

        self.pending_drops.push(SmallVec::new());
    }

    fn enter_parent_scope(&mut self, parent_scope_id: ScopeId, register_tracker: RegisterTracker) {
        self.current_scope_id = parent_scope_id;
        self.register_tracker = register_tracker;
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

    fn add_constant(&mut self, constant: ConstantEmission) -> u16 {
        match constant {
            ConstantEmission::Boolean(boolean) => boolean as u16,
            ConstantEmission::Character(character) => {
                self.constants.add_character(character).inner()
            }
            ConstantEmission::U8(integer) => integer as u16,
            ConstantEmission::I8(integer) => integer as u16,
            ConstantEmission::U16(integer) => integer,
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
            SyntaxKind::AdditionExpression => {
                left_constant.add(right_constant)?.ok_or_else(create_error)
            }
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
            SyntaxKind::ExponentExpression => {
                check_for_division_by_zero()?;

                left_constant.power(right_constant).ok_or_else(create_error)
            }
            SyntaxKind::EqualExpression => {
                left_constant.equal(right_constant).ok_or_else(create_error)
            }
            SyntaxKind::NotEqualExpression => left_constant
                .not_equal(right_constant)
                .ok_or_else(create_error),
            SyntaxKind::LessThanExpression => {
                left_constant.less(right_constant).ok_or_else(create_error)
            }
            SyntaxKind::GreaterThanExpression => left_constant
                .greater(right_constant)
                .ok_or_else(create_error),
            SyntaxKind::LessThanOrEqualExpression => left_constant
                .less_equal(right_constant)
                .ok_or_else(create_error),
            SyntaxKind::GreaterThanOrEqualExpression => left_constant
                .greater_equal(right_constant)
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
                let operand_index = self.add_constant(constant);
                let move_instruction = Instruction::r#move(
                    0,
                    constant.operand_type(),
                    MemoryKind::CONSTANT,
                    operand_index,
                );

                self.emit_instruction(move_instruction);
            }
            Emission::Place(Place::Constant {
                operand_type: r#type,
                index,
            }) => {
                let move_instruction = Instruction::r#move(0, r#type, MemoryKind::CONSTANT, index);

                self.emit_instruction(move_instruction);
            }
            Emission::Place(Place::Register(RegisterAllocation::Single { register, .. })) => {
                let move_instruction = Instruction::r#move(
                    0,
                    register.operand_type,
                    MemoryKind::REGISTER,
                    register.index,
                );

                self.emit_instruction(move_instruction);
            }
            Emission::Place(Place::Register(RegisterAllocation::Multiple {
                registers, ..
            })) => {
                for register in registers {
                    let destination = self
                        .register_tracker
                        .allocate_next_local(RegisterWidth::from(register.operand_type));

                    let move_instruction = Instruction::r#move(
                        destination,
                        register.operand_type,
                        MemoryKind::REGISTER,
                        register.index,
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
        operator: SyntaxKind,
        node: &SyntaxReader,
    ) -> Result<(MemoryKind, u16, OperandType), ErrorKind> {
        match emission {
            Emission::Constant(constant) => Ok((
                MemoryKind::CONSTANT,
                self.add_constant(constant),
                constant.operand_type(),
            )),
            Emission::Place(Place::Constant {
                operand_type,
                index,
            }) => Ok((MemoryKind::CONSTANT, index, operand_type)),
            Emission::Place(Place::Register(RegisterAllocation::Single { register, .. })) => {
                Ok((MemoryKind::REGISTER, register.index, register.operand_type))
            }
            Emission::Instructions(operand_instructions) => {
                if let Some(registers) = &operand_instructions.target
                    && registers.temporary()
                {
                    self.free_temporary_registers(registers);
                }

                instructions.merge(operand_instructions);

                let (index, operand_type) = match &instructions.target {
                    Some(RegisterAllocation::Single { register, .. }) => {
                        (register.index, register.operand_type)
                    }
                    Some(RegisterAllocation::Multiple { .. }) => {
                        let type_id = *self.resolver.get_type_binding(&node.id)?;

                        return Err(ErrorKind::Compile(CompileError::CannotApplyOperator {
                            operator,
                            type_id,
                            operand_position: node.position(),
                        }));
                    }
                    None => {
                        return Err(ErrorKind::Compile(CompileError::ExpectedValue {
                            node_kind: node.kind(),
                            position: node.position(),
                        }));
                    }
                };

                Ok((MemoryKind::REGISTER, index, operand_type))
            }
            Emission::NativeFunction(_) => Err(ErrorKind::Compile(
                CompileError::ExpectedNativeFunctionCall {
                    position: node.position(),
                },
            )),
            Emission::Place(Place::Register(RegisterAllocation::Multiple { .. })) => {
                let type_id = *self.resolver.get_type_binding(&node.id)?;

                Err(ErrorKind::Compile(CompileError::CannotApplyOperator {
                    operator,
                    type_id,
                    operand_position: node.position(),
                }))
            }
            Emission::None => Err(ErrorKind::Compile(CompileError::ExpectedValue {
                node_kind: node.kind(),
                position: node.position(),
            })),
        }
    }

    fn handle_member_emission(
        &mut self,
        emission: Emission,
        node: &SyntaxReader,
    ) -> Result<Place, ErrorKind> {
        match emission {
            Emission::Constant(constant) => {
                let operand_index = self.add_constant(constant);

                Ok(Place::Constant {
                    operand_type: constant.operand_type(),
                    index: operand_index,
                })
            }
            Emission::Place(place) => Ok(place),
            Emission::Instructions(instructions) => {
                let Some(registers) = instructions.target else {
                    return Err(ErrorKind::Compile(CompileError::ExpectedValue {
                        node_kind: node.kind(),
                        position: node.position(),
                    }));
                };

                Ok(Place::Register(registers))
            }
            Emission::NativeFunction(_) => Err(ErrorKind::Compile(
                CompileError::ExpectedNativeFunctionCall {
                    position: node.position(),
                },
            )),
            _ => Err(ErrorKind::Compile(CompileError::ExpectedValue {
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
                let operand_index = self.add_constant(constant);
                let test_instruction =
                    Instruction::test(true, MemoryKind::CONSTANT, operand_index, 1);

                instructions.push(test_instruction);
            }
            Emission::Place(Place::Constant { index, .. }) => {
                let test_instruction = Instruction::test(true, MemoryKind::CONSTANT, index, 1);

                instructions.push(test_instruction);
            }
            Emission::Place(Place::Register(RegisterAllocation::Single { register, .. })) => {
                let test_instruction =
                    Instruction::test(true, MemoryKind::REGISTER, register.index, 1);

                instructions.push(test_instruction);
            }
            Emission::Instructions(mut condition_instructions) => {
                let length = condition_instructions.length();

                if condition_instructions.length() >= 3 {
                    let condition_instruction =
                        &mut condition_instructions.instructions[length - 3].0;

                    match condition_instruction.operation() {
                        Operation::LESS | Operation::LESS_EQUAL | Operation::EQUAL => {
                            // Remove the MOVE instructions, leaving only the comparison to act as
                            // control flow.

                            condition_instructions.instructions.truncate(length - 2);

                            if let Some(registers) = &condition_instructions.target
                                && registers.temporary()
                            {
                                self.free_temporary_registers(registers);
                            }
                        }
                        Operation::TEST => {
                            //

                            let first_move_instruction =
                                condition_instructions.instructions[length - 2].0;

                            let Move {
                                operand_memory,
                                operand_index,
                                ..
                            } = Move::from(&first_move_instruction);

                            let new_test_instruction =
                                Instruction::test(true, operand_memory, operand_index, 1);

                            condition_instructions.instructions.truncate(length - 3);
                            condition_instructions.push(new_test_instruction);

                            if let Some(registers) = &condition_instructions.target
                                && registers.temporary()
                            {
                                self.free_temporary_registers(registers);
                            }
                        }
                        _ => {
                            let target_register =
                                if let Some(RegisterAllocation::Single { register, .. }) =
                                    &condition_instructions.target
                                {
                                    register
                                } else {
                                    return Err(ErrorKind::Compile(
                                        CompileError::ExpectedBooleanExpression {
                                            found: *self.resolver.get_type_binding(&node.id)?,
                                            node_kind: node.kind(),
                                            position: node.position(),
                                        },
                                    ));
                                };
                            let test_instruction = Instruction::test(
                                true,
                                MemoryKind::REGISTER,
                                target_register.index,
                                1,
                            );

                            condition_instructions.push(test_instruction);
                        }
                    }
                }

                instructions.merge(condition_instructions);
            }
            _ => {
                return Err(ErrorKind::Compile(
                    CompileError::ExpectedBooleanExpression {
                        found: *self.resolver.get_type_binding(&node.id)?,
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
        branch_emission: Emission,
        instructions: &mut InstructionsEmission,
        target: &RegisterAllocation,
        node: SyntaxReader,
    ) -> Result<(), ErrorKind> {
        match branch_emission {
            Emission::Constant(constant) => {
                let destination = target.expect_single()?;
                let operand_index = self.add_constant(constant);
                let move_instruction = Instruction::r#move(
                    destination.index,
                    constant.operand_type(),
                    MemoryKind::CONSTANT,
                    operand_index,
                );

                instructions.push(move_instruction);
            }
            Emission::Place(Place::Constant {
                operand_type: r#type,
                index,
            }) => {
                let destination = target.expect_single()?;
                let move_instruction =
                    Instruction::r#move(destination.index, r#type, MemoryKind::CONSTANT, index);

                instructions.push(move_instruction);
            }
            Emission::Place(Place::Register(operand_registers)) => {
                let (destination_registers, _) = target.expect_multiple(operand_registers.len())?;

                for (destination, operand) in
                    destination_registers.iter().zip(operand_registers.iter())
                {
                    let move_instruction = Instruction::r#move(
                        destination.index,
                        operand.operand_type,
                        MemoryKind::REGISTER,
                        operand.index,
                    );

                    instructions.push(move_instruction);
                }
            }
            Emission::Instructions(branch_instructions) => {
                instructions.merge(branch_instructions);
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
        match emission {
            Emission::Constant(constant) => {
                let operand_type = constant.operand_type();
                let destination = self
                    .register_tracker
                    .allocate_next_reserved(RegisterWidth::from(operand_type));
                let move_instruction = Instruction::r#move(
                    destination,
                    operand_type,
                    MemoryKind::CONSTANT,
                    self.add_constant(constant),
                );
                let return_instruction = Instruction::r#return(true, 1);

                return_instructions.push(move_instruction);
                return_instructions.push(return_instruction);

                Ok(())
            }
            Emission::Place(Place::Constant {
                operand_type,
                index,
            }) => {
                let destination = self
                    .register_tracker
                    .allocate_next_reserved(RegisterWidth::from(operand_type));
                let move_instruction =
                    Instruction::r#move(destination, operand_type, MemoryKind::CONSTANT, index);
                let return_instruction = Instruction::r#return(true, 1);

                return_instructions.push(move_instruction);
                return_instructions.push(return_instruction);

                Ok(())
            }
            Emission::Place(Place::Register(RegisterAllocation::Single { register, .. })) => {
                let destination = self
                    .register_tracker
                    .allocate_next_reserved(RegisterWidth::from(register.operand_type));
                let move_instruction = Instruction::r#move(
                    destination,
                    register.operand_type,
                    MemoryKind::REGISTER,
                    register.index,
                );
                let return_instruction = Instruction::r#return(true, 1);

                return_instructions.push(move_instruction);
                return_instructions.push(return_instruction);

                Ok(())
            }
            Emission::Place(Place::Register(RegisterAllocation::Multiple {
                registers, ..
            })) => {
                let return_instruction = Instruction::r#return(true, registers.len() as u16);

                for register in registers {
                    let destination = self
                        .register_tracker
                        .allocate_next_reserved(RegisterWidth::from(register.operand_type));
                    let move_instruction = Instruction::r#move(
                        destination,
                        register.operand_type,
                        MemoryKind::REGISTER,
                        register.index,
                    );

                    return_instructions.push(move_instruction);
                }

                return_instructions.push(return_instruction);

                Ok(())
            }
            Emission::Instructions(mut instructions) => {
                if let Some(registers) = &return_instructions.target {
                    for register in registers.iter() {
                        let destination = self
                            .register_tracker
                            .allocate_next_reserved(RegisterWidth::from(register.operand_type));

                        let move_instruction = Instruction::r#move(
                            destination,
                            register.operand_type,
                            MemoryKind::REGISTER,
                            register.index,
                        );

                        instructions.push(move_instruction);
                    }

                    let return_instruction = Instruction::r#return(true, registers.len() as u16);

                    return_instructions.merge(instructions);
                    return_instructions.push(return_instruction);

                    Ok(())
                } else {
                    let return_instruction = Instruction::r#return(false, 0);

                    return_instructions.merge(instructions);
                    return_instructions.push(return_instruction);

                    Ok(())
                }
            }
            Emission::None => {
                let return_instruction = Instruction::r#return(false, 0);

                return_instructions.push(return_instruction);

                Ok(())
            }
            Emission::NativeFunction(_) => Err(ErrorKind::Compile(
                CompileError::ExpectedNativeFunctionCall {
                    position: node.position(),
                },
            )),
        }
    }

    fn handle_implicit_return(
        &mut self,
        node: SyntaxReader,
        target: Option<RegisterAllocation>,
    ) -> Result<Emission, ErrorKind> {
        let mut return_emission = InstructionsEmission::new();

        if node.is_expression() {
            let expression_emission = self.visit_expression(node, target)?;

            self.handle_return_emission(&mut return_emission, expression_emission, node)?;
        } else {
            if node.is_item() {
                self.visit_item(node)?;
            } else {
                self.visit_statement(node)?;
            }

            let return_instruction = Instruction::r#return(false, 0);

            return_emission.push(return_instruction);
        }

        Ok(Emission::Instructions(return_emission))
    }
}

impl SyntaxVisitor for Emitter<'_> {
    type RootOutput = Emission;
    type StatementOutput = InstructionsEmission;
    type ExpressionInput = RegisterAllocation;
    type ExpressionOutput = Emission;
    type TypeOutput = ();
    type PathInput = ();
    type PathOutput = DeclarationId;

    fn visit_root(&mut self, _: SyntaxReader) -> Result<Self::RootOutput, ErrorKind> {
        unreachable!("Emitter should never visit root nodes");
    }

    fn visit_module_item(&mut self, _: SyntaxReader<'_>) -> Result<(), ErrorKind> {
        Ok(())
    }

    fn visit_function_item(&mut self, node: SyntaxReader<'_>) -> Result<(), ErrorKind> {
        debug!("Visting function item");

        let (function_name, function_expression) = node.binary_children()?;
        let (signature, body) = function_expression.binary_children()?;
        let mut singature_children = signature.children()?;
        let parameters = singature_children.expect_next()?;

        let function_scope_id = *self.resolver.get_scope_binding(&body.id)?;
        let prototype_id = self.prototypes.reserve_slot();
        let declaration_id = *self.resolver.get_declaration_binding(&function_name.id)?;

        let function_emitter = Emitter::new(
            node,
            Some(declaration_id),
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

        self.locals.insert(
            declaration_id,
            Place::Constant {
                operand_type: OperandType::FUNCTION,
                index: prototype_id.inner(),
            },
        );

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

        let target = self.allocate_registers(type_id, false, &expression)?;
        let mut expression_emission = self.visit_expression(expression, Some(target))?;
        let target = expression_emission
            .take_target()
            .expect("Failed to set provided target");

        match expression_emission {
            Emission::Constant(constant) => {
                let destination = target.expect_single()?;
                let operand_index = self.add_constant(constant);
                let move_instruction = Instruction::r#move(
                    destination.index,
                    constant.operand_type(),
                    MemoryKind::CONSTANT,
                    operand_index,
                );

                let_statement_instructions.push(move_instruction);
            }
            Emission::Place(Place::Constant {
                operand_type: r#type,
                index,
            }) => {
                if target.len() != 1 {
                    return Err(ErrorKind::Internal(InternalError::InvalidRegisterCount {
                        expected: 1,
                        found: target.len(),
                    }));
                }

                let destination = target.index();
                let move_instruction =
                    Instruction::r#move(destination, r#type, MemoryKind::CONSTANT, index);

                let_statement_instructions.push(move_instruction);
            }
            Emission::Place(Place::Register(RegisterAllocation::Single { register, .. })) => {
                if target.len() != 1 {
                    return Err(ErrorKind::Internal(InternalError::InvalidRegisterCount {
                        expected: 1,
                        found: target.len(),
                    }));
                }

                let destination = target.index();
                let move_instruction = Instruction::r#move(
                    destination,
                    register.operand_type,
                    MemoryKind::REGISTER,
                    register.index,
                );

                let_statement_instructions.push(move_instruction);
            }
            Emission::Place(Place::Register(RegisterAllocation::Multiple {
                registers: ref operand_registers,
                ..
            })) => {
                if target.len() != operand_registers.len() {
                    return Err(ErrorKind::Internal(InternalError::InvalidRegisterCount {
                        expected: operand_registers.len(),
                        found: target.len(),
                    }));
                }

                for (destination, operand) in target.iter().zip(operand_registers.iter()) {
                    let move_instruction = Instruction::r#move(
                        destination.index,
                        operand.operand_type,
                        MemoryKind::REGISTER,
                        operand.index,
                    );

                    let_statement_instructions.push(move_instruction);
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

        self.locals.insert(declaration_id, Place::Register(target));
        let_statement_instructions.set_target(None);

        Ok(let_statement_instructions)
    }

    fn visit_assignment_expression(
        &mut self,
        node: SyntaxReader<'_>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting reassignment statement");

        let (path, expression_statement) = node.binary_children()?;
        let expression = expression_statement.child()?;

        let declaration_id = self.resolver.get_declaration_binding(&path.id)?;
        let local = self
            .locals
            .get(declaration_id)
            .ok_or_else(|| {
                ErrorKind::Compile(CompileError::OutOfScopeId {
                    declaration_id: *declaration_id,
                    usage_position: path.position(),
                })
            })?
            .clone();

        let mut reassignment_instructions = InstructionsEmission::new();

        let destination_registers = local.expect_register(&path)?;
        let expression_emission = self.visit_expression(expression, Some(destination_registers))?;

        match expression_emission {
            Emission::Constant(constant) => {
                let destination_registers = expression_emission
                    .target()
                    .expect("Failed to set provided target");

                for allocation in destination_registers.iter() {
                    let operand_index = self.add_constant(constant);
                    let move_instruction = Instruction::r#move(
                        allocation.index,
                        allocation.operand_type,
                        MemoryKind::CONSTANT,
                        operand_index,
                    );

                    reassignment_instructions.push(move_instruction);

                    if allocation.operand_type == OperandType::STRING {
                        self.add_drop(allocation.index);
                    }
                }
            }
            Emission::Place(Place::Constant {
                operand_type,
                index,
            }) => {
                let destination = expression_emission
                    .target()
                    .expect("Failed to set provided target")
                    .expect_single()?;

                let move_instruction = Instruction::r#move(
                    destination.index,
                    operand_type,
                    MemoryKind::CONSTANT,
                    index,
                );

                reassignment_instructions.push(move_instruction);
            }
            Emission::Place(Place::Register(RegisterAllocation::Single { register, .. })) => {
                let destination = expression_emission
                    .target()
                    .expect("Failed to set provided target")
                    .expect_single()?;

                let move_instruction = Instruction::r#move(
                    destination.index,
                    register.operand_type,
                    MemoryKind::REGISTER,
                    register.index,
                );

                reassignment_instructions.push(move_instruction);
            }
            Emission::Place(Place::Register(RegisterAllocation::Multiple {
                registers: ref operand_registers,
                ..
            })) => {
                let (destination_registers, _) = expression_emission
                    .target()
                    .expect("Failed to set provided target")
                    .expect_multiple(operand_registers.len())?;

                for (destination, operand) in
                    destination_registers.into_iter().zip(operand_registers)
                {
                    let move_instruction = Instruction::r#move(
                        destination.index,
                        operand.operand_type,
                        MemoryKind::REGISTER,
                        operand.index,
                    );

                    reassignment_instructions.push(move_instruction);
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

        Ok(Emission::Instructions(reassignment_instructions))
    }

    fn visit_compound_assignment_expression(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting binary assignment statement");

        let emission = self.visit_math_expression(node, None)?;
        let instructions = if let Emission::Instructions(mut instructions) = emission {
            instructions.set_target(None);

            instructions
        } else {
            InstructionsEmission::new()
        };

        Ok(Emission::Instructions(instructions))
    }

    fn visit_boolean_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting boolean expression");

        Ok(Emission::Constant(ConstantEmission::Boolean(
            node.payload().decode_boolean(),
        )))
    }

    fn visit_byte_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting byte expression");

        Ok(Emission::Constant(ConstantEmission::U8(
            node.payload().decode_byte(),
        )))
    }

    fn visit_character_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting character expression");

        Ok(Emission::Constant(ConstantEmission::Character(
            node.payload().decode_character(),
        )))
    }

    fn visit_float_expression(
        &mut self,
        node: SyntaxReader,
        target: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting float expression");

        let float_str = self.source.get_file_content(&node.position())?;
        let float_constant = match target {
            Some(RegisterAllocation::Single { register, .. })
                if register.operand_type == OperandType::F_32 =>
            {
                let float = parse_with_options::<f32, RUST_LITERAL>(
                    float_str.as_bytes(),
                    &ParseFloatOptions::default(),
                )
                .unwrap_or_default();

                ConstantEmission::F32(float)
            }
            Some(RegisterAllocation::Single { register, .. })
                if register.operand_type != OperandType::F_64 =>
            {
                return Err(ErrorKind::Internal(InternalError::ExpectedFloatRegister));
            }
            _ => {
                let float = parse_with_options::<f64, RUST_LITERAL>(
                    float_str.as_bytes(),
                    &ParseFloatOptions::default(),
                )
                .unwrap_or_default();

                ConstantEmission::F64(float)
            }
        };

        Ok(Emission::Constant(float_constant))
    }

    fn visit_integer_expression(
        &mut self,
        node: SyntaxReader,
        target: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting integer expression");

        let integer_str = self.source.get_file_content(&node.position())?;
        let integer_constant = match target {
            Some(RegisterAllocation::Single { register, .. })
                if register.operand_type == OperandType::U_8 =>
            {
                let integer = parse_with_options::<u8, RUST_LITERAL>(
                    integer_str.as_bytes(),
                    &ParseIntegerOptions::default(),
                )
                .unwrap_or_default();

                ConstantEmission::U8(integer)
            }
            Some(RegisterAllocation::Single { register, .. })
                if register.operand_type == OperandType::I_8 =>
            {
                let integer = parse_with_options::<i8, RUST_LITERAL>(
                    integer_str.as_bytes(),
                    &ParseIntegerOptions::default(),
                )
                .unwrap_or_default();

                ConstantEmission::I8(integer)
            }
            Some(RegisterAllocation::Single { register, .. })
                if register.operand_type == OperandType::U_16 =>
            {
                let integer = parse_with_options::<u16, RUST_LITERAL>(
                    integer_str.as_bytes(),
                    &ParseIntegerOptions::default(),
                )
                .unwrap_or_default();

                ConstantEmission::U16(integer)
            }
            Some(RegisterAllocation::Single { register, .. })
                if register.operand_type == OperandType::I_16 =>
            {
                let integer = parse_with_options::<i16, RUST_LITERAL>(
                    integer_str.as_bytes(),
                    &ParseIntegerOptions::default(),
                )
                .unwrap_or_default();

                ConstantEmission::I16(integer)
            }
            Some(RegisterAllocation::Single { register, .. })
                if register.operand_type == OperandType::U_32 =>
            {
                let integer = parse_with_options::<u32, RUST_LITERAL>(
                    integer_str.as_bytes(),
                    &ParseIntegerOptions::default(),
                )
                .unwrap_or_default();

                ConstantEmission::U32(integer)
            }
            Some(RegisterAllocation::Single { register, .. })
                if register.operand_type == OperandType::I_32 =>
            {
                let integer = parse_with_options::<i32, RUST_LITERAL>(
                    integer_str.as_bytes(),
                    &ParseIntegerOptions::default(),
                )
                .unwrap_or_default();

                ConstantEmission::I32(integer)
            }
            Some(RegisterAllocation::Single { register, .. })
                if register.operand_type == OperandType::U_64 =>
            {
                let integer = parse_with_options::<u64, RUST_LITERAL>(
                    integer_str.as_bytes(),
                    &ParseIntegerOptions::default(),
                )
                .unwrap_or_default();

                ConstantEmission::U64(integer)
            }
            Some(RegisterAllocation::Single { register, .. })
                if register.operand_type == OperandType::I_64 =>
            {
                let integer = parse_with_options::<i64, RUST_LITERAL>(
                    integer_str.as_bytes(),
                    &ParseIntegerOptions::default(),
                )
                .unwrap_or_default();

                ConstantEmission::I64(integer)
            }
            Some(RegisterAllocation::Single { register, .. })
                if register.operand_type == OperandType::U_128 =>
            {
                let integer = parse_with_options::<u128, RUST_LITERAL>(
                    integer_str.as_bytes(),
                    &ParseIntegerOptions::default(),
                )
                .unwrap_or_default();

                ConstantEmission::U128(integer)
            }
            Some(RegisterAllocation::Single { register, .. })
                if register.operand_type == OperandType::I_128 =>
            {
                let integer = parse_with_options::<i128, RUST_LITERAL>(
                    integer_str.as_bytes(),
                    &ParseIntegerOptions::default(),
                )
                .unwrap_or_default();

                ConstantEmission::I128(integer)
            }
            None => {
                let integer = parse_with_options::<i32, RUST_LITERAL>(
                    integer_str.as_bytes(),
                    &ParseIntegerOptions::default(),
                )
                .unwrap_or_default();

                ConstantEmission::I32(integer)
            }
            _ => {
                return Err(ErrorKind::Internal(InternalError::ExpectedIntegerRegister));
            }
        };

        Ok(Emission::Constant(integer_constant))
    }

    fn visit_string_expression(
        &mut self,
        node: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting string expression");

        let str = self
            .source
            .get_file(node.file_id())?
            .content_str(node.span().shrink(1))?;
        let constant_id = self.constants.add_string(str);

        self.resolver.add_type_binding(node.id, TypeId::STRING);

        Ok(Emission::Place(Place::Constant {
            operand_type: OperandType::STRING,
            index: constant_id.inner(),
        }))
    }

    fn visit_list_expression(
        &mut self,
        node: SyntaxReader,
        target: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting list expression");

        let elements = node.children()?;

        let target = if let Some(target) = target {
            target
        } else {
            let type_id = *self.resolver.get_type_binding(&node.id)?;

            self.allocate_registers(type_id, true, &node)?
        };
        let destination = target.expect_single()?;
        let mut list_instructions = {
            let mut emission = InstructionsEmission::with_capacity(elements.len());

            emission.push(Instruction::no_op()); // Placeholder for NEW_LIST

            emission
        };

        let mut element_count = 0;
        let mut element_type = None;
        let mut element_size = 0;

        let mut handle_element_emission =
            |emitter: &mut Emitter,
             element_emission: Emission,
             element_index: usize,
             element_node: &SyntaxReader| {
                match element_emission {
                    Emission::Constant(constant) => {
                        let operand = emitter.add_constant(constant);
                        let operand_type = constant.operand_type();
                        let list_index = emitter.constants.add_u64(element_index as u64).inner();
                        let set_list_instruction = Instruction::set_list(
                            destination.index,
                            operand_type,
                            MemoryKind::CONSTANT,
                            operand,
                            MemoryKind::CONSTANT,
                            list_index,
                        );

                        list_instructions.push(set_list_instruction);

                        element_type = Some(operand_type);
                        element_count += 1;
                    }
                    Emission::Place(Place::Constant {
                        index,
                        operand_type,
                    }) => {
                        let list_index = emitter.constants.add_u64(element_index as u64).inner();
                        let set_list_instruction = Instruction::set_list(
                            destination.index,
                            operand_type,
                            MemoryKind::CONSTANT,
                            index,
                            MemoryKind::CONSTANT,
                            list_index,
                        );

                        list_instructions.push(set_list_instruction);

                        element_type = Some(operand_type);
                        element_count += 1;
                    }
                    Emission::Place(Place::Register(RegisterAllocation::Single {
                        register,
                        ..
                    })) => {
                        let list_index = emitter.constants.add_u64(element_index as u64).inner();
                        let set_list_instruction = Instruction::set_list(
                            destination.index,
                            register.operand_type,
                            MemoryKind::REGISTER,
                            register.index,
                            MemoryKind::CONSTANT,
                            list_index,
                        );

                        list_instructions.push(set_list_instruction);

                        element_type = Some(register.operand_type);
                        element_count += 1;
                    }
                    Emission::Place(Place::Register(RegisterAllocation::Multiple {
                        registers,
                        ..
                    })) => {
                        element_size = registers
                            .iter()
                            .map(|register| register.operand_type.size_in_bytes())
                            .sum();
                        let mut offset = element_index * element_size;

                        for register in registers {
                            let field_offset_index =
                                emitter.constants.add_u64(offset as u64).inner();
                            offset += register.operand_type.size_in_bytes();

                            let set_list_instruction = Instruction::set_list(
                                destination.index,
                                register.operand_type,
                                MemoryKind::REGISTER,
                                register.index,
                                MemoryKind::CONSTANT,
                                field_offset_index,
                            );

                            list_instructions.push(set_list_instruction);
                        }

                        element_type = Some(OperandType::STRUCT);
                        element_count += 1;
                    }
                    Emission::Instructions(element_instructions) => {
                        let target_register = if let Some(target) = &element_instructions.target {
                            target.expect_single()?
                        } else {
                            return Err(ErrorKind::Internal(
                                InternalError::ExpectedEmissionTarget {
                                    node_kind: element_node.kind(),
                                },
                            ));
                        };
                        let set_list_instruction = Instruction::set_list(
                            destination.index,
                            target_register.operand_type,
                            MemoryKind::REGISTER,
                            target.index(),
                            MemoryKind::CONSTANT,
                            emitter.constants.add_u64(element_index as u64).inner(),
                        );

                        list_instructions.merge(element_instructions);
                        list_instructions.push(set_list_instruction);

                        element_type = Some(target_register.operand_type);
                        element_count += 1;
                    }
                    Emission::NativeFunction(_) => {
                        return Err(ErrorKind::Compile(
                            CompileError::ExpectedNativeFunctionCall {
                                position: element_node.position(),
                            },
                        ));
                    }
                    Emission::None => {
                        return Err(ErrorKind::Compile(CompileError::ExpectedValue {
                            node_kind: element_node.kind(),
                            position: element_node.position(),
                        }));
                    }
                }

                Ok(())
            };

        for (index, element) in elements.enumerate() {
            let element_emission = self.visit_expression(element, None)?;

            handle_element_emission(self, element_emission, index, &element)?;
        }

        let element_operand_type = if let Some(element_type) = element_type {
            element_type
        } else {
            destination.operand_type
        };
        let element_size: u16 = if let Ok(size) = u16::try_from(element_size) {
            size
        } else {
            let first_element = node.children()?.next().unwrap();
            let type_id = *self.resolver.get_type_binding(&first_element.id)?;

            return Err(ErrorKind::Compile(CompileError::ListElementSizeOverflow {
                type_id,
                size: element_size,
                position: first_element.position(),
            }));
        };
        let element_count_index = self.constants.add_u64(element_count as u64).inner();

        let new_list_instruction = Instruction::new_list(
            destination.index,
            element_operand_type,
            MemoryKind::CONSTANT,
            element_count_index,
            element_size,
        );

        list_instructions.instructions[0] = (new_list_instruction, Vec::new());

        list_instructions.set_target(Some(target));

        Ok(Emission::Instructions(list_instructions))
    }

    fn visit_index_expression(
        &mut self,
        node: SyntaxReader,
        target: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting index expression");

        let (list_expression, index_expression) = node.binary_children()?;

        let left_emission = self.visit_expression(list_expression, None)?;
        let right_emission = self.visit_expression(index_expression, None)?;

        let mut index_emission = InstructionsEmission::new();

        let list_place = self.handle_member_emission(left_emission, &list_expression)?;
        let list_index = match list_place {
            Place::Register(RegisterAllocation::Single { register, .. }) => register.index,
            _ => {
                let type_id = *self.resolver.get_type_binding(&list_expression.id)?;

                return Err(ErrorKind::Compile(CompileError::CannotIndex {
                    type_id,
                    position: list_expression.position(),
                }));
            }
        };
        let index_place = self.handle_member_emission(right_emission, &index_expression)?;
        let (index_memory, index_index) = match index_place {
            Place::Constant { index, .. } => (MemoryKind::CONSTANT, index),
            Place::Register(RegisterAllocation::Single { register, .. }) => {
                (MemoryKind::REGISTER, register.index)
            }
            _ => {
                let type_id = *self.resolver.get_type_binding(&index_expression.id)?;

                return Err(ErrorKind::Compile(CompileError::ExpectedIntegerIndex {
                    found: type_id,
                    position: index_expression.position(),
                }));
            }
        };

        let target = if let Some(target) = target {
            target
        } else {
            let type_id = *self.resolver.get_type_binding(&node.id)?;

            self.allocate_registers(type_id, true, &node)?
        };
        let register = target.expect_single()?;
        let get_list_instruction = Instruction::get_list(
            register.index,
            register.operand_type,
            list_index,
            index_memory,
            index_index,
        );

        index_emission.push(get_list_instruction);
        index_emission.set_target(Some(target));

        Ok(Emission::Instructions(index_emission))
    }

    fn visit_path_expression(
        &mut self,
        path_expression: SyntaxReader,
        _: Option<Self::ExpressionInput>,
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
        target: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting struct expression");

        let (_, struct_fields) = node.binary_children()?;

        let target = if let Some(target) = target {
            target
        } else {
            let type_id = *self.resolver.get_type_binding(&node.id)?;

            self.allocate_registers(type_id, true, &node)?
        };

        let mut struct_instructions = InstructionsEmission::new();

        let fields_and_registers = struct_fields
            .children()?
            .array_chunks::<2>()
            .zip(target.iter());

        for ([_, field_expression], destination) in fields_and_registers {
            let field_emission = self.visit_expression(field_expression, None)?;
            let field_place = self.handle_member_emission(field_emission, &field_expression)?;

            match field_place {
                Place::Constant {
                    operand_type,
                    index,
                } => {
                    let move_instruction = Instruction::r#move(
                        destination.index,
                        operand_type,
                        MemoryKind::CONSTANT,
                        index,
                    );

                    struct_instructions.push(move_instruction);
                }
                Place::Register(RegisterAllocation::Single { register, .. }) => {
                    let move_instruction = Instruction::r#move(
                        destination.index,
                        register.operand_type,
                        MemoryKind::REGISTER,
                        register.index,
                    );

                    struct_instructions.push(move_instruction);
                }
                Place::Register(RegisterAllocation::Multiple { registers, .. }) => {
                    for register in registers {
                        let move_instruction = Instruction::r#move(
                            destination.index,
                            register.operand_type,
                            MemoryKind::REGISTER,
                            register.index,
                        );

                        struct_instructions.push(move_instruction);
                    }
                }
            }
        }

        struct_instructions.set_target(Some(target));

        Ok(Emission::Instructions(struct_instructions))
    }

    fn visit_block_expression(
        &mut self,
        node: SyntaxReader<'_>,
        target: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting block expression");

        let children = node.children()?;

        let block_scope_id = *self.resolver.get_scope_binding(&node.id)?;
        let parent_scope_id = self.current_scope_id;
        let parent_scope_tracker = self.register_tracker;

        self.enter_child_scope(block_scope_id);

        let child_count = children.len();
        let mut block_instructions = InstructionsEmission::new();

        for (index, child) in children.enumerate() {
            let is_last = index == child_count - 1;

            if child.is_statement() {
                if let Some(instructions) = self.visit_statement(child)? {
                    block_instructions.merge(instructions);
                }

                continue;
            }

            if !is_last {
                let expression_emission = self.visit_expression(child, None)?;

                if let Emission::Instructions(expression_instructions) = expression_emission {
                    block_instructions.merge(expression_instructions);
                }

                continue;
            }

            let mut last_emission = self.visit_expression(child, target)?;

            if block_instructions.is_empty() {
                return Ok(last_emission);
            }

            let target = if let Some(target) = last_emission.take_target() {
                target
            } else {
                let type_id = *self.resolver.get_type_binding(&child.id)?;

                self.allocate_registers(type_id, true, &child)?
            };

            match last_emission {
                Emission::Constant(constant) => {
                    let destination = target.expect_single()?;
                    let operand_type = constant.operand_type();
                    let operand_index = self.add_constant(constant);
                    let move_instruction = Instruction::r#move(
                        destination.index,
                        operand_type,
                        MemoryKind::CONSTANT,
                        operand_index,
                    );

                    block_instructions.push(move_instruction);
                    block_instructions.set_target(Some(target));
                }
                Emission::Place(Place::Constant {
                    operand_type: r#type,
                    index,
                }) => {
                    let destination = target.expect_single()?;
                    let move_instruction =
                        Instruction::r#move(destination.index, r#type, MemoryKind::CONSTANT, index);

                    block_instructions.push(move_instruction);
                    block_instructions.set_target(Some(target));
                }
                Emission::Place(Place::Register(RegisterAllocation::Single {
                    register: operand_register,
                    ..
                })) => {
                    let destination = target.expect_single()?;
                    let move_instruction = Instruction::r#move(
                        destination.index,
                        destination.operand_type,
                        MemoryKind::REGISTER,
                        operand_register.index,
                    );

                    block_instructions.push(move_instruction);
                    block_instructions.set_target(Some(target));
                }
                Emission::Place(Place::Register(RegisterAllocation::Multiple {
                    registers: operand_registers,
                    ..
                })) => {
                    let (destinations, _) = target.expect_multiple(operand_registers.len())?;

                    for (destination, operand) in destinations.iter().zip(operand_registers) {
                        let move_instruction = Instruction::r#move(
                            destination.index,
                            destination.operand_type,
                            MemoryKind::REGISTER,
                            operand.index,
                        );

                        block_instructions.push(move_instruction);
                    }

                    block_instructions.set_target(Some(target));
                }
                Emission::Instructions(instructions) => {
                    block_instructions.merge(instructions);
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
        }

        self.enter_parent_scope(parent_scope_id, parent_scope_tracker);
        self.handle_drops(&mut block_instructions);

        Ok(Emission::Instructions(block_instructions))
    }

    fn visit_if_expression(
        &mut self,
        node: SyntaxReader<'_>,
        target: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting if expression");

        let mut children = node.children()?;
        let condition = children.expect_next()?;
        let then_block = children.expect_next()?;
        let else_block = children.next();

        let mut if_instructions = InstructionsEmission::new();

        let condition_emission = self.visit_expression(condition, None)?;

        self.handle_condition_emission(&mut if_instructions, condition_emission, &condition)?;

        let target = if let Some(target) = target {
            target
        } else {
            let type_id = *self.resolver.get_type_binding(&node.id)?;

            self.allocate_registers(type_id, true, &node)?
        };
        let jump_over_then_id = self.create_jump_id();
        let start_else_anchor_count = self.jump_over_else_anchor_ids.len();

        if_instructions.push_drop_anchor(JumpAnchor::ForwardFromHere {
            id: jump_over_then_id,
        });

        let mut then_emission = self.visit_block_expression(then_block, Some(target))?;
        let target = then_emission
            .take_target()
            .expect("Failed to set provided target");

        self.handle_branch_emission(then_emission, &mut if_instructions, &target, then_block)?;

        if_instructions.push_drop_anchor(JumpAnchor::ForwardToNext {
            id: jump_over_then_id,
        });

        if let Some(else_block) = else_block {
            let mut else_emission = self.visit_block_expression(else_block, Some(target))?;
            let target = else_emission
                .take_target()
                .expect("Failed to set provided target");

            let jump_over_else_id = self.create_jump_id();

            self.jump_over_else_anchor_ids.push(jump_over_else_id);

            if_instructions.push_drop_anchor(JumpAnchor::ForwardFromHere {
                id: jump_over_else_id,
            });

            self.handle_branch_emission(else_emission, &mut if_instructions, &target, else_block)?;

            if_instructions.push_drop_anchor(JumpAnchor::ForwardToNext {
                id: jump_over_else_id,
            });

            if_instructions.set_target(Some(target));
        } else {
            if_instructions.set_target(Some(target));
        }

        let end_else_anchor_count = self.jump_over_else_anchor_ids.len();

        for index in start_else_anchor_count..end_else_anchor_count {
            let jump_id = self.jump_over_else_anchor_ids[index];

            if_instructions.push_drop_anchor(JumpAnchor::ForwardToNext { id: jump_id });
        }

        Ok(Emission::Instructions(if_instructions))
    }

    fn visit_math_expression(
        &mut self,
        node: SyntaxReader,
        target: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting math binary expression");

        let (left_expression, right_expression) = node.binary_children()?;

        let mut left_emission = self.visit_expression(left_expression, None)?;
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

        let left_target = left_emission.take_target();
        let (left_memory, left_index, _) = self.handle_operand_emission(
            &mut math_emission,
            left_emission,
            node.kind(),
            &left_expression,
        )?;
        let (right_memory, right_index, _) = self.handle_operand_emission(
            &mut math_emission,
            right_emission,
            node.kind(),
            &right_expression,
        )?;

        let type_id = *self.resolver.get_type_binding(&node.id)?;
        let mut handle_target_register = |target, node| -> Result<Register, ErrorKind> {
            let target = if let Some(target) = target {
                target
            } else {
                self.allocate_registers(type_id, true, node)?
            };

            let register = target.expect_single()?;

            math_emission.set_target(Some(target));

            Ok(register)
        };

        let math_instruction = match node.kind() {
            SyntaxKind::AdditionExpression => {
                let register = handle_target_register(target, &node)?;

                if type_id == TypeId::STRING {
                    self.pending_drops.last_mut().unwrap().push(register.index);
                }

                Instruction::add(
                    register.index,
                    register.operand_type,
                    left_memory,
                    left_index,
                    right_memory,
                    right_index,
                )
            }
            SyntaxKind::AdditionAssignmentExpression => {
                let regsiter = handle_target_register(left_target, &left_expression)?;

                Instruction::add(
                    left_index,
                    regsiter.operand_type,
                    left_memory,
                    left_index,
                    right_memory,
                    right_index,
                )
            }
            SyntaxKind::SubtractionExpression => {
                let register = handle_target_register(target, &node)?;

                Instruction::subtract(
                    register.index,
                    register.operand_type,
                    left_memory,
                    left_index,
                    right_memory,
                    right_index,
                )
            }
            SyntaxKind::SubtractionAssignmentExpression => {
                let register = handle_target_register(left_target, &left_expression)?;

                Instruction::subtract(
                    register.index,
                    register.operand_type,
                    left_memory,
                    left_index,
                    right_memory,
                    right_index,
                )
            }
            SyntaxKind::MultiplicationExpression => {
                let register = handle_target_register(target, &node)?;

                Instruction::multiply(
                    register.index,
                    register.operand_type,
                    left_memory,
                    left_index,
                    right_memory,
                    right_index,
                )
            }
            SyntaxKind::MultiplicationAssignmentExpression => {
                let register = handle_target_register(left_target, &left_expression)?;

                Instruction::multiply(
                    register.index,
                    register.operand_type,
                    left_memory,
                    left_index,
                    right_memory,
                    right_index,
                )
            }
            SyntaxKind::DivisionExpression => {
                let register = handle_target_register(target, &node)?;

                Instruction::divide(
                    register.index,
                    register.operand_type,
                    left_memory,
                    left_index,
                    right_memory,
                    right_index,
                )
            }
            SyntaxKind::DivisionAssignmentExpression => {
                let register = handle_target_register(left_target, &left_expression)?;

                Instruction::divide(
                    register.index,
                    register.operand_type,
                    left_memory,
                    left_index,
                    right_memory,
                    right_index,
                )
            }
            SyntaxKind::ModuloExpression => {
                let register = handle_target_register(target, &node)?;

                Instruction::modulo(
                    register.index,
                    register.operand_type,
                    left_memory,
                    left_index,
                    right_memory,
                    right_index,
                )
            }
            SyntaxKind::ModuloAssignmentExpression => {
                let register = handle_target_register(left_target, &left_expression)?;

                Instruction::modulo(
                    register.index,
                    register.operand_type,
                    left_memory,
                    left_index,
                    right_memory,
                    right_index,
                )
            }
            SyntaxKind::ExponentExpression => {
                let register = handle_target_register(target, &node)?;

                Instruction::power(
                    register.index,
                    register.operand_type,
                    left_memory,
                    left_index,
                    right_memory,
                    right_index,
                )
            }
            SyntaxKind::ExponentAssignmentExpression => {
                let register = handle_target_register(left_target, &left_expression)?;

                Instruction::power(
                    register.index,
                    register.operand_type,
                    left_memory,
                    left_index,
                    right_memory,
                    right_index,
                )
            }
            _ => unreachable!("Expected math expression, found {}", node.kind()),
        };

        math_emission.push(math_instruction);

        Ok(Emission::Instructions(math_emission))
    }

    fn visit_comparison_expression(
        &mut self,
        node: SyntaxReader,
        target: Option<Self::ExpressionInput>,
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

        let (left_memory, left_index, _) = self.handle_operand_emission(
            &mut comparison_emission,
            left_emission,
            node.kind(),
            &left_expression,
        )?;
        let (right_memory, right_index, _) = self.handle_operand_emission(
            &mut comparison_emission,
            right_emission,
            node.kind(),
            &right_expression,
        )?;

        let target = if let Some(target) = target {
            target
        } else {
            let type_id = *self.resolver.get_type_binding(&node.id)?;

            self.allocate_registers(type_id, true, &node)?
        };
        let register = target.expect_single()?;
        let comparison_instruction = match node.kind() {
            SyntaxKind::EqualExpression => Instruction::equal(
                true,
                register.operand_type,
                left_memory,
                left_index,
                right_memory,
                right_index,
            ),
            SyntaxKind::NotEqualExpression => Instruction::equal(
                false,
                register.operand_type,
                left_memory,
                left_index,
                right_memory,
                right_index,
            ),
            SyntaxKind::LessThanExpression => Instruction::less(
                true,
                register.operand_type,
                left_memory,
                left_index,
                right_memory,
                right_index,
            ),
            SyntaxKind::GreaterThanExpression => Instruction::less(
                false,
                register.operand_type,
                left_memory,
                left_index,
                right_memory,
                right_index,
            ),
            SyntaxKind::LessThanOrEqualExpression => Instruction::less(
                true,
                register.operand_type,
                left_memory,
                left_index,
                right_memory,
                right_index,
            ),
            SyntaxKind::GreaterThanOrEqualExpression => Instruction::less(
                false,
                register.operand_type,
                left_memory,
                left_index,
                right_memory,
                right_index,
            ),
            _ => unreachable!("Expected comparison expression, found {}", node.kind()),
        };
        let load_false_instruction = Instruction::move_with_jump(
            register.index,
            OperandType::BOOLEAN,
            MemoryKind::CONSTANT,
            false as u16,
            1,
            true,
        );
        let load_true_instruction = Instruction::r#move(
            register.index,
            OperandType::BOOLEAN,
            MemoryKind::CONSTANT,
            true as u16,
        );

        comparison_emission.push(comparison_instruction);
        comparison_emission.push(load_false_instruction);
        comparison_emission.push(load_true_instruction);
        comparison_emission.set_target(Some(target));

        Ok(Emission::Instructions(comparison_emission))
    }

    fn visit_logic_expression(
        &mut self,
        node: SyntaxReader<'_>,
        target: Option<Self::ExpressionInput>,
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

        let mut logic_instructions = InstructionsEmission::new();

        let (left_memory, left_index, _) = self.handle_operand_emission(
            &mut logic_instructions,
            left_emission,
            node.kind(),
            &left_expression,
        )?;
        let (right_memory, right_index, _) = self.handle_operand_emission(
            &mut logic_instructions,
            right_emission,
            node.kind(),
            &right_expression,
        )?;

        let target = if let Some(target) = target {
            target.clone()
        } else {
            let type_id = *self.resolver.get_type_binding(&node.id)?;

            self.allocate_registers(type_id, true, &node)?
        };
        let register = target.expect_single()?;

        let test_instruction = match node.kind() {
            SyntaxKind::AndExpression => Instruction::test(false, left_memory, left_index, 1),
            SyntaxKind::OrExpression => Instruction::test(true, left_memory, left_index, 1),
            _ => unreachable!("Expected logical expression, found {}", node.kind()),
        };
        let right_move_instruction = Instruction::move_with_jump(
            register.index,
            OperandType::BOOLEAN,
            right_memory,
            right_index,
            1,
            true,
        );
        let left_move_instruction = Instruction::r#move(
            register.index,
            OperandType::BOOLEAN,
            left_memory,
            left_index,
        );

        logic_instructions.push(test_instruction);
        logic_instructions.push(right_move_instruction);
        logic_instructions.push(left_move_instruction);
        logic_instructions.set_target(Some(target));

        Ok(Emission::Instructions(logic_instructions))
    }

    fn visit_negation_expression(
        &mut self,
        node: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting unary negation expression");

        let expression = node.child()?;

        let expression_emission = self.visit_expression(expression, None)?;

        if let Emission::Constant(constant) = expression_emission {
            let negated = constant.negate().ok_or_else(|| {
                ErrorKind::Compile(CompileError::CannotApplyOperator {
                    operator: node.kind(),
                    type_id: constant.type_id(),
                    operand_position: expression.position(),
                })
            })?;

            return Ok(Emission::Constant(negated));
        }

        let mut negation_emission = InstructionsEmission::new();

        let (operand_memory, operand_index, _) = self.handle_operand_emission(
            &mut negation_emission,
            expression_emission,
            node.kind(),
            &expression,
        )?;
        let target = if let Some(target) = input {
            target.clone()
        } else {
            let type_id = *self.resolver.get_type_binding(&node.id)?;

            self.allocate_registers(type_id, true, &node)?
        };
        let register = target.expect_single()?;

        let negate_instruction = Instruction::negate(
            register.index,
            register.operand_type,
            operand_memory,
            operand_index,
        );

        negation_emission.push(negate_instruction);
        negation_emission.set_target(Some(target));

        Ok(Emission::Instructions(negation_emission))
    }

    fn visit_while_expression(
        &mut self,
        node: SyntaxReader<'_>,
        _: Option<Self::ExpressionInput>,
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
        _target: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting function expression");

        let (signature, body) = node.binary_children()?;
        let mut singature_children = signature.children()?;
        let parameters = singature_children.expect_next()?;
        let function_scope_id = *self.resolver.get_scope_binding(&body.id)?;
        let prototype_id = self.prototypes.reserve_slot();

        let function_emitter = Emitter::new(
            body,
            None,
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

        Ok(Emission::Place(Place::Constant {
            index: prototype_id.inner(),
            operand_type: OperandType::FUNCTION,
        }))
    }

    fn visit_call_expression(
        &mut self,
        node: SyntaxReader<'_>,
        target: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        debug!("Visting call expression");

        let (callee, argument_list) = node.binary_children()?;
        let arguments = argument_list.children()?;

        let return_type_id = *self.resolver.get_type_binding(&node.id)?;
        let target = if target.is_some() {
            target
        } else if return_type_id != TypeId::UNIT {
            Some(self.allocate_registers(return_type_id, true, &node)?)
        } else {
            None
        };
        let destination = target
            .as_ref()
            .map(|target| target.index())
            .unwrap_or(u16::MAX);

        let mut call_instructions = InstructionsEmission::new();

        let callee_emission = self.visit_expression(callee, None)?;

        let arguments_start = self.register_tracker.next_temporary;
        let mut argument_count = 0u16;

        for argument in arguments {
            let argument_emission = self.visit_expression(argument, None)?;
            let argument_place = self.handle_member_emission(argument_emission, &argument)?;

            match argument_place {
                Place::Constant {
                    operand_type,
                    index,
                } => {
                    let destination = self
                        .register_tracker
                        .allocate_next_reserved(RegisterWidth::from(operand_type));
                    let move_instruction =
                        Instruction::r#move(destination, operand_type, MemoryKind::CONSTANT, index);

                    call_instructions.push(move_instruction);

                    argument_count += 1;
                }
                Place::Register(RegisterAllocation::Single { register, .. }) => {
                    let destination = self
                        .register_tracker
                        .allocate_next_reserved(RegisterWidth::from(register.operand_type));
                    let move_instruction = Instruction::r#move(
                        destination,
                        register.operand_type,
                        MemoryKind::REGISTER,
                        register.index,
                    );

                    call_instructions.push(move_instruction);

                    argument_count += 1;
                }
                Place::Register(RegisterAllocation::Multiple { registers, .. }) => {
                    for register in registers {
                        let destination = self
                            .register_tracker
                            .allocate_next_reserved(RegisterWidth::from(register.operand_type));
                        let move_instruction = Instruction::r#move(
                            destination,
                            register.operand_type,
                            MemoryKind::REGISTER,
                            register.index,
                        );

                        call_instructions.push(move_instruction);

                        argument_count += 1;
                    }
                }
            }
        }

        let callee_place = match callee_emission {
            Emission::Place(place) => place,
            Emission::Instructions(instructions) => {
                let Some(registers) = instructions.target else {
                    return Err(ErrorKind::Compile(CompileError::ExpectedValue {
                        node_kind: node.kind(),
                        position: node.position(),
                    }));
                };

                Place::Register(registers)
            }
            Emission::NativeFunction(native_function) => {
                let call_native_instruction = Instruction::call_native(
                    destination,
                    native_function,
                    arguments_start,
                    argument_count,
                );

                call_instructions.push(call_native_instruction);

                return Ok(Emission::Instructions(call_instructions));
            }
            _ => {
                return Err(ErrorKind::Compile(CompileError::ExpectedValue {
                    node_kind: node.kind(),
                    position: node.position(),
                }));
            }
        };
        let (callee_memory, callee_index) = match callee_place {
            Place::Constant { index, .. } => (MemoryKind::CONSTANT, index),
            Place::Register(RegisterAllocation::Single { register, .. }) => {
                (MemoryKind::REGISTER, register.index)
            }
            Place::Register(RegisterAllocation::Multiple { .. }) => {
                return Err(ErrorKind::Compile(CompileError::ExpectedFunction {
                    node_kind: callee.kind(),
                    position: callee.position(),
                }));
            }
        };

        let call_instruction = Instruction::call(
            destination,
            callee_memory,
            callee_index,
            arguments_start,
            argument_count,
        );

        call_instructions.push(call_instruction);
        call_instructions.set_target(target);

        Ok(Emission::Instructions(call_instructions))
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
    fn target(&self) -> Option<&RegisterAllocation> {
        match self {
            Emission::Instructions(emission) => emission.target.as_ref(),
            _ => None,
        }
    }

    fn take_target(&mut self) -> Option<RegisterAllocation> {
        match self {
            Emission::Instructions(emission) => emission.take_target(),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct InstructionsEmission {
    instructions: Vec<(Instruction, Vec<JumpAnchor>)>,
    target: Option<RegisterAllocation>,
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

    fn set_target(&mut self, target: Option<RegisterAllocation>) {
        self.target = target;
    }

    fn take_target(&mut self) -> Option<RegisterAllocation> {
        self.target.take()
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

#[derive(Clone, Debug)]
pub enum Place {
    Constant {
        operand_type: OperandType,
        index: u16,
    },
    Register(RegisterAllocation),
}

impl Place {
    fn expect_register(self, node: &SyntaxReader) -> Result<RegisterAllocation, ErrorKind> {
        match self {
            Place::Register(target) => Ok(target),
            _ => Err(ErrorKind::Compile(CompileError::CannotMutate {
                position: node.position(),
            })),
        }
    }
}

#[derive(Clone, Debug)]
pub enum RegisterAllocation {
    Single {
        register: Register,
        temporary: bool,
    },
    Multiple {
        registers: SmallVec<[Register; 8]>,
        temporary: bool,
    },
}

impl RegisterAllocation {
    fn temporary(&self) -> bool {
        match self {
            RegisterAllocation::Single { temporary, .. }
            | RegisterAllocation::Multiple { temporary, .. } => *temporary,
        }
    }

    fn index(&self) -> u16 {
        match self {
            RegisterAllocation::Single { register, .. } => register.index,
            RegisterAllocation::Multiple { registers, .. } => registers[0].index,
        }
    }

    fn len(&self) -> usize {
        match self {
            RegisterAllocation::Single { .. } => 1,
            RegisterAllocation::Multiple { registers, .. } => registers.len(),
        }
    }

    fn iter(&self) -> RegisterSpanIterator<'_> {
        RegisterSpanIterator {
            span: self,
            index: 0,
        }
    }

    fn expect_single(&self) -> Result<Register, ErrorKind> {
        match self {
            RegisterAllocation::Single { register, .. } => Ok(*register),
            _ => Err(ErrorKind::Internal(InternalError::InvalidRegisterCount {
                expected: 1,
                found: self.len(),
            })),
        }
    }

    fn expect_multiple(
        &self,
        expected: usize,
    ) -> Result<(&SmallVec<[Register; 8]>, bool), ErrorKind> {
        match self {
            RegisterAllocation::Multiple {
                registers,
                temporary,
            } if registers.len() == expected => Ok((registers, *temporary)),
            _ => Err(ErrorKind::Internal(InternalError::InvalidRegisterCount {
                expected: 2,
                found: self.len(),
            })),
        }
    }
}

struct RegisterSpanIterator<'a> {
    span: &'a RegisterAllocation,
    index: usize,
}

impl<'a> Iterator for RegisterSpanIterator<'a> {
    type Item = &'a Register;

    fn next(&mut self) -> Option<Self::Item> {
        match self.span {
            RegisterAllocation::Single { register, .. } => {
                if self.index == 0 {
                    self.index += 1;

                    Some(register)
                } else {
                    None
                }
            }
            RegisterAllocation::Multiple { registers, .. } => {
                let allocation = registers.get(self.index)?;
                self.index += 1;

                Some(allocation)
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Register {
    operand_type: OperandType,
    index: u16,
}

#[derive(Clone, Copy, Debug)]
pub enum ConstantEmission {
    Boolean(bool),
    Character(char),
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

    fn type_id(&self) -> TypeId {
        match self {
            ConstantEmission::Boolean(_) => TypeId::BOOLEAN,
            ConstantEmission::Character(_) => TypeId::CHARACTER,
            ConstantEmission::U8(_) => TypeId::U_8,
            ConstantEmission::I8(_) => TypeId::I_8,
            ConstantEmission::U16(_) => TypeId::U_16,
            ConstantEmission::I16(_) => TypeId::I_16,
            ConstantEmission::U32(_) => TypeId::U_32,
            ConstantEmission::I32(_) => TypeId::I_32,
            ConstantEmission::U64(_) => TypeId::U_64,
            ConstantEmission::I64(_) => TypeId::I_64,
            ConstantEmission::U128(_) => TypeId::U_128,
            ConstantEmission::I128(_) => TypeId::I_128,
            ConstantEmission::F32(_) => TypeId::F_32,
            ConstantEmission::F64(_) => TypeId::F_64,
        }
    }

    fn add(self, other: Self) -> Result<Option<Self>, ErrorKind> {
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

    fn equal(self, other: Self) -> Option<Self> {
        match (self, other) {
            (ConstantEmission::Boolean(left), ConstantEmission::Boolean(right)) => {
                Some(ConstantEmission::Boolean(left == right))
            }
            (ConstantEmission::Character(left), ConstantEmission::Character(right)) => {
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

    fn not_equal(self, other: Self) -> Option<Self> {
        self.equal(other).map(|equality| match equality {
            ConstantEmission::Boolean(value) => ConstantEmission::Boolean(!value),
            _ => unreachable!("Expected boolean constant from equality comparison"),
        })
    }

    fn less(self, other: Self) -> Option<Self> {
        match (self, other) {
            (ConstantEmission::Character(left), ConstantEmission::Character(right)) => {
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

    fn greater(self, other: Self) -> Option<Self> {
        self.less_equal(other).map(|less| match less {
            ConstantEmission::Boolean(value) => ConstantEmission::Boolean(!value),
            _ => unreachable!("Expected boolean constant from less comparison"),
        })
    }

    fn less_equal(self, other: Self) -> Option<Self> {
        match (self, other) {
            (ConstantEmission::Character(left), ConstantEmission::Character(right)) => {
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

    fn greater_equal(self, other: Self) -> Option<Self> {
        self.less(other).map(|less| match less {
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
    reserved: u16,

    next_local: u16,
    next_temporary: u16,
    next_reserved: u16,

    max: u16,
}

impl RegisterTracker {
    fn new(argument_count: u16, return_count: u16) -> Self {
        let reserved = argument_count.max(return_count);

        Self {
            reserved,
            next_local: reserved,
            next_temporary: reserved,
            next_reserved: 0,
            max: reserved,
        }
    }

    fn allocate_next_local(&mut self, width: RegisterWidth) -> u16 {
        let next = self.next_local;
        self.next_local += u16::from(width);
        self.next_temporary = self.next_temporary.max(self.next_local);
        self.max = self.max.max(self.next_local);

        next
    }

    fn allocate_next_temporary(&mut self, width: RegisterWidth) -> u16 {
        let next = self.next_temporary;
        self.next_temporary += u16::from(width);
        self.max = self.max.max(self.next_temporary);

        next
    }

    fn allocate_next_reserved(&mut self, width: RegisterWidth) -> u16 {
        let next = self.next_reserved.min(self.reserved);
        self.next_reserved += u16::from(width);

        debug_assert!(self.next_reserved <= self.reserved);

        next
    }

    fn free_temporary(&mut self, registers: &RegisterAllocation) {
        debug_assert!(registers.temporary());

        for register in registers.iter() {
            debug_assert!(register.index < self.next_temporary);

            self.next_temporary = self.next_temporary.min(register.index);
        }
    }
}

enum RegisterWidth {
    Single,
    Double,
    Quad,
}

impl From<OperandType> for RegisterWidth {
    fn from(operand_type: OperandType) -> Self {
        match operand_type {
            OperandType::U_64 | OperandType::I_64 | OperandType::F_64 => RegisterWidth::Double,
            OperandType::U_128 | OperandType::I_128 => RegisterWidth::Quad,
            _ => RegisterWidth::Single,
        }
    }
}

impl From<RegisterWidth> for u16 {
    fn from(width: RegisterWidth) -> Self {
        match width {
            RegisterWidth::Single => 1,
            RegisterWidth::Double => 2,
            RegisterWidth::Quad => 4,
        }
    }
}
