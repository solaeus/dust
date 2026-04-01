use std::collections::HashMap;

use lexical_core::{
    ParseFloatOptions, ParseIntegerOptions, format::RUST_LITERAL, parse_with_options,
};
use rustc_hash::FxBuildHasher;
use smallvec::SmallVec;
use tracing::{debug, trace};

use crate::{
    compiler::{CompilationRequest, error::CompileError},
    constant_list::ConstantListBuilder,
    instruction::{Drop, Instruction, Jump, MemoryKind, Move, OperandType, Operation, Test},
    native_function::NativeFunction,
    prototype::{Prototype, PrototypeId, PrototypeList},
    resolver::{
        Resolver,
        declarations::{DeclarationId, Definition},
        error::ResolverError,
        scopes::ScopeId,
        symbols::SymbolId,
        types::{FloatType, SignedIntegerType, Type, TypeId, TypeMembers, UnsignedIntegerType},
    },
    source::{Position, Source, Span},
    syntax::{
        components::{
            ArrayExpression, ArrayRepeatExpression, AssignmentExpression, CallExpression,
            ComparisonExpression, ExpressionStatement, FunctionItem, FunctionParameters,
            IndexExpression, LogicExpression, MathExpression, NegationExpression, NotExpression,
            RangeExpression,
        },
        node::SyntaxKind,
        reader::SyntaxReader,
        visitor::SyntaxVisitor,
    },
};

#[derive(Debug)]
pub struct Emitter<'a> {
    source: &'a Source<'a>,

    constants: &'a mut ConstantListBuilder,

    resolver: &'a mut Resolver,

    prototypes: &'a mut PrototypeList,

    compilation_stack: &'a mut Vec<CompilationRequest>,

    argument_count: u16,

    return_types: Vec<OperandType>,

    /// Emitted bytecode instructions, filled during compilation.
    instructions: Vec<Instruction>,

    /// Local variables declared in the function.
    locals: HashMap<DeclarationId, Place, FxBuildHasher>,

    /// Concatenated list of register indices that are referenced by DROP and JUMP instructions.
    drop_lists: Vec<u16>,

    /// Stack of register index lists that need to be dropped when exiting scopes.
    pending_drops: Vec<SmallVec<[u16; 8]>>,

    register_tracker: RegisterTracker,

    jump_placements: HashMap<JumpId, JumpPlacement, FxBuildHasher>,

    jump_over_branch_ids: Vec<JumpId>,

    current_scope_id: ScopeId,

    next_jump_id: JumpId,

    debug_symbol_id: Option<SymbolId>,

    debug_position: Position,
}

