#[cfg(test)]
mod tests;

use std::collections::HashMap;

use rustc_hash::FxBuildHasher;
use smallvec::{SmallVec, smallvec};
use tracing::{debug, trace};

use crate::{
    compiler::{
        CompilationRequest,
        error::CompileError,
        resolver::{
            PrototypeId, Resolver,
            declarations::{DeclarationId, Definition},
            scopes::{ScopeId, ScopeKind},
            types::{
                FloatType, InferredTypeConstraint, SignedIntegerType, Type, TypeId, TypeMembers,
                UnsignedIntegerType,
            },
        },
        value_creation::{
            create_char, create_f32_from_decimal, create_f64_from_decimal, create_i8_from_decimal,
            create_i16_from_decimal, create_i32_from_decimal, create_i64_from_decimal,
            create_i128_from_decimal, create_u8_from_decimal, create_u8_from_hexadecimal,
            create_u16_from_decimal, create_u32_from_decimal, create_u64_from_decimal,
            create_u128_from_decimal, create_usize_from_decimal,
        },
    },
    constants::{ConstantsBuilder, value::ConstantValue},
    instruction::{
        Address, Drop, Instruction, Jump, MemoryKind, Move, OperandType, Operation, Test,
    },
    native_function::NativeFunction,
    prototype::Prototype,
    source::Source,
    syntax::{
        components::{
            ArrayExpression, ArrayRepeatExpression, AssignmentExpression, CallExpression,
            ComparisonExpression, ConstItem, ExpressionStatement, FieldAccessExpression, FnItem,
            IndexExpression, LetStatement, LogicExpression, MathExpression, NegationExpression,
            NotExpression, RangeExpression, ValueParameters,
        },
        node::{SyntaxFlags, SyntaxKind},
        reader::SyntaxReader,
    },
};

#[derive(Debug)]
pub struct Emitter<'a> {
    source: &'a Source<'a>,

    constants: &'a mut ConstantsBuilder,

    resolver: &'a mut Resolver,

    compilation_stack: &'a mut Vec<CompilationRequest>,

    argument_count: u16,

    return_type_id: TypeId,

    return_operand_types: OperandType::SmallVec,

    /// Emitted bytecode instructions, filled during compilation.
    instructions: Vec<Instruction>,

    /// Local variables declared in the function.
    locals: HashMap<DeclarationId, Local, FxBuildHasher>,

    /// Concatenated list of register indices that are referenced by DROP and JUMP instructions.
    drop_lists: Vec<u16>,

    /// Stack of register index lists that need to be dropped when exiting scopes.
    pending_drops: Vec<SmallVec<[u16; 8]>>,

    register_tracker: RegisterTracker,

    jump_placements: HashMap<JumpId, JumpPlacement, FxBuildHasher>,

    jump_over_branch_ids: Vec<JumpId>,

    current_scope_id: ScopeId,

    next_jump_id: JumpId,
}

impl<'a> Emitter<'a> {
    pub fn new(
        declaration_id: Option<DeclarationId>,
        prototype_id: PrototypeId,
        return_type_id: TypeId,
        (source, constants, resolver, compilation_stack): (
            &'a Source,
            &'a mut ConstantsBuilder,
            &'a mut Resolver,
            &'a mut Vec<CompilationRequest>,
        ),
        value_parameters: Option<SyntaxReader>,
    ) -> Result<Self, CompileError> {
        let argument_register_count = if let Some(value_parameters) = value_parameters {
            let ValueParameters { name_type_pairs } = value_parameters.as_component()?;

            let mut count = 0;

            for [parameter_name, _] in name_type_pairs {
                let declaration_id = *resolver.get_declaration_binding(&parameter_name.id)?;
                let declaration = resolver.declarations.get_declaration(declaration_id)?;
                let Definition::Local { type_id, .. } = declaration.definition else {
                    return Err(CompileError::ExpectedLocalDefinition);
                };
                let concrete_type_id = resolver.resolve_type(type_id)?;
                let operand_types = resolver.get_operand_types(concrete_type_id)?;
                let register_size = operand_types
                    .iter()
                    .map(|operand_type| operand_type.register_width().as_u16())
                    .sum::<u16>();

                count += register_size as u16;
            }

            count
        } else {
            0
        };
        let return_operand_types = resolver.get_operand_types(return_type_id)?;
        let return_register_count = return_operand_types
            .iter()
            .map(|operand_type| operand_type.register_width().as_u16())
            .sum();

        let mut emitter = Self {
            source,
            constants,
            resolver,
            compilation_stack,
            instructions: Vec::new(),
            locals: HashMap::default(),
            drop_lists: Vec::new(),
            pending_drops: Vec::new(),
            argument_count: argument_register_count,
            register_tracker: RegisterTracker::new(argument_register_count, return_register_count),
            return_type_id,
            return_operand_types,
            jump_placements: HashMap::default(),
            jump_over_branch_ids: Vec::new(),
            current_scope_id: ScopeId::NONE,
            next_jump_id: JumpId(0),
        };

        if let Some(declaration_id) = declaration_id {
            emitter.locals.insert(
                declaration_id,
                Local::Place(Place::Constant {
                    operand_type: OperandType::FUNCTION,
                    index: prototype_id.inner(),
                }),
            );
        }

        if let Some(value_parameters) = value_parameters {
            let ValueParameters { name_type_pairs } = value_parameters.as_component()?;

            for [parameter_name, _] in name_type_pairs {
                let declaration_id = *emitter
                    .resolver
                    .get_declaration_binding(&parameter_name.id)?;
                let declaration = emitter
                    .resolver
                    .declarations
                    .get_declaration(declaration_id)?;
                let Definition::Local { type_id, .. } = declaration.definition else {
                    return Err(CompileError::ExpectedLocal);
                };
                let concrete_type_id = emitter.resolver.resolve_type(type_id)?;
                let allocation =
                    emitter.allocate_registers(concrete_type_id, AllocationKind::Reserved)?;

                emitter
                    .locals
                    .insert(declaration_id, Local::Place(Place::Register(allocation)));
            }
        }

        emitter.register_tracker.free_reserved();

        Ok(emitter)
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
                Operation::NO_OP => {
                    *instruction = Instruction::jump(distance, forward);
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
                Operation::MOVE => {
                    let r#move = Move::from(&*instruction);
                    let total_distance = r#move.jump_distance + distance;

                    *instruction = Instruction::move_with_jump(
                        r#move.destination,
                        r#move.operand_type,
                        r#move.operand_memory,
                        r#move.operand_index,
                        total_distance,
                        forward,
                    );
                }
                _ => {}
            }
        }

