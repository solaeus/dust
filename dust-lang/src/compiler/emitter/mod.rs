#[cfg(test)]
mod tests;

use std::collections::HashMap;

use rustc_hash::FxBuildHasher;
use smallvec::{SmallVec, smallvec};
use tracing::trace;

use crate::{
    compiler::{
        CompilationRequest,
        error::CompileError,
        resolver::{
            PrototypeId, Resolver,
            declarations::{DeclarationId, Definition},
            scopes::ScopeKind,
            types::{
                FloatType, InferredTypeConstraint, SignedIntegerType, Type, TypeId,
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
    optimal_small_vec_inline_capacity,
    prototype::Prototype,
    source::Source,
    syntax::{
        components::{
            ArrayExpression, ArrayRepeatExpression, AssignmentExpression, BlockExpression,
            CallExpression, ComparisonExpression, ConstItem, ExpressionStatement,
            FieldAccessExpression, GroupedExpression, IfExpression, IndexExpression, LetStatement,
            LogicExpression, MathExpression, NegationExpression, NotExpression, RangeExpression,
            StructExpression, StructExpressionStructFields, ValueParameters, WhileExpression,
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

    /// Concatenated lists of register indices that need to be dropped when exiting scopes.
    pending_drops: Vec<u16>,

    /// Stack of start indices into `pending_drops`. When exiting a drop context, the index is used
    /// to flush the indices into a DROP instruction.
    drop_stack: Vec<usize>,

    register_tracker: RegisterTracker,

    jump_placements: HashMap<JumpId, JumpPlacement, FxBuildHasher>,

    jump_over_branch_ids: Vec<JumpId>,

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

            for (parameter_name, _) in name_type_pairs {
                let declaration_id = *resolver.get_declaration_binding(&parameter_name.id)?;
                let declaration = resolver.declarations.get_declaration(declaration_id)?;
                let Definition::Local { type_id, .. } = declaration.definition else {
                    return Err(CompileError::ExpectedLocalDefinition(declaration_id));
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
            pending_drops: Vec::new(),
            drop_stack: Vec::new(),
            argument_count: argument_register_count,
            register_tracker: RegisterTracker::new(argument_register_count, return_register_count),
            return_type_id,
            return_operand_types,
            jump_placements: HashMap::default(),
            jump_over_branch_ids: Vec::new(),
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

            for (parameter_name, _) in name_type_pairs {
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
                    emitter.allocate_registers(concrete_type_id, RegisterKind::Reserved)?;

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

    fn enter_independent_expression(&mut self) -> u16 {
        self.register_tracker.next_temporary
    }

    fn enter_drop_context(&mut self) {
        self.drop_stack.push(self.pending_drops.len());
    }

    fn add_drop(&mut self, register_index: u16) {
        self.pending_drops.push(register_index);
    }

    fn exit_drop_context(&mut self, instructions: &mut InstructionsEmission) {
        let Some(start_index) = self.drop_stack.pop() else {
            return;
        };
        let Some((last_instruction, _)) = instructions.instructions.last_mut() else {
            return;
        };
        let (start_register, end_register) = self
            .pending_drops
            .drain(start_index..)
            .fold((0, 1), |(min, max), register_index| {
                (min.min(register_index), max.max(register_index + 1))
            });

        match last_instruction.operation() {
            Operation::DROP => {
                let Drop {
                    drop_list_start,
                    drop_list_end,
                } = Drop::from(&*last_instruction);

                let register_range = start_register..=end_register;

                if register_range.contains(&drop_list_start)
                    || register_range.contains(&drop_list_end)
                {
                    *last_instruction = Instruction::drop(
                        drop_list_start.min(start_register),
                        drop_list_end.max(end_register),
                    );
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
                    *last_instruction = Instruction::jump_with_drops(
                        offset,
                        is_positive,
                        start_register,
                        end_register,
                    );
                }
            }
            _ => {
                let drop_instruction = Instruction::drop(start_register, end_register);

                instructions.push(drop_instruction);
            }
        }
    }

    fn create_jump_id(&mut self) -> JumpId {
        let next = self.next_jump_id;

        self.next_jump_id.0 += 1;

        next
    }

    fn allocate_registers(
        &mut self,
        type_id: TypeId,
        kind: RegisterKind,
    ) -> Result<RegisterClaims, CompileError> {
        fn collect_registers(
            type_id: TypeId,
            kind: RegisterKind,
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
                Type::Tuple { element_types } => {
                    for index in element_types.as_range() {
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
                            RegisterKind::Local => {
                                emitter.register_tracker.allocate_next_local(operand_type)
                            }
                            RegisterKind::Temporary => emitter
                                .register_tracker
                                .allocate_next_temporary(operand_type),
                            RegisterKind::Reserved => emitter
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
                RegisterKind::Local => emitter.register_tracker.allocate_next_local(operand_type),
                RegisterKind::Temporary => emitter
                    .register_tracker
                    .allocate_next_temporary(operand_type),
                RegisterKind::Reserved => emitter
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

        Ok(RegisterClaims {
            claims: registers,
            kind,
        })
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
                target_registers: target,
                instructions,
                pending_drops,
            }) => {
                target_instructions.instructions.extend(instructions);
                target_instructions.pending_drops.extend(pending_drops);

                if let Some(allocation) = target {
                    Ok(Place::Register(allocation))
                } else {
                    Err(CompileError::ExpectedValue {
                        source_id: syntax.source_id(),
                        syntax_id: syntax.id,
                    })
                }
            }
            Emission::NativeFunction(_) => Err(CompileError::ExpectedNativeFunctionCall {
                position: syntax.position(),
            }),
            _ => Err(CompileError::ExpectedValue {
                source_id: syntax.source_id(),
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
                allocation.expect_base_index()?,
                allocation.operand_types(),
            )),
            Emission::Instructions(operand_instructions) => {
                instructions.merge(operand_instructions);

                match &instructions.target_registers {
                    Some(allocation) => Ok((
                        MemoryKind::REGISTER,
                        allocation.expect_base_index()?,
                        allocation.operand_types(),
                    )),
                    None => Err(CompileError::ExpectedValue {
                        source_id: operand.source_id(),
                        syntax_id: operand.id,
                    }),
                }
            }
            Emission::NativeFunction(_) => Err(CompileError::ExpectedNativeFunctionCall {
                position: operand.position(),
            }),
            Emission::Never => Err(CompileError::ExpectedValue {
                source_id: operand.source_id(),
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
                let test_instruction = Instruction::test(
                    comparator,
                    MemoryKind::REGISTER,
                    allocation.expect_base_index()?,
                    0,
                );

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

                            if let Some(registers) = &condition_instructions.target_registers {
                                self.register_tracker.deallocate(registers);
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

                            if let Some(registers) = &condition_instructions.target_registers {
                                self.register_tracker.deallocate(registers);
                            }
                        }
                        _ => {
                            let target_register_index = if let Some(allocation) =
                                &condition_instructions.target_registers
                                && allocation.len() == 1
                            {
                                allocation.claims[0].index
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
        target: &RegisterClaims,
        node: SyntaxReader,
    ) -> Result<(), CompileError> {
        match branch_emission {
            Emission::Value(constant) => {
                let address = self.materialize_value(constant)?;
                let move_instruction = Instruction::r#move(
                    target.expect_base_index()?,
                    constant.operand_type(),
                    address.memory,
                    address.index,
                );

                instructions.push(move_instruction);
            }
            Emission::Place(Place::Constant {
                operand_type,
                index,
            }) => {
                let destination = target.expect_base_index()?;
                let move_instruction =
                    Instruction::r#move(destination, operand_type, MemoryKind::CONSTANT, index);

                instructions.push(move_instruction);
            }
            Emission::Place(Place::Register(operand_registers)) => {
                for (destination, operand) in target.claims.iter().zip(operand_registers.claims) {
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
                    self.allocate_registers(self.return_type_id, RegisterKind::Reserved)?;
                let address = self.materialize_value(value)?;
                let move_instruction = Instruction::r#move(
                    allocation.expect_base_index()?,
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
                    self.allocate_registers(self.return_type_id, RegisterKind::Reserved)?;
                let move_instruction = Instruction::r#move(
                    allocation.expect_base_index()?,
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
                    self.allocate_registers(self.return_type_id, RegisterKind::Reserved)?;

                for (emission_register, target_register) in emission_allocation
                    .claims
                    .into_iter()
                    .zip(allocation.claims)
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
                let Some(instructions) = self.emit_statement(child)? else {
                    continue;
                };

                self.handle_function_body_instructions(instructions)?;

                continue;
            }

            if index == child_count - 1 {
                let return_expression_emission = self
                    .emit_expression(child, ExpressionTarget::Unclaimed(RegisterKind::Reserved))?;
                let return_instructions =
                    self.create_return_instructions(return_expression_emission, &child)?;

                self.handle_function_body_instructions(return_instructions)?;
            }

            if let Emission::Instructions(instructions) =
                self.emit_expression(child, ExpressionTarget::Unclaimed(RegisterKind::Temporary))?
            {
                self.handle_function_body_instructions(instructions)?;
            }
        }

        Ok(())
    }

    fn emit_statement(
        &mut self,
        reader: SyntaxReader,
    ) -> Result<Option<InstructionsEmission>, CompileError> {
        match reader.node.kind {
            SyntaxKind::LetStatement => self.emit_let_statement(reader),
            SyntaxKind::ExpressionStatement => self.emit_expression_statement(reader),
            _ => Err(CompileError::UnexpectedSyntax {
                expected: &[SyntaxKind::LetStatement, SyntaxKind::ExpressionStatement],
                found: reader.node.kind,
            }),
        }
    }

    fn emit_expression(
        &mut self,
        reader: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        match reader.node.kind {
            SyntaxKind::AssignmentExpression => self.emit_assignment_expression(reader, target),
            SyntaxKind::BooleanExpression => self.emit_boolean_expression(reader, target),
            SyntaxKind::HexadecimalExpression => self.emit_hexadecimal_expression(reader, target),
            SyntaxKind::CharacterExpression => self.emit_character_expression(reader, target),
            SyntaxKind::FloatExpression => self.emit_float_expression(reader, target),
            SyntaxKind::IntegerExpression => self.emit_integer_expression(reader, target),
            SyntaxKind::StringExpression => self.emit_string_expression(reader, target),
            SyntaxKind::ArrayExpression => self.emit_array_expression(reader, target),
            SyntaxKind::ArrayRepeatExpression => self.emit_array_repeat_expression(reader, target),
            SyntaxKind::IndexExpression => self.emit_index_expression(reader, target),
            SyntaxKind::RangeExpression | SyntaxKind::RangeInclusiveExpression => {
                self.emit_range_expression(reader, target)
            }
            SyntaxKind::PathExpression => self.emit_path_expression(reader, target),
            SyntaxKind::StructExpression => self.emit_struct_expression(reader, target),
            SyntaxKind::GroupedExpression => self.emit_grouped_expression(reader, target),
            SyntaxKind::BlockExpression => self.emit_block_expression(reader, target),
            SyntaxKind::IfExpression => self.emit_if_expression(reader, target),
            SyntaxKind::NegationExpression => self.emit_negation_expression(reader, target),
            SyntaxKind::NotExpression => self.emit_not_expression(reader, target),
            SyntaxKind::WhileExpression => self.emit_while_expression(reader, target),
            SyntaxKind::BreakExpression => self.emit_break_expression(reader, target),
            SyntaxKind::CallExpression => self.emit_call_expression(reader, target),
            SyntaxKind::FieldAccessExpression => self.emit_field_access_expression(reader, target),
            SyntaxKind::AdditionExpression
            | SyntaxKind::AdditionAssignmentExpression
            | SyntaxKind::SubtractionExpression
            | SyntaxKind::SubtractionAssignmentExpression
            | SyntaxKind::MultiplicationExpression
            | SyntaxKind::MultiplicationAssignmentExpression
            | SyntaxKind::DivisionExpression
            | SyntaxKind::DivisionAssignmentExpression
            | SyntaxKind::ModuloExpression
            | SyntaxKind::ModuloAssignmentExpression
            | SyntaxKind::ExponentExpression
            | SyntaxKind::ExponentAssignmentExpression => self.emit_math_expression(reader, target),
            SyntaxKind::GreaterThanExpression
            | SyntaxKind::LessThanExpression
            | SyntaxKind::GreaterThanOrEqualExpression
            | SyntaxKind::LessThanOrEqualExpression
            | SyntaxKind::EqualExpression
            | SyntaxKind::NotEqualExpression => self.emit_comparison_expression(reader, target),
            SyntaxKind::AndExpression | SyntaxKind::OrExpression => {
                self.emit_logic_expression(reader, target)
            }
            _ => Err(CompileError::UnexpectedSyntax {
                expected: &[],
                found: reader.node.kind,
            }),
        }
    }

    fn emit_root(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
        Ok(())
    }

    fn emit_module_item(&mut self, _: SyntaxReader<'_>) -> Result<(), CompileError> {
        Ok(())
    }

    fn emit_function_item(&mut self, _: SyntaxReader<'_>) -> Result<(), CompileError> {
        Ok(())
    }

    fn emit_use_item(&mut self, _: SyntaxReader<'_>) -> Result<(), CompileError> {
        Ok(())
    }

    fn emit_struct_item(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
        Ok(())
    }

    fn emit_enum_item(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
        Ok(())
    }

    fn emit_const_item(&mut self, syntax: SyntaxReader) -> Result<(), CompileError> {
        let ConstItem { name, value, .. } = syntax.as_component()?;
        let value = value.ok_or(CompileError::ExpectedValue {
            source_id: syntax.source_id(),
            syntax_id: syntax.id,
        })?;

        let expression_emission = self.emit_expression(value, ExpressionTarget::None)?;

        let declaration_id = *self.resolver.get_declaration_binding(&name.id)?;
        let constant_value = if let Emission::Value(constant) = expression_emission {
            constant
        } else {
            return Err(CompileError::ExpectedValue {
                source_id: value.source_id(),
                syntax_id: value.id,
            });
        };

        self.resolver
            .add_constant_item_value(declaration_id, constant_value);

        Ok(())
    }

    fn emit_type_item(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
        Ok(())
    }

    fn emit_impl_item(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
        Ok(())
    }

    fn emit_impl_trait_item(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
        Ok(())
    }

    fn emit_trait_item(&mut self, _: SyntaxReader) -> Result<(), CompileError> {
        Ok(())
    }

    fn emit_expression_statement(
        &mut self,
        syntax: SyntaxReader<'_>,
    ) -> Result<Option<InstructionsEmission>, CompileError> {
        let ExpressionStatement { expression } = syntax.as_component()?;

        let expression_emission = self.emit_expression(
            expression,
            ExpressionTarget::Unclaimed(RegisterKind::Temporary),
        )?;

        if let Emission::Instructions(mut instructions_emission) = expression_emission {
            instructions_emission.set_target(None);

            Ok(Some(instructions_emission))
        } else {
            Ok(None)
        }
    }

    fn emit_let_statement(
        &mut self,
        syntax: SyntaxReader,
    ) -> Result<Option<InstructionsEmission>, CompileError> {
        let LetStatement {
            name, expression, ..
        } = syntax.as_component()?;

        let declaration_id = *self.resolver.get_declaration_binding(&name.id)?;
        let emission =
            self.emit_expression(expression, ExpressionTarget::Unclaimed(RegisterKind::Local))?;

        match emission {
            Emission::Value(value) => {
                let address = self.materialize_value(value)?;

                self.locals.insert(
                    declaration_id,
                    Local::Place(Place::Constant {
                        operand_type: value.operand_type(),
                        index: address.index,
                    }),
                );

                Ok(None)
            }
            Emission::Place(place) => {
                self.locals.insert(declaration_id, Local::Place(place));

                Ok(None)
            }
            Emission::Instructions(InstructionsEmission {
                instructions,
                target_registers,
                pending_drops,
            }) => {
                let registers = target_registers.ok_or_else(|| CompileError::ExpectedValue {
                    source_id: expression.source_id(),
                    syntax_id: expression.id,
                })?;

                self.locals
                    .insert(declaration_id, Local::Place(Place::Register(registers)));

                Ok(Some(InstructionsEmission {
                    instructions,
                    target_registers: None,
                    pending_drops,
                }))
            }
            Emission::Never => Ok(None),
            Emission::NativeFunction(_) => Err(CompileError::ExpectedNativeFunctionCall {
                position: syntax.position(),
            }),
        }
    }

    fn emit_assignment_expression(
        &mut self,
        reader: SyntaxReader<'_>,
        _: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        let AssignmentExpression { target, source } = reader.as_component()?;

        let mut assignment_instructions = InstructionsEmission::new();

        let target_allocation = if target.node.kind == SyntaxKind::IndexExpression {
            let target_emission = self.emit_index_expression(
                target,
                ExpressionTarget::Unclaimed(RegisterKind::Temporary),
            )?;

            match target_emission {
                Emission::Place(Place::Register(allocation)) => allocation,
                Emission::Instructions(InstructionsEmission {
                    instructions,
                    target_registers: Some(target_allocation),
                    pending_drops,
                }) => {
                    assignment_instructions.instructions.extend(instructions);
                    assignment_instructions.pending_drops.extend(pending_drops);

                    target_allocation
                }
                Emission::Instructions(_) => {
                    return Err(CompileError::ExpectedValue {
                        source_id: target.source_id(),
                        syntax_id: target.id,
                    });
                }
                _ => {
                    return Err(CompileError::CannotMutate {
                        position: target.position(),
                    });
                }
            }
        } else {
            let declaration_id = self.resolver.get_declaration_binding(&target.id)?;
            let local = self.locals.get(declaration_id).ok_or_else(|| {
                CompileError::DeclarationOutOfScope {
                    declaration_id: *declaration_id,
                    usage_position: target.position(),
                }
            })?;

            if let Local::Place(Place::Register(registers)) = local {
                registers.clone()
            } else {
                return Err(CompileError::CannotMutate {
                    position: target.position(),
                });
            }
        };

        let source_emission =
            self.emit_expression(source, ExpressionTarget::Claimed(target_allocation.clone()))?;

        match source_emission {
            Emission::Value(value) => {
                let operand_type = value.operand_type();
                let address = self.materialize_value(value)?;
                let move_instruction = Instruction::r#move(
                    target_allocation.expect_base_index()?,
                    operand_type,
                    address.memory,
                    address.index,
                );

                assignment_instructions.push(move_instruction);

                if operand_type == OperandType::POINTER {
                    self.add_drop(address.index);
                }
            }
            Emission::Place(Place::Constant {
                operand_type,
                index,
            }) => {
                for destination in target_allocation.claims {
                    let move_instruction = Instruction::r#move(
                        destination.index,
                        operand_type,
                        MemoryKind::CONSTANT,
                        index,
                    );

                    assignment_instructions.push(move_instruction);

                    if destination.operand_type == OperandType::POINTER {
                        self.add_drop(destination.index);
                    }
                }
            }
            Emission::Place(Place::Register(operand_allocation)) => {
                for (destination, operand) in target_allocation
                    .claims
                    .into_iter()
                    .zip(operand_allocation.claims)
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
            Emission::Never => {
                return Err(CompileError::ExpectedValue {
                    source_id: source.source_id(),
                    syntax_id: source.id,
                });
            }
        }

        Ok(Emission::Instructions(assignment_instructions))
    }

    fn emit_boolean_expression(
        &mut self,
        reader: SyntaxReader,
        _: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        let boolean = reader.node.flags.get_flag(SyntaxFlags::BOOLEAN_TRUE);

        Ok(Emission::Value(ConstantValue::Boolean(boolean)))
    }

    fn emit_hexadecimal_expression(
        &mut self,
        reader: SyntaxReader,
        _: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        let text = &self.source.get_content(reader.position())?[2..];
        let byte = create_u8_from_hexadecimal(text)?;

        Ok(Emission::Value(ConstantValue::U8(byte)))
    }

    fn emit_character_expression(
        &mut self,
        reader: SyntaxReader,
        _: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        let text = self.source.get_content(reader.position().shrink(1))?;
        let character = create_char(text)?;

        Ok(Emission::Value(ConstantValue::Character(character)))
    }

    fn emit_float_expression(
        &mut self,
        reader: SyntaxReader,
        _: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        let type_id = *self.resolver.get_type_binding(&reader.id)?;
        let text = self.source.get_content(reader.position())?;

        match type_id {
            TypeId::F_32 => {
                let float = create_f32_from_decimal(text)?;

                Ok(Emission::Value(ConstantValue::F32(float)))
            }
            TypeId::F_64 => {
                let float = create_f64_from_decimal(text)?;

                Ok(Emission::Value(ConstantValue::F64(float)))
            }
            _ => Err(CompileError::InvalidTypeBinding(type_id)),
        }
    }

    fn emit_integer_expression(
        &mut self,
        reader: SyntaxReader,
        _: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        let type_id = *self.resolver.get_type_binding(&reader.id)?;
        let text = self.source.get_content(reader.position())?;

        match type_id {
            TypeId::I_8 => Ok(Emission::Value(ConstantValue::I8(create_i8_from_decimal(
                text,
            )?))),
            TypeId::I_16 => Ok(Emission::Value(ConstantValue::I16(
                create_i16_from_decimal(text)?,
            ))),
            TypeId::I_32 => Ok(Emission::Value(ConstantValue::I32(
                create_i32_from_decimal(text)?,
            ))),
            TypeId::I_64 => Ok(Emission::Value(ConstantValue::I64(
                create_i64_from_decimal(text)?,
            ))),
            TypeId::I_128 => Ok(Emission::Value(ConstantValue::I128(
                create_i128_from_decimal(text)?,
            ))),
            #[cfg(target_pointer_width = "32")]
            TypeId::I_SIZE => Ok(Emission::Value(ConstantValue::I32(
                create_i32_from_decimal(text)?,
            ))),
            #[cfg(target_pointer_width = "64")]
            TypeId::I_SIZE => Ok(Emission::Value(ConstantValue::I64(
                create_i64_from_decimal(text)?,
            ))),
            TypeId::U_8 => Ok(Emission::Value(ConstantValue::U8(create_u8_from_decimal(
                text,
            )?))),
            TypeId::U_16 => Ok(Emission::Value(ConstantValue::U16(
                create_u16_from_decimal(text)?,
            ))),
            TypeId::U_32 => Ok(Emission::Value(ConstantValue::U32(
                create_u32_from_decimal(text)?,
            ))),
            TypeId::U_64 => Ok(Emission::Value(ConstantValue::U64(
                create_u64_from_decimal(text)?,
            ))),
            TypeId::U_128 => Ok(Emission::Value(ConstantValue::U128(
                create_u128_from_decimal(text)?,
            ))),
            #[cfg(target_pointer_width = "32")]
            TypeId::U_SIZE => Ok(Emission::Value(ConstantValue::U32(
                create_u32_from_decimal(text)?,
            ))),
            #[cfg(target_pointer_width = "64")]
            TypeId::U_SIZE => Ok(Emission::Value(ConstantValue::U64(
                create_u64_from_decimal(text)?,
            ))),
            _ => Err(CompileError::InvalidTypeBinding(type_id)),
        }
    }

    fn emit_string_expression(
        &mut self,
        _: SyntaxReader,
        _: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        todo!()
    }

    fn emit_array_expression(
        &mut self,
        reader: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        let ArrayExpression { elements } = reader.as_component()?;

        let mut array_instructions = InstructionsEmission::new();

        let type_id = *self.resolver.get_type_binding(&reader.id)?;
        let array_registers = match target {
            ExpressionTarget::Claimed(registers) => registers,
            ExpressionTarget::Unclaimed(allocation_kind) => {
                self.allocate_registers(type_id, allocation_kind)?
            }
            ExpressionTarget::None => return Err(CompileError::ExpectedAllocation),
        };
        let registers_per_element = array_registers.len() / elements.len();

        for (index, element) in elements.into_iter().enumerate() {
            let register_start = index * registers_per_element;
            let register_end = register_start + registers_per_element;

            let element_target = ExpressionTarget::Claimed(RegisterClaims {
                claims: array_registers.claims[register_start..register_end].into(),
                kind: array_registers.kind,
            });
            let element_emission = self.emit_expression(element, element_target)?;

            match element_emission {
                Emission::Instructions(instructions) => array_instructions.merge(instructions),
                _ => return Err(CompileError::InvalidEmission),
            }
        }

        Ok(Emission::Instructions(array_instructions))
    }

    fn emit_array_repeat_expression(
        &mut self,
        reader: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        let ArrayRepeatExpression { element, .. } = reader.as_component()?;

        let mut array_instructions = InstructionsEmission::new();

        let array_type_id = *self.resolver.get_type_binding(&reader.id)?;
        let array_length =
            if let Type::Array { length, .. } = *self.resolver.types.get_type(array_type_id)? {
                length
            } else {
                return Err(CompileError::ExpectedArrayType(array_type_id));
            };
        let array_registers = match target {
            ExpressionTarget::Claimed(allocation) => allocation,
            ExpressionTarget::Unclaimed(allocation_kind) => {
                self.allocate_registers(array_type_id, allocation_kind)?
            }
            ExpressionTarget::None => return Err(CompileError::ExpectedAllocation),
        };
        let registers_per_element = array_registers.len() / array_length as usize;
        let first_element_target = ExpressionTarget::Claimed(RegisterClaims {
            claims: array_registers.claims[0..registers_per_element].into(),
            kind: array_registers.kind,
        });

        match self.emit_expression(element, first_element_target)? {
            Emission::Instructions(instructions) => {
                array_instructions.merge(instructions);
            }
            _ => return Err(CompileError::InvalidEmission),
        }

        let first_element_registers = &array_registers.claims[0..registers_per_element];

        for index in 1..array_length {
            let register_start = index * registers_per_element;
            let register_end = register_start + registers_per_element;
            let element_registers = &array_registers.claims[register_start..register_end];

            for (source_register, destination_register) in
                first_element_registers.iter().zip(element_registers)
            {
                let move_instruction = Instruction::r#move(
                    destination_register.index,
                    source_register.operand_type,
                    MemoryKind::REGISTER,
                    source_register.index,
                );

                array_instructions.push(move_instruction);
            }
        }

        array_instructions.set_target(Some(array_registers));

        Ok(Emission::Instructions(array_instructions))
    }

    fn emit_index_expression(
        &mut self,
        reader: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        let IndexExpression { collection, index } = reader.as_component()?;

        let collection_emission = self.emit_expression(
            collection,
            ExpressionTarget::Unclaimed(RegisterKind::Temporary),
        )?;

        let (list_registers, list_instructions) =
            match collection_emission {
                Emission::Place(Place::Register(registers)) => (registers, None),
                Emission::Instructions(instructions) => {
                    let registers = instructions.target_registers.clone().ok_or(
                        CompileError::ExpectedValue {
                            source_id: collection.source_id(),
                            syntax_id: collection.id,
                        },
                    )?;

                    (registers, Some(instructions))
                }
                _ => {
                    return Err(CompileError::ExpectedValue {
                        source_id: collection.source_id(),
                        syntax_id: collection.id,
                    });
                }
            };

        let list_type_id = *self.resolver.get_type_binding(&collection.id)?;
        let list_type = *self.resolver.types.get_type(list_type_id)?;

        let (element_type_id, array_length) = match list_type {
            Type::Array {
                element_type_id,
                length,
            } => (element_type_id, length),
            _ => {
                return Err(CompileError::CannotIndex {
                    type_id: list_type_id,
                    position: collection.position(),
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

            let range_start_str = self.source.get_content(range_start.position())?;
            let range_end_str = self.source.get_content(range_end.position())?;

            let start_index = create_usize_from_decimal(range_start_str)?;
            let end_index = create_usize_from_decimal(range_end_str)?;

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

            let slice_registers: SmallVec<[RegisterClaim; 4]> = list_registers
                .claims
                .iter()
                .skip(start_register_offset)
                .take(slice_register_count)
                .copied()
                .collect();

            let slice_allocation = RegisterClaims {
                claims: slice_registers,
                kind: RegisterKind::Local,
            };

            if let Some(mut instructions) = list_instructions {
                instructions.set_target(Some(slice_allocation));

                return Ok(Emission::Instructions(instructions));
            }

            return Ok(Emission::Place(Place::Register(slice_allocation)));
        }

        if index.node.kind == SyntaxKind::IntegerExpression {
            let index_str = self.source.get_content(index.position())?;
            let constant_index = create_usize_from_decimal(index_str)?;

            if constant_index >= array_length {
                return Err(CompileError::IndexOutOfBounds {
                    index: constant_index,
                    length: array_length,
                    position: index.position(),
                });
            }

            let register_offset = constant_index * element_register_count;

            let element_registers: SmallVec<[RegisterClaim; 4]> = list_registers
                .claims
                .iter()
                .skip(register_offset)
                .take(element_register_count)
                .copied()
                .collect();

            let element_allocation = RegisterClaims {
                claims: element_registers,
                kind: RegisterKind::Local,
            };

            if let Some(mut instructions) = list_instructions {
                instructions.set_target(Some(element_allocation));

                return Ok(Emission::Instructions(instructions));
            }

            return Ok(Emission::Place(Place::Register(element_allocation)));
        }

        let index_emission =
            self.emit_expression(index, ExpressionTarget::Unclaimed(RegisterKind::Temporary))?;

        let (index_memory, index_index) = if let Emission::Value(constant) = &index_emission
            && let Some(encoded) = constant.encoded_u16()
        {
            (MemoryKind::ENCODED, encoded)
        } else {
            let mut index_instructions_tmp = InstructionsEmission::new();
            let index_place =
                self.place_emission(index_emission, &mut index_instructions_tmp, &index)?;

            match index_place {
                Place::Constant { index, .. } => (MemoryKind::CONSTANT, index),
                Place::Register(ref allocation) if allocation.len() == 1 => {
                    (MemoryKind::REGISTER, allocation.claims[0].index)
                }
                _ => {
                    return Err(CompileError::ExpectedIntegerIndex {
                        found: *self.resolver.get_type_binding(&index.id)?,
                        position: index.position(),
                    });
                }
            }
        };

        let base_index = list_registers.expect_base_index()?;

        let element_type_id = *self.resolver.get_type_binding(&reader.id)?;
        let destination = match target {
            ExpressionTarget::Claimed(registers) => registers,
            ExpressionTarget::Unclaimed(allocation_kind) => {
                self.allocate_registers(element_type_id, allocation_kind)?
            }
            ExpressionTarget::None => return Err(CompileError::ExpectedAllocation),
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

    fn emit_range_expression(
        &mut self,
        reader: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        let RangeExpression { start, end } = reader.as_component()?;

        let type_id = *self.resolver.get_type_binding(&reader.id)?;
        let target_registers = match target {
            ExpressionTarget::Claimed(registers) => registers,
            ExpressionTarget::Unclaimed(allocation_kind) => {
                self.allocate_registers(type_id, allocation_kind)?
            }
            ExpressionTarget::None => return Err(CompileError::ExpectedAllocation),
        };

        let mut range_instructions = InstructionsEmission::new();

        for (field_expression, destination) in
            [start, end].into_iter().zip(&target_registers.claims)
        {
            let field_emission = self.emit_expression(
                field_expression,
                ExpressionTarget::Unclaimed(RegisterKind::Temporary),
            )?;

            if let Emission::Value(constant) = &field_emission
                && let Some(encoded) = constant.encoded_u16()
            {
                let move_instruction = Instruction::r#move(
                    destination.index,
                    constant.operand_type(),
                    MemoryKind::ENCODED,
                    encoded,
                );

                range_instructions.push(move_instruction);
                continue;
            }

            let field_place =
                self.place_emission(field_emission, &mut range_instructions, &field_expression)?;

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
                Place::Register(ref allocation) => {
                    for register in &allocation.claims {
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

        range_instructions.set_target(Some(target_registers));

        Ok(Emission::Instructions(range_instructions))
    }

    fn emit_path_expression(
        &mut self,
        reader: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        let declaration_id = *self.resolver.get_declaration_binding(&reader.id)?;

        if let Some(local) = self.locals.get(&declaration_id) {
            match local {
                Local::Place(place) => return Ok(Emission::Place(place.clone())),
                Local::Constant(value) => return Ok(Emission::Value(*value)),
            }
        }

        let declaration = self.resolver.declarations.get_declaration(declaration_id)?;

        match declaration.definition {
            Definition::Function { .. } => {
                let type_id = *self.resolver.get_type_binding(&reader.id)?;
                let callee_type = *self.resolver.types.get_type(type_id)?;

                let type_arguments =
                    if let Type::FunctionDefinition { type_arguments, .. } = callee_type {
                        type_arguments
                            .as_range()
                            .map(|index| {
                                let type_id = *self.resolver.types.get_type_member(index)?;

                                self.resolver.resolve_type(type_id)
                            })
                            .try_collect::<SmallVec<[TypeId; 4]>>()?
                    } else {
                        SmallVec::new()
                    };

                let cache_key = (declaration_id, type_arguments);
                let prototype_id =
                    if let Some(existing) = self.resolver.get_cached_prototype(&cache_key) {
                        existing
                    } else {
                        let reserved = self.resolver.reserve_prototype_id();

                        self.resolver.cache_prototype(cache_key, reserved);
                        self.compilation_stack.push(CompilationRequest {
                            declaration_id,
                            prototype_id: reserved,
                        });

                        reserved
                    };

                Ok(Emission::Place(Place::Constant {
                    operand_type: OperandType::FUNCTION,
                    index: prototype_id.inner(),
                }))
            }
            Definition::Constant { .. } => {
                let value = self
                    .resolver
                    .get_constant_item_value(&declaration_id)
                    .ok_or_else(|| CompileError::ExpectedValue {
                        source_id: reader.source_id(),
                        syntax_id: reader.id,
                    })?;

                Ok(Emission::Value(value))
            }
            _ => Err(CompileError::ExpectedValue {
                source_id: reader.source_id(),
                syntax_id: reader.id,
            }),
        }
    }

    fn emit_struct_expression(
        &mut self,
        reader: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        let StructExpression {
            path: _,
            fields: struct_fields,
        } = reader.as_component()?;
        let StructExpressionStructFields {
            name_expression_pairs,
        } = struct_fields.as_component()?;

        let type_id = *self.resolver.get_type_binding(&reader.id)?;
        let target_registers = match target {
            ExpressionTarget::Claimed(registers) => registers,
            ExpressionTarget::Unclaimed(allocation_kind) => {
                self.allocate_registers(type_id, allocation_kind)?
            }
            ExpressionTarget::None => return Err(CompileError::ExpectedAllocation),
        };

        let mut struct_instructions = InstructionsEmission::new();

        for ((_, field_expression), destination) in
            name_expression_pairs.zip(&target_registers.claims)
        {
            let field_emission = self.emit_expression(
                field_expression,
                ExpressionTarget::Unclaimed(RegisterKind::Temporary),
            )?;

            if let Emission::Value(constant) = &field_emission
                && let Some(encoded) = constant.encoded_u16()
            {
                let move_instruction = Instruction::r#move(
                    destination.index,
                    constant.operand_type(),
                    MemoryKind::ENCODED,
                    encoded,
                );

                struct_instructions.push(move_instruction);
                continue;
            }

            let field_place =
                self.place_emission(field_emission, &mut struct_instructions, &field_expression)?;

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
                Place::Register(ref allocation) => {
                    for register in &allocation.claims {
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

        struct_instructions.set_target(Some(target_registers));

        Ok(Emission::Instructions(struct_instructions))
    }

    fn emit_grouped_expression(
        &mut self,
        reader: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        let GroupedExpression { expression } = reader.as_component()?;
        let expression = expression.ok_or(CompileError::ExpectedValue {
            source_id: reader.source_id(),
            syntax_id: reader.id,
        })?;

        self.emit_expression(expression, target)
    }

    fn emit_block_expression(
        &mut self,
        reader: SyntaxReader<'_>,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        let BlockExpression { children } = reader.as_component()?;
        let saved_register_tracker = self.register_tracker;
        let mut block_instructions = InstructionsEmission::new();
        let mut last_emission = None;

        self.enter_drop_context();

        let child_count = children.len();

        let type_id = *self.resolver.get_type_binding(&reader.id)?;
        let target_registers = match target {
            ExpressionTarget::Claimed(registers) => registers,
            ExpressionTarget::Unclaimed(allocation_kind) => {
                self.allocate_registers(type_id, allocation_kind)?
            }
            ExpressionTarget::None => return Err(CompileError::ExpectedAllocation),
        };

        for (index, child) in children.enumerate() {
            let is_last = index == child_count - 1;

            if child.node.kind.is_statement() {
                if let Some(instructions) = self.emit_statement(child)? {
                    block_instructions.merge(instructions);
                }

                continue;
            }

            if !is_last {
                let expression_emission = self
                    .emit_expression(child, ExpressionTarget::Unclaimed(RegisterKind::Temporary))?;

                if let Emission::Instructions(expression_instructions) = expression_emission {
                    block_instructions.merge(expression_instructions);
                }

                continue;
            }

            last_emission = Some(
                self.emit_expression(child, ExpressionTarget::Claimed(target_registers.clone()))?,
            );
            break;
        }

        let mut result_emission = if let Some(last_emission) = last_emission {
            match last_emission {
                Emission::Instructions(instructions) => {
                    if block_instructions.is_empty() {
                        Emission::Instructions(instructions)
                    } else {
                        block_instructions.merge(instructions);

                        Emission::Instructions(block_instructions)
                    }
                }
                _ => return Err(CompileError::InvalidEmission),
            }
        } else {
            Emission::Instructions(block_instructions)
        };

        if let Emission::Instructions(instructions) = &mut result_emission {
            self.exit_drop_context(instructions);
        } else {
            self.drop_stack.pop();
        }

        self.register_tracker = saved_register_tracker;

        let target_allocation = match &result_emission {
            Emission::Instructions(instructions) => instructions.target_registers.as_ref(),
            Emission::Place(Place::Register(target_allocation)) => Some(target_allocation),
            _ => None,
        };

        if let Some(target_allocation) = target_allocation
            && let Some(last_register) = target_allocation.claims.last()
        {
            let target_end =
                last_register.index + last_register.operand_type.register_width().as_u16();

            match target_allocation.kind {
                RegisterKind::Local => {
                    self.register_tracker.next_local =
                        self.register_tracker.next_local.max(target_end);
                    self.register_tracker.next_temporary = self
                        .register_tracker
                        .next_temporary
                        .max(self.register_tracker.next_local);
                }
                RegisterKind::Temporary => {
                    self.register_tracker.next_temporary =
                        self.register_tracker.next_temporary.max(target_end);
                }
                RegisterKind::Reserved => {
                    self.register_tracker.next_reserved =
                        self.register_tracker.next_reserved.max(target_end);
                }
            }

            self.register_tracker.max = self.register_tracker.max.max(target_end);
        }

        Ok(result_emission)
    }

    fn emit_if_expression(
        &mut self,
        reader: SyntaxReader<'_>,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        let IfExpression {
            condition,
            then_branch,
            else_branch,
        } = reader.as_component()?;

        let mut if_instructions = InstructionsEmission::new();

        let condition_emission = self.emit_expression(
            condition,
            ExpressionTarget::Unclaimed(RegisterKind::Temporary),
        )?;

        self.handle_condition_emission(&mut if_instructions, condition_emission, &condition, true)?;

        let type_id = *self.resolver.get_type_binding(&reader.id)?;
        let target_registers = match target {
            ExpressionTarget::Claimed(registers) => registers,
            ExpressionTarget::Unclaimed(allocation_kind) => {
                self.allocate_registers(type_id, allocation_kind)?
            }
            ExpressionTarget::None => return Err(CompileError::ExpectedAllocation),
        };
        let jump_over_then_id = self.create_jump_id();
        let start_else_anchor_count = self.jump_over_branch_ids.len();

        if_instructions.push_jump_anchor(JumpAnchor::ForwardFromHere {
            id: jump_over_then_id,
        });

        {
            let saved_register_tracker = self.register_tracker;

            let then_emission = self.emit_block_expression(
                then_branch,
                ExpressionTarget::Claimed(target_registers.clone()),
            )?;
            let then_register_tracker = self.register_tracker;

            self.handle_branch_emission(
                then_emission,
                &mut if_instructions,
                &target_registers,
                then_branch,
            )?;

            if let Some(else_branch) = else_branch {
                let jump_over_else_id = self.create_jump_id();
                self.jump_over_branch_ids.push(jump_over_else_id);

                if_instructions.push(Instruction::no_op());
                if_instructions.push_jump_anchor(JumpAnchor::ForwardFromHere {
                    id: jump_over_else_id,
                });
                if_instructions.push_jump_anchor(JumpAnchor::ForwardToNext {
                    id: jump_over_then_id,
                });

                self.register_tracker = saved_register_tracker;
                let else_emission = self.emit_block_expression(
                    else_branch,
                    ExpressionTarget::Claimed(target_registers.clone()),
                )?;
                let else_register_tracker = self.register_tracker;

                self.handle_branch_emission(
                    else_emission,
                    &mut if_instructions,
                    &target_registers,
                    else_branch,
                )?;

                self.register_tracker = else_register_tracker;
                self.register_tracker.max =
                    self.register_tracker.max.max(then_register_tracker.max);
                if_instructions.set_target(Some(target_registers));
            } else {
                self.register_tracker = then_register_tracker;
                if_instructions.push_jump_anchor(JumpAnchor::ForwardToNext {
                    id: jump_over_then_id,
                });
                if_instructions.set_target(Some(target_registers));
            }
        }

        let end_else_anchor_count = self.jump_over_branch_ids.len();

        for index in start_else_anchor_count..end_else_anchor_count {
            let jump_id = self.jump_over_branch_ids[index];

            if_instructions.push_jump_anchor(JumpAnchor::ForwardToNext { id: jump_id });
        }

        Ok(Emission::Instructions(if_instructions))
    }

    fn emit_math_expression(
        &mut self,
        reader: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        let MathExpression { left, right } = reader.as_component()?;

        let left_emission =
            self.emit_expression(left, ExpressionTarget::Unclaimed(RegisterKind::Temporary))?;
        let right_emission =
            self.emit_expression(right, ExpressionTarget::Unclaimed(RegisterKind::Temporary))?;

        if let (Emission::Value(left_value), Emission::Value(right_value)) =
            (&left_emission, &right_emission)
        {
            let combined = match reader.node.kind {
                SyntaxKind::AdditionExpression | SyntaxKind::AdditionAssignmentExpression => {
                    left_value.add(*right_value, &reader)?
                }
                SyntaxKind::SubtractionExpression | SyntaxKind::SubtractionAssignmentExpression => {
                    left_value.subtract(*right_value, &reader)?
                }
                SyntaxKind::MultiplicationExpression
                | SyntaxKind::MultiplicationAssignmentExpression => {
                    left_value.multiply(*right_value, &reader)?
                }
                SyntaxKind::DivisionExpression | SyntaxKind::DivisionAssignmentExpression => {
                    left_value.divide(*right_value, &reader)?
                }
                SyntaxKind::ModuloExpression | SyntaxKind::ModuloAssignmentExpression => {
                    left_value.modulo(*right_value, &reader)?
                }
                SyntaxKind::ExponentExpression | SyntaxKind::ExponentAssignmentExpression => {
                    left_value.exponentiate(*right_value, &reader)?
                }
                _ => {
                    return Err(CompileError::UnexpectedSyntax {
                        expected: &[],
                        found: reader.node.kind,
                    });
                }
            };

            return Ok(Emission::Value(combined));
        }

        let mut math_emission = InstructionsEmission::new();

        let place_target = if let Emission::Place(Place::Register(allocation)) = &left_emission {
            Some(allocation.kind)
        } else if let Emission::Instructions(instructions) = &left_emission {
            instructions.target_registers.as_ref().map(|r| r.kind)
        } else {
            None
        };
        let (left_memory, left_index, _) =
            self.handle_operand_emission(&mut math_emission, left_emission, &left)?;
        let (right_memory, right_index, _) =
            self.handle_operand_emission(&mut math_emission, right_emission, &right)?;

        let type_id = *self.resolver.get_type_binding(&reader.id)?;

        let is_assignment = matches!(
            reader.node.kind,
            SyntaxKind::AdditionAssignmentExpression
                | SyntaxKind::SubtractionAssignmentExpression
                | SyntaxKind::MultiplicationAssignmentExpression
                | SyntaxKind::DivisionAssignmentExpression
                | SyntaxKind::ModuloAssignmentExpression
                | SyntaxKind::ExponentAssignmentExpression
        );

        let register = if is_assignment {
            let kind = place_target.unwrap_or(RegisterKind::Temporary);
            let allocation = self.allocate_registers(type_id, kind)?;
            let reg = allocation.expect_single()?;
            math_emission.set_target(Some(allocation));
            reg
        } else {
            let target_registers = match target {
                ExpressionTarget::Claimed(registers) => registers,
                ExpressionTarget::Unclaimed(allocation_kind) => {
                    self.allocate_registers(type_id, allocation_kind)?
                }
                ExpressionTarget::None => return Err(CompileError::ExpectedAllocation),
            };
            let reg = target_registers.expect_single()?;
            math_emission.set_target(Some(target_registers));
            reg
        };

        let destination = if is_assignment {
            left_index
        } else {
            register.index
        };

        let math_instruction = match reader.node.kind {
            SyntaxKind::AdditionExpression | SyntaxKind::AdditionAssignmentExpression => {
                Instruction::add(
                    destination,
                    register.operand_type,
                    left_memory,
                    left_index,
                    right_memory,
                    right_index,
                )
            }
            SyntaxKind::SubtractionExpression | SyntaxKind::SubtractionAssignmentExpression => {
                Instruction::subtract(
                    destination,
                    register.operand_type,
                    left_memory,
                    left_index,
                    right_memory,
                    right_index,
                )
            }
            SyntaxKind::MultiplicationExpression
            | SyntaxKind::MultiplicationAssignmentExpression => Instruction::multiply(
                destination,
                register.operand_type,
                left_memory,
                left_index,
                right_memory,
                right_index,
            ),
            SyntaxKind::DivisionExpression | SyntaxKind::DivisionAssignmentExpression => {
                Instruction::divide(
                    destination,
                    register.operand_type,
                    left_memory,
                    left_index,
                    right_memory,
                    right_index,
                )
            }
            SyntaxKind::ModuloExpression | SyntaxKind::ModuloAssignmentExpression => {
                Instruction::modulo(
                    destination,
                    register.operand_type,
                    left_memory,
                    left_index,
                    right_memory,
                    right_index,
                )
            }
            SyntaxKind::ExponentExpression | SyntaxKind::ExponentAssignmentExpression => {
                Instruction::power(
                    destination,
                    register.operand_type,
                    left_memory,
                    left_index,
                    right_memory,
                    right_index,
                )
            }
            _ => {
                return Err(CompileError::UnexpectedSyntax {
                    expected: &[
                        SyntaxKind::AdditionExpression,
                        SyntaxKind::SubtractionExpression,
                        SyntaxKind::MultiplicationExpression,
                        SyntaxKind::DivisionExpression,
                        SyntaxKind::ModuloExpression,
                        SyntaxKind::ExponentExpression,
                    ],
                    found: reader.node.kind,
                });
            }
        };

        math_emission.push(math_instruction);

        Ok(Emission::Instructions(math_emission))
    }

    fn emit_comparison_expression(
        &mut self,
        reader: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        let ComparisonExpression { left, right } = reader.as_component()?;

        let left_emission =
            self.emit_expression(left, ExpressionTarget::Unclaimed(RegisterKind::Temporary))?;
        let right_emission =
            self.emit_expression(right, ExpressionTarget::Unclaimed(RegisterKind::Temporary))?;

        if let Emission::Value(left_constant) = left_emission
            && let Emission::Value(right_constant) = right_emission
        {
            let combined = match reader.node.kind {
                SyntaxKind::EqualExpression => left_constant.equal(right_constant, &reader)?,
                SyntaxKind::NotEqualExpression => {
                    left_constant.not_equal(right_constant, &reader)?
                }
                SyntaxKind::LessThanExpression => {
                    left_constant.less_than(right_constant, &reader)?
                }
                SyntaxKind::GreaterThanExpression => {
                    left_constant.greater_than(right_constant, &reader)?
                }
                SyntaxKind::LessThanOrEqualExpression => {
                    left_constant.less_than_or_equal(right_constant, &reader)?
                }
                SyntaxKind::GreaterThanOrEqualExpression => {
                    left_constant.greater_than_or_equal(right_constant, &reader)?
                }
                _ => {
                    return Err(CompileError::UnexpectedSyntax {
                        expected: &[],
                        found: reader.node.kind,
                    });
                }
            };

            return Ok(Emission::Value(combined));
        }

        let mut comparison_emission = InstructionsEmission::new();

        let (left_memory, left_index, left_operand_types) =
            self.handle_operand_emission(&mut comparison_emission, left_emission, &left)?;
        let left_operand_type = if left_operand_types.len() == 1 {
            left_operand_types[0]
        } else {
            return Err(CompileError::CannotApplyOperator {
                operator: reader.node.kind,
                type_id: *self.resolver.get_type_binding(&left.id)?,
                operand_position: left.position(),
            });
        };
        let (right_memory, right_index, _) =
            self.handle_operand_emission(&mut comparison_emission, right_emission, &right)?;

        let type_id = *self.resolver.get_type_binding(&reader.id)?;
        let target_registers = match target {
            ExpressionTarget::Claimed(registers) => registers,
            ExpressionTarget::Unclaimed(allocation_kind) => {
                self.allocate_registers(type_id, allocation_kind)?
            }
            ExpressionTarget::None => return Err(CompileError::ExpectedAllocation),
        };
        let register = target_registers.expect_single()?;

        let comparison_instruction = match reader.node.kind {
            SyntaxKind::EqualExpression => Instruction::equal(
                true,
                left_operand_type,
                left_memory,
                left_index,
                right_memory,
                right_index,
            ),
            SyntaxKind::NotEqualExpression => Instruction::equal(
                false,
                left_operand_type,
                left_memory,
                left_index,
                right_memory,
                right_index,
            ),
            SyntaxKind::LessThanExpression => Instruction::less(
                true,
                left_operand_type,
                left_memory,
                left_index,
                right_memory,
                right_index,
            ),
            SyntaxKind::GreaterThanExpression => Instruction::less_equal(
                false,
                left_operand_type,
                left_memory,
                left_index,
                right_memory,
                right_index,
            ),
            SyntaxKind::LessThanOrEqualExpression => Instruction::less_equal(
                true,
                left_operand_type,
                left_memory,
                left_index,
                right_memory,
                right_index,
            ),
            SyntaxKind::GreaterThanOrEqualExpression => Instruction::less(
                false,
                left_operand_type,
                left_memory,
                left_index,
                right_memory,
                right_index,
            ),
            _ => {
                return Err(CompileError::UnexpectedSyntax {
                    expected: &[
                        SyntaxKind::EqualExpression,
                        SyntaxKind::NotEqualExpression,
                        SyntaxKind::LessThanExpression,
                        SyntaxKind::GreaterThanExpression,
                        SyntaxKind::LessThanOrEqualExpression,
                        SyntaxKind::GreaterThanOrEqualExpression,
                    ],
                    found: reader.node.kind,
                });
            }
        };
        let load_false_instruction = Instruction::move_with_jump(
            register.index,
            OperandType::BOOLEAN,
            MemoryKind::ENCODED,
            false as u16,
            1,
            true,
        );
        let load_true_instruction = Instruction::r#move(
            register.index,
            OperandType::BOOLEAN,
            MemoryKind::ENCODED,
            true as u16,
        );

        comparison_emission.push(comparison_instruction);
        comparison_emission.push(load_false_instruction);
        comparison_emission.push(load_true_instruction);
        comparison_emission.set_target(Some(target_registers));

        Ok(Emission::Instructions(comparison_emission))
    }

    fn emit_logic_expression(
        &mut self,
        reader: SyntaxReader<'_>,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        let LogicExpression { left, right } = reader.as_component()?;

        let left_emission =
            self.emit_expression(left, ExpressionTarget::Unclaimed(RegisterKind::Temporary))?;
        let right_emission =
            self.emit_expression(right, ExpressionTarget::Unclaimed(RegisterKind::Temporary))?;

        if let Emission::Value(left_constant) = left_emission
            && let Emission::Value(right_constant) = right_emission
        {
            let combined = match reader.node.kind {
                SyntaxKind::AndExpression => left_constant.and(right_constant, &reader)?,
                SyntaxKind::OrExpression => left_constant.or(right_constant, &reader)?,
                _ => {
                    return Err(CompileError::UnexpectedSyntax {
                        expected: &[],
                        found: reader.node.kind,
                    });
                }
            };

            return Ok(Emission::Value(combined));
        }

        let mut logic_instructions = InstructionsEmission::new();

        let (left_memory, left_index, _) =
            self.handle_operand_emission(&mut logic_instructions, left_emission, &left)?;
        let (right_memory, right_index, _) =
            self.handle_operand_emission(&mut logic_instructions, right_emission, &right)?;

        let type_id = *self.resolver.get_type_binding(&reader.id)?;
        let target_registers = match target {
            ExpressionTarget::Claimed(registers) => registers,
            ExpressionTarget::Unclaimed(allocation_kind) => {
                self.allocate_registers(type_id, allocation_kind)?
            }
            ExpressionTarget::None => return Err(CompileError::ExpectedAllocation),
        };
        let register = target_registers.expect_single()?;

        let test_instruction = match reader.node.kind {
            SyntaxKind::AndExpression => Instruction::test(false, left_memory, left_index, 1),
            SyntaxKind::OrExpression => Instruction::test(true, left_memory, left_index, 1),
            _ => {
                return Err(CompileError::UnexpectedSyntax {
                    expected: &[SyntaxKind::AndExpression, SyntaxKind::OrExpression],
                    found: reader.node.kind,
                });
            }
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
        logic_instructions.set_target(Some(target_registers));

        Ok(Emission::Instructions(logic_instructions))
    }

    fn emit_negation_expression(
        &mut self,
        reader: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        let NegationExpression { operand } = reader.as_component()?;

        let expression_emission = self.emit_expression(
            operand,
            ExpressionTarget::Unclaimed(RegisterKind::Temporary),
        )?;

        if let Emission::Value(constant) = &expression_emission
            && operand.node.kind != SyntaxKind::PathExpression
        {
            let negated = constant.negate(&operand)?;

            return Ok(Emission::Value(negated));
        }

        let mut negation_emission = InstructionsEmission::new();

        let (operand_memory, operand_index, _) =
            self.handle_operand_emission(&mut negation_emission, expression_emission, &operand)?;
        let type_id = *self.resolver.get_type_binding(&reader.id)?;
        let target_registers = match target {
            ExpressionTarget::Claimed(registers) => registers,
            ExpressionTarget::Unclaimed(allocation_kind) => {
                self.allocate_registers(type_id, allocation_kind)?
            }
            ExpressionTarget::None => return Err(CompileError::ExpectedAllocation),
        };
        let register = target_registers.expect_single()?;

        let negate_instruction = Instruction::negate(
            register.index,
            register.operand_type,
            operand_memory,
            operand_index,
        );

        negation_emission.push(negate_instruction);
        negation_emission.set_target(Some(target_registers));

        Ok(Emission::Instructions(negation_emission))
    }

    fn emit_not_expression(
        &mut self,
        reader: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        let NotExpression { operand } = reader.as_component()?;

        let expression_emission = self.emit_expression(
            operand,
            ExpressionTarget::Unclaimed(RegisterKind::Temporary),
        )?;

        if let Emission::Value(constant) = &expression_emission
            && operand.node.kind != SyntaxKind::PathExpression
        {
            let negated = constant.negate(&operand)?;

            return Ok(Emission::Value(negated));
        }

        let mut negation_emission = InstructionsEmission::new();

        let (operand_memory, operand_index, _) =
            self.handle_operand_emission(&mut negation_emission, expression_emission, &operand)?;
        let type_id = *self.resolver.get_type_binding(&reader.id)?;
        let target_registers = match target {
            ExpressionTarget::Claimed(registers) => registers,
            ExpressionTarget::Unclaimed(allocation_kind) => {
                self.allocate_registers(type_id, allocation_kind)?
            }
            ExpressionTarget::None => return Err(CompileError::ExpectedAllocation),
        };
        let register = target_registers.expect_single()?;

        let negate_instruction = Instruction::negate(
            register.index,
            register.operand_type,
            operand_memory,
            operand_index,
        );

        negation_emission.push(negate_instruction);
        negation_emission.set_target(Some(target_registers));

        Ok(Emission::Instructions(negation_emission))
    }

    fn emit_while_expression(
        &mut self,
        reader: SyntaxReader<'_>,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        let WhileExpression { condition, body } = reader.as_component()?;

        let mut while_instructions = InstructionsEmission::new();

        let condition_emission = self.emit_expression(
            condition,
            ExpressionTarget::Unclaimed(RegisterKind::Temporary),
        )?;

        self.handle_condition_emission(
            &mut while_instructions,
            condition_emission,
            &condition,
            false,
        )?;

        let jump_forward_id = self.create_jump_id();
        let jump_backward_id = self.create_jump_id();

        while_instructions.push_jump_anchor(JumpAnchor::LoopStartHere {
            forward_id: jump_forward_id,
        });

        let body_emission = self.emit_block_expression(body, target)?;

        if let Emission::Instructions(instructions) = body_emission {
            while_instructions.merge(instructions);
        }

        for break_id in self.jump_over_branch_ids.drain(..) {
            while_instructions.push_jump_anchor(JumpAnchor::ForwardToNext { id: break_id });
        }

        while_instructions.push_jump_anchor(JumpAnchor::LoopEndOnNext {
            forward_id: jump_forward_id,
            backward_id: jump_backward_id,
        });
        while_instructions.set_target(None);

        Ok(Emission::Instructions(while_instructions))
    }

    fn emit_break_expression(
        &mut self,
        _: SyntaxReader<'_>,
        _: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        let break_id = self.create_jump_id();

        self.jump_over_branch_ids.push(break_id);

        let mut break_emission = InstructionsEmission::new();

        break_emission.push(Instruction::no_op());
        break_emission.push_jump_anchor(JumpAnchor::ForwardFromHere { id: break_id });

        Ok(Emission::Instructions(break_emission))
    }

    fn emit_call_expression(
        &mut self,
        reader: SyntaxReader<'_>,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        let CallExpression { callee, arguments } = reader.as_component()?;

        todo!()
    }

    fn emit_field_access_expression(
        &mut self,
        reader: SyntaxReader,
        _: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        let FieldAccessExpression {
            struct_expression,
            field_name,
        } = reader.as_component()?;

        let operand_emission = self.emit_expression(
            struct_expression,
            ExpressionTarget::Unclaimed(RegisterKind::Temporary),
        )?;

        let struct_registers = match operand_emission {
            Emission::Place(Place::Register(registers)) => registers,
            emission => {
                let mut field_access_instructions = InstructionsEmission::new();
                let place = self.place_emission(
                    emission,
                    &mut field_access_instructions,
                    &struct_expression,
                )?;

                match place {
                    Place::Register(registers) => registers,
                    _ => {
                        return Err(CompileError::ExpectedValue {
                            source_id: struct_expression.source_id(),
                            syntax_id: struct_expression.id,
                        });
                    }
                }
            }
        };

        let field_declaration_id = *self.resolver.get_declaration_binding(&field_name.id)?;
        let field_declaration = self
            .resolver
            .declarations
            .get_declaration(field_declaration_id)?;

        let parent_struct = match field_declaration.definition {
            Definition::Field { parent_struct, .. } => parent_struct,
            _ => {
                return Err(CompileError::ExpectedValue {
                    source_id: field_name.source_id(),
                    syntax_id: field_name.id,
                });
            }
        };

        let struct_declaration = self.resolver.declarations.get_declaration(parent_struct)?;

        let fields = match struct_declaration.definition {
            Definition::StructType {
                fields: Some(fields),
                ..
            } => fields,
            _ => {
                return Err(CompileError::ExpectedValue {
                    source_id: field_name.source_id(),
                    syntax_id: field_name.id,
                });
            }
        };

        let field_entries = self.resolver.scopes.get_members(fields);

        let mut register_offset = 0usize;

        for &(_, field_id) in field_entries {
            if field_id == field_declaration_id {
                break;
            }

            let field_declaration = self.resolver.declarations.get_declaration(field_id)?;

            if let Definition::Field { type_id, .. } = field_declaration.definition {
                let operand_types = self.resolver.get_operand_types(type_id)?;
                register_offset += operand_types.len();
            }
        }

        let field_register = struct_registers.claims.get(register_offset);

        match field_register {
            Some(register) => Ok(Emission::Place(Place::Register(RegisterClaims {
                claims: smallvec![*register],
                kind: struct_registers.kind,
            }))),
            None => Err(CompileError::ExpectedValue {
                source_id: field_name.source_id(),
                syntax_id: field_name.id,
            }),
        }
    }
}

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
    instructions: Vec<(Instruction, JumpAnchor::SmallVec)>,
    target_registers: Option<RegisterClaims>,
    pending_drops: Vec<u16>,
}

impl InstructionsEmission {
    fn new() -> Self {
        Self {
            instructions: Vec::new(),
            target_registers: None,
            pending_drops: Vec::new(),
        }
    }

    fn with_instruction(instruction: Instruction) -> Self {
        Self {
            instructions: vec![(instruction, SmallVec::new())],
            target_registers: None,
            pending_drops: Vec::new(),
        }
    }

    fn with_instruction_and_target(instruction: Instruction, target: RegisterClaims) -> Self {
        Self {
            instructions: vec![(instruction, SmallVec::new())],
            target_registers: Some(target),
            pending_drops: Vec::new(),
        }
    }

    fn length(&self) -> usize {
        self.instructions.len()
    }

    fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }

    fn push(&mut self, instruction: Instruction) {
        self.instructions.push((instruction, SmallVec::new()));
    }

    fn set_target(&mut self, target: Option<RegisterClaims>) {
        self.target_registers = target;
    }

    fn push_jump_anchor(&mut self, anchor: JumpAnchor) {
        if let Some((_, anchors)) = self.instructions.last_mut() {
            anchors.push(anchor);
        }
    }

    fn merge(&mut self, other: InstructionsEmission) {
        self.instructions.extend(other.instructions);
        self.pending_drops.extend(other.pending_drops);
        self.target_registers = other.target_registers;
    }
}

#[derive(Clone, Debug)]
pub enum Place {
    Constant {
        index: u16,
        operand_type: OperandType,
    },
    Register(RegisterClaims),
}

impl Place {
    fn expect_allocation(self, node: &SyntaxReader) -> Result<RegisterClaims, CompileError> {
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
pub struct RegisterClaims {
    claims: RegisterClaim::SmallVec,
    kind: RegisterKind,
}

impl RegisterClaims {
    fn expect_base_index(&self) -> Result<u16, CompileError> {
        self.claims
            .first()
            .map(|claim| claim.index)
            .ok_or(CompileError::ExpectedAllocation)
    }

    fn expect_single(&self) -> Result<RegisterClaim, CompileError> {
        if self.claims.len() == 1 {
            Ok(self.claims[0])
        } else {
            Err(CompileError::ExpectedAllocation)
        }
    }

    fn len(&self) -> usize {
        self.claims.len()
    }

    fn operand_types(&self) -> OperandType::SmallVec {
        self.claims
            .iter()
            .map(|register| register.operand_type)
            .collect()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RegisterClaim {
    index: u16,
    operand_type: OperandType,
}

impl RegisterClaim {
    type SmallVec = SmallVec<[Self; optimal_small_vec_inline_capacity::<Self>()]>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RegisterKind {
    Local,
    Temporary,
    Reserved,
}

enum ExpressionTarget {
    Claimed(RegisterClaims),
    Unclaimed(RegisterKind),
    None,
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

impl JumpAnchor {
    type SmallVec = SmallVec<[JumpAnchor; optimal_small_vec_inline_capacity::<JumpAnchor>()]>;
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

#[derive(Clone, Copy, Debug)]
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

    fn deallocate(&mut self, allocation: &RegisterClaims) {
        match allocation.kind {
            RegisterKind::Local => {
                self.next_local = self.next_local.saturating_sub(allocation.claims[0].index);
            }
            RegisterKind::Temporary => {
                self.next_temporary = self
                    .next_temporary
                    .saturating_sub(allocation.claims[allocation.claims.len() - 1].index);
            }
            RegisterKind::Reserved => {
                self.next_reserved = 0;
            }
        }

        self.max = self.max.min(self.next_temporary);
    }
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