impl<'a> Emitter<'a> {
    pub fn new(
        declaration_id: Option<DeclarationId>,
        prototype_id: PrototypeId,
        argument_count: u16,
        return_types: Vec<OperandType>,
        starting_scope_id: ScopeId,
        debug_info: (Option<SymbolId>, Position),
        (source, constants, resolver, prototypes, compilation_stack): (
            &'a Source,
            &'a mut ConstantListBuilder,
            &'a mut Resolver,
            &'a mut PrototypeList,
            &'a mut Vec<CompilationRequest>,
        ),
    ) -> Result<Self, CompileError> {
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

        let return_register_count = return_types
            .iter()
            .map(|r#type| RegisterWidth::from(*r#type).as_u16())
            .sum::<u16>();

        Ok(Self {
            source,
            constants,
            resolver,
            prototypes,
            compilation_stack,
            instructions: Vec::new(),
            locals,
            drop_lists: Vec::new(),
            pending_drops: Vec::new(),
            argument_count,
            register_tracker: RegisterTracker::new(argument_count, return_register_count),
            return_types,
            jump_placements: HashMap::default(),
            jump_over_branch_ids: Vec::new(),
            current_scope_id: starting_scope_id,
            next_jump_id: JumpId(0),
            debug_symbol_id: debug_info.0,
            debug_position: debug_info.1,
        })
    }

    pub fn bind_parameters(&mut self, parameters: SyntaxReader<'_>) -> Result<(), CompileError> {
        let FunctionParameters {
            type_parameters: _,
            value_parameters,
        } = parameters.as_component()?;

        let mut next_register_index: u16 = 0;

        for [parameter_name, _parameter_type] in value_parameters.children().array_chunks() {
            let declaration_id = *self.resolver.get_declaration_binding(&parameter_name.id)?;
            let declaration = self.resolver.declarations.get_declaration(declaration_id)?;
            let Definition::Local { type_id, .. } = declaration.definition else {
                return Err(CompileError::ExpectedLocalDefinition);
            };
            let concrete_type_id = self.resolver.resolve_type(type_id)?;
            let operand_types = self.resolver.get_operand_types(concrete_type_id)?;

            let mut registers = SmallVec::new();

            for operand_type in operand_types {
                let width = RegisterWidth::from(operand_type).as_u16();
                let register = Register {
                    operand_type,
                    index: next_register_index,
                };

                registers.push(register);
                next_register_index += width;
            }

            if registers.is_empty() {
                continue;
            }

            let allocation = if registers.len() == 1 {
                RegisterAllocation::Single {
                    register: registers[0],
                    temporary: false,
                }
            } else {
                RegisterAllocation::Multiple {
                    registers,
                    temporary: false,
                }
            };

            self.locals
                .insert(declaration_id, Place::Register(allocation));
        }

        debug_assert_eq!(next_register_index, self.argument_count);

        Ok(())
    }

    pub fn finish(mut self) -> Result<Prototype, CompileError> {
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
                        "Invalid jump anchor instruction: {}",
                        instruction.operation()
                    );
                }
            }
        }

        Ok(Prototype {
            instructions: self.instructions,
            return_types: self.return_types,
            register_count: self.register_tracker.max,
            argument_count: self.argument_count,
            debug_symbol_id: self.debug_symbol_id,
            debug_position: self.debug_position,
        })
    }

    fn emit_instruction(&mut self, instruction: Instruction) {
        trace!("Emitting {} instruction", instruction.operation());

        self.instructions.push(instruction);
    }

    fn create_jump_id(&mut self) -> JumpId {
        let next = self.next_jump_id;

        self.next_jump_id.0 += 1;

        next
    }

    fn allocate_registers(
        &mut self,
        type_id: TypeId,
        temporary: bool,
        reader: &SyntaxReader,
    ) -> Result<RegisterAllocation, CompileError> {
        fn collect_registers(
            type_id: TypeId,
            temporary: bool,
            registers: &mut SmallVec<[Register; 8]>,
            emitter: &mut Emitter,
            reader: &SyntaxReader,
        ) -> Result<(), CompileError> {
            let type_node = emitter.resolver.types.get_type(type_id)?;

            let (operand_type, width) = match type_node {
                Type::Boolean => (OperandType::BOOLEAN, RegisterWidth::Single),
                Type::SignedInteger(SignedIntegerType::I8) => {
                    (OperandType::I_8, RegisterWidth::Single)
                }
                Type::SignedInteger(SignedIntegerType::I16) => {
                    (OperandType::I_16, RegisterWidth::Single)
                }
                Type::SignedInteger(SignedIntegerType::I32) => {
                    (OperandType::I_32, RegisterWidth::Single)
                }
                Type::SignedInteger(SignedIntegerType::I64) => {
                    (OperandType::I_64, RegisterWidth::Single)
                }
                Type::SignedInteger(SignedIntegerType::I128) => {
                    (OperandType::I_128, RegisterWidth::Double)
                }
                Type::UnsignedInteger(UnsignedIntegerType::U8) => {
                    (OperandType::U_8, RegisterWidth::Single)
                }
                Type::UnsignedInteger(UnsignedIntegerType::U16) => {
                    (OperandType::U_16, RegisterWidth::Single)
                }
                Type::UnsignedInteger(UnsignedIntegerType::U32) => {
                    (OperandType::U_32, RegisterWidth::Single)
                }
                Type::UnsignedInteger(UnsignedIntegerType::U64) => {
                    (OperandType::U_64, RegisterWidth::Single)
                }
                Type::UnsignedInteger(UnsignedIntegerType::U128) => {
                    (OperandType::U_128, RegisterWidth::Double)
                }
                Type::Float(FloatType::F32) => (OperandType::F_32, RegisterWidth::Single),
                Type::Float(FloatType::F64) => (OperandType::F_64, RegisterWidth::Double),
                Type::Never => todo!(),
                Type::Character => (OperandType::CHARACTER, RegisterWidth::Single),
                Type::FunctionDefinition { .. } => (OperandType::FUNCTION, RegisterWidth::Single),
                Type::Inferred { resolved, .. } => {
                    if let Some(resolved) = resolved {
                        collect_registers(*resolved, temporary, registers, emitter, reader)?;

                        return Ok(());
                    } else {
                        return Err(CompileError::CannotInferType {
                            type_id,
                            position: reader.position(),
                        });
                    }
                }
                Type::Algebraic { .. } | Type::Array { .. } => {
                    let operand_types = emitter.resolver.get_operand_types(type_id)?;

                    for operand_type in operand_types {
                        let width = RegisterWidth::from(operand_type);
                        let next_register_index = if temporary {
                            emitter.register_tracker.allocate_next_temporary(width)
                        } else {
                            emitter.register_tracker.allocate_next_local(width)
                        };

                        registers.push(Register {
                            operand_type,
                            index: next_register_index,
                        });
                    }

                    return Ok(());
                }
                _ => todo!(),
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

        collect_registers(type_id, temporary, &mut allocations, self, reader)?;

        match allocations.len() {
            0 => Err(CompileError::ExpectedValue {
                node_kind: reader.node.kind,
                position: reader.position(),
            }),
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
            registers.is_temporary(),
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
        if let Some(drops) = self.pending_drops.last_mut() {
            drops.push(register)
        }
    }

    fn handle_drops(&mut self, instructions: &mut InstructionsEmission) {
        let start = self.drop_lists.len() as u16;
        let Some(pending_drops_for_scope) = self.pending_drops.last_mut() else {
            return;
        };

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
        left: &SyntaxReader,
        right_constant: ConstantEmission,
        right: &SyntaxReader,
    ) -> Result<ConstantEmission, CompileError> {
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
                Err(CompileError::DivisionByZero {
                    position: Position::new(
                        left.file_id(),
                        Span::join(&left.node.span, &right.node.span),
                    ),
                })
            } else {
                Ok(())
            }
        };
        let create_error = || {
            let left_type = left_constant.operand_type();
            let right_type = right_constant.operand_type();

            CompileError::CannotApplyBinaryOperator {
                operator: operator.node.kind,
                operand_position: operator.position(),
                left_type,
                left_position: left.position(),
                right_type,
                right_position: right.position(),
            }
        };

        match operator.node.kind {
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
            _ => unreachable!("Invalid binary operator: {:?}", operator.node.kind),
        }
    }

    fn handle_top_emission(
        &mut self,
        emission: Emission,
        node: SyntaxReader,
    ) -> Result<(), CompileError> {
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
                                let placement = self
                                    .jump_placements
                                    .get_mut(&id)
                                    .ok_or(CompileError::ExpectedJumpPlacement(id))?;
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
                                let forward_index = self
                                    .jump_placements
                                    .get(&forward_id)
                                    .ok_or(CompileError::ExpectedJumpPlacement(forward_id))?
                                    .index;
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

                                let forward_placement =
                                    self.jump_placements
                                        .get_mut(&forward_id)
                                        .ok_or(CompileError::ExpectedJumpPlacement(forward_id))?;

                                forward_placement.distance = forward_distance;

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
        operator: SyntaxKind,
        operand: &SyntaxReader,
    ) -> Result<(MemoryKind, u16, OperandType), CompileError> {
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
                    && registers.is_temporary()
                {
                    self.free_temporary_registers(registers);
                }

                instructions.merge(operand_instructions);

                let (index, operand_type) = match &instructions.target {
                    Some(RegisterAllocation::Single { register, .. }) => {
                        (register.index, register.operand_type)
                    }
                    Some(RegisterAllocation::Multiple { .. }) => {
                        let type_id = *self.resolver.get_type_binding(&operand.id)?;

                        return Err(CompileError::CannotApplyOperator {
                            operator,
                            type_id,
                            operand_position: operand.position(),
                        });
                    }
                    None => {
                        return Err(CompileError::ExpectedValue {
                            node_kind: operand.node.kind,
                            position: operand.position(),
                        });
                    }
                };

                Ok((MemoryKind::REGISTER, index, operand_type))
            }
            Emission::NativeFunction(_) => Err(CompileError::ExpectedNativeFunctionCall {
                position: operand.position(),
            }),
            Emission::Place(Place::Register(RegisterAllocation::Multiple { .. })) => {
                let type_id = *self.resolver.get_type_binding(&operand.id)?;

                Err(CompileError::CannotApplyOperator {
                    operator,
                    type_id,
                    operand_position: operand.position(),
                })
            }
            Emission::None => Err(CompileError::ExpectedValue {
                node_kind: operand.node.kind,
                position: operand.position(),
            }),
        }
    }

    fn handle_member_emission(
        &mut self,
        emission: Emission,
        member: &SyntaxReader,
    ) -> Result<Place, CompileError> {
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
                    return Err(CompileError::ExpectedValue {
                        node_kind: member.node.kind,
                        position: member.position(),
                    });
                };

                Ok(Place::Register(registers))
            }
            Emission::NativeFunction(_) => Err(CompileError::ExpectedNativeFunctionCall {
                position: member.position(),
            }),
            _ => Err(CompileError::ExpectedValue {
                node_kind: member.node.kind,
                position: member.position(),
            }),
        }
    }

    fn handle_condition_emission(
        &mut self,
        instructions: &mut InstructionsEmission,
        emission: Emission,
        condition: &SyntaxReader,
    ) -> Result<(), CompileError> {
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
                                && registers.is_temporary()
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
                                && registers.is_temporary()
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
                                    return Err(CompileError::ExpectedBooleanExpression {
                                        found: *self.resolver.get_type_binding(&condition.id)?,
                                        node_kind: condition.node.kind,
                                        position: condition.position(),
                                    });
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
                return Err(CompileError::ExpectedBooleanExpression {
                    found: *self.resolver.get_type_binding(&condition.id)?,
                    node_kind: condition.node.kind,
                    position: condition.position(),
                });
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
    ) -> Result<(), CompileError> {
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
                let return_instruction = Instruction::r#return();

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
                let return_instruction = Instruction::r#return();

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
                let return_instruction = Instruction::r#return();

                return_instructions.push(move_instruction);
                return_instructions.push(return_instruction);

                Ok(())
            }
            Emission::Place(Place::Register(RegisterAllocation::Multiple {
                registers, ..
            })) => {
                let return_instruction = Instruction::r#return();

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
                if let Some(registers) = instructions.target.take() {
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

                    let return_instruction = Instruction::r#return();

                    return_instructions.merge(instructions);
                    return_instructions.push(return_instruction);

                    Ok(())
                } else {
                    let return_instruction = Instruction::r#return();

                    return_instructions.merge(instructions);
                    return_instructions.push(return_instruction);

                    Ok(())
                }
            }
            Emission::None => {
                let return_instruction = Instruction::r#return();

                return_instructions.push(return_instruction);

                Ok(())
            }
            Emission::NativeFunction(_) => Err(CompileError::ExpectedNativeFunctionCall {
                position: node.position(),
            }),
        }
    }

    pub fn emit_function_body(&mut self, body: SyntaxReader) -> Result<(), CompileError> {
        let children = body.children();
        let child_count = children.len();

        if child_count == 0 {
            self.emit_instruction(Instruction::r#return());
            return Ok(());
        }

        for (index, child) in children.enumerate() {
            let is_last = index == child_count - 1;

            if is_last {
                let return_target = self.allocate_return_target();
                let emission = self.handle_implicit_return(child, return_target)?;
                self.handle_top_emission(emission, child)?;
                return Ok(());
            }

            if child.is_statement() {
                if let Some(instructions) = self.visit_statement(child)? {
                    self.handle_top_emission(Emission::Instructions(instructions), child)?;
                }
            } else {
                let emission = self.visit_expression(child, None)?;
                if let Emission::Instructions(instructions) = emission {
                    self.handle_top_emission(Emission::Instructions(instructions), child)?;
                }
            }
        }

        Ok(())
    }

    fn allocate_return_target(&mut self) -> Option<RegisterAllocation> {
        if self.return_types.is_empty() {
            return None;
        }

        if self.return_types.len() == 1 {
            let operand_type = self.return_types[0];
            let index = self
                .register_tracker
                .allocate_next_reserved(RegisterWidth::from(operand_type));

            return Some(RegisterAllocation::Single {
                register: Register {
                    operand_type,
                    index,
                },
                temporary: false,
            });
        }

        let mut registers = SmallVec::new();

        for &operand_type in &self.return_types {
            let index = self
                .register_tracker
                .allocate_next_reserved(RegisterWidth::from(operand_type));

            registers.push(Register {
                operand_type,
                index,
            });
        }

        Some(RegisterAllocation::Multiple {
            registers,
            temporary: false,
        })
    }

    fn handle_implicit_return(
        &mut self,
        node: SyntaxReader,
        target: Option<RegisterAllocation>,
    ) -> Result<Emission, CompileError> {
        let mut return_emission = InstructionsEmission::new();

        if node.is_expression() {
            let return_target_first_index = target
                .as_ref()
                .and_then(|target| target.iter().next().copied());
            let mut expression_emission = self.visit_expression(node, target)?;

            if let Some(return_register) = return_target_first_index {
                self.register_tracker.reset_reserved();

                if let Emission::Instructions(ref mut instructions) = expression_emission
                    && instructions.target.as_ref().is_some_and(|target| {
                        target
                            .iter()
                            .any(|register| register.index == return_register.index)
                    })
                {
                    instructions.target.take();
                }
            }

            self.handle_return_emission(&mut return_emission, expression_emission, node)?;
        } else {
            if node.is_item() {
                self.visit_item(node)?;
            } else {
                self.visit_statement(node)?;
            }

            let return_instruction = Instruction::r#return();

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

    fn visit_root(&mut self, _: SyntaxReader) -> Result<Self::RootOutput, CompileError> {
        unreachable!("Emitter should never visit root nodes");
    }

    fn visit_module_item(&mut self, _: SyntaxReader<'_>) -> Result<(), CompileError> {
        todo!()
    }

    fn visit_function_item(&mut self, reader: SyntaxReader<'_>) -> Result<(), CompileError> {
        let FunctionItem { name, .. } = reader.as_component()?;

        let declaration_id = *self.resolver.get_declaration_binding(&name.id)?;
        let declaration = self.resolver.declarations.get_declaration(declaration_id)?;
        let Definition::Function {
            type_parameters, ..
        } = declaration.definition
        else {
            return Err(CompileError::ExpectedFunctionType {
                found: TypeId::UNIT,
                position: reader.position(),
            });
        };

        if !type_parameters.is_empty() {
            return Ok(());
        }

        let cache_key = (declaration_id, SmallVec::new());
        let prototype_id = if let Some(existing) = self.resolver.get_cached_prototype(&cache_key) {
            existing
        } else {
            let reserved = self.prototypes.reserve();

            self.resolver.cache_prototype(cache_key, reserved);
            self.compilation_stack.push(CompilationRequest {
                declaration_id,
                prototype_id: reserved,
            });

            reserved
        };

        self.locals.insert(
            declaration_id,
            Place::Constant {
                operand_type: OperandType::FUNCTION,
                index: prototype_id.inner(),
            },
        );

        Ok(())
    }

    fn visit_use_item(&mut self, _: SyntaxReader<'_>) -> Result<(), CompileError> {
        todo!()
    }

    fn visit_struct_item(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
        todo!()
    }

    fn visit_enum_item(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
        todo!()
    }

    fn visit_const_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        Err(CompileError::Unimplemented {
            syntax_kind: reader.node.kind,
            position: reader.position(),
        })
    }

    fn visit_type_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        Err(CompileError::Unimplemented {
            syntax_kind: reader.node.kind,
            position: reader.position(),
        })
    }

    fn visit_impl_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        Err(CompileError::Unimplemented {
            syntax_kind: reader.node.kind,
            position: reader.position(),
        })
    }

    fn visit_impl_trait_item(&mut self, reader: SyntaxReader) -> Result<(), CompileError> {
        Err(CompileError::Unimplemented {
            syntax_kind: reader.node.kind,
            position: reader.position(),
        })
    }

    fn visit_expression_statement(
        &mut self,
        reader: SyntaxReader<'_>,
    ) -> Result<Self::StatementOutput, CompileError> {
        let ExpressionStatement { expression } = reader.as_component()?;

        let expression_emission = self.visit_expression(expression, None)?;

        if let Emission::Instructions(mut instructions_emission) = expression_emission {
            instructions_emission.set_target(None);

            Ok(instructions_emission)
        } else {
            Ok(InstructionsEmission::new())
        }
    }

    fn visit_let_statement(
        &mut self,
        reader: SyntaxReader,
    ) -> Result<Self::StatementOutput, CompileError> {
        debug!("Visting let statement");

        let mut children = reader.children();
        let path = children.expect_next()?;
        let expression = children.expect_next()?;

        let type_id = *self.resolver.get_type_binding(&expression.id)?;

        let mut let_statement_instructions = InstructionsEmission::new();

        let target = self.allocate_registers(type_id, false, &expression)?;
        let expression_emission = self.visit_expression(expression, Some(target.clone()))?;

        let place = match expression_emission {
            Emission::Place(place @ Place::Register(..)) => place,
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

                Place::Register(target)
            }
            Emission::Place(Place::Constant {
                operand_type: r#type,
                index,
            }) => {
                let destination = target.index();
                let move_instruction =
                    Instruction::r#move(destination, r#type, MemoryKind::CONSTANT, index);

                let_statement_instructions.push(move_instruction);

                Place::Register(target)
            }
            Emission::Instructions(expression_instructions) => {
                let_statement_instructions.merge(expression_instructions);

                Place::Register(target)
            }
            Emission::NativeFunction(_) => {
                return Err(CompileError::ExpectedNativeFunctionCall {
                    position: reader.position(),
                });
            }
            Emission::None => {
                return Err(CompileError::ExpectedValue {
                    node_kind: expression.node.kind,
                    position: expression.position(),
                });
            }
        };

        let declaration_id = *self.resolver.get_declaration_binding(&path.id)?;

        self.locals.insert(declaration_id, place);
        let_statement_instructions.set_target(None);

        Ok(let_statement_instructions)
    }

    fn visit_assignment_expression(
        &mut self,
        reader: SyntaxReader<'_>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let AssignmentExpression { target, value } = reader.as_component()?;

        if target.node.kind == SyntaxKind::IndexExpression {
            let IndexExpression { list, index } = target.as_component()?;

            let list_emission = self.visit_expression(list, None)?;

            let (list_registers, list_instructions) = match list_emission {
                Emission::Place(Place::Register(registers)) => (registers, None),
                Emission::Instructions(instructions) => {
                    let registers =
                        instructions
                            .target
                            .clone()
                            .ok_or(CompileError::ExpectedValue {
                                node_kind: list.node.kind,
                                position: list.position(),
                            })?;

                    (registers, Some(instructions))
                }
                _ => {
                    return Err(CompileError::ExpectedValue {
                        node_kind: list.node.kind,
                        position: list.position(),
                    });
                }
            };

            let list_type_id = *self.resolver.get_type_binding(&list.id)?;
            let list_type = *self.resolver.types.get_type(list_type_id)?;

            let (element_type_id, array_length) = match list_type {
                Type::Array {
                    element_type_id,
                    length,
                } => (element_type_id, length),
                _ => {
                    return Err(CompileError::CannotIndex {
                        type_id: list_type_id,
                        position: list.position(),
                    });
                }
            };

            let element_operand_types = self.resolver.get_operand_types(element_type_id)?;
            let element_register_count = element_operand_types.len();

            if index.node.kind == SyntaxKind::IntegerExpression {
                let index_str = self.source.get_file_content(&index.position())?;
                let index_type_id = *self.resolver.get_type_binding(&index.id)?;
                let constant_index = parse_with_options::<usize, RUST_LITERAL>(
                    index_str.as_bytes(),
                    &ParseIntegerOptions::default(),
                )
                .map_err(|_| CompileError::ExpectedIntegerIndex {
                    found: index_type_id,
                    position: index.position(),
                })?;

                if constant_index >= array_length {
                    return Err(CompileError::IndexOutOfBounds {
                        index: constant_index,
                        length: array_length,
                        position: index.position(),
                    });
                }

                let register_offset = constant_index * element_register_count;

                let element_registers: SmallVec<[Register; 8]> = list_registers
                    .iter()
                    .skip(register_offset)
                    .take(element_register_count)
                    .copied()
                    .collect();

                let destination = match element_registers.len() {
                    1 => RegisterAllocation::Single {
                        register: element_registers[0],
                        temporary: false,
                    },
                    _ => RegisterAllocation::Multiple {
                        registers: element_registers,
                        temporary: false,
                    },
                };

                let value_emission = self.visit_expression(value, Some(destination.clone()))?;

                let mut assignment_instructions =
                    list_instructions.unwrap_or_else(InstructionsEmission::new);

                match value_emission {
                    Emission::Constant(constant) => {
                        for register in destination.iter() {
                            let operand_index = self.add_constant(constant);
                            let move_instruction = Instruction::r#move(
                                register.index,
                                register.operand_type,
                                MemoryKind::CONSTANT,
                                operand_index,
                            );

                            assignment_instructions.push(move_instruction);

                            if register.operand_type == OperandType::POINTER {
                                self.add_drop(register.index);
                            }
                        }
                    }
                    Emission::Place(Place::Constant {
                        operand_type,
                        index,
                    }) => {
                        let destination_register = destination.expect_single()?;
                        let move_instruction = Instruction::r#move(
                            destination_register.index,
                            operand_type,
                            MemoryKind::CONSTANT,
                            index,
                        );

                        assignment_instructions.push(move_instruction);
                    }
                    Emission::Place(Place::Register(RegisterAllocation::Single {
                        register,
                        ..
                    })) => {
                        let destination_register = destination.expect_single()?;
                        let move_instruction = Instruction::r#move(
                            destination_register.index,
                            register.operand_type,
                            MemoryKind::REGISTER,
                            register.index,
                        );

                        assignment_instructions.push(move_instruction);
                    }
                    Emission::Place(Place::Register(RegisterAllocation::Multiple {
                        registers: ref source_registers,
                        ..
                    })) => {
                        let (destination_registers, _) =
                            destination.expect_multiple(source_registers.len())?;

                        for (destination_register, source_register) in
                            destination_registers.into_iter().zip(source_registers)
                        {
                            let move_instruction = Instruction::r#move(
                                destination_register.index,
                                source_register.operand_type,
                                MemoryKind::REGISTER,
                                source_register.index,
                            );

                            assignment_instructions.push(move_instruction);
                        }
                    }
                    Emission::Instructions(instructions) => {
                        assignment_instructions.merge(instructions);
                    }
                    Emission::NativeFunction(_) => {
                        return Err(CompileError::ExpectedNativeFunctionCall {
                            position: reader.position(),
                        });
                    }
                    Emission::None => {
                        return Err(CompileError::ExpectedValue {
                            node_kind: value.node.kind,
                            position: value.position(),
                        });
                    }
                }

                assignment_instructions.set_target(None);

                return Ok(Emission::Instructions(assignment_instructions));
            }

            let index_emission = self.visit_expression(index, None)?;
            let index_place = self.handle_member_emission(index_emission, &index)?;

            let (index_memory, index_index) = match index_place {
                Place::Constant { index, .. } => (MemoryKind::CONSTANT, index),
                Place::Register(RegisterAllocation::Single { register, .. }) => {
                    (MemoryKind::REGISTER, register.index)
                }
                _ => {
                    return Err(CompileError::ExpectedIntegerIndex {
                        found: *self.resolver.get_type_binding(&index.id)?,
                        position: index.position(),
                    });
                }
            };

            let value_emission = self.visit_expression(value, None)?;
            let value_place = self.handle_member_emission(value_emission, &value)?;

            let (source_memory, source_index) = match &value_place {
                Place::Constant { index, .. } => (MemoryKind::CONSTANT, *index),
                Place::Register(allocation) => (MemoryKind::REGISTER, allocation.index()),
            };

            let base_index = list_registers.index();
            let element_operand_type = element_operand_types[0];

            let mut assignment_instructions =
                list_instructions.unwrap_or_else(InstructionsEmission::new);

            let check_index_instruction =
                Instruction::check_index(index_memory, index_index, array_length as u16);

            assignment_instructions.push(check_index_instruction);

            let set_index_instruction = Instruction::set_index(
                base_index,
                element_operand_type,
                index_memory,
                index_index,
                source_memory,
                source_index,
            );

            assignment_instructions.push(set_index_instruction);
            assignment_instructions.set_target(None);

            return Ok(Emission::Instructions(assignment_instructions));
        }

        let declaration_id = self.resolver.get_declaration_binding(&target.id)?;
        let local = self
            .locals
            .get(declaration_id)
            .ok_or_else(|| CompileError::DeclarationOutOfScope {
                declaration_id: *declaration_id,
                usage_position: target.position(),
            })?
            .clone();

        let mut assignment_instructions = InstructionsEmission::new();

        let destination_registers = local.expect_register(&target)?;
        let expression_emission = self.visit_expression(value, Some(destination_registers))?;

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

                    assignment_instructions.push(move_instruction);

                    if allocation.operand_type == OperandType::POINTER {
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

                assignment_instructions.push(move_instruction);
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

                assignment_instructions.push(move_instruction);
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

                    assignment_instructions.push(move_instruction);
                }
            }
            Emission::Instructions(instructions) => {
                assignment_instructions.merge(instructions);
                assignment_instructions.set_target(None);
            }
            Emission::NativeFunction(_) => {
                return Err(CompileError::ExpectedNativeFunctionCall {
                    position: reader.position(),
                });
            }
            Emission::None => {
                return Err(CompileError::ExpectedValue {
                    node_kind: value.node.kind,
                    position: value.position(),
                });
            }
        }

        Ok(Emission::Instructions(assignment_instructions))
    }

    fn visit_compound_assignment_expression(
        &mut self,
        reader: SyntaxReader,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting binary assignment statement");

        let emission = self.visit_math_expression(reader, None)?;
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
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        todo!()
    }

    fn visit_byte_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        todo!()
    }

    fn visit_character_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        todo!()
    }

    fn visit_float_expression(
        &mut self,
        reader: SyntaxReader,
        target: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting float expression");

        let float_str = self.source.get_file_content(&reader.position())?;
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
                return Err(CompileError::ExpectedFloatRegister);
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
        reader: SyntaxReader,
        target: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting integer expression");

        let integer_str = self.source.get_file_content(&reader.position())?;
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
            Some(RegisterAllocation::Multiple { registers, .. })
                if registers[0].operand_type == OperandType::U_64 =>
            {
                let integer = parse_with_options::<u64, RUST_LITERAL>(
                    integer_str.as_bytes(),
                    &ParseIntegerOptions::default(),
                )
                .unwrap_or_default();

                ConstantEmission::U64(integer)
            }
            Some(RegisterAllocation::Multiple { registers, .. })
                if registers[0].operand_type == OperandType::I_64 =>
            {
                let integer = parse_with_options::<i64, RUST_LITERAL>(
                    integer_str.as_bytes(),
                    &ParseIntegerOptions::default(),
                )
                .unwrap_or_default();

                ConstantEmission::I64(integer)
            }
            Some(RegisterAllocation::Multiple { registers, .. })
                if registers[0].operand_type == OperandType::U_128 =>
            {
                let integer = parse_with_options::<u128, RUST_LITERAL>(
                    integer_str.as_bytes(),
                    &ParseIntegerOptions::default(),
                )
                .unwrap_or_default();

                ConstantEmission::U128(integer)
            }
            Some(RegisterAllocation::Multiple { registers, .. })
                if registers[0].operand_type == OperandType::I_128 =>
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
                return Err(CompileError::ExpectedIntegerRegister);
            }
        };

        Ok(Emission::Constant(integer_constant))
    }

    fn visit_string_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        todo!()
    }

    fn visit_array_expression(
        &mut self,
        reader: SyntaxReader,
        target: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let ArrayExpression { elements } = reader.as_component()?;

        let target = if let Some(target) = target {
            target
        } else {
            let type_id = *self.resolver.get_type_binding(&reader.id)?;

            self.allocate_registers(type_id, true, &reader)?
        };

        let mut array_instructions = InstructionsEmission::new();

        for (element, destination) in elements.zip(target.iter()) {
            let element_target = RegisterAllocation::Single {
                register: *destination,
                temporary: false,
            };
            let element_emission = self.visit_expression(element, Some(element_target))?;

            match element_emission {
                Emission::Instructions(instructions) => {
                    array_instructions.merge(instructions);
                }
                Emission::Constant(constant) => {
                    let constant_index = self.add_constant(constant);
                    let move_instruction = Instruction::r#move(
                        destination.index,
                        constant.operand_type(),
                        MemoryKind::CONSTANT,
                        constant_index,
                    );

                    array_instructions.push(move_instruction);
                }
                Emission::Place(place) => match place {
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

                        array_instructions.push(move_instruction);
                    }
                    Place::Register(RegisterAllocation::Single { register, .. }) => {
                        let move_instruction = Instruction::r#move(
                            destination.index,
                            register.operand_type,
                            MemoryKind::REGISTER,
                            register.index,
                        );

                        array_instructions.push(move_instruction);
                    }
                    Place::Register(RegisterAllocation::Multiple { registers, .. }) => {
                        for register in registers {
                            let move_instruction = Instruction::r#move(
                                destination.index,
                                register.operand_type,
                                MemoryKind::REGISTER,
                                register.index,
                            );

                            array_instructions.push(move_instruction);
                        }
                    }
                },
                Emission::None | Emission::NativeFunction(_) => {}
            }
        }

        array_instructions.set_target(Some(target));

        Ok(Emission::Instructions(array_instructions))
    }

    fn visit_array_repeat_expression(
        &mut self,
        reader: SyntaxReader,
        target: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let ArrayRepeatExpression { element, .. } = reader.as_component()?;

        let target = if let Some(target) = target {
            target
        } else {
            let type_id = *self.resolver.get_type_binding(&reader.id)?;

            self.allocate_registers(type_id, true, &reader)?
        };

        let first_destination = target.iter().next();
        let element_target = first_destination.map(|destination| RegisterAllocation::Single {
            register: *destination,
            temporary: false,
        });
        let element_emission = self.visit_expression(element, element_target)?;
        let element_place = self.handle_member_emission(element_emission, &element)?;

        let mut array_instructions = InstructionsEmission::new();

        for destination in target.iter().skip(1) {
            match &element_place {
                Place::Constant {
                    operand_type,
                    index,
                } => {
                    let move_instruction = Instruction::r#move(
                        destination.index,
                        *operand_type,
                        MemoryKind::CONSTANT,
                        *index,
                    );

                    array_instructions.push(move_instruction);
                }
                Place::Register(RegisterAllocation::Single { register, .. }) => {
                    let move_instruction = Instruction::r#move(
                        destination.index,
                        register.operand_type,
                        MemoryKind::REGISTER,
                        register.index,
                    );

                    array_instructions.push(move_instruction);
                }
                Place::Register(RegisterAllocation::Multiple { registers, .. }) => {
                    for register in registers {
                        let move_instruction = Instruction::r#move(
                            destination.index,
                            register.operand_type,
                            MemoryKind::REGISTER,
                            register.index,
                        );

                        array_instructions.push(move_instruction);
                    }
                }
            }
        }

        array_instructions.set_target(Some(target));

        Ok(Emission::Instructions(array_instructions))
    }

    fn visit_index_expression(
        &mut self,
        reader: SyntaxReader,
        target: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let IndexExpression { list, index } = reader.as_component()?;

        let list_emission = self.visit_expression(list, None)?;

        let (list_registers, list_instructions) = match list_emission {
            Emission::Place(Place::Register(registers)) => (registers, None),
            Emission::Instructions(instructions) => {
                let registers = instructions
                    .target
                    .clone()
                    .ok_or(CompileError::ExpectedValue {
                        node_kind: list.node.kind,
                        position: list.position(),
                    })?;

                (registers, Some(instructions))
            }
            _ => {
                return Err(CompileError::ExpectedValue {
                    node_kind: list.node.kind,
                    position: list.position(),
                });
            }
        };

        let list_type_id = *self.resolver.get_type_binding(&list.id)?;
        let list_type = *self.resolver.types.get_type(list_type_id)?;

        let (element_type_id, array_length) = match list_type {
            Type::Array {
                element_type_id,
                length,
            } => (element_type_id, length),
            _ => {
                return Err(CompileError::CannotIndex {
                    type_id: list_type_id,
                    position: list.position(),
                });
            }
        };

        let element_operand_types = self.resolver.get_operand_types(element_type_id)?;
        let element_register_count = element_operand_types.len();

        if matches!(
            index.node.kind,
            SyntaxKind::RangeExpression | SyntaxKind::RangeInclusiveExpression
        ) {
            let RangeExpression {
                start: range_start,
                end: range_end,
            } = index.as_component()?;

            let range_start_str = self.source.get_file_content(&range_start.position())?;
            let range_end_str = self.source.get_file_content(&range_end.position())?;

            let range_start_type_id = *self.resolver.get_type_binding(&range_start.id)?;
            let range_end_type_id = *self.resolver.get_type_binding(&range_end.id)?;

            let start_index = parse_with_options::<usize, RUST_LITERAL>(
                range_start_str.as_bytes(),
                &ParseIntegerOptions::default(),
            )
            .map_err(|_| CompileError::ExpectedIntegerIndex {
                found: range_start_type_id,
                position: range_start.position(),
            })?;

            let end_index = parse_with_options::<usize, RUST_LITERAL>(
                range_end_str.as_bytes(),
                &ParseIntegerOptions::default(),
            )
            .map_err(|_| CompileError::ExpectedIntegerIndex {
                found: range_end_type_id,
                position: range_end.position(),
            })?;

            let slice_end = if index.node.kind == SyntaxKind::RangeInclusiveExpression {
                end_index + 1
            } else {
                end_index
            };

            if start_index > slice_end || slice_end > array_length {
                return Err(CompileError::IndexOutOfBounds {
                    index: slice_end,
                    length: array_length,
                    position: index.position(),
                });
            }

            let start_register_offset = start_index * element_register_count;
            let slice_register_count = (slice_end - start_index) * element_register_count;

            let slice_registers: SmallVec<[Register; 8]> = list_registers
                .iter()
                .skip(start_register_offset)
                .take(slice_register_count)
                .copied()
                .collect();

            let slice_allocation = match slice_registers.len() {
                1 => RegisterAllocation::Single {
                    register: slice_registers[0],
                    temporary: false,
                },
                _ => RegisterAllocation::Multiple {
                    registers: slice_registers,
                    temporary: false,
                },
            };

            if let Some(mut instructions) = list_instructions {
                instructions.set_target(Some(slice_allocation));

                return Ok(Emission::Instructions(instructions));
            }

            return Ok(Emission::Place(Place::Register(slice_allocation)));
        }

        if index.node.kind == SyntaxKind::IntegerExpression {
            let index_str = self.source.get_file_content(&index.position())?;
            let index_type_id = *self.resolver.get_type_binding(&index.id)?;
            let constant_index = parse_with_options::<usize, RUST_LITERAL>(
                index_str.as_bytes(),
                &ParseIntegerOptions::default(),
            )
            .map_err(|_| CompileError::ExpectedIntegerIndex {
                found: index_type_id,
                position: index.position(),
            })?;

            if constant_index >= array_length {
                return Err(CompileError::IndexOutOfBounds {
                    index: constant_index,
                    length: array_length,
                    position: index.position(),
                });
            }

            let register_offset = constant_index * element_register_count;

            let element_registers: SmallVec<[Register; 8]> = list_registers
                .iter()
                .skip(register_offset)
                .take(element_register_count)
                .copied()
                .collect();

            let element_allocation = match element_registers.len() {
                1 => RegisterAllocation::Single {
                    register: element_registers[0],
                    temporary: false,
                },
                _ => RegisterAllocation::Multiple {
                    registers: element_registers,
                    temporary: false,
                },
            };

            if let Some(mut instructions) = list_instructions {
                instructions.set_target(Some(element_allocation));

                return Ok(Emission::Instructions(instructions));
            }

            return Ok(Emission::Place(Place::Register(element_allocation)));
        }

        let index_emission = self.visit_expression(index, None)?;
        let index_place = self.handle_member_emission(index_emission, &index)?;

        let (index_memory, index_index) = match index_place {
            Place::Constant { index, .. } => (MemoryKind::CONSTANT, index),
            Place::Register(RegisterAllocation::Single { register, .. }) => {
                (MemoryKind::REGISTER, register.index)
            }
            _ => {
                return Err(CompileError::ExpectedIntegerIndex {
                    found: *self.resolver.get_type_binding(&index.id)?,
                    position: index.position(),
                });
            }
        };

        let base_index = list_registers.index();

        let element_type_id = *self.resolver.get_type_binding(&reader.id)?;

        let destination = if let Some(target) = target {
            target
        } else {
            self.allocate_registers(element_type_id, true, &reader)?
        };

        let destination_register = destination.expect_single()?;

        let mut index_instructions = if let Some(instructions) = list_instructions {
            instructions
        } else {
            InstructionsEmission::new()
        };

        let check_index_instruction =
            Instruction::check_index(index_memory, index_index, array_length as u16);

        index_instructions.push(check_index_instruction);

        let get_index_instruction = Instruction::get_index(
            destination_register.index,
            destination_register.operand_type,
            base_index,
            index_memory,
            index_index,
        );

        index_instructions.push(get_index_instruction);
        index_instructions.set_target(Some(destination));

        Ok(Emission::Instructions(index_instructions))
    }

    fn visit_range_expression(
        &mut self,
        reader: SyntaxReader,
        target: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let RangeExpression { start, end } = reader.as_component()?;

        let target = if let Some(target) = target {
            target
        } else {
            let type_id = *self.resolver.get_type_binding(&reader.id)?;

            self.allocate_registers(type_id, true, &reader)?
        };

        let mut range_instructions = InstructionsEmission::new();

        for (field_expression, destination) in [start, end].into_iter().zip(target.iter()) {
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

                    range_instructions.push(move_instruction);
                }
                Place::Register(RegisterAllocation::Single { register, .. }) => {
                    let move_instruction = Instruction::r#move(
                        destination.index,
                        register.operand_type,
                        MemoryKind::REGISTER,
                        register.index,
                    );

                    range_instructions.push(move_instruction);
                }
                Place::Register(RegisterAllocation::Multiple { registers, .. }) => {
                    for register in registers {
                        let move_instruction = Instruction::r#move(
                            destination.index,
                            register.operand_type,
                            MemoryKind::REGISTER,
                            register.index,
                        );

                        range_instructions.push(move_instruction);
                    }
                }
            }
        }

        range_instructions.set_target(Some(target));

        Ok(Emission::Instructions(range_instructions))
    }

    fn visit_path_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let declaration_id = self.resolver.get_declaration_binding(&reader.id)?;
        let declaration = self
            .resolver
            .declarations
            .get_declaration(*declaration_id)?;

        if let Definition::Variant {
            discriminant,
            fields,
            ..
        } = declaration.definition
            && fields.is_empty()
        {
            let type_id = *self.resolver.get_type_binding(&reader.id)?;
            let target = self.allocate_registers(type_id, true, &reader)?;
            let discriminant_constant = self.add_constant(ConstantEmission::U32(discriminant));
            let destination = target.index();
            let move_instruction = Instruction::r#move(
                destination,
                OperandType::U_32,
                MemoryKind::CONSTANT,
                discriminant_constant,
            );

            let mut instructions = InstructionsEmission::new();

            instructions.push(move_instruction);
            instructions.set_target(Some(target));

            return Ok(Emission::Instructions(instructions));
        }

        let local = self
            .locals
            .get(declaration_id)
            .ok_or_else(|| CompileError::DeclarationOutOfScope {
                declaration_id: *declaration_id,
                usage_position: reader.position(),
            })?
            .clone();

        Ok(Emission::Place(local))
    }

    fn visit_struct_expression(
        &mut self,
        reader: SyntaxReader,
        target: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting struct expression");

        let (_, struct_fields) = reader.binary_children()?;

        let target = if let Some(target) = target {
            target
        } else {
            let type_id = *self.resolver.get_type_binding(&reader.id)?;

            self.allocate_registers(type_id, true, &reader)?
        };

        let mut struct_instructions = InstructionsEmission::new();

        let fields_and_registers = struct_fields
            .children()
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

    fn visit_grouped_expression(
        &mut self,
        reader: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        todo!()
    }

    fn visit_block_expression(
        &mut self,
        reader: SyntaxReader<'_>,
        target: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting block expression");

        let children = reader.children();

        let block_scope_id = *self.resolver.get_scope_binding(&reader.id)?;
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
                    return Err(CompileError::ExpectedNativeFunctionCall {
                        position: reader.position(),
                    });
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
        reader: SyntaxReader<'_>,
        target: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting if expression");

        let mut children = reader.children();
        let condition = children.expect_next()?;
        let then_block = children.expect_next()?;
        let else_block = children.next();

        let mut if_instructions = InstructionsEmission::new();

        let condition_emission = self.visit_expression(condition, None)?;

        self.handle_condition_emission(&mut if_instructions, condition_emission, &condition)?;

        let target = if let Some(target) = target {
            target
        } else {
            let type_id = *self.resolver.get_type_binding(&reader.id)?;

            self.allocate_registers(type_id, true, &reader)?
        };
        let jump_over_then_id = self.create_jump_id();
        let start_else_anchor_count = self.jump_over_branch_ids.len();

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

            self.jump_over_branch_ids.push(jump_over_else_id);

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

        let end_else_anchor_count = self.jump_over_branch_ids.len();

        for index in start_else_anchor_count..end_else_anchor_count {
            let jump_id = self.jump_over_branch_ids[index];

            if_instructions.push_drop_anchor(JumpAnchor::ForwardToNext { id: jump_id });
        }

        Ok(Emission::Instructions(if_instructions))
    }

    fn visit_math_expression(
        &mut self,
        reader: SyntaxReader,
        target: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let MathExpression { left, right } = reader.as_component()?;

        let mut left_emission = self.visit_expression(left, None)?;
        let right_emission = self.visit_expression(right, None)?;

        if target.is_none()
            && let (Emission::Constant(left_value), Emission::Constant(right_value)) =
                (&left_emission, &right_emission)
        {
            let combined =
                self.combine_constants(&reader, *left_value, &left, *right_value, &right)?;

            return Ok(Emission::Constant(combined));
        }

        let mut math_emission = InstructionsEmission::new();

        let left_target = left_emission.take_target();
        let (left_memory, left_index, _) = self.handle_operand_emission(
            &mut math_emission,
            left_emission,
            reader.node.kind,
            &left,
        )?;
        let (right_memory, right_index, _) = self.handle_operand_emission(
            &mut math_emission,
            right_emission,
            reader.node.kind,
            &right,
        )?;

        let type_id = *self.resolver.get_type_binding(&reader.id)?;
        let mut handle_target_register = |target, node| -> Result<Register, CompileError> {
            let target = if let Some(target) = target {
                target
            } else {
                self.allocate_registers(type_id, true, node)?
            };

            let register = target.expect_single()?;

            math_emission.set_target(Some(target));

            Ok(register)
        };

        let math_instruction = match reader.node.kind {
            SyntaxKind::AdditionExpression => {
                let register = handle_target_register(target, &reader)?;

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
                let regsiter = handle_target_register(left_target, &left)?;

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
                let register = handle_target_register(target, &reader)?;

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
                let register = handle_target_register(left_target, &left)?;

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
                let register = handle_target_register(target, &reader)?;

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
                let register = handle_target_register(left_target, &left)?;

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
                let register = handle_target_register(target, &reader)?;

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
                let register = handle_target_register(left_target, &left)?;

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
                let register = handle_target_register(target, &reader)?;

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
                let register = handle_target_register(left_target, &left)?;

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
                let register = handle_target_register(target, &reader)?;

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
                let register = handle_target_register(left_target, &left)?;

                Instruction::power(
                    register.index,
                    register.operand_type,
                    left_memory,
                    left_index,
                    right_memory,
                    right_index,
                )
            }
            _ => unreachable!("Expected math expression, found {}", reader.node.kind),
        };

        math_emission.push(math_instruction);

        Ok(Emission::Instructions(math_emission))
    }

    fn visit_comparison_expression(
        &mut self,
        reader: SyntaxReader,
        target: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let ComparisonExpression { left, right } = reader.as_component()?;

        let left_emission = self.visit_expression(left, None)?;
        let right_emission = self.visit_expression(right, None)?;

        if let Emission::Constant(left_constant) = left_emission
            && let Emission::Constant(right_constant) = right_emission
        {
            let combined =
                self.combine_constants(&reader, left_constant, &left, right_constant, &right)?;

            return Ok(Emission::Constant(combined));
        }

        let mut comparison_emission = InstructionsEmission::new();

        let (left_memory, left_index, _) = self.handle_operand_emission(
            &mut comparison_emission,
            left_emission,
            reader.node.kind,
            &left,
        )?;
        let (right_memory, right_index, _) = self.handle_operand_emission(
            &mut comparison_emission,
            right_emission,
            reader.node.kind,
            &right,
        )?;

        let target = if let Some(target) = target {
            target
        } else {
            let type_id = *self.resolver.get_type_binding(&reader.id)?;

            self.allocate_registers(type_id, true, &reader)?
        };
        let register = target.expect_single()?;
        let comparison_instruction = match reader.node.kind {
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
            SyntaxKind::GreaterThanExpression => Instruction::less_equal(
                false,
                register.operand_type,
                left_memory,
                left_index,
                right_memory,
                right_index,
            ),
            SyntaxKind::LessThanOrEqualExpression => Instruction::less_equal(
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
            _ => unreachable!("Expected comparison expression, found {}", reader.node.kind),
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
        reader: SyntaxReader<'_>,
        target: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let LogicExpression { left, right } = reader.as_component()?;

        let left_emission = self.visit_expression(left, None)?;
        let right_emission = self.visit_expression(right, None)?;

        if let Emission::Constant(left_constant) = left_emission
            && let Emission::Constant(right_constant) = right_emission
        {
            let combined =
                self.combine_constants(&reader, left_constant, &left, right_constant, &right)?;

            return Ok(Emission::Constant(combined));
        }

        let mut logic_instructions = InstructionsEmission::new();

        let (left_memory, left_index, _) = self.handle_operand_emission(
            &mut logic_instructions,
            left_emission,
            reader.node.kind,
            &left,
        )?;
        let (right_memory, right_index, _) = self.handle_operand_emission(
            &mut logic_instructions,
            right_emission,
            reader.node.kind,
            &right,
        )?;

        let target = if let Some(target) = target {
            target.clone()
        } else {
            let type_id = *self.resolver.get_type_binding(&reader.id)?;

            self.allocate_registers(type_id, true, &reader)?
        };
        let register = target.expect_single()?;

        let test_instruction = match reader.node.kind {
            SyntaxKind::AndExpression => Instruction::test(false, left_memory, left_index, 1),
            SyntaxKind::OrExpression => Instruction::test(true, left_memory, left_index, 1),
            _ => unreachable!("Expected logical expression, found {}", reader.node.kind),
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
        reader: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let NegationExpression { operand } = reader.as_component()?;

        let expression_emission = self.visit_expression(operand, None)?;

        if let Emission::Constant(constant) = expression_emission {
            let negated = constant
                .negate()
                .ok_or_else(|| CompileError::CannotApplyOperator {
                    operator: reader.node.kind,
                    type_id: constant.type_id(),
                    operand_position: operand.position(),
                })?;

            return Ok(Emission::Constant(negated));
        }

        let mut negation_emission = InstructionsEmission::new();

        let (operand_memory, operand_index, _) = self.handle_operand_emission(
            &mut negation_emission,
            expression_emission,
            reader.node.kind,
            &operand,
        )?;
        let target = if let Some(target) = input {
            target.clone()
        } else {
            let type_id = *self.resolver.get_type_binding(&reader.id)?;

            self.allocate_registers(type_id, true, &reader)?
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

    fn visit_not_expression(
        &mut self,
        reader: SyntaxReader,
        input: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let NotExpression { operand } = reader.as_component()?;

        let expression_emission = self.visit_expression(operand, None)?;

        if let Emission::Constant(constant) = expression_emission {
            let negated = constant
                .negate()
                .ok_or_else(|| CompileError::CannotApplyOperator {
                    operator: reader.node.kind,
                    type_id: constant.type_id(),
                    operand_position: operand.position(),
                })?;

            return Ok(Emission::Constant(negated));
        }

        let mut negation_emission = InstructionsEmission::new();

        let (operand_memory, operand_index, _) = self.handle_operand_emission(
            &mut negation_emission,
            expression_emission,
            reader.node.kind,
            &operand,
        )?;
        let target = if let Some(target) = input {
            target.clone()
        } else {
            let type_id = *self.resolver.get_type_binding(&reader.id)?;

            self.allocate_registers(type_id, true, &reader)?
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
        reader: SyntaxReader<'_>,
        target: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        debug!("Visting while expression");

        let (condition, body) = reader.binary_children()?;

        let mut while_emission = InstructionsEmission::new();
        let condition_emission = self.visit_expression(condition, None)?;

        self.handle_condition_emission(&mut while_emission, condition_emission, &condition)?;

        let jump_forward_id = self.create_jump_id();
        let jump_backward_id = self.create_jump_id();

        while_emission.push_drop_anchor(JumpAnchor::LoopStartHere {
            forward_id: jump_forward_id,
        });

        let body_emission = self.visit_block_expression(body, target)?;

        if let Emission::Instructions(instructions) = body_emission {
            while_emission.merge(instructions);
        }

        while_emission.push_drop_anchor(JumpAnchor::LoopEndOnNext {
            forward_id: jump_forward_id,
            backward_id: jump_backward_id,
        });
        while_emission.set_target(None);

        Ok(Emission::Instructions(while_emission))
    }

    fn visit_call_expression(
        &mut self,
        reader: SyntaxReader<'_>,
        target: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        let CallExpression { callee, arguments } = reader.as_component()?;

        let declaration_id = *self.resolver.get_declaration_binding(&callee.id)?;
        let definition = self
            .resolver
            .declarations
            .get_declaration(declaration_id)?
            .definition;

        if let Definition::Variant {
            discriminant,
            fields,
            ..
        } = definition
            && !fields.is_empty()
        {
            let return_type_id = *self.resolver.get_type_binding(&reader.id)?;
            let target = if let Some(target) = target {
                target
            } else {
                self.allocate_registers(return_type_id, true, &reader)?
            };

            let discriminant_constant = self.add_constant(ConstantEmission::U32(discriminant));
            let destination = target.index();
            let move_instruction = Instruction::r#move(
                destination,
                OperandType::U_32,
                MemoryKind::CONSTANT,
                discriminant_constant,
            );

            let mut variant_instructions = InstructionsEmission::new();

            variant_instructions.push(move_instruction);

            let field_registers = target.iter().skip(1);

            for (argument, field_register) in arguments.children().zip(field_registers) {
                let argument_emission = self.visit_expression(argument, None)?;
                let argument_place = self.handle_member_emission(argument_emission, &argument)?;

                match argument_place {
                    Place::Constant {
                        operand_type,
                        index,
                    } => {
                        let move_instruction = Instruction::r#move(
                            field_register.index,
                            operand_type,
                            MemoryKind::CONSTANT,
                            index,
                        );

                        variant_instructions.push(move_instruction);
                    }
                    Place::Register(RegisterAllocation::Single { register, .. }) => {
                        let move_instruction = Instruction::r#move(
                            field_register.index,
                            register.operand_type,
                            MemoryKind::REGISTER,
                            register.index,
                        );

                        variant_instructions.push(move_instruction);
                    }
                    Place::Register(RegisterAllocation::Multiple { registers, .. }) => {
                        for register in registers {
                            let move_instruction = Instruction::r#move(
                                field_register.index,
                                register.operand_type,
                                MemoryKind::REGISTER,
                                register.index,
                            );

                            variant_instructions.push(move_instruction);
                        }
                    }
                }
            }

            variant_instructions.set_target(Some(target));

            return Ok(Emission::Instructions(variant_instructions));
        }

        let return_type_id = *self.resolver.get_type_binding(&reader.id)?;
        let target = if target.is_some() {
            target
        } else if return_type_id != TypeId::UNIT {
            Some(self.allocate_registers(return_type_id, true, &reader)?)
        } else {
            None
        };
        let destination = target
            .as_ref()
            .map(|target| target.index())
            .unwrap_or(u16::MAX);

        let mut call_instructions = InstructionsEmission::new();

        let monomorphized_prototype_id = if let Definition::Function {
            value_parameters, ..
        } = definition
        {
            let mut concrete_type_arguments = SmallVec::<[TypeId; 4]>::new();

            for (argument, parameter_index) in arguments.children().zip(value_parameters.as_range())
            {
                let parameter_type_id = *self.resolver.types.get_type_member(parameter_index)?;
                let parameter_type = *self.resolver.types.get_type(parameter_type_id)?;

                if matches!(parameter_type, Type::Slice { .. }) {
                    let argument_type_id = *self.resolver.get_type_binding(&argument.id)?;

                    concrete_type_arguments.push(argument_type_id);
                }
            }

            if !concrete_type_arguments.is_empty() {
                let cache_key = (declaration_id, concrete_type_arguments);
                let prototype_id =
                    if let Some(existing) = self.resolver.get_cached_prototype(&cache_key) {
                        existing
                    } else {
                        let reserved = self.prototypes.reserve();

                        self.resolver.cache_prototype(cache_key, reserved);
                        self.compilation_stack.push(CompilationRequest {
                            declaration_id,
                            prototype_id: reserved,
                        });

                        reserved
                    };

                Some(prototype_id)
            } else {
                None
            }
        } else {
            None
        };

        let arguments_start = self.register_tracker.next_temporary;

        for argument in arguments.children() {
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
                    }
                }
            }
        }

        let (callee_memory, callee_index) = if let Some(prototype_id) = monomorphized_prototype_id {
            (MemoryKind::CONSTANT, prototype_id.inner())
        } else {
            let callee_emission = self.visit_expression(callee, None)?;

            let callee_place = match callee_emission {
                Emission::Place(place) => place,
                Emission::Instructions(instructions) => {
                    let Some(registers) = instructions.target else {
                        return Err(CompileError::ExpectedFunction {
                            node_kind: callee.node.kind,
                            position: callee.position(),
                        });
                    };

                    Place::Register(registers)
                }
                Emission::NativeFunction(native_function) => {
                    let call_native_instruction = todo!();
                }
                _ => {
                    return Err(CompileError::ExpectedFunction {
                        node_kind: callee.node.kind,
                        position: callee.position(),
                    });
                }
            };

            match callee_place {
                Place::Constant { index, .. } => (MemoryKind::CONSTANT, index),
                Place::Register(RegisterAllocation::Single { register, .. }) => {
                    (MemoryKind::REGISTER, register.index)
                }
                Place::Register(RegisterAllocation::Multiple { .. }) => {
                    return Err(CompileError::ExpectedFunction {
                        node_kind: callee.node.kind,
                        position: callee.position(),
                    });
                }
            }
        };

        let call_instruction =
            Instruction::call(destination, callee_memory, callee_index, arguments_start);

        call_instructions.push(call_instruction);
        call_instructions.set_target(target);

        Ok(Emission::Instructions(call_instructions))
    }

    fn visit_field_access_expression(
        &mut self,
        reader: SyntaxReader,
        _: Option<Self::ExpressionInput>,
    ) -> Result<Self::ExpressionOutput, CompileError> {
        Err(CompileError::Unimplemented {
            syntax_kind: reader.node.kind,
            position: reader.position(),
        })
    }

    fn visit_type(&mut self, _: SyntaxReader) -> Result<Self::TypeOutput, CompileError> {
        Ok(())
    }

    fn visit_path(
        &mut self,
        reader: SyntaxReader,
        _: Self::PathInput,
    ) -> Result<Self::PathOutput, CompileError> {
        todo!()
    }

    fn visit_simple_path(
        &mut self,
        reader: SyntaxReader,
        _: Self::PathInput,
    ) -> Result<Self::PathOutput, CompileError> {
        todo!()
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

    fn with_instruction(instruction: Instruction) -> Self {
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
    fn expect_register(self, node: &SyntaxReader) -> Result<RegisterAllocation, CompileError> {
        match self {
            Place::Register(target) => Ok(target),
            _ => Err(CompileError::CannotMutate {
                position: node.position(),
            }),
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
    fn is_temporary(&self) -> bool {
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

    fn expect_single(&self) -> Result<Register, CompileError> {
        match self {
            RegisterAllocation::Single { register, .. } => Ok(*register),
            _ => Err(CompileError::InvalidRegisterCount {
                expected: 1,
                found: self.len(),
            }),
        }
    }

    fn expect_multiple(
        &self,
        expected: usize,
    ) -> Result<(&SmallVec<[Register; 8]>, bool), CompileError> {
        match self {
            RegisterAllocation::Multiple {
                registers,
                temporary,
            } if registers.len() == expected => Ok((registers, *temporary)),
            _ => Err(CompileError::InvalidRegisterCount {
                expected: 2,
                found: self.len(),
            }),
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

    fn add(self, other: Self) -> Result<Option<Self>, CompileError> {
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
    ForwardFromHere {
        id: JumpId,
    },
    LoopStartHere {
        forward_id: JumpId,
    },
    ForwardToNext {
        id: JumpId,
    },
    LoopEndOnNext {
        forward_id: JumpId,
        backward_id: JumpId,
    },
}

#[derive(Clone, Copy, Debug)]
struct JumpPlacement {
    index: usize,
    distance: u16,
    forward: bool,
    coalesce: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JumpId(u16);

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
        self.next_local += width.as_u16();
        self.next_temporary = self.next_temporary.max(self.next_local);
        self.max = self.max.max(self.next_local);

        next
    }

    fn allocate_next_temporary(&mut self, width: RegisterWidth) -> u16 {
        let next = self.next_temporary;
        self.next_temporary += width.as_u16();
        self.max = self.max.max(self.next_temporary);

        next
    }

    fn allocate_next_reserved(&mut self, width: RegisterWidth) -> u16 {
        let next = self.next_reserved.min(self.reserved);
        self.next_reserved += width.as_u16();

        next
    }

    fn reset_reserved(&mut self) {
        self.next_reserved = 0;
    }

    fn free_temporary(&mut self, registers: &RegisterAllocation) {
        debug_assert!(registers.is_temporary());

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

impl RegisterWidth {
    fn as_u16(&self) -> u16 {
        match self {
            RegisterWidth::Single => 1,
            RegisterWidth::Double => 2,
            RegisterWidth::Quad => 4,
        }
    }
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

pub fn get_byte_size(
    type_id: TypeId,
    type_arguments: Option<&TypeMembers>,
    resolver: &Resolver,
) -> Result<Option<usize>, CompileError> {
    fn get_definition_type_size(
        declaration_id: DeclarationId,
        type_arguments: Option<&TypeMembers>,
        resolver: &Resolver,
    ) -> Result<Option<usize>, CompileError> {
        let declaration = resolver.declarations.get_declaration(declaration_id)?;

        match &declaration.definition {
            Definition::StructType { fields, .. } => {
                let field_declaration_ids =
                    resolver.declarations.get_declaration_members(fields)?;
                let mut total_size = 0;

                for field_declaration_id in field_declaration_ids {
                    let field_declaration = resolver
                        .declarations
                        .get_declaration(*field_declaration_id)?;
                    let Definition::Field {
                        type_id: field_type_id,
                        ..
                    } = field_declaration.definition
                    else {
                        return Err(CompileError::Resolver(
                            ResolverError::ExpectedFieldDeclaration(*field_declaration_id),
                        ));
                    };

                    let byte_size = if let Some(size) =
                        get_byte_size(field_type_id, type_arguments, resolver)?
                    {
                        size
                    } else {
                        return Ok(None);
                    };

                    total_size += byte_size;
                }

                Ok(Some(total_size))
            }
            Definition::EnumType { variants, .. } => {
                let discriminant_size = 4;
                let variant_declaration_ids =
                    resolver.declarations.get_declaration_members(variants)?;
                let mut max_variant_size = 0;

                for variant_declaration_id in variant_declaration_ids {
                    let variant_declaration = resolver
                        .declarations
                        .get_declaration(*variant_declaration_id)?;
                    let Definition::Variant { fields, .. } = &variant_declaration.definition else {
                        continue;
                    };

                    let field_declaration_ids =
                        resolver.declarations.get_declaration_members(fields)?;
                    let mut variant_size = 0;

                    for field_declaration_id in field_declaration_ids {
                        let field_declaration = resolver
                            .declarations
                            .get_declaration(*field_declaration_id)?;
                        let Definition::Field {
                            type_id: field_type_id,
                            ..
                        } = field_declaration.definition
                        else {
                            continue;
                        };

                        let byte_size =
                            get_byte_size(field_type_id, type_arguments, resolver)?.unwrap_or(0);

                        variant_size += byte_size;
                    }

                    max_variant_size = max_variant_size.max(variant_size);
                }

                Ok(Some(discriminant_size + max_variant_size))
            }
            Definition::TypeParameter => {
                todo!()
            }
            _ => todo!("Handle byte size for declaration: {:?}", declaration),
        }
    }

    let r#type = resolver.types.get_type(type_id)?;

    match r#type {
        Type::Never => Ok(Some(0)),
        Type::Boolean
        | Type::SignedInteger(SignedIntegerType::I8)
        | Type::UnsignedInteger(UnsignedIntegerType::U8) => Ok(Some(1)),
        Type::SignedInteger(SignedIntegerType::I16)
        | Type::UnsignedInteger(UnsignedIntegerType::U16)
        | Type::FunctionDefinition { .. }
        | Type::Closure { .. }
        | Type::Function { .. } => Ok(Some(2)),
        Type::Character
        | Type::SignedInteger(SignedIntegerType::I32)
        | Type::UnsignedInteger(UnsignedIntegerType::U32)
        | Type::Float(FloatType::F32) => Ok(Some(4)),
        Type::Slice { .. } | Type::Pointer { .. } => Ok(Some(8)),
        Type::SignedInteger(SignedIntegerType::I64)
        | Type::UnsignedInteger(UnsignedIntegerType::U64)
        | Type::Float(FloatType::F64) => Ok(Some(8)),
        Type::SignedInteger(SignedIntegerType::I128)
        | Type::UnsignedInteger(UnsignedIntegerType::U128) => Ok(Some(16)),
        Type::Tuple { element_type_ids } => {
            let type_ids = resolver.types.get_type_members(*element_type_ids)?;
            let mut total_size = 0;

            for type_id in type_ids {
                let byte_size =
                    if let Some(size) = get_byte_size(*type_id, type_arguments, resolver)? {
                        size
                    } else {
                        return Ok(None);
                    };

                total_size += byte_size;
            }

            Ok(Some(total_size))
        }
        Type::Array {
            element_type_id,
            length,
        } => {
            let element_size =
                if let Some(size) = get_byte_size(*element_type_id, type_arguments, resolver)? {
                    size
                } else {
                    return Ok(None);
                };

            Ok(Some(element_size * (*length)))
        }
        Type::Algebraic {
            declaration_id,
            type_arguments,
        } => get_definition_type_size(*declaration_id, Some(type_arguments), resolver),
        Type::Generic { declaration_id } => {
            get_definition_type_size(*declaration_id, None, resolver)
        }
        Type::Inferred {
            resolved: Some(resolved),
            ..
        } => get_byte_size(*resolved, type_arguments, resolver),
        Type::Inferred { resolved: None, .. } => Ok(None),
        _ => todo!("Handle byte size for type: {:?}", r#type),
    }
}

pub fn get_register_size(
    type_id: TypeId,
    type_arguments: Option<&TypeMembers>,
    resolver: &Resolver,
) -> Result<Option<usize>, CompileError> {
    get_byte_size(type_id, type_arguments, resolver)
        .map(|byte_size| byte_size.map(|size| size.div_ceil(4)))
}
