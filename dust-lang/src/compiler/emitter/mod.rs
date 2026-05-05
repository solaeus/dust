#[cfg(test)]
mod tests;

use std::collections::HashMap;

use rustc_hash::FxBuildHasher;
use smallvec::{SmallVec, smallvec};
use tracing::trace;

use crate::{
    compiler::{
        error::CompileError,
        resolver::{
            PrototypeId, Resolver,
            declarations::{DeclarationId, Definition},
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

    current_self_place: Option<Place>,
}

impl<'a> Emitter<'a> {
    pub fn new(
        declaration_id: DeclarationId,
        prototype_id: PrototypeId,
        return_type_id: TypeId,
        (source, constants, resolver): (&'a Source, &'a mut ConstantsBuilder, &'a mut Resolver),
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

                count += register_size;
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
            current_self_place: None,
        };

        emitter.locals.insert(
            declaration_id,
            Local::Place(Place::Constant {
                operand_type: OperandType::FUNCTION,
                index: prototype_id.inner(),
            }),
        );

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
                    emitter.claim_registers(concrete_type_id, RegisterKind::Reserved)?;

                emitter
                    .locals
                    .insert(declaration_id, Local::Place(Place::Register(allocation)));
            }
        }

        emitter.register_tracker.free_reserved();

        Ok(emitter)
    }

    pub fn finish(mut self) -> Result<Prototype, CompileError> {
        for (_, jump) in self.jump_placements {
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

                if index == child_count - 1 {
                    self.emit_instruction(Instruction::r#return());
                }

                continue;
            }

            if index == child_count - 1 {
                let return_registers =
                    self.claim_registers(self.return_type_id, RegisterKind::Reserved)?;
                let return_expression_emission = self.emit_expression(
                    child,
                    ExpressionTarget::ClaimedRegister(return_registers.clone()),
                )?;
                let return_instructions = self.create_return_instructions(
                    return_expression_emission,
                    return_registers,
                    &child,
                )?;

                self.handle_function_body_instructions(return_instructions)?;
            } else if let Emission::Instructions(instructions) = self.emit_expression(
                child,
                ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
            )? {
                self.handle_function_body_instructions(instructions)?;
            }
        }

        Ok(())
    }

    fn emit_instruction(&mut self, instruction: Instruction) {
        trace!("Emitting {} instruction", instruction.operation());

        self.instructions.push(instruction);
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

        let mut drops = self.pending_drops.drain(start_index..);

        let Some(first_register) = drops.next() else {
            return;
        };
        let (start_register, end_register) = drops.fold(
            (first_register, first_register + 1),
            |(min, max), register_index| (min.min(register_index), max.max(register_index + 1)),
        );
        let Some((last_instruction, _)) = instructions.instructions.last_mut() else {
            let drop_instruction = Instruction::drop(start_register, end_register);

            instructions.push(drop_instruction);

            return;
        };

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

    fn claim_registers(
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
                Type::Pointer { .. } => OperandType::POINTER,
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
        if let Some(encoded) = value.encoded_u16() {
            return Ok(Address {
                memory: MemoryKind::ENCODED,
                index: encoded,
            });
        }

        match value {
            ConstantValue::I32(integer) => Ok(Address {
                memory: MemoryKind::CONSTANT,
                index: self.constants.add_i32(integer).inner(),
            }),
            ConstantValue::I64(integer) => Ok(Address {
                memory: MemoryKind::CONSTANT,
                index: self.constants.add_i64(integer).inner(),
            }),
            ConstantValue::I128(integer) => Ok(Address {
                memory: MemoryKind::CONSTANT,
                index: self.constants.add_i128(integer).inner(),
            }),
            ConstantValue::U32(integer) => Ok(Address {
                memory: MemoryKind::CONSTANT,
                index: self.constants.add_u32(integer).inner(),
            }),
            ConstantValue::U64(integer) => Ok(Address {
                memory: MemoryKind::CONSTANT,
                index: self.constants.add_u64(integer).inner(),
            }),
            ConstantValue::U128(integer) => Ok(Address {
                memory: MemoryKind::CONSTANT,
                index: self.constants.add_u128(integer).inner(),
            }),
            ConstantValue::F32(float) => Ok(Address {
                memory: MemoryKind::CONSTANT,
                index: self.constants.add_f32(float).inner(),
            }),
            ConstantValue::F64(float) => Ok(Address {
                memory: MemoryKind::CONSTANT,
                index: self.constants.add_f64(float).inner(),
            }),
            ConstantValue::Character(character) => Ok(Address {
                memory: MemoryKind::CONSTANT,
                index: self.constants.add_character(character).inner(),
            }),
            ConstantValue::Function { prototype_id, .. } => Ok(Address {
                memory: MemoryKind::ENCODED,
                index: prototype_id.inner(),
            }),
            _ => Err(CompileError::ExpectedEncodedValue { found: value }),
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

    fn create_emission_from_value(
        &mut self,
        value: ConstantValue,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        match target {
            ExpressionTarget::ClaimedRegister(register_claims) => {
                let register = register_claims.expect_single()?;
                let address = self.materialize_value(value)?;
                let move_instruction = Instruction::r#move(
                    register.index,
                    register.operand_type,
                    address.memory,
                    address.index,
                );

                Ok(Emission::Instructions(
                    InstructionsEmission::with_instruction_and_target(
                        move_instruction,
                        register_claims,
                    ),
                ))
            }
            _ => Ok(Emission::Value(value)),
        }
    }

    fn create_emission_from_registers(
        &mut self,
        source_registers: RegisterClaims,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        match target {
            ExpressionTarget::ClaimedRegister(target_registers) => {
                if source_registers.claims.len() == target_registers.claims.len()
                    && source_registers.claims[0].index == target_registers.claims[0].index
                {
                    return Ok(Emission::Place(Place::Register(source_registers)));
                }

                let mut instructions = InstructionsEmission::new();

                for (destination, operand) in
                    target_registers.claims.iter().zip(&source_registers.claims)
                {
                    let move_instruction = Instruction::r#move(
                        destination.index,
                        operand.operand_type,
                        MemoryKind::REGISTER,
                        operand.index,
                    );

                    instructions.push(move_instruction);
                }

                instructions.target_registers = Some(source_registers);

                Ok(Emission::Instructions(instructions))
            }
            _ => Ok(Emission::Place(Place::Register(source_registers))),
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
    ) -> Result<(MemoryKind, u16), CompileError> {
        match emission {
            Emission::Value(value) => {
                let address = self.materialize_value(value)?;

                Ok((address.memory, address.index))
            }
            Emission::Place(Place::Constant { index, .. }) => Ok((MemoryKind::CONSTANT, index)),
            Emission::Place(Place::Register(allocation)) => {
                Ok((MemoryKind::REGISTER, allocation.expect_base_index()?))
            }
            Emission::Instructions(operand_instructions) => {
                instructions.merge(operand_instructions);

                match &instructions.target_registers {
                    Some(allocation) => Ok((MemoryKind::REGISTER, allocation.expect_base_index()?)),
                    None => Err(CompileError::ExpectedValue {
                        source_id: operand.source_id(),
                        syntax_id: operand.id,
                    }),
                }
            }
            Emission::NativeFunction(_) => Err(CompileError::ExpectedNativeFunctionCall {
                position: operand.position(),
            }),
            Emission::Never | Emission::None => Err(CompileError::ExpectedValue {
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
            Emission::Place(Place::Register(allocation)) if allocation.claims.len() == 1 => {
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
                                && allocation.claims.len() == 1
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
        syntax: &SyntaxReader,
    ) -> Result<(), CompileError> {
        match branch_emission {
            Emission::Instructions(branch_instructions) => {
                instructions.merge(branch_instructions);

                Ok(())
            }
            Emission::NativeFunction(_) => Err(CompileError::ExpectedNativeFunctionCall {
                position: syntax.position(),
            }),
            _ => Ok(()),
        }
    }

    fn create_return_instructions(
        &mut self,
        expression_emission: Emission,
        target_registers: RegisterClaims,
        syntax: &SyntaxReader,
    ) -> Result<InstructionsEmission, CompileError> {
        match expression_emission {
            Emission::Value(value) => {
                let address = self.materialize_value(value)?;
                let move_instruction = Instruction::r#move(
                    target_registers.expect_base_index()?,
                    value.operand_type(),
                    address.memory,
                    address.index,
                );

                let mut instructions = InstructionsEmission::with_instruction_and_target(
                    move_instruction,
                    target_registers,
                );
                instructions.push(Instruction::r#return());

                Ok(instructions)
            }
            Emission::Place(Place::Constant {
                operand_type,
                index,
            }) => {
                let move_instruction = Instruction::r#move(
                    target_registers.expect_base_index()?,
                    operand_type,
                    MemoryKind::CONSTANT,
                    index,
                );

                Ok(InstructionsEmission {
                    instructions: vec![
                        (move_instruction, SmallVec::new()),
                        (Instruction::r#return(), SmallVec::new()),
                    ],
                    target_registers: Some(target_registers),
                    pending_drops: Vec::new(),
                })
            }
            Emission::Place(Place::Register(emission_allocation)) => {
                let mut return_instructions = InstructionsEmission::new();

                if emission_allocation.claims.len() == target_registers.claims.len()
                    && emission_allocation.claims[0].index == target_registers.claims[0].index
                {
                    return Ok(InstructionsEmission::with_instruction(
                        Instruction::r#return(),
                    ));
                }

                for (emission_register, target_register) in emission_allocation
                    .claims
                    .into_iter()
                    .zip(target_registers.claims)
                {
                    let move_instruction = Instruction::r#move(
                        target_register.index,
                        emission_register.operand_type,
                        MemoryKind::REGISTER,
                        emission_register.index,
                    );

                    return_instructions.push(move_instruction);
                }

                return_instructions.push(Instruction::r#return());

                Ok(return_instructions)
            }
            Emission::Instructions(mut instructions) => {
                instructions.push(Instruction::r#return());

                Ok(instructions)
            }
            Emission::Never | Emission::None => {
                let return_instruction = Instruction::r#return();

                Ok(InstructionsEmission::with_instruction(return_instruction))
            }
            Emission::NativeFunction(_) => Err(CompileError::ExpectedNativeFunctionCall {
                position: syntax.position(),
            }),
        }
    }

    fn emit_statement(
        &mut self,
        reader: SyntaxReader,
    ) -> Result<Option<InstructionsEmission>, CompileError> {
        match reader.node.kind {
            SyntaxKind::ConstItem => self.emit_const_item(reader).map(|()| None),
            SyntaxKind::ModItem
            | SyntaxKind::FnItem
            | SyntaxKind::UseItem
            | SyntaxKind::StructItem
            | SyntaxKind::EnumItem
            | SyntaxKind::TypeItem
            | SyntaxKind::ImplItem
            | SyntaxKind::TraitItem => Ok(None),
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

    fn emit_const_item(&mut self, syntax: SyntaxReader) -> Result<(), CompileError> {
        let ConstItem { name, value, .. } = syntax.as_component()?;

        let Some(value) = value else {
            return Ok(());
        };

        let expression_emission = self.emit_expression(
            value,
            ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
        )?;

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

    fn emit_let_statement(
        &mut self,
        syntax: SyntaxReader,
    ) -> Result<Option<InstructionsEmission>, CompileError> {
        let LetStatement {
            mutable,
            name,
            expression,
            ..
        } = syntax.as_component()?;

        let expression_target = if mutable {
            let type_id = *self.resolver.get_type_binding(&expression.id)?;
            let registers = self.claim_registers(type_id, RegisterKind::Local)?;

            ExpressionTarget::ClaimedRegister(registers)
        } else {
            ExpressionTarget::UnclaimedRegister(RegisterKind::Local)
        };
        let emission = self.emit_expression(expression, expression_target)?;

        match emission {
            Emission::Value(value) => {
                let declaration_id = *self.resolver.get_declaration_binding(&name.id)?;

                self.locals.insert(declaration_id, Local::Constant(value));

                Ok(None)
            }
            Emission::Place(place) => {
                let declaration_id = *self.resolver.get_declaration_binding(&name.id)?;

                self.locals.insert(declaration_id, Local::Place(place));

                Ok(None)
            }
            Emission::Instructions(InstructionsEmission {
                instructions,
                target_registers,
                pending_drops,
            }) => {
                let declaration_id = *self.resolver.get_declaration_binding(&name.id)?;
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
            Emission::Never | Emission::None => Ok(None),
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

        let declaration_id = self.resolver.get_declaration_binding(&target.id)?;
        let local =
            self.locals
                .get(declaration_id)
                .ok_or_else(|| CompileError::DeclarationOutOfScope {
                    declaration_id: *declaration_id,
                    usage_position: target.position(),
                })?;

        let Local::Place(Place::Register(target_registers)) = local.clone() else {
            return Err(CompileError::CannotMutate {
                position: target.position(),
            });
        };

        let source_emission = self.emit_expression(
            source,
            ExpressionTarget::ClaimedRegister(target_registers.clone()),
        )?;

        match source_emission {
            Emission::Value(value) => {
                let operand_type = value.operand_type();
                let address = self.materialize_value(value)?;
                let move_instruction = Instruction::r#move(
                    target_registers.expect_base_index()?,
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
                for destination in target_registers.claims {
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
                for (destination, operand) in target_registers
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
            Emission::Never | Emission::None => {
                return Err(CompileError::ExpectedValue {
                    source_id: source.source_id(),
                    syntax_id: source.id,
                });
            }
        }

        Ok(Emission::Instructions(assignment_instructions))
    }

    fn emit_expression_statement(
        &mut self,
        syntax: SyntaxReader<'_>,
    ) -> Result<Option<InstructionsEmission>, CompileError> {
        let ExpressionStatement { expression } = syntax.as_component()?;

        let expression_emission = self.emit_expression(
            expression,
            ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
        )?;

        if let Emission::Instructions(mut instructions_emission) = expression_emission {
            instructions_emission.set_target(None);

            Ok(Some(instructions_emission))
        } else {
            Ok(None)
        }
    }

    fn emit_boolean_expression(
        &mut self,
        reader: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        let boolean = reader.node.flags.get_flag(SyntaxFlags::TRUE);

        self.create_emission_from_value(ConstantValue::Boolean(boolean), target)
    }

    fn emit_hexadecimal_expression(
        &mut self,
        reader: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        let text = &self.source.get_content(reader.position())?[2..];
        let byte = create_u8_from_hexadecimal(text)?;

        self.create_emission_from_value(ConstantValue::U8(byte), target)
    }

    fn emit_character_expression(
        &mut self,
        reader: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        let text = self.source.get_content(reader.position().shrink(1))?;
        let character = create_char(text)?;

        self.create_emission_from_value(ConstantValue::Character(character), target)
    }

    fn emit_float_expression(
        &mut self,
        reader: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        let type_id = *self.resolver.get_type_binding(&reader.id)?;
        let text = self.source.get_content(reader.position())?;
        let value = match type_id {
            TypeId::F_32 => {
                let float = create_f32_from_decimal(text)?;

                ConstantValue::F32(float)
            }
            TypeId::F_64 => {
                let float = create_f64_from_decimal(text)?;

                ConstantValue::F64(float)
            }
            _ => match &target {
                ExpressionTarget::ClaimedRegister(register_claims) => {
                    let register = register_claims.expect_single()?;

                    match register.operand_type {
                        OperandType::F_32 => ConstantValue::F32(create_f32_from_decimal(text)?),
                        OperandType::F_64 => ConstantValue::F64(create_f64_from_decimal(text)?),
                        _ => {
                            return Err(CompileError::InvalidTypeBinding(type_id));
                        }
                    }
                }
                _ => {
                    let float = create_f32_from_decimal(text)?;

                    ConstantValue::F32(float)
                }
            },
        };

        self.create_emission_from_value(value, target)
    }

    fn emit_integer_expression(
        &mut self,
        reader: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        let type_id = *self.resolver.get_type_binding(&reader.id)?;
        let text = self.source.get_content(reader.position())?;
        let value = match type_id {
            TypeId::I_8 => ConstantValue::I8(create_i8_from_decimal(text)?),
            TypeId::I_16 => ConstantValue::I16(create_i16_from_decimal(text)?),
            TypeId::I_32 => ConstantValue::I32(create_i32_from_decimal(text)?),
            TypeId::I_64 => ConstantValue::I64(create_i64_from_decimal(text)?),
            TypeId::I_128 => ConstantValue::I128(create_i128_from_decimal(text)?),
            TypeId::I_SIZE => {
                if cfg!(target_pointer_width = "64") {
                    ConstantValue::I64(create_i64_from_decimal(text)?)
                } else {
                    ConstantValue::I32(create_i32_from_decimal(text)?)
                }
            }
            TypeId::U_8 => ConstantValue::U8(create_u8_from_decimal(text)?),
            TypeId::U_16 => ConstantValue::U16(create_u16_from_decimal(text)?),
            TypeId::U_32 => ConstantValue::U32(create_u32_from_decimal(text)?),
            TypeId::U_64 => ConstantValue::U64(create_u64_from_decimal(text)?),
            TypeId::U_128 => ConstantValue::U128(create_u128_from_decimal(text)?),
            TypeId::U_SIZE => {
                if cfg!(target_pointer_width = "64") {
                    ConstantValue::U64(create_u64_from_decimal(text)?)
                } else {
                    ConstantValue::U32(create_u32_from_decimal(text)?)
                }
            }
            _ => match &target {
                ExpressionTarget::ClaimedRegister(register_claims) => {
                    let register = register_claims.expect_single()?;

                    match register.operand_type {
                        OperandType::I_8 => ConstantValue::I8(create_i8_from_decimal(text)?),
                        OperandType::I_16 => ConstantValue::I16(create_i16_from_decimal(text)?),
                        OperandType::I_32 => ConstantValue::I32(create_i32_from_decimal(text)?),
                        OperandType::I_64 => ConstantValue::I64(create_i64_from_decimal(text)?),
                        OperandType::I_128 => ConstantValue::I128(create_i128_from_decimal(text)?),
                        OperandType::U_8 => ConstantValue::U8(create_u8_from_decimal(text)?),
                        OperandType::U_16 => ConstantValue::U16(create_u16_from_decimal(text)?),
                        OperandType::U_32 => ConstantValue::U32(create_u32_from_decimal(text)?),
                        OperandType::U_64 => ConstantValue::U64(create_u64_from_decimal(text)?),
                        OperandType::U_128 => ConstantValue::U128(create_u128_from_decimal(text)?),
                        _ => {
                            return Err(CompileError::InvalidTypeBinding(type_id));
                        }
                    }
                }
                _ => ConstantValue::I32(create_i32_from_decimal(text)?),
            },
        };

        self.create_emission_from_value(value, target)
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

        let array_registers = match target {
            ExpressionTarget::ClaimedRegister(registers) => registers,
            ExpressionTarget::UnclaimedRegister(register_kind) => {
                let type_id = *self.resolver.get_type_binding(&reader.id)?;

                self.claim_registers(type_id, register_kind)?
            }
            ExpressionTarget::Any => {
                let type_id = *self.resolver.get_type_binding(&reader.id)?;

                self.claim_registers(type_id, RegisterKind::Temporary)?
            }
        };
        let registers_per_element = array_registers.claims.len() / elements.len();

        for (index, element) in elements.into_iter().enumerate() {
            let register_start = index * registers_per_element;
            let register_end = register_start + registers_per_element;

            let element_target = ExpressionTarget::ClaimedRegister(RegisterClaims {
                claims: array_registers.claims[register_start..register_end].into(),
                kind: array_registers.kind,
            });
            let element_emission = self.emit_expression(element, element_target)?;

            match element_emission {
                Emission::Instructions(instructions) => array_instructions.merge(instructions),
                _ => return Err(CompileError::InvalidEmission),
            }
        }

        array_instructions.set_target(Some(array_registers));

        Ok(Emission::Instructions(array_instructions))
    }

    fn emit_array_repeat_expression(
        &mut self,
        reader: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        let ArrayRepeatExpression { element, .. } = reader.as_component()?;

        let mut array_instructions = InstructionsEmission::new();

        let type_id = *self.resolver.get_type_binding(&reader.id)?;
        let array_length =
            if let Type::Array { length, .. } = *self.resolver.types.get_type(type_id)? {
                length
            } else {
                return Err(CompileError::ExpectedArrayType(type_id));
            };
        let array_registers = match target {
            ExpressionTarget::ClaimedRegister(registers) => registers,
            ExpressionTarget::UnclaimedRegister(register_kind) => {
                self.claim_registers(type_id, register_kind)?
            }
            ExpressionTarget::Any => self.claim_registers(type_id, RegisterKind::Temporary)?,
        };
        let registers_per_element = array_registers.claims.len() / array_length;
        let first_element_target = ExpressionTarget::ClaimedRegister(RegisterClaims {
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
            ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
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

            return self.create_emission_from_registers(element_allocation, target);
        }

        let index_emission = self.emit_expression(
            index,
            ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
        )?;

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
                Place::Register(allocation) if allocation.claims.len() == 1 => {
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
            ExpressionTarget::ClaimedRegister(registers) => registers,
            ExpressionTarget::UnclaimedRegister(register_kind) => {
                self.claim_registers(element_type_id, register_kind)?
            }
            ExpressionTarget::Any => {
                self.claim_registers(element_type_id, RegisterKind::Temporary)?
            }
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
            ExpressionTarget::ClaimedRegister(registers) => registers,
            ExpressionTarget::UnclaimedRegister(register_kind) => {
                self.claim_registers(type_id, register_kind)?
            }
            ExpressionTarget::Any => self.claim_registers(type_id, RegisterKind::Temporary)?,
        };

        let mut range_instructions = InstructionsEmission::new();

        for (field_expression, destination) in
            [start, end].into_iter().zip(&target_registers.claims)
        {
            let field_emission = self.emit_expression(
                field_expression,
                ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
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

        if let Some(local) = self.locals.get(&declaration_id).cloned() {
            match local {
                Local::Place(Place::Register(registers)) => {
                    return self.create_emission_from_registers(registers, target);
                }
                Local::Place(Place::Constant {
                    operand_type,
                    index,
                }) => match target {
                    ExpressionTarget::ClaimedRegister(register_claims) => {
                        let mut instructions = InstructionsEmission::new();

                        for register in register_claims.claims {
                            let move_instruction = Instruction::r#move(
                                register.index,
                                register.operand_type,
                                MemoryKind::CONSTANT,
                                index,
                            );

                            instructions.push(move_instruction);
                        }

                        return Ok(Emission::Instructions(instructions));
                    }
                    ExpressionTarget::UnclaimedRegister(register_kind) => {
                        let type_id = *self.resolver.get_type_binding(&reader.id)?;
                        let registers = self.claim_registers(type_id, register_kind)?;

                        let mut instructions = InstructionsEmission::new();

                        for register in &registers.claims {
                            let move_instruction = Instruction::r#move(
                                register.index,
                                register.operand_type,
                                MemoryKind::CONSTANT,
                                index,
                            );

                            instructions.push(move_instruction);
                        }

                        instructions.set_target(Some(registers));

                        return Ok(Emission::Instructions(instructions));
                    }
                    ExpressionTarget::Any => {
                        return Ok(Emission::Place(Place::Constant {
                            operand_type,
                            index,
                        }));
                    }
                },
                Local::Constant(value) => return self.create_emission_from_value(value, target),
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
                let prototype_id = self
                    .resolver
                    .add_monomorphized_function(declaration_id, type_arguments);

                Ok(Emission::Value(ConstantValue::Function {
                    prototype_id,
                    type_id,
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
            Definition::Variant {
                discriminant,
                fields: None,
                ..
            } => {
                let type_id = *self.resolver.get_type_binding(&reader.id)?;
                let register = match target {
                    ExpressionTarget::ClaimedRegister(registers) => registers,
                    ExpressionTarget::UnclaimedRegister(register_kind) => {
                        self.claim_registers(type_id, register_kind)?
                    }
                    ExpressionTarget::Any => {
                        self.claim_registers(type_id, RegisterKind::Temporary)?
                    }
                }
                .expect_single()?;
                let move_instruction = Instruction::r#move(
                    register.index,
                    register.operand_type,
                    MemoryKind::ENCODED,
                    discriminant,
                );

                Ok(Emission::Instructions(
                    InstructionsEmission::with_instruction(move_instruction),
                ))
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
            ExpressionTarget::ClaimedRegister(registers) => registers,
            ExpressionTarget::UnclaimedRegister(register_kind) => {
                self.claim_registers(type_id, register_kind)?
            }
            ExpressionTarget::Any => self.claim_registers(type_id, RegisterKind::Temporary)?,
        };

        let mut struct_instructions = InstructionsEmission::new();

        for ((_, field_expression), destination) in
            name_expression_pairs.zip(&target_registers.claims)
        {
            let field_emission = self.emit_expression(
                field_expression,
                ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
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

        if let Some(expression) = expression {
            self.emit_expression(expression, target)
        } else {
            Ok(Emission::None)
        }
    }

    fn emit_block_expression(
        &mut self,
        reader: SyntaxReader<'_>,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        let BlockExpression { children } = reader.as_component()?;

        let mut block_instructions = InstructionsEmission::new();

        let type_id = *self.resolver.get_type_binding(&reader.id)?;
        let target_registers = match target {
            ExpressionTarget::ClaimedRegister(registers) => registers,
            ExpressionTarget::UnclaimedRegister(register_kind) => {
                self.claim_registers(type_id, register_kind)?
            }
            ExpressionTarget::Any => self.claim_registers(type_id, RegisterKind::Temporary)?,
        };

        let child_count = children.len();
        let saved_register_tracker = self.register_tracker;
        let mut last_emission = None;

        self.enter_drop_context();

        for (index, child) in children.enumerate() {
            let is_last = index == child_count - 1;

            if child.node.kind.is_statement() {
                if let Some(instructions) = self.emit_statement(child)? {
                    block_instructions.merge(instructions);
                }

                continue;
            }

            if !is_last {
                let expression_emission = self.emit_expression(
                    child,
                    ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
                )?;

                if let Emission::Instructions(expression_instructions) = expression_emission {
                    block_instructions.merge(expression_instructions);
                }

                continue;
            }

            last_emission = Some(
                self.emit_expression(child, ExpressionTarget::ClaimedRegister(target_registers))?,
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
            ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
        )?;

        self.handle_condition_emission(&mut if_instructions, condition_emission, &condition, true)?;

        let target_registers = match target {
            ExpressionTarget::ClaimedRegister(registers) => registers,
            ExpressionTarget::UnclaimedRegister(register_kind) => {
                let type_id = *self.resolver.get_type_binding(&reader.id)?;

                self.claim_registers(type_id, register_kind)?
            }
            ExpressionTarget::Any => {
                let type_id = *self.resolver.get_type_binding(&reader.id)?;

                self.claim_registers(type_id, RegisterKind::Temporary)?
            }
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
                ExpressionTarget::ClaimedRegister(target_registers.clone()),
            )?;
            let then_register_tracker = self.register_tracker;

            self.handle_branch_emission(then_emission, &mut if_instructions, &then_branch)?;

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
                    ExpressionTarget::ClaimedRegister(target_registers.clone()),
                )?;
                let else_register_tracker = self.register_tracker;

                self.handle_branch_emission(else_emission, &mut if_instructions, &else_branch)?;

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

        let left_emission = self.emit_expression(
            left,
            ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
        )?;
        let right_emission = self.emit_expression(
            right,
            ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
        )?;

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

            return self.create_emission_from_value(combined, target);
        }

        let mut math_emission = InstructionsEmission::new();

        let (left_memory, left_index) =
            self.handle_operand_emission(&mut math_emission, left_emission, &left)?;
        let (right_memory, right_index) =
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
        let (destination, operand_type, registers) = if is_assignment {
            let operand_type = self
                .resolver
                .get_operand_types(type_id)?
                .first()
                .copied()
                .ok_or_else(|| CompileError::CannotApplyOperator {
                    operator: reader.node.kind,
                    type_id,
                    operand_position: reader.position(),
                })?;

            (left_index, operand_type, None)
        } else {
            let target_registers = match target {
                ExpressionTarget::ClaimedRegister(registers) => registers,
                ExpressionTarget::UnclaimedRegister(register_kind) => {
                    self.claim_registers(type_id, register_kind)?
                }
                ExpressionTarget::Any => self.claim_registers(type_id, RegisterKind::Temporary)?,
            };
            let register = target_registers.expect_single()?;

            (
                register.index,
                register.operand_type,
                Some(target_registers),
            )
        };

        let math_instruction = match reader.node.kind {
            SyntaxKind::AdditionExpression | SyntaxKind::AdditionAssignmentExpression => {
                Instruction::add(
                    destination,
                    operand_type,
                    left_memory,
                    left_index,
                    right_memory,
                    right_index,
                )
            }
            SyntaxKind::SubtractionExpression | SyntaxKind::SubtractionAssignmentExpression => {
                Instruction::subtract(
                    destination,
                    operand_type,
                    left_memory,
                    left_index,
                    right_memory,
                    right_index,
                )
            }
            SyntaxKind::MultiplicationExpression
            | SyntaxKind::MultiplicationAssignmentExpression => Instruction::multiply(
                destination,
                operand_type,
                left_memory,
                left_index,
                right_memory,
                right_index,
            ),
            SyntaxKind::DivisionExpression | SyntaxKind::DivisionAssignmentExpression => {
                Instruction::divide(
                    destination,
                    operand_type,
                    left_memory,
                    left_index,
                    right_memory,
                    right_index,
                )
            }
            SyntaxKind::ModuloExpression | SyntaxKind::ModuloAssignmentExpression => {
                Instruction::modulo(
                    destination,
                    operand_type,
                    left_memory,
                    left_index,
                    right_memory,
                    right_index,
                )
            }
            SyntaxKind::ExponentExpression | SyntaxKind::ExponentAssignmentExpression => {
                Instruction::power(
                    destination,
                    operand_type,
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
        math_emission.set_target(registers);

        Ok(Emission::Instructions(math_emission))
    }

    fn emit_comparison_expression(
        &mut self,
        reader: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        let ComparisonExpression { left, right } = reader.as_component()?;

        let left_emission = self.emit_expression(
            left,
            ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
        )?;
        let right_emission = self.emit_expression(
            right,
            ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
        )?;

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

            return self.create_emission_from_value(combined, target);
        }

        let mut comparison_emission = InstructionsEmission::new();

        let (left_memory, left_index) =
            self.handle_operand_emission(&mut comparison_emission, left_emission, &left)?;
        let left_operand_types = self
            .resolver
            .get_operand_types(*self.resolver.get_type_binding(&left.id)?)?;
        let left_operand_type = if left_operand_types.len() == 1 {
            left_operand_types[0]
        } else {
            return Err(CompileError::CannotApplyOperator {
                operator: reader.node.kind,
                type_id: *self.resolver.get_type_binding(&left.id)?,
                operand_position: left.position(),
            });
        };
        let (right_memory, right_index) =
            self.handle_operand_emission(&mut comparison_emission, right_emission, &right)?;

        let type_id = *self.resolver.get_type_binding(&reader.id)?;
        let target_registers = match target {
            ExpressionTarget::ClaimedRegister(registers) => registers,
            ExpressionTarget::UnclaimedRegister(register_kind) => {
                self.claim_registers(type_id, register_kind)?
            }
            ExpressionTarget::Any => self.claim_registers(type_id, RegisterKind::Temporary)?,
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

        let left_emission = self.emit_expression(
            left,
            ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
        )?;
        let right_emission = self.emit_expression(
            right,
            ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
        )?;

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

            return self.create_emission_from_value(combined, target);
        }

        let mut logic_instructions = InstructionsEmission::new();

        let (left_memory, left_index) =
            self.handle_operand_emission(&mut logic_instructions, left_emission, &left)?;
        let (right_memory, right_index) =
            self.handle_operand_emission(&mut logic_instructions, right_emission, &right)?;

        let target_registers = match target {
            ExpressionTarget::ClaimedRegister(registers) => registers,
            ExpressionTarget::UnclaimedRegister(register_kind) => {
                let type_id = *self.resolver.get_type_binding(&reader.id)?;

                self.claim_registers(type_id, register_kind)?
            }
            ExpressionTarget::Any => {
                let type_id = *self.resolver.get_type_binding(&reader.id)?;

                self.claim_registers(type_id, RegisterKind::Temporary)?
            }
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
            ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
        )?;

        if let Emission::Value(constant) = &expression_emission
            && operand.node.kind != SyntaxKind::PathExpression
        {
            let negated = constant.negate(&operand)?;

            return self.create_emission_from_value(negated, target);
        }

        let mut negation_emission = InstructionsEmission::new();

        let (operand_memory, operand_index) =
            self.handle_operand_emission(&mut negation_emission, expression_emission, &operand)?;
        let target_registers = match target {
            ExpressionTarget::ClaimedRegister(registers) => registers,
            ExpressionTarget::UnclaimedRegister(register_kind) => {
                let type_id = *self.resolver.get_type_binding(&reader.id)?;

                self.claim_registers(type_id, register_kind)?
            }
            ExpressionTarget::Any => {
                let type_id = *self.resolver.get_type_binding(&reader.id)?;

                self.claim_registers(type_id, RegisterKind::Temporary)?
            }
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
            ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
        )?;

        if let Emission::Value(constant) = &expression_emission
            && operand.node.kind != SyntaxKind::PathExpression
        {
            let negated = constant.negate(&operand)?;

            return Ok(Emission::Value(negated));
        }

        let mut negation_emission = InstructionsEmission::new();

        let (operand_memory, operand_index) =
            self.handle_operand_emission(&mut negation_emission, expression_emission, &operand)?;
        let target_registers = match target {
            ExpressionTarget::ClaimedRegister(registers) => registers,
            ExpressionTarget::UnclaimedRegister(register_kind) => {
                let type_id = *self.resolver.get_type_binding(&reader.id)?;

                self.claim_registers(type_id, register_kind)?
            }
            ExpressionTarget::Any => {
                let type_id = *self.resolver.get_type_binding(&reader.id)?;

                self.claim_registers(type_id, RegisterKind::Temporary)?
            }
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
            ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
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

        let mut call_instructions = InstructionsEmission::new();

        let callee_emission = self.emit_expression(
            callee,
            ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
        )?;
        let (callee_memory, callee_index) =
            self.handle_operand_emission(&mut call_instructions, callee_emission, &callee)?;

        let mut arguments_start = u16::MAX;

        for argument in arguments.children() {
            let argument_emission = self.emit_expression(
                argument,
                ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
            )?;
            let (argument_memory, argument_index) =
                self.handle_operand_emission(&mut call_instructions, argument_emission, &argument)?;

            let argument_destination = if argument_memory == MemoryKind::REGISTER {
                argument_index
            } else {
                let argument_type_id = *self.resolver.get_type_binding(&argument.id)?;
                let argument_allocation =
                    self.claim_registers(argument_type_id, RegisterKind::Temporary)?;

                for (offset, register) in argument_allocation.claims.iter().enumerate() {
                    let move_instruction = Instruction::r#move(
                        register.index,
                        register.operand_type,
                        argument_memory,
                        argument_index + offset as u16,
                    );

                    call_instructions.push(move_instruction);
                }

                argument_allocation.expect_base_index()?
            };

            if arguments_start == u16::MAX {
                arguments_start = argument_destination;
            }
        }

        let (destination, registers) = match target {
            ExpressionTarget::ClaimedRegister(registers) => {
                (registers.expect_base_index()?, Some(registers))
            }
            ExpressionTarget::UnclaimedRegister(register_kind) => {
                let return_type_id = *self.resolver.get_type_binding(&reader.id)?;

                if matches!(return_type_id, TypeId::UNIT | TypeId::NEVER) {
                    (u16::MAX, None)
                } else {
                    let registers = self.claim_registers(return_type_id, register_kind)?;

                    (registers.expect_base_index()?, Some(registers))
                }
            }
            ExpressionTarget::Any => {
                let return_type_id = *self.resolver.get_type_binding(&reader.id)?;

                if matches!(return_type_id, TypeId::UNIT | TypeId::NEVER) {
                    (u16::MAX, None)
                } else {
                    let registers =
                        self.claim_registers(return_type_id, RegisterKind::Temporary)?;

                    (registers.expect_base_index()?, Some(registers))
                }
            }
        };
        let call_instruction =
            Instruction::call(destination, callee_memory, callee_index, arguments_start);

        call_instructions.push(call_instruction);
        call_instructions.set_target(registers);

        Ok(Emission::Instructions(call_instructions))
    }

    fn emit_field_access_expression(
        &mut self,
        reader: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        let FieldAccessExpression {
            struct_expression,
            field_name,
        } = reader.as_component()?;

        let operand_emission = self.emit_expression(
            struct_expression,
            ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
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

        for &field_id in field_entries {
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
            Some(register) => self.create_emission_from_registers(
                RegisterClaims {
                    claims: smallvec![*register],
                    kind: struct_registers.kind,
                },
                target,
            ),
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
    None,
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
    // TODO: Remove this or make infallible
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
            Err(CompileError::InvalidRegisterAllocation)
        }
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
    ClaimedRegister(RegisterClaims),
    UnclaimedRegister(RegisterKind),
    Any,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum JumpAnchor {
    ForwardFromHere {
        id: JumpId,
    },
    LoopStartHere {
        forward_id: JumpId,
    },
    LoopEndOnNext {
        forward_id: JumpId,
        backward_id: JumpId,
    },
    ForwardToNext {
        id: JumpId,
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