        Ok(Prototype {
            instructions: self.instructions,
            return_types: self.return_operand_types,
            register_count: self.register_tracker.max,
            argument_count: self.argument_count,
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
        kind: AllocationKind,
    ) -> Result<RegisterAllocation, CompileError> {
        fn collect_registers(
            type_id: TypeId,
            kind: AllocationKind,
            registers: &mut SmallVec<[RegisterClaim; 4]>,
            emitter: &mut Emitter,
        ) -> Result<(), CompileError> {
            let type_node = emitter.resolver.types.get_type(type_id)?;

            let operand_type = match type_node {
                Type::Never => return Ok(()),
                Type::Boolean => OperandType::BOOLEAN,
                Type::SignedInteger(SignedIntegerType::I8) => OperandType::I_8,
                Type::SignedInteger(SignedIntegerType::I16) => OperandType::I_16,
                Type::SignedInteger(SignedIntegerType::I32) => OperandType::I_32,
                Type::SignedInteger(SignedIntegerType::I64) => OperandType::I_64,
                Type::SignedInteger(SignedIntegerType::I128) => OperandType::I_128,
                Type::SignedInteger(SignedIntegerType::ISize) => {
                    #[cfg(target_pointer_width = "64")]
                    {
                        OperandType::I_64
                    }

                    #[cfg(target_pointer_width = "32")]
                    {
                        (OperandType::I_32, RegisterWidth::Single)
                    }
                }
                Type::UnsignedInteger(UnsignedIntegerType::U8) => OperandType::U_8,
                Type::UnsignedInteger(UnsignedIntegerType::U16) => OperandType::U_16,
                Type::UnsignedInteger(UnsignedIntegerType::U32) => OperandType::U_32,
                Type::UnsignedInteger(UnsignedIntegerType::U64) => OperandType::U_64,
                Type::UnsignedInteger(UnsignedIntegerType::U128) => OperandType::U_128,
                Type::UnsignedInteger(UnsignedIntegerType::USize) => {
                    #[cfg(target_pointer_width = "64")]
                    {
                        OperandType::U_64
                    }

                    #[cfg(target_pointer_width = "32")]
                    {
                        OperandType::U_32
                    }
                }
                Type::Float(FloatType::F32) => OperandType::F_32,
                Type::Float(FloatType::F64) => OperandType::F_64,
                Type::Character => OperandType::CHARACTER,
                Type::Tuple { element_type_ids } => {
                    let element_type_ids = *element_type_ids;

                    for index in element_type_ids.as_range() {
                        let element_type = *emitter.resolver.types.get_type_member(index)?;

                        collect_registers(element_type, kind, registers, emitter)?;
                    }

                    return Ok(());
                }
                Type::Array {
                    element_type_id,
                    length,
                } => {
                    let element_type_id = *element_type_id;
                    let length = *length;

                    for _ in 0..length {
                        collect_registers(element_type_id, kind, registers, emitter)?;
                    }

                    return Ok(());
                }
                Type::FunctionDefinition { .. } | Type::Closure { .. } | Type::Function { .. } => {
                    OperandType::FUNCTION
                }
                Type::Slice { .. } | Type::Pointer { .. } => OperandType::POINTER,
                Type::Inferred {
                    resolved: Some(resolved),
                    ..
                } => {
                    collect_registers(*resolved, kind, registers, emitter)?;

                    return Ok(());
                }
                Type::Algebraic { .. } => {
                    let operand_types = emitter.resolver.get_operand_types(type_id)?;

                    for operand_type in operand_types {
                        let next_register_index = match kind {
                            AllocationKind::Local => {
                                emitter.register_tracker.allocate_next_local(operand_type)
                            }
                            AllocationKind::Temporary => emitter
                                .register_tracker
                                .allocate_next_temporary(operand_type),
                            AllocationKind::Reserved => emitter
                                .register_tracker
                                .allocate_next_reserved(operand_type),
                        };

                        registers.push(RegisterClaim {
                            operand_type,
                            index: next_register_index,
                        });
                    }

                    return Ok(());
                }
                Type::Inferred {
                    constraint: Some(InferredTypeConstraint::Integer),
                    resolved: None,
                    ..
                } => OperandType::I_32,
                Type::Inferred {
                    constraint: Some(InferredTypeConstraint::Float),
                    resolved: None,
                    ..
                } => OperandType::F_64,
                Type::Inferred { .. } | Type::Generic { .. } => {
                    return Err(CompileError::CannotInferType { type_id });
                }
            };

            let next_register_index = match kind {
                AllocationKind::Local => emitter.register_tracker.allocate_next_local(operand_type),
                AllocationKind::Temporary => emitter
                    .register_tracker
                    .allocate_next_temporary(operand_type),
                AllocationKind::Reserved => emitter
                    .register_tracker
                    .allocate_next_reserved(operand_type),
            };

            registers.push(RegisterClaim {
                operand_type,
                index: next_register_index,
            });

            Ok(())
        }

        let mut registers = SmallVec::new();

        collect_registers(type_id, kind, &mut registers, self)?;

        Ok(RegisterAllocation { registers, kind })
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

    fn materialize_value(&mut self, value: ConstantValue) -> Result<Address, CompileError> {
        fn encode<I: TryInto<u16>>(integer: I) -> Option<u16> {
            integer.try_into().ok()
        }

        match value {
            ConstantValue::Boolean(boolean) => Ok(Address {
                memory: MemoryKind::ENCODED,
                index: boolean as u16,
            }),
            ConstantValue::I8(integer) => Ok(Address {
                memory: MemoryKind::ENCODED,
                index: integer as u16,
            }),
            ConstantValue::I16(integer) => Ok(Address {
                memory: MemoryKind::ENCODED,
                index: integer as u16,
            }),
            ConstantValue::I32(integer) => {
                if let Some(encoded) = encode(integer) {
                    Ok(Address {
                        memory: MemoryKind::ENCODED,
                        index: encoded,
                    })
                } else {
                    Ok(Address {
                        memory: MemoryKind::CONSTANT,
                        index: self.constants.add_i32(integer).inner(),
                    })
                }
            }
            ConstantValue::I64(integer) => {
                if let Some(encoded) = encode(integer) {
                    Ok(Address {
                        memory: MemoryKind::ENCODED,
                        index: encoded,
                    })
                } else {
                    Ok(Address {
                        memory: MemoryKind::CONSTANT,
                        index: self.constants.add_i64(integer).inner(),
                    })
                }
            }
            ConstantValue::I128(integer) => {
                if let Some(encoded) = encode(integer) {
                    Ok(Address {
                        memory: MemoryKind::ENCODED,
                        index: encoded,
                    })
                } else {
                    Ok(Address {
                        memory: MemoryKind::CONSTANT,
                        index: self.constants.add_i128(integer).inner(),
                    })
                }
            }
            ConstantValue::U8(integer) => Ok(Address {
                memory: MemoryKind::ENCODED,
                index: integer as u16,
            }),
            ConstantValue::U16(integer) => Ok(Address {
                memory: MemoryKind::ENCODED,
                index: integer as u16,
            }),
            ConstantValue::U32(integer) => {
                if let Some(encoded) = encode(integer) {
                    Ok(Address {
                        memory: MemoryKind::ENCODED,
                        index: encoded,
                    })
                } else {
                    Ok(Address {
                        memory: MemoryKind::CONSTANT,
                        index: self.constants.add_u32(integer).inner(),
                    })
                }
            }
            ConstantValue::U64(integer) => {
                if let Some(encoded) = encode(integer) {
                    Ok(Address {
                        memory: MemoryKind::ENCODED,
                        index: encoded,
                    })
                } else {
                    Ok(Address {
                        memory: MemoryKind::CONSTANT,
                        index: self.constants.add_u64(integer).inner(),
                    })
                }
            }
            ConstantValue::U128(integer) => {
                if let Some(encoded) = encode(integer) {
                    Ok(Address {
                        memory: MemoryKind::ENCODED,
                        index: encoded,
                    })
                } else {
                    Ok(Address {
                        memory: MemoryKind::CONSTANT,
                        index: self.constants.add_u128(integer).inner(),
                    })
                }
            }
            ConstantValue::F32(float) => {
                if let Some(encoded) = encode(float.to_bits()) {
                    Ok(Address {
                        memory: MemoryKind::ENCODED,
                        index: encoded,
                    })
                } else {
                    Ok(Address {
                        memory: MemoryKind::CONSTANT,
                        index: self.constants.add_f32(float).inner(),
                    })
                }
            }
            ConstantValue::F64(float) => {
                if let Some(encoded) = encode(float.to_bits()) {
                    Ok(Address {
                        memory: MemoryKind::ENCODED,
                        index: encoded,
                    })
                } else {
                    Ok(Address {
                        memory: MemoryKind::CONSTANT,
                        index: self.constants.add_f64(float).inner(),
                    })
                }
            }
            ConstantValue::Character(character) => {
                if let Some(encoded) = encode(character) {
                    Ok(Address {
                        memory: MemoryKind::ENCODED,
                        index: encoded,
                    })
                } else {
                    Ok(Address {
                        memory: MemoryKind::CONSTANT,
                        index: self.constants.add_character(character).inner(),
                    })
                }
            }
        }
    }

    fn place_emission(
        &mut self,
        source_emission: Emission,
        target_instructions: &mut InstructionsEmission,
        syntax: &SyntaxReader,
    ) -> Result<Place, CompileError> {
        match source_emission {
            Emission::Value(constant) => {
                let address = self.materialize_value(constant)?;

                Ok(Place::Constant {
                    operand_type: constant.operand_type(),
                    index: address.index,
                })
            }
            Emission::Place(place) => Ok(place),
            Emission::Instructions(InstructionsEmission {
                target,
                instructions,
            }) => {
                if let Some(registers) = &target
                    && registers.is_temporary()
                {
                    self.free_temporary_registers(registers);
                }

                target_instructions.extend(instructions);

                if let Some(allocation) = target {
                    Ok(Place::Register(allocation))
                } else {
                    Err(CompileError::ExpectedValue {
                        file_id: syntax.file_id(),
                        syntax_id: syntax.id,
                    })
                }
            }
            Emission::NativeFunction(_) => Err(CompileError::ExpectedNativeFunctionCall {
                position: syntax.position(),
            }),
            _ => Err(CompileError::ExpectedValue {
                file_id: syntax.file_id(),
                syntax_id: syntax.id,
            }),
        }
    }

    fn handle_function_body_instructions(
        &mut self,
        instructions: InstructionsEmission,
    ) -> Result<(), CompileError> {
        for (instruction, mut jump_anchors) in instructions.instructions {
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
                        let coalesce = if instruction.is_coallescible_with_jump(true) {
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

                        let forward_placement = self
                            .jump_placements
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

        Ok(())
    }

    fn handle_operand_emission(
        &mut self,
        instructions: &mut InstructionsEmission,
        emission: Emission,
        operand: &SyntaxReader,
    ) -> Result<(MemoryKind, u16, OperandType::SmallVec), CompileError> {
        match emission {
            Emission::Value(value) => {
                let address = self.materialize_value(value)?;

                Ok((
                    address.memory,
                    address.index,
                    smallvec![value.operand_type()],
                ))
            }
            Emission::Place(Place::Constant {
                operand_type,
                index,
            }) => Ok((MemoryKind::CONSTANT, index, smallvec![operand_type])),
            Emission::Place(Place::Register(allocation)) => Ok((
                MemoryKind::REGISTER,
                allocation.base_index(),
                allocation.operand_types(),
            )),
            Emission::Instructions(operand_instructions) => {
                if let Some(registers) = &operand_instructions.target
                    && registers.is_temporary()
                {
                    self.free_temporary_registers(registers);
                }

                instructions.merge(operand_instructions);

                match &instructions.target {
                    Some(allocation) => Ok((
                        MemoryKind::REGISTER,
                        allocation.base_index(),
                        allocation.operand_types(),
                    )),
                    None => Err(CompileError::ExpectedValue {
                        file_id: operand.file_id(),
                        syntax_id: operand.id,
                    }),
                }
            }
            Emission::NativeFunction(_) => Err(CompileError::ExpectedNativeFunctionCall {
                position: operand.position(),
            }),
            Emission::Never => Err(CompileError::ExpectedValue {
                file_id: operand.file_id(),
                syntax_id: operand.id,
            }),
        }
    }

    fn handle_condition_emission(
        &mut self,
        instructions: &mut InstructionsEmission,
        emission: Emission,
        condition: &SyntaxReader,
        comparator: bool,
    ) -> Result<(), CompileError> {
        match emission {
            Emission::Value(ConstantValue::Boolean(boolean)) => {
                let test_instruction =
                    Instruction::test(comparator, MemoryKind::ENCODED, boolean as u16, 0);

                instructions.push(test_instruction);

                Ok(())
            }
            Emission::Place(Place::Constant { index, .. }) => {
                let test_instruction =
                    Instruction::test(comparator, MemoryKind::CONSTANT, index, 0);

                instructions.push(test_instruction);

                Ok(())
            }
            Emission::Place(Place::Register(allocation)) if allocation.len() == 1 => {
                let test_instruction =
                    Instruction::test(comparator, MemoryKind::REGISTER, allocation.base_index(), 0);

                instructions.push(test_instruction);

                Ok(())
            }
            Emission::Instructions(mut condition_instructions) => {
                let length = condition_instructions.length();

                if condition_instructions.length() >= 3 {
                    let condition_instruction =
                        &mut condition_instructions.instructions[length - 3].0;

                    match condition_instruction.operation() {
                        Operation::LESS | Operation::LESS_EQUAL | Operation::EQUAL => {
                            condition_instructions.instructions.truncate(length - 2);

                            if let Some(registers) = &condition_instructions.target
                                && registers.is_temporary()
                            {
                                self.free_temporary_registers(registers);
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
                                Instruction::test(comparator, operand_memory, operand_index, 1);

                            condition_instructions.instructions.truncate(length - 3);
                            condition_instructions.push(new_test_instruction);

                            if let Some(registers) = &condition_instructions.target
                                && registers.is_temporary()
                            {
                                self.free_temporary_registers(registers);
                            }
                        }
                        _ => {
                            let target_register_index = if let Some(allocation) =
                                &condition_instructions.target
                                && allocation.len() == 1
                            {
                                allocation.registers[0].index
                            } else {
                                return Err(CompileError::ExpectedBooleanExpression {
                                    found: *self.resolver.get_type_binding(&condition.id)?,
                                    node_kind: condition.node.kind,
                                    position: condition.position(),
                                });
                            };
                            let test_instruction = Instruction::test(
                                comparator,
                                MemoryKind::REGISTER,
                                target_register_index,
                                1,
                            );

                            condition_instructions.push(test_instruction);
                        }
                    }
                }

                instructions.merge(condition_instructions);

                Ok(())
            }
            _ => Err(CompileError::ExpectedBooleanExpression {
                found: *self.resolver.get_type_binding(&condition.id)?,
                node_kind: condition.node.kind,
                position: condition.position(),
            }),
        }
    }

    fn handle_branch_emission(
        &mut self,
        branch_emission: Emission,
        instructions: &mut InstructionsEmission,
        allocation: &RegisterAllocation,
        node: SyntaxReader,
    ) -> Result<(), CompileError> {
        match branch_emission {
            Emission::Value(constant) => {
                let destination = allocation.expect_single()?;
                let address = self.materialize_value(constant)?;
                let move_instruction = Instruction::r#move(
                    destination.index,
                    constant.operand_type(),
                    address.memory,
                    address.index,
                );

                instructions.push(move_instruction);
            }
            Emission::Place(Place::Constant {
                operand_type: r#type,
                index,
            }) => {
                let destination = allocation.expect_single()?;
                let move_instruction =
                    Instruction::r#move(destination.index, r#type, MemoryKind::CONSTANT, index);

                instructions.push(move_instruction);
            }
            Emission::Place(Place::Register(operand_registers)) => {
                let destination_registers = allocation.expect_multiple(operand_registers.len())?;

                for (destination, operand) in destination_registers
                    .iter()
                    .zip(operand_registers.registers)
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
            Emission::Never => {}
        }

        Ok(())
    }

    fn create_return_instructions(
        &mut self,
        expression_emission: Emission,
        syntax: &SyntaxReader,
    ) -> Result<InstructionsEmission, CompileError> {
        match expression_emission {
            Emission::Value(value) => {
                let allocation =
                    self.allocate_registers(self.return_type_id, AllocationKind::Reserved)?;
                let address = self.materialize_value(value)?;
                let move_instruction = Instruction::r#move(
                    allocation.base_index(),
                    value.operand_type(),
                    address.memory,
                    address.index,
                );

                Ok(InstructionsEmission::with_instruction_and_target(
                    move_instruction,
                    allocation,
                ))
            }
            Emission::Place(Place::Constant {
                operand_type,
                index,
            }) => {
                let allocation =
                    self.allocate_registers(self.return_type_id, AllocationKind::Reserved)?;
                let move_instruction = Instruction::r#move(
                    allocation.base_index(),
                    operand_type,
                    MemoryKind::CONSTANT,
                    index,
                );

                Ok(InstructionsEmission::with_instruction_and_target(
                    move_instruction,
                    allocation,
                ))
            }
            Emission::Place(Place::Register(emission_allocation)) => {
                let mut return_instructions = InstructionsEmission::new();
                let allocation =
                    self.allocate_registers(self.return_type_id, AllocationKind::Reserved)?;

                for (emission_register, target_register) in emission_allocation
                    .registers
                    .into_iter()
                    .zip(allocation.registers)
                {
                    let move_instruction = Instruction::r#move(
                        target_register.index,
                        emission_register.operand_type,
                        MemoryKind::REGISTER,
                        emission_register.index,
                    );

                    return_instructions.push(move_instruction);
                }

                Ok(return_instructions)
            }
            Emission::Instructions(instructions) => Ok(instructions),
            Emission::Never => {
                let return_instruction = Instruction::r#return();

                Ok(InstructionsEmission::with_instruction(return_instruction))
            }
            Emission::NativeFunction(_) => Err(CompileError::ExpectedNativeFunctionCall {
                position: syntax.position(),
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
            if child.node.kind.is_statement() {
                let Some(instructions) = self.visit_statement(child)? else {
                    continue;
                };

                self.handle_function_body_instructions(instructions)?;

                continue;
            }

            if index == child_count - 1 {
                let return_expression_emission =
                    self.visit_expression(child, Some(AllocationKind::Reserved))?;
                let return_instructions =
                    self.create_return_instructions(return_expression_emission, &child)?;

                self.handle_function_body_instructions(return_instructions)?;
            }

            if let Emission::Instructions(instructions) =
                self.visit_expression(child, Some(AllocationKind::Temporary))?
            {
                self.handle_function_body_instructions(instructions)?;
            }
        }

        Ok(())
    }
}

// impl<'a> SyntaxVisitor for Emitter<'a> {
//     type RootOutput = ();
//     type StatementOutput = InstructionsEmission;
//     type ExpressionInput = Option<AllocationKind>;
//     type ExpressionOutput = Emission;
//     type TypeOutput = ();
//     type PathInput = ();
//     type PathOutput = ();

//     fn visit_root(&mut self, _: SyntaxReader) -> Result<Self::RootOutput, CompileError> {
//         Ok(())
//     }

//     fn visit_module_item(&mut self, _: SyntaxReader<'_>) -> Result<(), CompileError> {
//         Ok(())
//     }

//     fn visit_function_item(&mut self, _: SyntaxReader<'_>) -> Result<(), CompileError> {
//         Ok(())
//     }

//     fn visit_use_item(&mut self, _: SyntaxReader<'_>) -> Result<(), CompileError> {
//         Ok(())
//     }

//     fn visit_struct_item(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
//         Ok(())
//     }

//     fn visit_enum_item(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
//         Ok(())
//     }

//     fn visit_const_item(&mut self, syntax: SyntaxReader) -> Result<(), CompileError> {
//         let ConstItem { name, value, .. } = syntax.as_component()?;

//         let expression_emission = self.visit_expression(value, None)?;

//         let declaration_id = *self.resolver.get_declaration_binding(&name.id)?;
//         let constant_value = if let Emission::Value(constant) = expression_emission {
//             constant
//         } else {
//             return Err(CompileError::ExpectedValue {
//                 file_id: value.file_id(),
//                 syntax_id: value.id,
//             });
//         };

//         self.resolver
//             .add_constant_item_value(declaration_id, constant_value);

//         Ok(())
//     }

//     fn visit_type_item(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
//         Ok(())
//     }

//     fn visit_impl_item(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
//         Ok(())
//     }

//     fn visit_impl_trait_item(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
//         Ok(())
//     }

//     fn visit_trait_item(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
//         Ok(())
//     }

//     fn visit_expression_statement(
//         &mut self,
//         syntax: SyntaxReader<'_>,
//     ) -> Result<Self::StatementOutput, CompileError> {
//         let ExpressionStatement { expression } = syntax.as_component()?;

//         let expression_emission =
//             self.visit_expression(expression, Some(AllocationKind::Temporary))?;

//         if let Emission::Instructions(mut instructions_emission) = expression_emission {
//             instructions_emission.set_target(None);

//             Ok(instructions_emission)
//         } else {
//             Ok(InstructionsEmission::new())
//         }
//     }

//     fn visit_let_statement(
//         &mut self,
//         syntax: SyntaxReader,
//     ) -> Result<Self::StatementOutput, CompileError> {
//         let LetStatement {
//             name, expression, ..
//         } = syntax.as_component()?;

//         let declaration_id = *self.resolver.get_declaration_binding(&name.id)?;
//         let emission = self.visit_expression(expression, Some(AllocationKind::Local))?;

//         match emission {
//             Emission::Instructions(InstructionsEmission {
//                 instructions,
//                 target,
//             }) => {
//                 let target = target.ok_or_else(|| CompileError::ExpectedValue {
//                     file_id: expression.file_id(),
//                     syntax_id: expression.id,
//                 })?;

//                 self.locals
//                     .insert(declaration_id, Local::Place(Place::Register(target)));

//                 Ok(InstructionsEmission {
//                     instructions,
//                     target: None,
//                 })
//             }
//             Emission::NativeFunction(_) => Err(CompileError::ExpectedNativeFunctionCall {
//                 position: syntax.position(),
//             }),
//             _ => Err(CompileError::InvalidEmission),
//         }
//     }

//     fn visit_assignment_expression(
//         &mut self,
//         reader: SyntaxReader<'_>,
//         _: Option<AllocationKind>,
//     ) -> Result<Self::ExpressionOutput, CompileError> {
//         let AssignmentExpression { target, source } = reader.as_component()?;

//         let mut assignment_instructions = InstructionsEmission::new();

//         let target_allocation = if target.node.kind == SyntaxKind::IndexExpression {
//             let target_emission = self.visit_index_expression(target, None)?;

//             match target_emission {
//                 Emission::Place(Place::Register(allocation)) => allocation,
//                 Emission::Instructions(InstructionsEmission {
//                     instructions,
//                     target: Some(allocation),
//                 }) => {
//                     assignment_instructions.extend(instructions);

//                     allocation
//                 }
//                 Emission::Instructions(_) => {
//                     return Err(CompileError::ExpectedValue {
//                         file_id: target.file_id(),
//                         syntax_id: target.id,
//                     });
//                 }
//                 _ => {
//                     return Err(CompileError::CannotMutate {
//                         position: target.position(),
//                     });
//                 }
//             }
//         } else {
//             let declaration_id = self.resolver.get_declaration_binding(&target.id)?;
//             let local = self.locals.get(declaration_id).ok_or_else(|| {
//                 CompileError::DeclarationOutOfScope {
//                     declaration_id: *declaration_id,
//                     usage_position: target.position(),
//                 }
//             })?;

//             if let Local::Place(Place::Register(allocation)) = local {
//                 allocation.clone()
//             } else {
//                 return Err(CompileError::CannotMutate {
//                     position: target.position(),
//                 });
//             }
//         };

//         let source_emission = self.visit_expression(source, None)?;

//         match source_emission {
//             Emission::Value(value) => {
//                 let operand_type = value.operand_type();
//                 let address = self.materialize_value(value)?;
//                 let move_instruction = Instruction::r#move(
//                     target_allocation.base_index(),
//                     operand_type,
//                     address.memory,
//                     address.index,
//                 );

//                 assignment_instructions.push(move_instruction);

//                 if operand_type == OperandType::POINTER {
//                     self.add_drop(address.index);
//                 }
//             }
//             Emission::Place(Place::Constant {
//                 operand_type,
//                 index,
//             }) => {
//                 for destination in &target_allocation.registers {
//                     let move_instruction = Instruction::r#move(
//                         destination.index,
//                         operand_type,
//                         MemoryKind::CONSTANT,
//                         index,
//                     );

//                     assignment_instructions.push(move_instruction);

//                     if destination.operand_type == OperandType::POINTER {
//                         self.add_drop(destination.index);
//                     }
//                 }
//             }
//             Emission::Place(Place::Register(operand_allocation)) => {
//                 for (destination, operand) in target_allocation
//                     .registers
//                     .into_iter()
//                     .zip(operand_allocation.registers)
//                 {
//                     let move_instruction = Instruction::r#move(
//                         destination.index,
//                         operand.operand_type,
//                         MemoryKind::REGISTER,
//                         operand.index,
//                     );

//                     assignment_instructions.push(move_instruction);
//                 }
//             }
//             Emission::Instructions(instructions) => {
//                 assignment_instructions.merge(instructions);
//                 assignment_instructions.set_target(None);
//             }
//             Emission::NativeFunction(_) => {
//                 return Err(CompileError::ExpectedNativeFunctionCall {
//                     position: reader.position(),
//                 });
//             }
//             Emission::Never => {
//                 return Err(CompileError::ExpectedValue {
//                     file_id: source.file_id(),
//                     syntax_id: source.id,
//                 });
//             }
//         }

//         Ok(Emission::Instructions(assignment_instructions))
//     }

//     fn visit_boolean_expression(
//         &mut self,
//         reader: SyntaxReader,
//         target_kind: Option<AllocationKind>,
//     ) -> Result<Self::ExpressionOutput, CompileError> {
//         let text = self.source.get_content(&reader.position())?;
//         let boolean = text == "true";
//     }

//     fn visit_hexadecimal_expression(
//         &mut self,
//         reader: SyntaxReader,
//         target_kind: Option<AllocationKind>,
//     ) -> Result<Self::ExpressionOutput, CompileError> {
//         let text = &self.source.get_content(&reader.position())?[2..];
//         let byte = create_u8_from_hexadecimal(text)?;

//         Ok(Emission::Value(ConstantValue::U8(byte)))
//     }

//     fn visit_character_expression(
//         &mut self,
//         reader: SyntaxReader,
//         _: (),
//     ) -> Result<Self::ExpressionOutput, CompileError> {
//         let text = self.source.get_content(&reader.position().shrink(1))?;
//         let character = create_char(text)?;

//         Ok(Emission::Value(ConstantValue::Character(character)))
//     }

//     fn visit_float_expression(
//         &mut self,
//         reader: SyntaxReader,
//         _: (),
//     ) -> Result<Self::ExpressionOutput, CompileError> {
//         debug!("Visting float expression");

//         let type_id = *self.resolver.get_type_binding(&reader.id)?;
//         let text = self.source.get_content(&reader.position())?;

//         match type_id {
//             TypeId::F_32 => {
//                 let float = create_f32_from_decimal(text)?;

//                 Ok(Emission::Value(ConstantValue::F32(float)))
//             }
//             TypeId::F_64 => {
//                 let float = create_f64_from_decimal(text)?;

//                 Ok(Emission::Value(ConstantValue::F64(float)))
//             }
//             _ => Err(CompileError::InvalidTypeBinding(type_id)),
//         }
//     }

//     fn visit_integer_expression(
//         &mut self,
//         reader: SyntaxReader,
//         _: (),
//     ) -> Result<Self::ExpressionOutput, CompileError> {
//         debug!("Visting integer expression");

//         let type_id = *self.resolver.get_type_binding(&reader.id)?;
//         let text = self.source.get_content(&reader.position())?;

//         match type_id {
//             TypeId::I_8 => Ok(Emission::Value(ConstantValue::I8(create_i8_from_decimal(
//                 text,
//             )?))),
//             TypeId::I_16 => Ok(Emission::Value(ConstantValue::I16(
//                 create_i16_from_decimal(text)?,
//             ))),
//             TypeId::I_32 => Ok(Emission::Value(ConstantValue::I32(
//                 create_i32_from_decimal(text)?,
//             ))),
//             TypeId::I_64 => Ok(Emission::Value(ConstantValue::I64(
//                 create_i64_from_decimal(text)?,
//             ))),
//             TypeId::I_128 => Ok(Emission::Value(ConstantValue::I128(
//                 create_i128_from_decimal(text)?,
//             ))),
//             #[cfg(target_pointer_width = "32")]
//             TypeId::I_SIZE => Ok(Emission::Value(ConstantValue::I32(
//                 create_i32_from_decimal(text)?,
//             ))),
//             #[cfg(target_pointer_width = "64")]
//             TypeId::I_SIZE => Ok(Emission::Value(ConstantValue::I64(
//                 create_i64_from_decimal(text)?,
//             ))),
//             TypeId::U_8 => Ok(Emission::Value(ConstantValue::U8(create_u8_from_decimal(
//                 text,
//             )?))),
//             TypeId::U_16 => Ok(Emission::Value(ConstantValue::U16(
//                 create_u16_from_decimal(text)?,
//             ))),
//             TypeId::U_32 => Ok(Emission::Value(ConstantValue::U32(
//                 create_u32_from_decimal(text)?,
//             ))),
//             TypeId::U_64 => Ok(Emission::Value(ConstantValue::U64(
//                 create_u64_from_decimal(text)?,
//             ))),
//             TypeId::U_128 => Ok(Emission::Value(ConstantValue::U128(
//                 create_u128_from_decimal(text)?,
//             ))),
//             #[cfg(target_pointer_width = "32")]
//             TypeId::U_SIZE => Ok(Emission::Value(ConstantValue::U32(
//                 create_u32_from_decimal(text)?,
//             ))),
//             #[cfg(target_pointer_width = "64")]
//             TypeId::U_SIZE => Ok(Emission::Value(ConstantValue::U64(
//                 create_u64_from_decimal(text)?,
//             ))),
//             _ => Err(CompileError::InvalidTypeBinding(type_id)),
//         }
//     }

//     fn visit_string_expression(
//         &mut self,
//         _: SyntaxReader,
//         _: (),
//     ) -> Result<Self::ExpressionOutput, CompileError> {
//         todo!()
//     }

//     fn visit_array_expression(
//         &mut self,
//         reader: SyntaxReader,
//         _: (),
//     ) -> Result<Self::ExpressionOutput, CompileError> {
//         let ArrayExpression { elements } = reader.as_component()?;

//         let (allocation_iter, temporary) = if let Some(allocation) = allocation {
//             (allocation.registers(), allocation.is_temporary())
//         } else {
//             let type_id = *self.resolver.get_type_binding(&reader.id)?;
//             let allocation_iter = self
//                 .allocate_registers(type_id, AllocationKind::Temporary)?
//                 .ok_or(CompileError::ExpectedValue {
//                     file_id: reader.file_id(),
//                     syntax_id: reader.id,
//                 })?
//                 .registers();

//             (allocation_iter, true)
//         };

//         let mut array_instructions = InstructionsEmission::new();

//         for (element, register) in elements.zip(allocation_iter) {
//             let element_target = RegisterAllocation::Single {
//                 register: *register,
//                 temporary,
//             };
//             let element_emission = self.visit_expression(element, &Some(element_target))?;

//             match element_emission {
//                 Emission::Instructions(instructions) => {
//                     array_instructions.merge(instructions);
//                 }
//                 Emission::Value(constant) => {
//                     let (memory_kind, operand_index) =
//                         if let Some(encoded) = constant.as_encoded_u16() {
//                             (MemoryKind::ENCODED, encoded)
//                         } else {
//                             (MemoryKind::CONSTANT, self.add_constant(constant))
//                         };
//                     let move_instruction = Instruction::r#move(
//                         register.index,
//                         constant.operand_type(),
//                         memory_kind,
//                         operand_index,
//                     );

//                     array_instructions.push(move_instruction);
//                 }
//                 Emission::Place(place) => match place {
//                     Place::Constant {
//                         operand_type,
//                         index,
//                     } => {
//                         let move_instruction = Instruction::r#move(
//                             register.index,
//                             operand_type,
//                             MemoryKind::CONSTANT,
//                             index,
//                         );

//                         array_instructions.push(move_instruction);
//                     }
//                     Place::Register(RegisterAllocation::Single { register, .. }) => {
//                         let move_instruction = Instruction::r#move(
//                             register.index,
//                             register.operand_type,
//                             MemoryKind::REGISTER,
//                             register.index,
//                         );

//                         array_instructions.push(move_instruction);
//                     }
//                     Place::Register(RegisterAllocation::Multiple { registers, .. }) => {
//                         for register in registers {
//                             let move_instruction = Instruction::r#move(
//                                 register.index,
//                                 register.operand_type,
//                                 MemoryKind::REGISTER,
//                                 register.index,
//                             );

//                             array_instructions.push(move_instruction);
//                         }
//                     }
//                 },
//                 Emission::Never | Emission::NativeFunction(_) => {}
//             }
//         }

//         array_instructions.set_target(Some(allocation));

//         Ok(Emission::Instructions(array_instructions))
//     }

//     fn visit_array_repeat_expression(
//         &mut self,
//         reader: SyntaxReader,
//         target_kind: AllocationKind,
//     ) -> Result<Self::ExpressionOutput, CompileError> {
//         let ArrayRepeatExpression { element, .. } = reader.as_component()?;

//         let array_type_id = *self.resolver.get_type_binding(&reader.id)?;
//         let array_length =
//             if let Type::Array { length, .. } = *self.resolver.types.get_type(array_type_id)? {
//                 length
//             } else {
//                 return Err(CompileError::ExpectedArrayType(array_type_id));
//             };
//         let element_type_id = *self.resolver.get_type_binding(&element.id)?;
//         let mut array_instructions = InstructionsEmission::new();

//         for register in &mut registers {
//             let element_emission = self.visit_expression(element, &Some(element_target))?;

//             match element_emission {
//                 Emission::Instructions(instructions) => {
//                     array_instructions.merge(instructions);
//                 }
//                 Emission::Value(constant) => {
//                     let (memory_kind, operand_index) =
//                         if let Some(encoded) = constant.as_encoded_u16() {
//                             (MemoryKind::ENCODED, encoded)
//                         } else {
//                             (MemoryKind::CONSTANT, self.add_constant(constant))
//                         };
//                     let move_instruction = Instruction::r#move(
//                         register.index,
//                         constant.operand_type(),
//                         memory_kind,
//                         operand_index,
//                     );

//                     array_instructions.push(move_instruction);
//                 }
//                 Emission::Place(place) => match place {
//                     Place::Constant {
//                         operand_type,
//                         index,
//                     } => {
//                         let move_instruction = Instruction::r#move(
//                             register.index,
//                             operand_type,
//                             MemoryKind::CONSTANT,
//                             index,
//                         );

//                         array_instructions.push(move_instruction);
//                     }
//                     Place::Register(RegisterAllocation::Single { register, .. }) => {
//                         let move_instruction = Instruction::r#move(
//                             register.index,
//                             register.operand_type,
//                             MemoryKind::REGISTER,
//                             register.index,
//                         );

//                         array_instructions.push(move_instruction);
//                     }
//                     Place::Register(RegisterAllocation::Multiple { registers, .. }) => {
//                         for register in registers {
//                             let move_instruction = Instruction::r#move(
//                                 register.index,
//                                 register.operand_type,
//                                 MemoryKind::REGISTER,
//                                 register.index,
//                             );

//                             array_instructions.push(move_instruction);
//                         }
//                     }
//                 },
//                 Emission::Never | Emission::NativeFunction(_) => {}
//             }
//         }

//         array_instructions.set_target(Some(allocation));

//         Ok(Emission::Instructions(array_instructions))
//     }

//     fn visit_index_expression(
//         &mut self,
//         reader: SyntaxReader,
//         allocation: Option<AllocationKind>,
//     ) -> Result<Self::ExpressionOutput, CompileError> {
//         let IndexExpression { collection, index } = reader.as_component()?;

//         let list_emission = self.visit_expression(collection, None)?;

//         let (list_registers, list_instructions) = match list_emission {
//             Emission::Place(Place::Register(registers)) => (registers, None),
//             Emission::Instructions(instructions) => {
//                 let registers = instructions
//                     .target
//                     .clone()
//                     .ok_or(CompileError::ExpectedValue {
//                         node_kind: collection.node.kind,
//                         position: collection.position(),
//                     })?;

//                 (registers, Some(instructions))
//             }
//             _ => {
//                 return Err(CompileError::ExpectedValue {
//                     node_kind: collection.node.kind,
//                     position: collection.position(),
//                 });
//             }
//         };

//         let list_type_id = *self.resolver.get_type_binding(&collection.id)?;
//         let list_type = *self.resolver.types.get_type(list_type_id)?;

//         let (element_type_id, array_length) = match list_type {
//             Type::Array {
//                 element_type_id,
//                 length,
//             } => (element_type_id, length),
//             _ => {
//                 return Err(CompileError::CannotIndex {
//                     type_id: list_type_id,
//                     position: collection.position(),
//                 });
//             }
//         };

//         let element_operand_types = self.resolver.get_operand_types(element_type_id)?;
//         let element_register_count = element_operand_types.len();

//         if matches!(
//             index.node.kind,
//             SyntaxKind::RangeExpression | SyntaxKind::RangeInclusiveExpression
//         ) {
//             let RangeExpression {
//                 start: range_start,
//                 end: range_end,
//             } = index.as_component()?;

//             let range_start_str = self.source.get_content(&range_start.position())?;
//             let range_end_str = self.source.get_content(&range_end.position())?;

//             let start_index = create_usize_from_decimal(range_start_str)?;
//             let end_index = create_usize_from_decimal(range_end_str)?;

//             let slice_end = if index.node.kind == SyntaxKind::RangeInclusiveExpression {
//                 end_index + 1
//             } else {
//                 end_index
//             };

//             if start_index > slice_end || slice_end > array_length {
//                 return Err(CompileError::IndexOutOfBounds {
//                     index: slice_end,
//                     length: array_length,
//                     position: index.position(),
//                 });
//             }

//             let start_register_offset = start_index * element_register_count;
//             let slice_register_count = (slice_end - start_index) * element_register_count;

//             let slice_registers: SmallVec<[RegisterClaim; 4]> = list_registers
//                 .registers()
//                 .skip(start_register_offset)
//                 .take(slice_register_count)
//                 .copied()
//                 .collect();

//             let slice_allocation = match slice_registers.len() {
//                 1 => RegisterAllocation::Single {
//                     register: slice_registers[0],
//                     temporary: false,
//                 },
//                 _ => RegisterAllocation::Multiple {
//                     registers: slice_registers,
//                     temporary: false,
//                 },
//             };

//             if let Some(mut instructions) = list_instructions {
//                 instructions.set_target(Some(slice_allocation));

//                 return Ok(Emission::Instructions(instructions));
//             }

//             return Ok(Emission::Place(Place::Register(slice_allocation)));
//         }

//         if index.node.kind == SyntaxKind::IntegerExpression {
//             let index_str = self.source.get_content(&index.position())?;
//             let constant_index = create_usize_from_decimal(index_str)?;

//             if constant_index >= array_length {
//                 return Err(CompileError::IndexOutOfBounds {
//                     index: constant_index,
//                     length: array_length,
//                     position: index.position(),
//                 });
//             }

//             let register_offset = constant_index * element_register_count;

//             let element_registers: SmallVec<[RegisterClaim; 4]> = list_registers
//                 .registers()
//                 .skip(register_offset)
//                 .take(element_register_count)
//                 .copied()
//                 .collect();

//             let element_allocation = match element_registers.len() {
//                 1 => RegisterAllocation::Single {
//                     register: element_registers[0],
//                     temporary: false,
//                 },
//                 _ => RegisterAllocation::Multiple {
//                     registers: element_registers,
//                     temporary: false,
//                 },
//             };

//             if let Some(mut instructions) = list_instructions {
//                 instructions.set_target(Some(element_allocation));

//                 return Ok(Emission::Instructions(instructions));
//             }

//             return Ok(Emission::Place(Place::Register(element_allocation)));
//         }

//         let index_emission = self.visit_expression(index, None)?;

//         let (index_memory, index_index) = if let Emission::Value(constant) = &index_emission
//             && let Some(encoded) = constant.as_encoded_u16()
//         {
//             (MemoryKind::ENCODED, encoded)
//         } else {
//             let index_place = self.place_emission(index_emission, &index)?;

//             match index_place {
//                 Place::Constant { index, .. } => (MemoryKind::CONSTANT, index),
//                 Place::Register(RegisterAllocation::Single { register, .. }) => {
//                     (MemoryKind::REGISTER, register.index)
//                 }
//                 _ => {
//                     return Err(CompileError::ExpectedIntegerIndex {
//                         found: *self.resolver.get_type_binding(&index.id)?,
//                         position: index.position(),
//                     });
//                 }
//             }
//         };

//         let base_index = list_registers.base_index();

//         let element_type_id = *self.resolver.get_type_binding(&reader.id)?;

//         let destination = if let Some(allocation) = allocation {
//             allocation
//         } else {
//             self.allocate_registers(element_type_id, AllocationKind::Temporary, &reader)?
//         };

//         let destination_register = destination.expect_single()?;

//         let mut index_instructions = if let Some(instructions) = list_instructions {
//             instructions
//         } else {
//             InstructionsEmission::new()
//         };

//         let check_index_instruction =
//             Instruction::check_index(index_memory, index_index, array_length as u16);

//         index_instructions.push(check_index_instruction);

//         let get_index_instruction = Instruction::get_index(
//             destination_register.index,
//             destination_register.operand_type,
//             base_index,
//             index_memory,
//             index_index,
//         );

//         index_instructions.push(get_index_instruction);
//         index_instructions.set_target(Some(destination));

//         Ok(Emission::Instructions(index_instructions))
//     }

//     fn visit_range_expression(
//         &mut self,
//         reader: SyntaxReader,
//         allocation: Option<AllocationKind>,
//     ) -> Result<Self::ExpressionOutput, CompileError> {
//         let RangeExpression { start, end } = reader.as_component()?;

//         let allocation = if let Some(allocation) = allocation {
//             allocation
//         } else {
//             let type_id = *self.resolver.get_type_binding(&reader.id)?;

//             self.allocate_registers(type_id, AllocationKind::Temporary, &reader)?
//         };

//         let mut range_instructions = InstructionsEmission::new();

//         for (field_expression, destination) in [start, end].into_iter().zip(allocation.iter()) {
//             let field_emission = self.visit_expression(field_expression, None)?;

//             if let Emission::Value(constant) = &field_emission
//                 && let Some(encoded) = constant.as_encoded_u16()
//             {
//                 let move_instruction = Instruction::r#move(
//                     destination.index,
//                     constant.operand_type(),
//                     MemoryKind::ENCODED,
//                     encoded,
//                 );

//                 range_instructions.push(move_instruction);
//                 continue;
//             }

//             let field_place = self.place_emission(field_emission, &field_expression)?;

//             match field_place {
//                 Place::Constant {
//                     operand_type,
//                     index,
//                 } => {
//                     let move_instruction = Instruction::r#move(
//                         destination.index,
//                         operand_type,
//                         MemoryKind::CONSTANT,
//                         index,
//                     );

//                     range_instructions.push(move_instruction);
//                 }
//                 Place::Register(RegisterAllocation::Single { register, .. }) => {
//                     let move_instruction = Instruction::r#move(
//                         destination.index,
//                         register.operand_type,
//                         MemoryKind::REGISTER,
//                         register.index,
//                     );

//                     range_instructions.push(move_instruction);
//                 }
//                 Place::Register(RegisterAllocation::Multiple { registers, .. }) => {
//                     for register in registers {
//                         let move_instruction = Instruction::r#move(
//                             destination.index,
//                             register.operand_type,
//                             MemoryKind::REGISTER,
//                             register.index,
//                         );

//                         range_instructions.push(move_instruction);
//                     }
//                 }
//             }
//         }

//         range_instructions.set_target(Some(allocation));

//         Ok(Emission::Instructions(range_instructions))
//     }

//     fn visit_path_expression(
//         &mut self,
//         reader: SyntaxReader,
//         allocation: Option<AllocationKind>,
//     ) -> Result<Self::ExpressionOutput, CompileError> {
//         let declaration_id = *self.resolver.get_declaration_binding(&reader.id)?;
//         let declaration = self.resolver.declarations.get_declaration(declaration_id)?;

//         if let Definition::Variant {
//             discriminant,
//             fields,
//             ..
//         } = declaration.definition
//             && fields.is_empty()
//         {
//             let type_id = *self.resolver.get_type_binding(&reader.id)?;
//             let allocation = if let Some(allocation) = allocation {
//                 allocation
//             } else {
//                 self.allocate_registers(type_id, AllocationKind::Temporary, &reader)?
//             };
//             let destination = allocation.index();
//             let move_instruction = Instruction::r#move(
//                 destination,
//                 OperandType::U_16,
//                 MemoryKind::ENCODED,
//                 discriminant,
//             );

//             let mut instructions = InstructionsEmission::new();

//             instructions.push(move_instruction);
//             instructions.set_target(Some(allocation));

//             return Ok(Emission::Instructions(instructions));
//         }

//         if let Some(local) = self.locals.get(&declaration_id) {
//             return match local {
//                 Local::Place(place) => Ok(Emission::Place(place.clone())),
//                 Local::Constant(value) => Ok(Emission::Value(*value)),
//             };
//         }

//         let declaration = self.resolver.declarations.get_declaration(declaration_id)?;

//         let place = match declaration.definition {
//             Definition::Function {
//                 type_parameters, ..
//             } => {
//                 let concrete_type_arguments = if !type_parameters.is_empty() {
//                     let type_id = *self.resolver.get_type_binding(&reader.id)?;
//                     let callee_type = *self.resolver.types.get_type(type_id)?;

//                     if let Type::FunctionDefinition { type_arguments, .. } = callee_type {
//                         type_arguments
//                             .as_range()
//                             .map(|index| {
//                                 let type_id = *self.resolver.types.get_type_member(index)?;

//                                 self.resolver.resolve_type(type_id)
//                             })
//                             .try_collect::<SmallVec<[TypeId; 4]>>()?
//                     } else {
//                         SmallVec::new()
//                     }
//                 } else {
//                     SmallVec::new()
//                 };

//                 let cache_key = (declaration_id, concrete_type_arguments);
//                 let prototype_id =
//                     if let Some(existing) = self.resolver.get_cached_prototype(&cache_key) {
//                         existing
//                     } else {
//                         let reserved = self.prototypes.reserve();

//                         self.resolver.cache_prototype(cache_key, reserved);
//                         self.compilation_stack.push(CompilationRequest {
//                             declaration_id,
//                             prototype_id: reserved,
//                         });

//                         reserved
//                     };

//                 Place::Constant {
//                     operand_type: OperandType::FUNCTION,
//                     index: prototype_id.inner(),
//                 }
//             }
//             Definition::Constant { .. } => {
//                 let value = self
//                     .resolver
//                     .get_constant_item_value(&declaration_id)
//                     .ok_or_else(|| CompileError::ExpectedValue {
//                         node_kind: reader.node.kind,
//                         position: reader.position(),
//                     })?;

//                 return Ok(Emission::Value(value));
//             }
//             _ => {
//                 return Err(CompileError::ExpectedValue {
//                     node_kind: reader.node.kind,
//                     position: reader.position(),
//                 });
//             }
//         };

//         Ok(Emission::Place(place))
//     }

//     fn visit_struct_expression(
//         &mut self,
//         reader: SyntaxReader,
//         allocation: Option<AllocationKind>,
//     ) -> Result<Self::ExpressionOutput, CompileError> {
//         debug!("Visting struct expression");

//         let (_, struct_fields) = reader.binary_children()?;

//         let allocation = if let Some(allocation) = allocation {
//             allocation
//         } else {
//             let type_id = *self.resolver.get_type_binding(&reader.id)?;

//             self.allocate_registers(type_id, AllocationKind::Temporary, &reader)?
//         };

//         let mut struct_instructions = InstructionsEmission::new();

//         let fields_and_registers = struct_fields
//             .children()
//             .array_chunks::<2>()
//             .zip(allocation.iter());

//         for ([_, field_expression], destination) in fields_and_registers {
//             let field_emission = self.visit_expression(field_expression, None)?;

//             if let Emission::Value(constant) = &field_emission
//                 && let Some(encoded) = constant.as_encoded_u16()
//             {
//                 let move_instruction = Instruction::r#move(
//                     destination.index,
//                     constant.operand_type(),
//                     MemoryKind::ENCODED,
//                     encoded,
//                 );

//                 struct_instructions.push(move_instruction);
//                 continue;
//             }

//             let field_place = self.place_emission(field_emission, &field_expression)?;

//             match field_place {
//                 Place::Constant {
//                     operand_type,
//                     index,
//                 } => {
//                     let move_instruction = Instruction::r#move(
//                         destination.index,
//                         operand_type,
//                         MemoryKind::CONSTANT,
//                         index,
//                     );

//                     struct_instructions.push(move_instruction);
//                 }
//                 Place::Register(RegisterAllocation::Single { register, .. }) => {
//                     let move_instruction = Instruction::r#move(
//                         destination.index,
//                         register.operand_type,
//                         MemoryKind::REGISTER,
//                         register.index,
//                     );

//                     struct_instructions.push(move_instruction);
//                 }
//                 Place::Register(RegisterAllocation::Multiple { registers, .. }) => {
//                     for register in registers {
//                         let move_instruction = Instruction::r#move(
//                             destination.index,
//                             register.operand_type,
//                             MemoryKind::REGISTER,
//                             register.index,
//                         );

//                         struct_instructions.push(move_instruction);
//                     }
//                 }
//             }
//         }

//         struct_instructions.set_target(Some(allocation));

//         Ok(Emission::Instructions(struct_instructions))
//     }

//     fn visit_grouped_expression(
//         &mut self,
//         reader: SyntaxReader,
//         input: (),
//     ) -> Result<Self::ExpressionOutput, CompileError> {
//         let child = reader.children().expect_next()?;

//         self.visit_expression(child, input)
//     }

//     fn visit_block_expression(
//         &mut self,
//         reader: SyntaxReader<'_>,
//         allocation: Option<AllocationKind>,
//     ) -> Result<Self::ExpressionOutput, CompileError> {
//         debug!("Visting block expression");

//         let children = reader.children();

//         let block_scope_id = *self.resolver.get_scope_binding(&reader.id)?;
//         let parent_scope_id = self.current_scope_id;
//         let parent_scope_tracker = self.register_tracker;

//         self.enter_child_scope(block_scope_id);

//         let child_count = children.len();
//         let mut block_instructions = InstructionsEmission::new();

//         for (index, child) in children.enumerate() {
//             let is_last = index == child_count - 1;

//             if child.is_statement() {
//                 if let Some(instructions) = self.visit_statement(child)? {
//                     block_instructions.merge(instructions);
//                 }

//                 continue;
//             }

//             if !is_last {
//                 let expression_emission = self.visit_expression(child, None)?;

//                 if let Emission::Instructions(expression_instructions) = expression_emission {
//                     block_instructions.merge(expression_instructions);
//                 }

//                 continue;
//             }

//             let mut last_emission = self.visit_expression(child, allocation)?;

//             if block_instructions.is_empty() {
//                 return Ok(last_emission);
//             }

//             let allocation = if let Some(allocation) = last_emission.take_allocation() {
//                 allocation
//             } else {
//                 let type_id = *self.resolver.get_type_binding(&child.id)?;

//                 self.allocate_registers(type_id, AllocationKind::Temporary, &child)?
//             };

//             match last_emission {
//                 Emission::Value(constant) => {
//                     let destination = allocation.expect_single()?;
//                     let operand_type = constant.operand_type();
//                     let (memory_kind, operand_index) =
//                         if let Some(encoded) = constant.as_encoded_u16() {
//                             (MemoryKind::ENCODED, encoded)
//                         } else {
//                             (MemoryKind::CONSTANT, self.add_constant(constant))
//                         };
//                     let move_instruction = Instruction::r#move(
//                         destination.index,
//                         operand_type,
//                         memory_kind,
//                         operand_index,
//                     );

//                     block_instructions.push(move_instruction);
//                     block_instructions.set_target(Some(allocation));
//                 }
//                 Emission::Place(Place::Constant {
//                     operand_type: r#type,
//                     index,
//                 }) => {
//                     let destination = allocation.expect_single()?;
//                     let move_instruction =
//                         Instruction::r#move(destination.index, r#type, MemoryKind::CONSTANT, index);

//                     block_instructions.push(move_instruction);
//                     block_instructions.set_target(Some(allocation));
//                 }
//                 Emission::Place(Place::Register(RegisterAllocation::Single {
//                     register: operand_register,
//                     ..
//                 })) => {
//                     let destination = allocation.expect_single()?;
//                     let move_instruction = Instruction::r#move(
//                         destination.index,
//                         destination.operand_type,
//                         MemoryKind::REGISTER,
//                         operand_register.index,
//                     );

//                     block_instructions.push(move_instruction);
//                     block_instructions.set_target(Some(allocation));
//                 }
//                 Emission::Place(Place::Register(RegisterAllocation::Multiple {
//                     registers: operand_registers,
//                     ..
//                 })) => {
//                     let (destinations, _) = allocation.expect_multiple(operand_registers.len())?;

//                     for (destination, operand) in destinations.iter().zip(operand_registers) {
//                         let move_instruction = Instruction::r#move(
//                             destination.index,
//                             destination.operand_type,
//                             MemoryKind::REGISTER,
//                             operand.index,
//                         );

//                         block_instructions.push(move_instruction);
//                     }

//                     block_instructions.set_target(Some(allocation));
//                 }
//                 Emission::Instructions(instructions) => {
//                     block_instructions.merge(instructions);
//                 }
//                 Emission::NativeFunction(_) => {
//                     return Err(CompileError::ExpectedNativeFunctionCall {
//                         position: reader.position(),
//                     });
//                 }
//                 Emission::Never => {}
//             }

//             break;
//         }

//         self.enter_parent_scope(parent_scope_id, parent_scope_tracker);
//         self.handle_drops(&mut block_instructions);

//         Ok(Emission::Instructions(block_instructions))
//     }

//     fn visit_if_expression(
//         &mut self,
//         reader: SyntaxReader<'_>,
//         allocation: Option<AllocationKind>,
//     ) -> Result<Self::ExpressionOutput, CompileError> {
//         debug!("Visting if expression");

//         let mut children = reader.children();
//         let condition = children.expect_next()?;
//         let then_block = children.expect_next()?;
//         let else_block = children.next();

//         let mut if_instructions = InstructionsEmission::new();

//         let condition_emission = self.visit_expression(condition, None)?;

//         self.handle_condition_emission(&mut if_instructions, condition_emission, &condition, true)?;

//         let allocation = if let Some(allocation) = allocation {
//             Some(allocation)
//         } else {
//             let type_id = *self.resolver.get_type_binding(&reader.id)?;

//             if type_id == TypeId::UNIT || type_id == TypeId::NEVER {
//                 None
//             } else {
//                 Some(self.allocate_registers(type_id, AllocationKind::Temporary, &reader)?)
//             }
//         };
//         let jump_over_then_id = self.create_jump_id();
//         let start_else_anchor_count = self.jump_over_branch_ids.len();

//         if_instructions.push_drop_anchor(JumpAnchor::ForwardFromHere {
//             id: jump_over_then_id,
//         });

//         if let Some(allocation) = allocation {
//             let target_clone = allocation.clone();
//             let mut then_emission = self.visit_block_expression(then_block, Some(allocation))?;
//             let allocation = then_emission.take_allocation().unwrap_or(target_clone);

//             self.handle_branch_emission(
//                 then_emission,
//                 &mut if_instructions,
//                 &allocation,
//                 then_block,
//             )?;

//             if let Some(else_block) = else_block {
//                 let jump_over_else_id = self.create_jump_id();
//                 self.jump_over_branch_ids.push(jump_over_else_id);

//                 if_instructions.push(Instruction::no_op());
//                 if_instructions.push_drop_anchor(JumpAnchor::ForwardFromHere {
//                     id: jump_over_else_id,
//                 });
//                 if_instructions.push_drop_anchor(JumpAnchor::ForwardToNext {
//                     id: jump_over_then_id,
//                 });

//                 let target_clone = allocation.clone();
//                 let mut else_emission =
//                     self.visit_block_expression(else_block, Some(allocation))?;
//                 let allocation = else_emission.take_allocation().unwrap_or(target_clone);

//                 self.handle_branch_emission(
//                     else_emission,
//                     &mut if_instructions,
//                     &allocation,
//                     else_block,
//                 )?;

//                 if_instructions.set_target(Some(allocation));
//             } else {
//                 if_instructions.push_drop_anchor(JumpAnchor::ForwardToNext {
//                     id: jump_over_then_id,
//                 });
//                 if_instructions.set_target(Some(allocation));
//             }
//         } else {
//             let then_emission = self.visit_block_expression(then_block, None)?;

//             if let Emission::Instructions(then_instructions) = then_emission {
//                 if_instructions.merge(then_instructions);
//             }

//             if_instructions.push_drop_anchor(JumpAnchor::ForwardToNext {
//                 id: jump_over_then_id,
//             });

//             if let Some(else_block) = else_block {
//                 let else_emission = self.visit_block_expression(else_block, None)?;

//                 if let Emission::Instructions(else_instructions) = else_emission {
//                     if_instructions.merge(else_instructions);
//                 }
//             }
//         }

//         let end_else_anchor_count = self.jump_over_branch_ids.len();

//         for index in start_else_anchor_count..end_else_anchor_count {
//             let jump_id = self.jump_over_branch_ids[index];

//             if_instructions.push_drop_anchor(JumpAnchor::ForwardToNext { id: jump_id });
//         }

//         Ok(Emission::Instructions(if_instructions))
//     }

//     fn visit_math_expression(
//         &mut self,
//         reader: SyntaxReader,
//         allocation: Option<AllocationKind>,
//     ) -> Result<Self::ExpressionOutput, CompileError> {
//         let MathExpression { left, right } = reader.as_component()?;

//         let mut left_emission = self.visit_expression(left, None)?;
//         let right_emission = self.visit_expression(right, None)?;

//         if let (Emission::Value(left_value), Emission::Value(right_value)) =
//             (&left_emission, &right_emission)
//         {
//             let combined = self.fold_values(&reader, *left_value, &left, *right_value, &right)?;

//             return Ok(Emission::Value(combined));
//         }

//         let mut math_emission = InstructionsEmission::new();

//         let place_target = if let Emission::Place(Place::Register(allocation)) = &left_emission {
//             Some(allocation.clone())
//         } else {
//             None
//         };
//         let left_target = left_emission.take_allocation().or(place_target);
//         let (left_memory, left_index, _) = self.handle_operand_emission(
//             &mut math_emission,
//             left_emission,
//             reader.node.kind,
//             &left,
//         )?;
//         let (right_memory, right_index, _) = self.handle_operand_emission(
//             &mut math_emission,
//             right_emission,
//             reader.node.kind,
//             &right,
//         )?;

//         let type_id = *self.resolver.get_type_binding(&reader.id)?;
//         let mut handle_target_register =
//             |allocation, node| -> Result<RegisterClaim, CompileError> {
//                 let allocation = if let Some(allocation) = allocation {
//                     allocation
//                 } else {
//                     self.allocate_registers(type_id, AllocationKind::Temporary, node)?
//                 };

//                 let register = allocation.expect_single()?;

//                 math_emission.set_target(Some(allocation));

//                 Ok(register)
//             };

//         let math_instruction = match reader.node.kind {
//             SyntaxKind::AdditionExpression => {
//                 let register = handle_target_register(allocation, &reader)?;

//                 Instruction::add(
//                     register.index,
//                     register.operand_type,
//                     left_memory,
//                     left_index,
//                     right_memory,
//                     right_index,
//                 )
//             }
//             SyntaxKind::AdditionAssignmentExpression => {
//                 let regsiter = handle_target_register(left_target, &left)?;

//                 Instruction::add(
//                     left_index,
//                     regsiter.operand_type,
//                     left_memory,
//                     left_index,
//                     right_memory,
//                     right_index,
//                 )
//             }
//             SyntaxKind::SubtractionExpression => {
//                 let register = handle_target_register(allocation, &reader)?;

//                 Instruction::subtract(
//                     register.index,
//                     register.operand_type,
//                     left_memory,
//                     left_index,
//                     right_memory,
//                     right_index,
//                 )
//             }
//             SyntaxKind::SubtractionAssignmentExpression => {
//                 let register = handle_target_register(left_target, &left)?;

//                 Instruction::subtract(
//                     register.index,
//                     register.operand_type,
//                     left_memory,
//                     left_index,
//                     right_memory,
//                     right_index,
//                 )
//             }
//             SyntaxKind::MultiplicationExpression => {
//                 let register = handle_target_register(allocation, &reader)?;

//                 Instruction::multiply(
//                     register.index,
//                     register.operand_type,
//                     left_memory,
//                     left_index,
//                     right_memory,
//                     right_index,
//                 )
//             }
//             SyntaxKind::MultiplicationAssignmentExpression => {
//                 let register = handle_target_register(left_target, &left)?;

//                 Instruction::multiply(
//                     register.index,
//                     register.operand_type,
//                     left_memory,
//                     left_index,
//                     right_memory,
//                     right_index,
//                 )
//             }
//             SyntaxKind::DivisionExpression => {
//                 let register = handle_target_register(allocation, &reader)?;

//                 Instruction::divide(
//                     register.index,
//                     register.operand_type,
//                     left_memory,
//                     left_index,
//                     right_memory,
//                     right_index,
//                 )
//             }
//             SyntaxKind::DivisionAssignmentExpression => {
//                 let register = handle_target_register(left_target, &left)?;

//                 Instruction::divide(
//                     register.index,
//                     register.operand_type,
//                     left_memory,
//                     left_index,
//                     right_memory,
//                     right_index,
//                 )
//             }
//             SyntaxKind::ModuloExpression => {
//                 let register = handle_target_register(allocation, &reader)?;

//                 Instruction::modulo(
//                     register.index,
//                     register.operand_type,
//                     left_memory,
//                     left_index,
//                     right_memory,
//                     right_index,
//                 )
//             }
//             SyntaxKind::ModuloAssignmentExpression => {
//                 let register = handle_target_register(left_target, &left)?;

//                 Instruction::modulo(
//                     register.index,
//                     register.operand_type,
//                     left_memory,
//                     left_index,
//                     right_memory,
//                     right_index,
//                 )
//             }
//             SyntaxKind::ExponentExpression => {
//                 let register = handle_target_register(allocation, &reader)?;

//                 Instruction::power(
//                     register.index,
//                     register.operand_type,
//                     left_memory,
//                     left_index,
//                     right_memory,
//                     right_index,
//                 )
//             }
//             SyntaxKind::ExponentAssignmentExpression => {
//                 let register = handle_target_register(left_target, &left)?;

//                 Instruction::power(
//                     register.index,
//                     register.operand_type,
//                     left_memory,
//                     left_index,
//                     right_memory,
//                     right_index,
//                 )
//             }
//             _ => {
//                 return Err(CompileError::UnexpectedSyntaxKind {
//                     expected: &[
//                         SyntaxKind::AdditionExpression,
//                         SyntaxKind::AdditionAssignmentExpression,
//                         SyntaxKind::SubtractionExpression,
//                         SyntaxKind::SubtractionAssignmentExpression,
//                         SyntaxKind::MultiplicationExpression,
//                         SyntaxKind::MultiplicationAssignmentExpression,
//                         SyntaxKind::DivisionExpression,
//                         SyntaxKind::DivisionAssignmentExpression,
//                         SyntaxKind::ModuloExpression,
//                         SyntaxKind::ModuloAssignmentExpression,
//                         SyntaxKind::ExponentExpression,
//                         SyntaxKind::ExponentAssignmentExpression,
//                     ],
//                     found: reader.node.kind,
//                 });
//             }
//         };

//         math_emission.push(math_instruction);

//         Ok(Emission::Instructions(math_emission))
//     }

//     fn visit_comparison_expression(
//         &mut self,
//         reader: SyntaxReader,
//         allocation: Option<AllocationKind>,
//     ) -> Result<Self::ExpressionOutput, CompileError> {
//         let ComparisonExpression { left, right } = reader.as_component()?;

//         let left_emission = self.visit_expression(left, None)?;
//         let right_emission = self.visit_expression(right, None)?;

//         if let Emission::Value(left_constant) = left_emission
//             && let Emission::Value(right_constant) = right_emission
//         {
//             let combined =
//                 self.fold_values(&reader, left_constant, &left, right_constant, &right)?;

//             return Ok(Emission::Value(combined));
//         }

//         let mut comparison_emission = InstructionsEmission::new();

//         let (left_memory, left_index, left_operand_type) = self.handle_operand_emission(
//             &mut comparison_emission,
//             left_emission,
//             reader.node.kind,
//             &left,
//         )?;
//         let (right_memory, right_index, _) = self.handle_operand_emission(
//             &mut comparison_emission,
//             right_emission,
//             reader.node.kind,
//             &right,
//         )?;

//         let allocation = if let Some(allocation) = allocation {
//             allocation
//         } else {
//             let type_id = *self.resolver.get_type_binding(&reader.id)?;

//             self.allocate_registers(type_id, AllocationKind::Temporary, &reader)?
//         };
//         let register = allocation.expect_single()?;
//         let comparison_instruction = match reader.node.kind {
//             SyntaxKind::EqualExpression => Instruction::equal(
//                 true,
//                 left_operand_type,
//                 left_memory,
//                 left_index,
//                 right_memory,
//                 right_index,
//             ),
//             SyntaxKind::NotEqualExpression => Instruction::equal(
//                 false,
//                 left_operand_type,
//                 left_memory,
//                 left_index,
//                 right_memory,
//                 right_index,
//             ),
//             SyntaxKind::LessThanExpression => Instruction::less(
//                 true,
//                 left_operand_type,
//                 left_memory,
//                 left_index,
//                 right_memory,
//                 right_index,
//             ),
//             SyntaxKind::GreaterThanExpression => Instruction::less_equal(
//                 false,
//                 left_operand_type,
//                 left_memory,
//                 left_index,
//                 right_memory,
//                 right_index,
//             ),
//             SyntaxKind::LessThanOrEqualExpression => Instruction::less_equal(
//                 true,
//                 left_operand_type,
//                 left_memory,
//                 left_index,
//                 right_memory,
//                 right_index,
//             ),
//             SyntaxKind::GreaterThanOrEqualExpression => Instruction::less(
//                 false,
//                 left_operand_type,
//                 left_memory,
//                 left_index,
//                 right_memory,
//                 right_index,
//             ),
//             _ => {
//                 return Err(CompileError::UnexpectedSyntaxKind {
//                     expected: &[
//                         SyntaxKind::EqualExpression,
//                         SyntaxKind::NotEqualExpression,
//                         SyntaxKind::LessThanExpression,
//                         SyntaxKind::GreaterThanExpression,
//                         SyntaxKind::LessThanOrEqualExpression,
//                         SyntaxKind::GreaterThanOrEqualExpression,
//                     ],
//                     found: reader.node.kind,
//                 });
//             }
//         };
//         let load_false_instruction = Instruction::move_with_jump(
//             register.index,
//             OperandType::BOOLEAN,
//             MemoryKind::ENCODED,
//             false as u16,
//             1,
//             true,
//         );
//         let load_true_instruction = Instruction::r#move(
//             register.index,
//             OperandType::BOOLEAN,
//             MemoryKind::ENCODED,
//             true as u16,
//         );

//         comparison_emission.push(comparison_instruction);
//         comparison_emission.push(load_false_instruction);
//         comparison_emission.push(load_true_instruction);
//         comparison_emission.set_target(Some(allocation));

//         Ok(Emission::Instructions(comparison_emission))
//     }

//     fn visit_logic_expression(
//         &mut self,
//         reader: SyntaxReader<'_>,
//         allocation: Option<AllocationKind>,
//     ) -> Result<Self::ExpressionOutput, CompileError> {
//         let LogicExpression { left, right } = reader.as_component()?;

//         let left_emission = self.visit_expression(left, None)?;
//         let right_emission = self.visit_expression(right, None)?;

//         if let Emission::Value(left_constant) = left_emission
//             && let Emission::Value(right_constant) = right_emission
//         {
//             let combined =
//                 self.fold_values(&reader, left_constant, &left, right_constant, &right)?;

//             return Ok(Emission::Value(combined));
//         }

//         let mut logic_instructions = InstructionsEmission::new();

//         let (left_memory, left_index, _) = self.handle_operand_emission(
//             &mut logic_instructions,
//             left_emission,
//             reader.node.kind,
//             &left,
//         )?;
//         let (right_memory, right_index, _) = self.handle_operand_emission(
//             &mut logic_instructions,
//             right_emission,
//             reader.node.kind,
//             &right,
//         )?;

//         let allocation = if let Some(allocation) = allocation {
//             allocation.clone()
//         } else {
//             let type_id = *self.resolver.get_type_binding(&reader.id)?;

//             self.allocate_registers(type_id, AllocationKind::Temporary, &reader)?
//         };
//         let register = allocation.expect_single()?;

//         let test_instruction = match reader.node.kind {
//             SyntaxKind::AndExpression => Instruction::test(false, left_memory, left_index, 1),
//             SyntaxKind::OrExpression => Instruction::test(true, left_memory, left_index, 1),
//             _ => {
//                 return Err(CompileError::UnexpectedSyntaxKind {
//                     expected: &[SyntaxKind::AndExpression, SyntaxKind::OrExpression],
//                     found: reader.node.kind,
//                 });
//             }
//         };
//         let right_move_instruction = Instruction::move_with_jump(
//             register.index,
//             OperandType::BOOLEAN,
//             right_memory,
//             right_index,
//             1,
//             true,
//         );
//         let left_move_instruction = Instruction::r#move(
//             register.index,
//             OperandType::BOOLEAN,
//             left_memory,
//             left_index,
//         );

//         logic_instructions.push(test_instruction);
//         logic_instructions.push(right_move_instruction);
//         logic_instructions.push(left_move_instruction);
//         logic_instructions.set_target(Some(allocation));

//         Ok(Emission::Instructions(logic_instructions))
//     }

//     fn visit_negation_expression(
//         &mut self,
//         reader: SyntaxReader,
//         input: (),
//     ) -> Result<Self::ExpressionOutput, CompileError> {
//         let NegationExpression { operand } = reader.as_component()?;

//         let expression_emission = self.visit_expression(operand, None)?;

//         if let Emission::Value(constant) = &expression_emission
//             && operand.node.kind != SyntaxKind::PathExpression
//         {
//             let negated = constant
//                 .negate()
//                 .ok_or_else(|| CompileError::CannotApplyOperator {
//                     operator: reader.node.kind,
//                     type_id: constant.type_id(),
//                     operand_position: operand.position(),
//                 })?;

//             return Ok(Emission::Value(negated));
//         }

//         let mut negation_emission = InstructionsEmission::new();

//         let (operand_memory, operand_index, _) = self.handle_operand_emission(
//             &mut negation_emission,
//             expression_emission,
//             reader.node.kind,
//             &operand,
//         )?;
//         let allocation = if let Some(allocation) = input {
//             allocation.clone()
//         } else {
//             let type_id = *self.resolver.get_type_binding(&reader.id)?;

//             self.allocate_registers(type_id, AllocationKind::Temporary, &reader)?
//         };
//         let register = allocation.expect_single()?;

//         let negate_instruction = Instruction::negate(
//             register.index,
//             register.operand_type,
//             operand_memory,
//             operand_index,
//         );

//         negation_emission.push(negate_instruction);
//         negation_emission.set_target(Some(allocation));

//         Ok(Emission::Instructions(negation_emission))
//     }

//     fn visit_not_expression(
//         &mut self,
//         reader: SyntaxReader,
//         input: Option<AllocationKind>,
//     ) -> Result<Self::ExpressionOutput, CompileError> {
//         let NotExpression { operand } = reader.as_component()?;

//         let expression_emission = self.visit_expression(operand, None)?;

//         if let Emission::Value(constant) = &expression_emission
//             && operand.node.kind != SyntaxKind::PathExpression
//         {
//             let negated = constant
//                 .negate()
//                 .ok_or_else(|| CompileError::CannotApplyOperator {
//                     operator: reader.node.kind,
//                     type_id: constant.type_id(),
//                     operand_position: operand.position(),
//                 })?;

//             return Ok(Emission::Value(negated));
//         }

//         let mut negation_emission = InstructionsEmission::new();

//         let (operand_memory, operand_index, _) = self.handle_operand_emission(
//             &mut negation_emission,
//             expression_emission,
//             reader.node.kind,
//             &operand,
//         )?;
//         let allocation = if let Some(allocation) = input {
//             allocation.clone()
//         } else {
//             let type_id = *self.resolver.get_type_binding(&reader.id)?;

//             self.allocate_registers(type_id, AllocationKind::Temporary, &reader)?
//         };
//         let register = allocation.expect_single()?;

//         let negate_instruction = Instruction::negate(
//             register.index,
//             register.operand_type,
//             operand_memory,
//             operand_index,
//         );

//         negation_emission.push(negate_instruction);
//         negation_emission.set_target(Some(allocation));

//         Ok(Emission::Instructions(negation_emission))
//     }

//     fn visit_while_expression(
//         &mut self,
//         reader: SyntaxReader<'_>,
//         allocation: Option<AllocationKind>,
//     ) -> Result<Self::ExpressionOutput, CompileError> {
//         debug!("Visting while expression");

//         let (condition, body) = reader.binary_children()?;

//         let mut while_emission = InstructionsEmission::new();
//         let condition_emission = self.visit_expression(condition, None)?;

//         self.handle_condition_emission(&mut while_emission, condition_emission, &condition, false)?;

//         let jump_forward_id = self.create_jump_id();
//         let jump_backward_id = self.create_jump_id();

//         while_emission.push_drop_anchor(JumpAnchor::LoopStartHere {
//             forward_id: jump_forward_id,
//         });

//         let body_emission = self.visit_block_expression(body, allocation)?;

//         if let Emission::Instructions(instructions) = body_emission {
//             while_emission.merge(instructions);
//         }

//         for break_id in self.jump_over_branch_ids.drain(..) {
//             while_emission.push_drop_anchor(JumpAnchor::ForwardToNext { id: break_id });
//         }

//         while_emission.push_drop_anchor(JumpAnchor::LoopEndOnNext {
//             forward_id: jump_forward_id,
//             backward_id: jump_backward_id,
//         });
//         while_emission.set_target(None);

//         Ok(Emission::Instructions(while_emission))
//     }

//     fn visit_break_expression(
//         &mut self,
//         _: SyntaxReader<'_>,
//         _: (),
//     ) -> Result<Self::ExpressionOutput, CompileError> {
//         let break_id = self.create_jump_id();

//         self.jump_over_branch_ids.push(break_id);

//         let mut break_emission = InstructionsEmission::new();

//         break_emission.push(Instruction::no_op());
//         break_emission.push_drop_anchor(JumpAnchor::ForwardFromHere { id: break_id });

//         Ok(Emission::Instructions(break_emission))
//     }

//     fn visit_call_expression(
//         &mut self,
//         reader: SyntaxReader<'_>,
//         allocation: Option<AllocationKind>,
//     ) -> Result<Self::ExpressionOutput, CompileError> {
//         let CallExpression { callee, arguments } = reader.as_component()?;

//         let mut declaration_id = *self.resolver.get_declaration_binding(&callee.id)?;
//         let definition = self
//             .resolver
//             .declarations
//             .get_declaration(declaration_id)?
//             .definition;

//         if let Definition::Variant {
//             discriminant,
//             fields,
//             ..
//         } = definition
//             && !fields.is_empty()
//         {
//             let return_type_id = *self.resolver.get_type_binding(&reader.id)?;
//             let allocation = if let Some(allocation) = allocation {
//                 allocation
//             } else {
//                 self.allocate_registers(return_type_id, AllocationKind::Temporary, &reader)?
//             };

//             let destination = allocation.index();
//             let move_instruction = Instruction::r#move(
//                 destination,
//                 OperandType::U_16,
//                 MemoryKind::ENCODED,
//                 discriminant,
//             );

//             let mut variant_instructions = InstructionsEmission::new();

//             variant_instructions.push(move_instruction);

//             let field_registers = allocation.iter().skip(1);

//             for (argument, field_register) in arguments.children().zip(field_registers) {
//                 let argument_emission = self.visit_expression(argument, None)?;

//                 if let Emission::Value(constant) = &argument_emission
//                     && let Some(encoded) = constant.as_encoded_u16()
//                 {
//                     let move_instruction = Instruction::r#move(
//                         field_register.index,
//                         constant.operand_type(),
//                         MemoryKind::ENCODED,
//                         encoded,
//                     );

//                     variant_instructions.push(move_instruction);
//                     continue;
//                 }

//                 let argument_place = self.place_emission(argument_emission, &argument)?;

//                 match argument_place {
//                     Place::Constant {
//                         operand_type,
//                         index,
//                     } => {
//                         let move_instruction = Instruction::r#move(
//                             field_register.index,
//                             operand_type,
//                             MemoryKind::CONSTANT,
//                             index,
//                         );

//                         variant_instructions.push(move_instruction);
//                     }
//                     Place::Register(RegisterAllocation::Single { register, .. }) => {
//                         let move_instruction = Instruction::r#move(
//                             field_register.index,
//                             register.operand_type,
//                             MemoryKind::REGISTER,
//                             register.index,
//                         );

//                         variant_instructions.push(move_instruction);
//                     }
//                     Place::Register(RegisterAllocation::Multiple { registers, .. }) => {
//                         for register in registers {
//                             let move_instruction = Instruction::r#move(
//                                 field_register.index,
//                                 register.operand_type,
//                                 MemoryKind::REGISTER,
//                                 register.index,
//                             );

//                             variant_instructions.push(move_instruction);
//                         }
//                     }
//                 }
//             }

//             variant_instructions.set_target(Some(allocation));

//             return Ok(Emission::Instructions(variant_instructions));
//         }

//         let return_type_id = *self.resolver.get_type_binding(&reader.id)?;
//         let allocation = if allocation.is_some() {
//             allocation
//         } else if return_type_id != TypeId::UNIT {
//             Some(self.allocate_registers(return_type_id, AllocationKind::Temporary, &reader)?)
//         } else {
//             None
//         };
//         let destination = allocation
//             .as_ref()
//             .map(|allocation| allocation.index())
//             .unwrap_or(u16::MAX);

//         let mut call_instructions = InstructionsEmission::new();

//         let monomorphized_prototype_id = if let Definition::Function {
//             value_parameters, ..
//         } = definition
//         {
//             let callee_type_id = *self.resolver.get_type_binding(&callee.id)?;
//             let callee_type = *self.resolver.types.get_type(callee_type_id)?;

//             let concrete_type_arguments = if let Type::FunctionDefinition { type_arguments, .. } =
//                 callee_type
//                 && !type_arguments.is_empty()
//             {
//                 type_arguments
//                     .as_range()
//                     .map(|index| {
//                         let type_id = *self.resolver.types.get_type_member(index)?;

//                         self.resolver.resolve_type(type_id)
//                     })
//                     .try_collect::<SmallVec<[TypeId; 4]>>()?
//             } else {
//                 let mut types = SmallVec::<[TypeId; 4]>::new();

//                 for (argument, parameter_index) in
//                     arguments.children().zip(value_parameters.as_range())
//                 {
//                     let parameter_type_id =
//                         *self.resolver.types.get_type_member(parameter_index)?;
//                     let parameter_type = *self.resolver.types.get_type(parameter_type_id)?;

//                     if matches!(parameter_type, Type::Slice { .. }) {
//                         let argument_type_id = *self.resolver.get_type_binding(&argument.id)?;

//                         types.push(argument_type_id);
//                     }
//                 }

//                 let callee_scope = self.resolver.scopes.get_scope(
//                     self.resolver
//                         .declarations
//                         .get_declaration(declaration_id)?
//                         .scope_id,
//                 )?;

//                 if callee_scope.kind == ScopeKind::Trait
//                     && callee.node.kind == SyntaxKind::FieldAccessExpression
//                 {
//                     let FieldAccessExpression {
//                         struct_expression, ..
//                     } = callee.as_component()?;
//                     let operand_type_id = *self.resolver.get_type_binding(&struct_expression.id)?;
//                     let concrete_self_type_id = self.resolver.resolve_type(operand_type_id)?;

//                     types.push(concrete_self_type_id);

//                     // Resolve the trait method's declaration_id to the impl's
//                     // concrete version. The trait may declare an abstract method
//                     // (TraitFunctionItem) that has no body — we need the impl's
//                     // FunctionItem instead.
//                     let trait_method_symbol_id = self
//                         .resolver
//                         .declarations
//                         .get_declaration(declaration_id)?
//                         .symbol_id;

//                     let concrete_self_type =
//                         *self.resolver.types.get_type(concrete_self_type_id)?;

//                     if let Type::Algebraic {
//                         declaration_id: type_decl_id,
//                         ..
//                     } = concrete_self_type
//                     {
//                         let type_scope_id = self
//                             .resolver
//                             .declarations
//                             .get_declaration(type_decl_id)?
//                             .scope_id;

//                         'impl_search: for (_, decl) in self.resolver.declarations.iter() {
//                             if decl.scope_id != type_scope_id {
//                                 continue;
//                             }

//                             if let Definition::TraitImplementation { declarations, .. }
//                             | Definition::InherentImplementation { declarations, .. } =
//                                 decl.definition
//                             {
//                                 let member_ids = self
//                                     .resolver
//                                     .declarations
//                                     .get_declaration_members(&declarations)?;

//                                 for &member_id in member_ids {
//                                     let member =
//                                         self.resolver.declarations.get_declaration(member_id)?;

//                                     if member.symbol_id == trait_method_symbol_id {
//                                         declaration_id = member_id;
//                                         break 'impl_search;
//                                     }
//                                 }
//                             }
//                         }
//                     }
//                 }

//                 types
//             };

//             let cache_key = (declaration_id, concrete_type_arguments);
//             let prototype_id =
//                 if let Some(existing) = self.resolver.get_cached_prototype(&cache_key) {
//                     existing
//                 } else {
//                     let reserved = self.prototypes.reserve();

//                     self.resolver.cache_prototype(cache_key, reserved);
//                     self.compilation_stack.push(CompilationRequest {
//                         declaration_id,
//                         prototype_id: reserved,
//                     });

//                     reserved
//                 };

//             Some(prototype_id)
//         } else {
//             None
//         };

//         let arguments_start_index = self.register_tracker.next_temporary;

//         for argument in arguments.children() {
//             let argument_type_id = *self.resolver.get_type_binding(&argument.id)?;
//             let argument_target =
//                 self.allocate_registers(argument_type_id, AllocationKind::Reserved, &argument)?;

//             let argument_emission = self.visit_expression(argument, Some(argument_target))?;

//             if let Emission::Value(constant) = &argument_emission
//                 && let Some(encoded) = constant.as_encoded_u16()
//             {
//                 let operand_type = constant.operand_type();
//                 let destination = self
//                     .register_tracker
//                     .allocate_next_temporary(RegisterWidth::from(operand_type));
//                 let move_instruction =
//                     Instruction::r#move(destination, operand_type, MemoryKind::ENCODED, encoded);

//                 call_instructions.push(move_instruction);
//                 continue;
//             }

//             if let Emission::Instructions(instructions) = argument_emission {
//                 call_instructions.merge(instructions);
//                 continue;
//             }

//             let argument_place = self.place_emission(argument_emission, &argument)?;

//             match argument_place {
//                 Place::Constant {
//                     operand_type,
//                     index,
//                 } => {
//                     let destination = self
//                         .register_tracker
//                         .allocate_next_temporary(RegisterWidth::from(operand_type));
//                     let move_instruction =
//                         Instruction::r#move(destination, operand_type, MemoryKind::CONSTANT, index);

//                     call_instructions.push(move_instruction);
//                 }
//                 Place::Register(RegisterAllocation::Single { register, .. }) => {
//                     let destination = self
//                         .register_tracker
//                         .allocate_next_temporary(RegisterWidth::from(register.operand_type));
//                     let move_instruction = Instruction::r#move(
//                         destination,
//                         register.operand_type,
//                         MemoryKind::REGISTER,
//                         register.index,
//                     );

//                     call_instructions.push(move_instruction);
//                 }
//                 Place::Register(RegisterAllocation::Multiple { registers, .. }) => {
//                     for register in registers {
//                         let destination = self
//                             .register_tracker
//                             .allocate_next_temporary(RegisterWidth::from(register.operand_type));
//                         let move_instruction = Instruction::r#move(
//                             destination,
//                             register.operand_type,
//                             MemoryKind::REGISTER,
//                             register.index,
//                         );

//                         call_instructions.push(move_instruction);
//                     }
//                 }
//             }
//         }

//         self.register_tracker.free_reserved();

//         let arguments_start = if arguments.has_children() {
//             arguments_start_index
//         } else {
//             u16::MAX
//         };

//         let (callee_memory, callee_index) = if let Some(prototype_id) = monomorphized_prototype_id {
//             (MemoryKind::ENCODED, prototype_id.inner())
//         } else {
//             let callee_emission = self.visit_expression(callee, None)?;

//             let callee_place = match callee_emission {
//                 Emission::Place(place) => place,
//                 Emission::Instructions(instructions) => {
//                     let Some(registers) = instructions.target else {
//                         return Err(CompileError::ExpectedFunction {
//                             node_kind: callee.node.kind,
//                             position: callee.position(),
//                         });
//                     };

//                     Place::Register(registers)
//                 }
//                 Emission::NativeFunction(_) => todo!(),
//                 _ => {
//                     return Err(CompileError::ExpectedFunction {
//                         node_kind: callee.node.kind,
//                         position: callee.position(),
//                     });
//                 }
//             };

//             match callee_place {
//                 Place::Constant { index, .. } => (MemoryKind::CONSTANT, index),
//                 Place::Register(RegisterAllocation::Single { register, .. }) => {
//                     (MemoryKind::REGISTER, register.index)
//                 }
//                 Place::Register(RegisterAllocation::Multiple { .. }) => {
//                     return Err(CompileError::ExpectedFunction {
//                         node_kind: callee.node.kind,
//                         position: callee.position(),
//                     });
//                 }
//             }
//         };

//         let call_instruction =
//             Instruction::call(destination, callee_memory, callee_index, arguments_start);

//         call_instructions.push(call_instruction);
//         call_instructions.set_target(allocation);

//         Ok(Emission::Instructions(call_instructions))
//     }

//     fn visit_field_access_expression(
//         &mut self,
//         reader: SyntaxReader,
//         _: (),
//     ) -> Result<Self::ExpressionOutput, CompileError> {
//         let FieldAccessExpression {
//             struct_expression,
//             field_name,
//         } = reader.as_component()?;

//         let operand_emission = self.visit_expression(struct_expression, None)?;

//         let struct_registers = match operand_emission {
//             Emission::Place(Place::Register(registers)) => registers,
//             emission => {
//                 let place = self.place_emission(emission, &struct_expression)?;

//                 match place {
//                     Place::Register(registers) => registers,
//                     _ => {
//                         return Err(CompileError::ExpectedValue {
//                             node_kind: struct_expression.node.kind,
//                             position: struct_expression.position(),
//                         });
//                     }
//                 }
//             }
//         };

//         let field_declaration_id = *self.resolver.get_declaration_binding(&field_name.id)?;
//         let field_declaration = self
//             .resolver
//             .declarations
//             .get_declaration(field_declaration_id)?;

//         let parent_struct = match field_declaration.definition {
//             Definition::Field { parent_struct, .. } => parent_struct,
//             _ => {
//                 return Err(CompileError::ExpectedValue {
//                     node_kind: field_name.node.kind,
//                     position: field_name.position(),
//                 });
//             }
//         };

//         let struct_declaration = self.resolver.declarations.get_declaration(parent_struct)?;

//         let fields = match struct_declaration.definition {
//             Definition::StructType { fields, .. } => fields,
//             _ => {
//                 return Err(CompileError::ExpectedValue {
//                     node_kind: field_name.node.kind,
//                     position: field_name.position(),
//                 });
//             }
//         };

//         let field_ids = self
//             .resolver
//             .declarations
//             .get_declaration_members(&fields)?;

//         let mut register_offset = 0usize;

//         for &field_id in field_ids {
//             if field_id == field_declaration_id {
//                 break;
//             }

//             let field_declaration = self.resolver.declarations.get_declaration(field_id)?;

//             if let Definition::Field { type_id, .. } = field_declaration.definition {
//                 let operand_types = self.resolver.get_operand_types(type_id)?;
//                 register_offset += operand_types.len();
//             }
//         }

//         let field_register = struct_registers.registers().nth(register_offset);

//         match field_register {
//             Some(register) => Ok(Emission::Place(Place::Register(
//                 RegisterAllocation::Single {
//                     register: *register,
//                     temporary: struct_registers.is_temporary(),
//                 },
//             ))),
//             None => Err(CompileError::ExpectedValue {
//                 node_kind: field_name.node.kind,
//                 position: field_name.position(),
//             }),
//         }
//     }

//     fn visit_type(&mut self, _: SyntaxReader) -> Result<Self::TypeOutput, CompileError> {
//         Ok(())
//     }

//     fn visit_path(
//         &mut self,
//         _: SyntaxReader,
//         _: Self::PathInput,
//     ) -> Result<Self::PathOutput, CompileError> {
//         Ok(())
//     }

//     fn visit_simple_path(
//         &mut self,
//         _: SyntaxReader,
//         _: Self::PathInput,
//     ) -> Result<Self::PathOutput, CompileError> {
//         Ok(())
//     }
// }

#[derive(Clone, Debug)]
pub enum Emission {
    Instructions(InstructionsEmission),
    Value(ConstantValue),
    NativeFunction(NativeFunction),
    Place(Place),
    Never,
}

impl From<ConstantValue> for Emission {
    fn from(value: ConstantValue) -> Self {
        Emission::Value(value)
    }
}

#[derive(Clone, Debug, Default)]
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

    fn with_instruction(instruction: Instruction) -> Self {
        Self {
            instructions: vec![(instruction, Vec::new())],
            target: None,
        }
    }

    fn with_instruction_and_target(instruction: Instruction, target: RegisterAllocation) -> Self {
        Self {
            instructions: vec![(instruction, Vec::new())],
            target: Some(target),
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

    fn push_drop_anchor(&mut self, anchor: JumpAnchor) {
        if let Some((_, anchors)) = self.instructions.last_mut() {
            anchors.push(anchor);
        }
    }

    fn extend(&mut self, instructions: Vec<(Instruction, Vec<JumpAnchor>)>) {
        self.instructions.extend(instructions);
    }

    fn merge(&mut self, other: InstructionsEmission) {
        self.instructions.extend(other.instructions);
        self.target = other.target;
    }
}

#[derive(Clone, Debug)]
pub enum Place {
    Constant {
        index: u16,
        operand_type: OperandType,
    },
    Register(RegisterAllocation),
}

impl Place {
    fn expect_allocation(self, node: &SyntaxReader) -> Result<RegisterAllocation, CompileError> {
        if let Place::Register(allocation) = self {
            Ok(allocation)
        } else {
            Err(CompileError::CannotMutate {
                position: node.position(),
            })
        }
    }
}

#[derive(Clone, Debug)]
pub enum Local {
    Place(Place),
    Constant(ConstantValue),
}

#[derive(Clone, Debug)]
pub struct RegisterAllocation {
    registers: SmallVec<[RegisterClaim; 4]>,
    kind: AllocationKind,
}

impl RegisterAllocation {
    fn is_temporary(&self) -> bool {
        self.kind == AllocationKind::Temporary
    }

    fn base_index(&self) -> u16 {
        self.registers[0].index
    }

    fn len(&self) -> usize {
        self.registers.len()
    }

    fn operand_types(&self) -> OperandType::SmallVec {
        self.registers
            .iter()
            .map(|register| register.operand_type)
            .collect()
    }

    fn expect_single(&self) -> Result<RegisterClaim, CompileError> {
        if self.registers.len() == 1 {
            Ok(self.registers[0])
        } else {
            Err(CompileError::InvalidRegisterCount {
                expected: 1,
                found: self.registers.len(),
            })
        }
    }

    fn expect_multiple(
        &self,
        expected: usize,
    ) -> Result<&SmallVec<[RegisterClaim; 4]>, CompileError> {
        if self.registers.len() == expected {
            Ok(&self.registers)
        } else {
            Err(CompileError::InvalidRegisterCount {
                expected,
                found: self.registers.len(),
            })
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RegisterClaim {
    index: u16,
    operand_type: OperandType,
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
            next_local: argument_count,
            next_temporary: argument_count,
            next_reserved: 0,
            max: reserved,
        }
    }

    fn allocate_next_local(&mut self, operand_type: OperandType) -> u16 {
        let next = self.next_local;
        self.next_local += operand_type.register_width().as_u16();
        self.next_temporary = self.next_temporary.max(self.next_local);
        self.max = self.max.max(self.next_local);

        next
    }

    fn allocate_next_temporary(&mut self, operand_type: OperandType) -> u16 {
        let next = self.next_temporary;
        self.next_temporary += operand_type.register_width().as_u16();
        self.max = self.max.max(self.next_temporary);

        next
    }

    fn allocate_next_reserved(&mut self, operand_type: OperandType) -> u16 {
        let next = self.next_reserved.min(self.reserved);
        self.next_reserved += operand_type.register_width().as_u16();

        next
    }

    fn free_reserved(&mut self) {
        self.next_reserved = 0;
    }

    fn free_temporary(&mut self, allocation: &RegisterAllocation) {
        debug_assert!(allocation.is_temporary());

        for register in &allocation.registers {
            debug_assert!(register.index < self.next_temporary);

            self.next_temporary = self.next_temporary.min(register.index);
        }

        self.max = self.max.min(self.next_temporary);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AllocationKind {
    Local,
    Temporary,
    Reserved,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum RegisterWidth {
    Single,
    Double,
    Quad,
}

impl RegisterWidth {
    pub fn as_u16(&self) -> u16 {
        match self {
            RegisterWidth::Single => 1,
            RegisterWidth::Double => 2,
            RegisterWidth::Quad => 4,
        }
    }
}

pub fn get_byte_size(
    type_id: TypeId,
    type_arguments: Option<&TypeMembers>,
    resolver: &Resolver,
) -> Result<Option<usize>, CompileError> {
    const DISCRIMINANT_BYTE_SIZE: usize = 2;

    fn get_definition_type_size(
        declaration_id: DeclarationId,
        type_arguments: Option<&TypeMembers>,
        resolver: &Resolver,
    ) -> Result<Option<usize>, CompileError> {
        let declaration = resolver.declarations.get_declaration(declaration_id)?;

        match &declaration.definition {
            Definition::StructType {
                type_parameters,
                fields,
                ..
            } => {
                let field_entries =
                    resolver.scopes.get_namespace_entries(*fields);
                let mut total_size = 0;

                for &(_, field_declaration_id) in field_entries {
                    let field_declaration = resolver
                        .declarations
                        .get_declaration(field_declaration_id)?;
                    let Definition::Field {
                        type_id: field_type_id,
                        ..
                    } = field_declaration.definition
                    else {
                        return Err(CompileError::ExpectedFieldDefinition(
                            field_declaration_id,
                        ));
                    };

                    let resolved_field_type_id = if let Some(type_arguments) = type_arguments {
                        let field_type = resolver.types.get_type(field_type_id)?;

                        if let Type::Generic {
                            declaration_id: parameter_declaration_id,
                        } = field_type
                        {
                            let type_param_entries =
                                resolver.scopes.get_namespace_entries(*type_parameters);
                            type_param_entries
                                .iter()
                                .zip(type_arguments.as_range())
                                .find_map(|(&(_, param_decl_id), argument_member_index)| {
                                    if &param_decl_id == parameter_declaration_id {
                                        let argument_type_id = resolver
                                            .types
                                            .get_type_member(argument_member_index)
                                            .ok()?;

                                        Some(*argument_type_id)
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or(field_type_id)
                        } else {
                            field_type_id
                        }
                    } else {
                        field_type_id
                    };

                    let byte_size = if let Some(size) =
                        get_byte_size(resolved_field_type_id, type_arguments, resolver)?
                    {
                        size
                    } else {
                        return Ok(None);
                    };

                    total_size += byte_size;
                }

                Ok(Some(total_size))
            }
            Definition::EnumType {
                type_parameters,
                variants,
                ..
            } => {
                let variant_entries =
                    resolver.scopes.get_namespace_entries(*variants);
                let mut max_variant_size = 0;

                for &(_, variant_declaration_id) in variant_entries {
                    let variant_declaration = resolver
                        .declarations
                        .get_declaration(variant_declaration_id)?;
                    let Definition::Variant { fields, .. } = &variant_declaration.definition else {
                        continue;
                    };

                    let field_entries =
                        resolver.scopes.get_namespace_entries(*fields);
                    let mut variant_size = 0;

                    for &(_, field_declaration_id) in field_entries {
                        let field_declaration = resolver
                            .declarations
                            .get_declaration(field_declaration_id)?;
                        let Definition::Field {
                            type_id: field_type_id,
                            ..
                        } = field_declaration.definition
                        else {
                            continue;
                        };

                        let resolved_field_type_id = if let Some(type_arguments) = type_arguments {
                            let field_type = resolver.types.get_type(field_type_id)?;

                            if let Type::Generic {
                                declaration_id: parameter_declaration_id,
                            } = field_type
                            {
                                let type_param_entries =
                                    resolver.scopes.get_namespace_entries(*type_parameters);
                                type_param_entries
                                    .iter()
                                    .zip(type_arguments.as_range())
                                    .find_map(|(&(_, param_decl_id), argument_index)| {
                                        if &param_decl_id == parameter_declaration_id {
                                            let argument_type_id = resolver
                                                .types
                                                .get_type_member(argument_index)
                                                .ok()?;

                                            Some(*argument_type_id)
                                        } else {
                                            None
                                        }
                                    })
                                    .unwrap_or(field_type_id)
                            } else {
                                field_type_id
                            }
                        } else {
                            field_type_id
                        };

                        let byte_size =
                            get_byte_size(resolved_field_type_id, type_arguments, resolver)?
                                .unwrap_or(0);

                        variant_size += byte_size;
                    }

                    max_variant_size = max_variant_size.max(variant_size);
                }

                Ok(Some(max_variant_size + DISCRIMINANT_BYTE_SIZE))
            }
            Definition::TypeParameter => {
                if let Some(&concrete_type_id) = resolver.type_parameter_map.get(&declaration_id) {
                    get_byte_size(concrete_type_id, type_arguments, resolver)
                } else {
                    Ok(None)
                }
            }
            Definition::TypeAlias {
                aliased_type_id, ..
            }
            | Definition::InherentAssociatedType {
                aliased_type_id, ..
            } => get_byte_size(*aliased_type_id, type_arguments, resolver),
            Definition::Constant { type_id, .. }
            | Definition::InherentAssociatedConstant { type_id, .. }
            | Definition::Local { type_id, .. }
            | Definition::Field { type_id, .. } => {
                get_byte_size(*type_id, type_arguments, resolver)
            }
            Definition::Use {
                source_declaration_id,
                ..
            } => get_definition_type_size(*source_declaration_id, type_arguments, resolver),
            Definition::Function { .. } | Definition::NativeFunction { .. } => Ok(Some(2)),
            _ => Ok(None),
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
        Type::SignedInteger(SignedIntegerType::I64 | SignedIntegerType::ISize)
        | Type::UnsignedInteger(UnsignedIntegerType::U64 | UnsignedIntegerType::USize)
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
