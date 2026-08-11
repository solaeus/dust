use std::{collections::HashMap, mem::take, range::Range};

use rustc_hash::FxBuildHasher;
use smallvec::{SmallVec, smallvec};
use tracing::trace;

use crate::{
    compiler::{
        context::{
            Context,
            declarations::{DeclarationId, Definition},
            types::{
                FloatType, InferredTypeConstraint, SignedIntegerType, Type, TypeId,
                UnsignedIntegerType,
            },
        },
        error::CompileError,
        prototypes::{PrototypeId, Prototypes},
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
        Address, Drop, Instruction, MemoryKind, Move, NativeFunction, OperandType, Operation, Test,
    },
    optimize_inline_capacity,
    prototype::Prototype,
    source::{CodeId, Source},
    syntax::{
        Syntax,
        components::{
            ArrayExpression, ArrayRepeatExpression, AssignmentExpression, BlockExpression,
            CallExpression, ComparisonExpression, ConstItem, ExpressionStatement,
            FieldAccessExpression, GroupedExpression, IfExpression, IndexExpression, LetStatement,
            LogicExpression, MathExpression, MethodCallExpression, NegationExpression,
            NotExpression, RangeExpression, ReferenceExpression, StructExpression,
            StructExpressionStructFields, ValueParameters, WhileExpression,
        },
        node::{SyntaxFlags, SyntaxKind},
        reader::SyntaxReader,
    },
};

#[derive(Debug)]
pub struct PrototypeEmitter<'a> {
    source: &'a Source<'a>,

    syntax: &'a Syntax,

    constants: &'a mut ConstantsBuilder,

    context: &'a mut Context,

    prototypes: &'a mut Prototypes,

    code_id: CodeId,

    argument_count: u16,

    self_registers: Option<RegisterClaims>,

    return_type_id: TypeId,

    return_operand_types: OperandType::SmallVec,

    /// Emitted bytecode instructions, filled during compilation.
    instructions: Vec<Instruction>,

    /// Concatenated lists of register indices that need to be dropped when exiting scopes.
    drops: Vec<u16>,

    /// Local variables declared in the function.
    locals: HashMap<DeclarationId, Local, FxBuildHasher>,

    register_tracker: RegisterTracker,

    jump_placements: HashMap<JumpId, JumpPlacement, FxBuildHasher>,

    jump_over_branch_ids: Vec<JumpId>,

    break_ids: Vec<JumpId>,

    next_jump_id: JumpId,
}

impl<'a> PrototypeEmitter<'a> {
    pub fn new(
        declaration_id: DeclarationId,
        prototype_id: PrototypeId,
        return_type_id: TypeId,
        value_parameters: Option<SyntaxReader>,
        code_id: CodeId,
        (source, syntax, constants, context, prototypes): (
            &'a Source,
            &'a Syntax,
            &'a mut ConstantsBuilder,
            &'a mut Context,
            &'a mut Prototypes,
        ),
    ) -> Result<Self, CompileError> {
        let return_type_id = context.get_inferred_type_id(return_type_id)?;
        let return_operand_types = context.get_operand_types(return_type_id)?;
        let return_register_count = return_operand_types
            .iter()
            .map(|operand_type| operand_type.register_width().as_u16())
            .sum();

        let mut prototype_emitter = Self {
            source,
            syntax,
            constants,
            context,
            prototypes,
            code_id,
            instructions: Vec::new(),
            drops: Vec::new(),
            locals: HashMap::default(),
            argument_count: 0,
            register_tracker: RegisterTracker::new(0, return_register_count),
            return_type_id,
            self_registers: None,
            return_operand_types,
            jump_placements: HashMap::default(),
            jump_over_branch_ids: Vec::new(),
            break_ids: Vec::new(),
            next_jump_id: JumpId(0),
        };

        prototype_emitter.register_tracker.reserved = u16::MAX;

        prototype_emitter.locals.insert(
            declaration_id,
            Local::Place(Place::Encoded {
                operand_type: OperandType::FUNCTION,
                index: prototype_id.index(),
            }),
        );

        if let Some(value_parameters) = value_parameters {
            let ValueParameters { name_type_pairs } = value_parameters.as_component()?;

            if value_parameters
                .node
                .flags
                .get_flag(SyntaxFlags::SELF_VALUE)
            {
                let self_type_id = *prototype_emitter
                    .context
                    .get_type_binding(&(value_parameters.code_id(), value_parameters.id))?;
                let concrete_self_type_id = prototype_emitter
                    .context
                    .get_inferred_type_id(self_type_id)?;
                let self_registers = prototype_emitter
                    .claim_registers(concrete_self_type_id, RegisterKind::Reserved)?;

                prototype_emitter.self_registers = Some(self_registers);
            }

            for (parameter_name, _) in name_type_pairs {
                let declaration_id = *prototype_emitter
                    .context
                    .get_declaration_binding(&(parameter_name.code_id(), parameter_name.id))?;
                let declaration = prototype_emitter
                    .context
                    .declarations
                    .get_declaration(declaration_id);
                let Definition::Local { type_id, .. } = declaration.definition else {
                    return Err(CompileError::ExpectedLocalDefinition(declaration_id));
                };
                let concrete_type_id = prototype_emitter.context.get_inferred_type_id(type_id)?;
                let allocation =
                    prototype_emitter.claim_registers(concrete_type_id, RegisterKind::Reserved)?;

                prototype_emitter
                    .locals
                    .insert(declaration_id, Local::Place(Place::Registers(allocation)));
            }
        }

        let argument_register_count = prototype_emitter.register_tracker.next_reserved;

        prototype_emitter.argument_count = argument_register_count;
        prototype_emitter.register_tracker =
            RegisterTracker::new(argument_register_count, return_register_count);

        Ok(prototype_emitter)
    }

    pub fn finish(mut self) -> Result<Prototype, CompileError> {
        for (_, jump) in self.jump_placements {
            let JumpPlacement {
                index,
                distance: base_distance,
                forward,
                coalesce,
            } = jump;

            if !coalesce {
                let jump_instruction = Instruction::jump(base_distance, forward);

                self.instructions[index] = jump_instruction;

                continue;
            }

            let instruction = &mut self.instructions[index];

            match instruction.operation() {
                Operation::DROP => {
                    let Drop {
                        start_register,
                        end_register,
                    } = Drop::from(*instruction);

                    *instruction = Instruction::jump_with_drops(
                        base_distance,
                        forward,
                        start_register,
                        end_register,
                    )
                }
                Operation::NO_OP => {
                    *instruction = Instruction::jump(base_distance, forward);
                }
                Operation::TEST => {
                    let Test {
                        comparator,
                        operand,
                        jump_distance,
                    } = Test::from(*instruction);
                    let total_distance = base_distance + jump_distance;

                    *instruction = Instruction::test(comparator, operand, total_distance);
                }
                Operation::MOVE => {
                    let Move {
                        destination,
                        operand_type,
                        operand,
                        jump_distance: move_jump_distance,
                        jump_forward,
                    } = Move::from(*instruction);

                    if move_jump_distance == 0 {
                        *instruction = Instruction::move_with_jump(
                            destination,
                            operand_type,
                            operand,
                            base_distance,
                            true,
                        );
                    } else if jump_forward == forward {
                        let total_distance = base_distance + move_jump_distance;

                        *instruction = Instruction::move_with_jump(
                            destination,
                            operand_type,
                            operand,
                            total_distance,
                            forward,
                        );
                    }
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

    pub fn visit_function_body(&mut self, body: SyntaxReader) -> Result<(), CompileError> {
        trace!("Visiting function body");

        let mut body_instructions = self.create_instruction_range();

        let children = body.children();
        let child_count = children.len();

        for (index, child) in children.enumerate() {
            if child.node.kind.is_statement() {
                if let Some(instructions) = self.visit_statement(child)? {
                    body_instructions.merge(instructions);
                }

                continue;
            }

            if index == child_count - 1 {
                let return_registers =
                    self.claim_registers(self.return_type_id, RegisterKind::Reserved)?;
                let return_expression_emission = self.visit_expression(
                    child,
                    ExpressionTarget::ClaimedRegister(return_registers.clone()),
                )?;

                self.create_return_instructions(
                    &mut body_instructions,
                    return_expression_emission,
                    return_registers,
                    &child,
                )?;

                break;
            }

            if let Emission::Instructions(instructions) =
                self.visit_expression(child, ExpressionTarget::Any)?
            {
                body_instructions.merge(instructions);
            }
        }

        if self.instructions.is_empty() {
            self.emit_instruction(Instruction::r#return(), &mut body_instructions);
        }

        Ok(())
    }

    fn create_instruction_range(&self) -> InstructionRange {
        let instruction_count = self.instructions.len() as u32;
        let drop_count = self.drops.len() as u32;

        InstructionRange {
            instructions: Range {
                start: instruction_count,
                end: instruction_count,
            },
            drops: Range {
                start: drop_count,
                end: drop_count,
            },
            target_registers: None,
        }
    }

    fn emit_instruction(&mut self, instruction: Instruction, range: &mut InstructionRange) {
        trace!("Emitting {} instruction", instruction.operation());

        self.instructions.push(instruction);

        range.instructions.end += 1;
    }

    fn discard_instructions(&mut self, count: usize, range: &mut InstructionRange) {
        debug_assert_eq!(range.instructions.end as usize, self.instructions.len());

        self.instructions.truncate(self.instructions.len() - count);

        range.instructions.end -= count as u32;
    }

    fn jump_forward_from_current_instruction(
        &mut self,
        id: JumpId,
        instructions: &mut InstructionRange,
    ) -> Result<(), CompileError> {
        let Some(last) = self.instructions.last().copied() else {
            return Err(CompileError::ExpectedInstruction {});
        };
        let coalesce = last.is_coallescible_with_jump(true);

        if !coalesce {
            self.emit_instruction(Instruction::no_op(), instructions);
        }

        self.jump_placements.insert(
            id,
            JumpPlacement {
                index: self.instructions.len() - 1,
                distance: 0,
                forward: true,
                coalesce,
            },
        );

        Ok(())
    }

    fn jump_forward_to_next_instruction(&mut self, id: JumpId) -> Result<(), CompileError> {
        let Some(jump) = self.jump_placements.get_mut(&id) else {
            return Err(CompileError::ExpectedJumpPlacement(id));
        };

        jump.distance = (self.instructions.len() - jump.index - 1) as u16;

        Ok(())
    }

    fn end_loop_on_next_instruction(
        &mut self,
        loop_start_index: usize,
        backward_id: JumpId,
        instructions: &mut InstructionRange,
    ) -> Result<(), CompileError> {
        let last = self
            .instructions
            .last()
            .ok_or(CompileError::ExpectedInstruction)?;
        let coalesce = last.is_coallescible_with_jump(true);

        if !coalesce {
            self.emit_instruction(Instruction::no_op(), instructions);
        }

        let index = self.instructions.len() - 1;
        let distance = (index - loop_start_index - 1) as u16;

        self.jump_placements.insert(
            backward_id,
            JumpPlacement {
                index,
                distance,
                forward: false,
                coalesce,
            },
        );

        Ok(())
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
            registers: &mut RegisterClaim::SmallVec,
            prototype_emitter: &mut PrototypeEmitter,
        ) -> Result<(), CompileError> {
            let type_node = prototype_emitter.context.types.get_type(type_id);

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
                    for index in element_types.as_usize_range() {
                        let element_type =
                            *prototype_emitter.context.types.get_type_member(index)?;

                        collect_registers(element_type, kind, registers, prototype_emitter)?;
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
                        collect_registers(element_type_id, kind, registers, prototype_emitter)?;
                    }

                    return Ok(());
                }
                Type::FunctionDefinition { .. } | Type::Closure { .. } | Type::Function { .. } => {
                    OperandType::FUNCTION
                }
                Type::Pointer { .. } => OperandType::HEAP_POINTER,
                Type::Inferred {
                    resolved_id: Some(resolved_id),
                    ..
                } => {
                    collect_registers(*resolved_id, kind, registers, prototype_emitter)?;

                    return Ok(());
                }
                Type::Algebraic { .. } => {
                    let operand_types = prototype_emitter.context.get_operand_types(type_id)?;

                    for operand_type in operand_types {
                        let next_register_index = match kind {
                            RegisterKind::Scoped => prototype_emitter
                                .register_tracker
                                .allocate_next_local(operand_type),
                            RegisterKind::Temporary => prototype_emitter
                                .register_tracker
                                .allocate_next_temporary(operand_type),
                            RegisterKind::Reserved => prototype_emitter
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
                    resolved_id: None,
                    ..
                } => OperandType::I_32,
                Type::Inferred {
                    constraint: Some(InferredTypeConstraint::Float),
                    resolved_id: None,
                    ..
                } => OperandType::F_64,
                Type::Reference { .. } => {
                    let register_index = match kind {
                        RegisterKind::Scoped => prototype_emitter
                            .register_tracker
                            .allocate_next_local(OperandType::HEAP_POINTER),
                        RegisterKind::Temporary => prototype_emitter
                            .register_tracker
                            .allocate_next_temporary(OperandType::HEAP_POINTER),
                        RegisterKind::Reserved => prototype_emitter
                            .register_tracker
                            .allocate_next_reserved(OperandType::HEAP_POINTER),
                    };
                    registers.push(RegisterClaim {
                        operand_type: OperandType::HEAP_POINTER,
                        index: register_index,
                    });

                    return Ok(());
                }
                Type::Inferred { .. } | Type::Generic { .. } | Type::Projection { .. } => {
                    return Err(CompileError::CannotInferType { type_id });
                }
            };

            let next_register_index = match kind {
                RegisterKind::Scoped => prototype_emitter
                    .register_tracker
                    .allocate_next_local(operand_type),
                RegisterKind::Temporary => prototype_emitter
                    .register_tracker
                    .allocate_next_temporary(operand_type),
                RegisterKind::Reserved => prototype_emitter
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

    fn claim_register_for_target(
        &mut self,
        target: ExpressionTarget,
        syntax: SyntaxReader,
    ) -> Result<(u16, Option<RegisterClaims>), CompileError> {
        match target {
            ExpressionTarget::ClaimedRegister(register_claims) => Ok((
                register_claims.expect_single()?.index,
                Some(register_claims),
            )),
            ExpressionTarget::UnclaimedRegister(register_kind) => {
                let type_id = *self.context.get_type_binding(&(self.code_id, syntax.id))?;

                if matches!(type_id, TypeId::UNIT | TypeId::NEVER) {
                    Ok((u16::MAX, None))
                } else {
                    let register_claims = self.claim_registers(type_id, register_kind)?;

                    Ok((register_claims.base_index()?, Some(register_claims)))
                }
            }
            ExpressionTarget::Any => {
                let type_id = *self.context.get_type_binding(&(self.code_id, syntax.id))?;

                if matches!(type_id, TypeId::UNIT | TypeId::NEVER) {
                    Ok((u16::MAX, None))
                } else {
                    let register_claims = self.claim_registers(type_id, RegisterKind::Temporary)?;

                    Ok((register_claims.base_index()?, Some(register_claims)))
                }
            }
        }
    }

    fn materialize_value(&mut self, value: ConstantValue) -> Result<Address, CompileError> {
        if let Some(encoded) = value.to_encoded_u16() {
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
                index: prototype_id.index(),
            }),
            _ => Err(CompileError::ExpectedEncodedValue { found: value }),
        }
    }

    fn place_emission(
        &mut self,
        source_emission: Emission,
        target_instructions: &mut InstructionRange,
        syntax: &SyntaxReader,
    ) -> Result<Place, CompileError> {
        match source_emission {
            Emission::Value(constant) => {
                let operand = self.materialize_value(constant)?;

                match operand.memory {
                    MemoryKind::ENCODED => Ok(Place::Encoded {
                        index: operand.index,
                        operand_type: constant.operand_type(),
                    }),
                    MemoryKind::CONSTANT => Ok(Place::Constant {
                        index: operand.index,
                        operand_type: constant.operand_type(),
                    }),
                    _ => Err(CompileError::ExpectedValue {
                        code_id: syntax.code_id(),
                        syntax_id: syntax.id,
                    }),
                }
            }
            Emission::Place(place) => Ok(place),
            Emission::Instructions(instructions) => {
                let target_registers = target_instructions.extend(instructions);

                if let Some(registers) = target_registers {
                    Ok(Place::Registers(registers))
                } else {
                    Err(CompileError::ExpectedValue {
                        code_id: syntax.code_id(),
                        syntax_id: syntax.id,
                    })
                }
            }
            Emission::NativeFunction(_) => Err(CompileError::ExpectedNativeFunctionCall {
                position: syntax.position(),
            }),
            _ => Err(CompileError::ExpectedValue {
                code_id: syntax.code_id(),
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
            ExpressionTarget::ClaimedRegister(registers) => {
                let mut instructions = self.create_instruction_range();
                let register = registers.expect_single()?;
                let operand = self.materialize_value(value)?;
                let move_instruction =
                    Instruction::r#move(register.index, register.operand_type, operand);

                self.emit_instruction(move_instruction, &mut instructions);

                instructions.target_registers = Some(registers);

                Ok(Emission::Instructions(instructions))
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
                    return Ok(Emission::Place(Place::Registers(source_registers)));
                }

                let mut instructions = self.create_instruction_range();

                for (destination, operand) in
                    target_registers.claims.iter().zip(&source_registers.claims)
                {
                    let move_instruction = Instruction::r#move(
                        destination.index,
                        operand.operand_type,
                        Address::new(MemoryKind::REGISTER, operand.index),
                    );

                    self.emit_instruction(move_instruction, &mut instructions);
                }

                instructions.target_registers = Some(source_registers);

                Ok(Emission::Instructions(instructions))
            }
            _ => Ok(Emission::Place(Place::Registers(source_registers))),
        }
    }

    fn handle_operand_emission(
        &mut self,
        emission: Emission,
        instructions: &mut InstructionRange,
        operand: &SyntaxReader,
    ) -> Result<Address, CompileError> {
        match emission {
            Emission::Value(value) => self.materialize_value(value),
            Emission::Place(place) => place.address(),
            Emission::Instructions(operand_instructions) => {
                instructions.merge(operand_instructions);

                match &instructions.target_registers {
                    Some(allocation) => Ok(Address {
                        memory: MemoryKind::REGISTER,
                        index: allocation.base_index()?,
                    }),
                    None => Err(CompileError::ExpectedValue {
                        code_id: operand.code_id(),
                        syntax_id: operand.id,
                    }),
                }
            }
            Emission::NativeFunction(_) => Err(CompileError::ExpectedNativeFunctionCall {
                position: operand.position(),
            }),
            Emission::Never | Emission::None => Err(CompileError::ExpectedValue {
                code_id: operand.code_id(),
                syntax_id: operand.id,
            }),
        }
    }

    fn handle_condition_emission(
        &mut self,
        target_instructions: &mut InstructionRange,
        emission: Emission,
        condition: &SyntaxReader,
        comparator: bool,
    ) -> Result<(), CompileError> {
        match emission {
            Emission::Value(ConstantValue::Boolean(boolean)) => {
                let operand = Address::new(MemoryKind::ENCODED, boolean as u16);
                let test_instruction = Instruction::test(comparator, operand, 0);

                self.emit_instruction(test_instruction, target_instructions);

                Ok(())
            }
            Emission::Place(Place::Constant { index, .. }) => {
                let test_instruction =
                    Instruction::test(comparator, Address::new(MemoryKind::CONSTANT, index), 0);

                self.emit_instruction(test_instruction, target_instructions);

                Ok(())
            }
            Emission::Place(Place::Registers(allocation)) if allocation.claims.len() == 1 => {
                let operand = Address::new(MemoryKind::REGISTER, allocation.base_index()?);
                let test_instruction = Instruction::test(comparator, operand, 0);

                self.emit_instruction(test_instruction, target_instructions);

                Ok(())
            }
            Emission::Instructions(instruction_range) => {
                let length = self.instructions.len();

                let target_registers = target_instructions.extend(instruction_range);

                if length >= 3 {
                    let condition_instruction = &mut self.instructions[length - 3];

                    match condition_instruction.operation() {
                        Operation::LESS | Operation::LESS_EQUAL | Operation::EQUAL => {
                            self.discard_instructions(2, target_instructions);

                            if let Some(registers) = &target_registers {
                                self.register_tracker.deallocate(registers);
                            }
                        }
                        Operation::TEST
                            if self.instructions[length - 2].operation() == Operation::MOVE
                                && self.instructions[length - 1].operation() == Operation::MOVE =>
                        {
                            let first_move_instruction = self.instructions[length - 2];
                            let operand = Move::from(first_move_instruction).operand;
                            let new_test_instruction = Instruction::test(comparator, operand, 0);

                            self.discard_instructions(3, target_instructions);
                            self.emit_instruction(new_test_instruction, target_instructions);

                            if let Some(registers) = &target_registers {
                                self.register_tracker.deallocate(registers);
                            }
                        }
                        _ => {
                            let operand = if let Some(allocation) = &target_registers
                                && allocation.claims.len() == 1
                            {
                                Address::new(MemoryKind::REGISTER, allocation.claims[0].index)
                            } else {
                                return Err(CompileError::ExpectedBooleanExpression {
                                    found: *self
                                        .context
                                        .get_type_binding(&(self.code_id, condition.id))?,
                                    node_kind: condition.node.kind,
                                    position: condition.position(),
                                });
                            };
                            let test_instruction = Instruction::test(comparator, operand, 0);

                            self.emit_instruction(test_instruction, target_instructions);
                        }
                    }
                } else {
                    let operand = if let Some(allocation) = &target_registers
                        && allocation.claims.len() == 1
                    {
                        Address::new(MemoryKind::REGISTER, allocation.claims[0].index)
                    } else {
                        return Err(CompileError::ExpectedBooleanExpression {
                            found: *self
                                .context
                                .get_type_binding(&(self.code_id, condition.id))?,
                            node_kind: condition.node.kind,
                            position: condition.position(),
                        });
                    };
                    let test_instruction = Instruction::test(comparator, operand, 0);

                    self.emit_instruction(test_instruction, target_instructions);
                }

                Ok(())
            }
            _ => Err(CompileError::ExpectedBooleanExpression {
                found: *self
                    .context
                    .get_type_binding(&(self.code_id, condition.id))?,
                node_kind: condition.node.kind,
                position: condition.position(),
            }),
        }
    }

    fn handle_branch_emission(
        &mut self,
        branch_emission: Emission,
        instructions: &mut InstructionRange,
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
        instructions: &mut InstructionRange,
        expression_emission: Emission,
        registers: RegisterClaims,
        syntax: &SyntaxReader,
    ) -> Result<(), CompileError> {
        match expression_emission {
            Emission::Value(value) => {
                let operand = self.materialize_value(value)?;
                let move_instruction =
                    Instruction::r#move(registers.base_index()?, value.operand_type(), operand);

                self.emit_instruction(move_instruction, instructions);
                self.emit_instruction(Instruction::r#return(), instructions);

                instructions.target_registers = Some(registers);

                Ok(())
            }
            Emission::Place(Place::Registers(emission_registers)) => {
                if emission_registers.kind != RegisterKind::Reserved {
                    self.emit_instruction(Instruction::r#return(), instructions);

                    return Ok(());
                }

                for (emission_register, target_register) in
                    emission_registers.claims.into_iter().zip(registers.claims)
                {
                    if emission_register.index == target_register.index {
                        continue;
                    }

                    let move_instruction = Instruction::r#move(
                        target_register.index,
                        emission_register.operand_type,
                        emission_register.address(),
                    );

                    self.emit_instruction(move_instruction, instructions);
                }

                self.emit_instruction(Instruction::r#return(), instructions);

                Ok(())
            }
            Emission::Place(
                place @ Place::Constant { operand_type, .. }
                | place @ Place::Encoded { operand_type, .. },
            ) => {
                let move_instruction =
                    Instruction::r#move(registers.base_index()?, operand_type, place.address()?);

                self.emit_instruction(move_instruction, instructions);
                self.emit_instruction(Instruction::r#return(), instructions);

                instructions.target_registers = Some(registers);

                Ok(())
            }
            Emission::Instructions(emission_instructions) => {
                instructions.merge(emission_instructions);

                self.emit_instruction(Instruction::r#return(), instructions);

                Ok(())
            }
            Emission::Never | Emission::None => {
                self.emit_instruction(Instruction::r#return(), instructions);

                Ok(())
            }
            Emission::NativeFunction(_) => Err(CompileError::ExpectedNativeFunctionCall {
                position: syntax.position(),
            }),
        }
    }

    fn visit_statement(
        &mut self,
        syntax: SyntaxReader,
    ) -> Result<Option<InstructionRange>, CompileError> {
        match syntax.node.kind {
            SyntaxKind::ConstItem => self.visit_const_item(syntax).map(|()| None),
            SyntaxKind::ModItem
            | SyntaxKind::FunctionItem
            | SyntaxKind::UseItem
            | SyntaxKind::StructItem
            | SyntaxKind::EnumItem
            | SyntaxKind::TypeItem
            | SyntaxKind::ImplItem
            | SyntaxKind::TraitItem => Ok(None),
            SyntaxKind::LetStatement => self.visit_let_statement(syntax),
            SyntaxKind::ExpressionStatement => self.visit_expression_statement(syntax),
            _ => Err(CompileError::UnexpectedSyntax {
                found: syntax.node.kind,
            }),
        }
    }

    fn visit_expression(
        &mut self,
        syntax: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        match syntax.node.kind {
            SyntaxKind::AssignmentExpression => self.visit_assignment_expression(syntax, target),
            SyntaxKind::BooleanExpression => self.visit_boolean_expression(syntax, target),
            SyntaxKind::HexadecimalExpression => self.visit_hexadecimal_expression(syntax, target),
            SyntaxKind::CharacterExpression => self.visit_character_expression(syntax, target),
            SyntaxKind::FloatExpression => self.visit_float_expression(syntax, target),
            SyntaxKind::IntegerExpression => self.visit_integer_expression(syntax, target),
            SyntaxKind::StringExpression => self.visit_string_expression(syntax, target),
            SyntaxKind::ArrayExpression => self.visit_array_expression(syntax, target),
            SyntaxKind::ArrayRepeatExpression => self.visit_array_repeat_expression(syntax, target),
            SyntaxKind::IndexExpression => self.visit_index_expression(syntax, target),
            SyntaxKind::RangeExpression | SyntaxKind::RangeInclusiveExpression => {
                self.visit_range_expression(syntax, target)
            }
            SyntaxKind::PathExpression => self.visit_path_expression(syntax, target),
            SyntaxKind::StructExpression => self.visit_struct_expression(syntax, target),
            SyntaxKind::GroupedExpression => self.visit_grouped_expression(syntax, target),
            SyntaxKind::BlockExpression => self.visit_block_expression(syntax, target),
            SyntaxKind::IfExpression => self.visit_if_expression(syntax, target),
            SyntaxKind::NegationExpression => self.visit_negation_expression(syntax, target),
            SyntaxKind::NotExpression => self.visit_not_expression(syntax, target),
            SyntaxKind::WhileExpression => self.visit_while_expression(syntax, target),
            SyntaxKind::BreakExpression => self.visit_break_expression(syntax, target),
            SyntaxKind::CallExpression => self.visit_call_expression(syntax, target),
            SyntaxKind::MethodCallExpression => self.visit_method_call_expression(syntax, target),
            SyntaxKind::FieldAccessExpression => self.visit_field_access_expression(syntax, target),
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
            | SyntaxKind::ExponentAssignmentExpression => {
                self.visit_math_expression(syntax, target)
            }
            SyntaxKind::GreaterThanExpression
            | SyntaxKind::LessThanExpression
            | SyntaxKind::GreaterThanOrEqualExpression
            | SyntaxKind::LessThanOrEqualExpression
            | SyntaxKind::EqualExpression
            | SyntaxKind::NotEqualExpression => self.visit_comparison_expression(syntax, target),
            SyntaxKind::SelfExpression => self.visit_self_expression(syntax, target),
            SyntaxKind::AndExpression | SyntaxKind::OrExpression => {
                self.visit_logic_expression(syntax, target)
            }
            SyntaxKind::ReferenceExpression => self.visit_reference_expression(syntax, target),
            _ => Err(CompileError::UnexpectedSyntax {
                found: syntax.node.kind,
            }),
        }
    }

    fn visit_const_item(&mut self, syntax: SyntaxReader) -> Result<(), CompileError> {
        trace!("Visiting const item");

        let ConstItem { name, value, .. } = syntax.as_component()?;

        let Some(value) = value else {
            return Ok(());
        };

        let expression_emission = self.visit_expression(
            value,
            ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
        )?;

        let declaration_id = *self
            .context
            .get_declaration_binding(&(self.code_id, name.id))?;
        let constant_value = if let Emission::Value(constant) = expression_emission {
            constant
        } else {
            return Err(CompileError::ExpectedValue {
                code_id: value.code_id(),
                syntax_id: value.id,
            });
        };

        self.context
            .add_constant_item_value(declaration_id, constant_value);

        Ok(())
    }

    fn visit_let_statement(
        &mut self,
        syntax: SyntaxReader,
    ) -> Result<Option<InstructionRange>, CompileError> {
        trace!("Visiting let statement");

        let LetStatement {
            mutable,
            name,
            expression,
            ..
        } = syntax.as_component()?;

        let expression_target = if mutable {
            let type_id = *self
                .context
                .get_type_binding(&(self.code_id, expression.id))?;
            let registers = self.claim_registers(type_id, RegisterKind::Scoped)?;

            ExpressionTarget::ClaimedRegister(registers)
        } else {
            ExpressionTarget::UnclaimedRegister(RegisterKind::Scoped)
        };
        let emission = self.visit_expression(expression, expression_target)?;

        match emission {
            Emission::Value(value) => {
                let declaration_id = *self
                    .context
                    .get_declaration_binding(&(self.code_id, name.id))?;

                self.locals.insert(declaration_id, Local::Constant(value));

                Ok(None)
            }
            Emission::Place(place) => {
                let declaration_id = *self
                    .context
                    .get_declaration_binding(&(self.code_id, name.id))?;

                self.locals.insert(declaration_id, Local::Place(place));

                Ok(None)
            }
            Emission::Instructions(mut instructions) => {
                let declaration_id = *self
                    .context
                    .get_declaration_binding(&(self.code_id, name.id))?;
                let registers =
                    instructions
                        .target_registers
                        .ok_or_else(|| CompileError::ExpectedValue {
                            code_id: expression.code_id(),
                            syntax_id: expression.id,
                        })?;

                self.locals
                    .insert(declaration_id, Local::Place(Place::Registers(registers)));

                instructions.target_registers = None;

                Ok(Some(instructions))
            }
            Emission::Never | Emission::None => Ok(None),
            Emission::NativeFunction(_) => Err(CompileError::ExpectedNativeFunctionCall {
                position: syntax.position(),
            }),
        }
    }

    fn visit_assignment_expression(
        &mut self,
        syntax: SyntaxReader<'_>,
        _: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        trace!("Visiting assignment expression");

        let AssignmentExpression { target, source } = syntax.as_component()?;

        let mut assignment_instructions = self.create_instruction_range();

        let target_registers = match target.node.kind {
            SyntaxKind::PathExpression => {
                let declaration_id = self
                    .context
                    .get_declaration_binding(&(self.code_id, target.id))?;
                let local = self.locals.get(declaration_id).ok_or_else(|| {
                    CompileError::DeclarationOutOfScope {
                        declaration_id: *declaration_id,
                        usage_position: target.position(),
                    }
                })?;

                if let Local::Place(Place::Registers(registers)) = local {
                    registers.clone()
                } else {
                    return Err(CompileError::CannotMutate {
                        position: target.position(),
                    });
                }
            }
            SyntaxKind::IndexExpression => {
                let IndexExpression { collection, index } = target.as_component()?;

                let collection_emission = self.visit_expression(
                    collection,
                    ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
                )?;
                let index_emission = self.visit_expression(
                    index,
                    ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
                )?;
                let source_emission = self.visit_expression(
                    source,
                    ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
                )?;

                let collection_place = self.place_emission(
                    collection_emission,
                    &mut assignment_instructions,
                    &collection,
                )?;
                let index_place =
                    self.place_emission(index_emission, &mut assignment_instructions, &index)?;
                let source_place =
                    self.place_emission(source_emission, &mut assignment_instructions, &source)?;

                let collection_base_register = match collection_place {
                    Place::Registers(registers) => registers.base_index()?,
                    _ => {
                        return Err(CompileError::CannotMutate {
                            position: collection.position(),
                        });
                    }
                };
                let type_id = *self
                    .context
                    .get_type_binding(&(self.code_id, collection.id))?;
                let collection_type = self.context.types.get_type(type_id);
                let operand_type = match collection_type {
                    Type::Array {
                        element_type_id, ..
                    } => {
                        let element_operand_types =
                            self.context.get_operand_types(*element_type_id)?;

                        if element_operand_types.len() == 1 {
                            element_operand_types[0]
                        } else {
                            OperandType::HEAP_POINTER
                        }
                    }
                    _ => {
                        return Err(CompileError::CannotIndex {
                            type_id,
                            position: collection.position(),
                        });
                    }
                };

                let set_instruction = Instruction::set(
                    collection_base_register,
                    operand_type,
                    index_place.address()?,
                    source_place.address()?,
                );

                self.emit_instruction(set_instruction, &mut assignment_instructions);

                return Ok(Emission::Instructions(assignment_instructions));
            }
            _ => {
                let target_emission = self.visit_expression(
                    target,
                    ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
                )?;

                match target_emission {
                    Emission::Place(Place::Registers(registers)) => registers,
                    Emission::Instructions(instructions) => assignment_instructions
                        .extend(instructions)
                        .ok_or_else(|| CompileError::ExpectedValue {
                            code_id: target.code_id(),
                            syntax_id: target.id,
                        })?,
                    _ => {
                        return Err(CompileError::CannotApplyOperator {
                            operator: SyntaxKind::AssignmentExpression,
                            type_id: *self.context.get_type_binding(&(self.code_id, target.id))?,
                            operand_position: target.position(),
                        });
                    }
                }
            }
        };
        let source_emission = self.visit_expression(
            source,
            ExpressionTarget::ClaimedRegister(target_registers.clone()),
        )?;

        match source_emission {
            Emission::Value(value) => {
                let operand_type = value.operand_type();
                let operand = self.materialize_value(value)?;
                let move_instruction =
                    Instruction::r#move(target_registers.base_index()?, operand_type, operand);

                self.emit_instruction(move_instruction, &mut assignment_instructions);
            }
            Emission::Place(
                place @ Place::Constant { operand_type, .. }
                | place @ Place::Encoded { operand_type, .. },
            ) => {
                for destination in target_registers.claims {
                    let move_instruction =
                        Instruction::r#move(destination.index, operand_type, place.address()?);

                    self.emit_instruction(move_instruction, &mut assignment_instructions);
                }
            }
            Emission::Place(Place::Registers(operand_allocation)) => {
                for (destination, operand) in target_registers
                    .claims
                    .into_iter()
                    .zip(operand_allocation.claims)
                {
                    let move_instruction = Instruction::r#move(
                        destination.index,
                        operand.operand_type,
                        Address::new(MemoryKind::REGISTER, operand.index),
                    );

                    self.emit_instruction(move_instruction, &mut assignment_instructions)
                }
            }
            Emission::Instructions(instructions) => {
                assignment_instructions.merge(instructions);
                assignment_instructions.target_registers = None;
            }
            Emission::NativeFunction(_) => {
                return Err(CompileError::ExpectedNativeFunctionCall {
                    position: syntax.position(),
                });
            }
            Emission::Never | Emission::None => {
                return Err(CompileError::ExpectedValue {
                    code_id: source.code_id(),
                    syntax_id: source.id,
                });
            }
        }

        Ok(Emission::Instructions(assignment_instructions))
    }

    fn visit_expression_statement(
        &mut self,
        syntax: SyntaxReader<'_>,
    ) -> Result<Option<InstructionRange>, CompileError> {
        trace!("Visiting expression statement");

        let ExpressionStatement { expression } = syntax.as_component()?;

        let expression_emission = self.visit_expression(
            expression,
            ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
        )?;

        if let Emission::Instructions(mut instructions_emission) = expression_emission {
            instructions_emission.target_registers = None;

            Ok(Some(instructions_emission))
        } else {
            Ok(None)
        }
    }

    fn visit_boolean_expression(
        &mut self,
        syntax: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        trace!("Visiting boolean expression");

        let boolean = syntax.node.flags.get_flag(SyntaxFlags::TRUE);

        self.create_emission_from_value(ConstantValue::Boolean(boolean), target)
    }

    fn visit_hexadecimal_expression(
        &mut self,
        syntax: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        trace!("Visiting hexadecimal expression");

        let bytes = &self
            .source
            .get_code(syntax.code_id())
            .get_bytes(syntax.node.span)?[2..];
        let byte = create_u8_from_hexadecimal(bytes, syntax)?;

        self.create_emission_from_value(ConstantValue::U8(byte), target)
    }

    fn visit_character_expression(
        &mut self,
        syntax: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        trace!("Visiting character expression");

        let text = self.source.get_content(syntax.position().shrink(1))?;
        let character = create_char(text)?;

        self.create_emission_from_value(ConstantValue::Character(character), target)
    }

    fn visit_float_expression(
        &mut self,
        syntax: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        trace!("Visiting float expression");

        let type_id = {
            let raw = *self.context.get_type_binding(&(self.code_id, syntax.id))?;

            self.context.get_inferred_type_id(raw)?
        };
        let bytes = self
            .source
            .get_code(syntax.code_id())
            .get_bytes(syntax.node.span)?;
        let value = match type_id {
            TypeId::F_32 => {
                let float = create_f32_from_decimal(bytes, syntax)?;

                ConstantValue::F32(float)
            }
            TypeId::F_64 => {
                let float = create_f64_from_decimal(bytes, syntax)?;

                ConstantValue::F64(float)
            }
            _ => match &target {
                ExpressionTarget::ClaimedRegister(register_claims) => {
                    let register = register_claims.expect_single()?;

                    match register.operand_type {
                        OperandType::F_32 => {
                            ConstantValue::F32(create_f32_from_decimal(bytes, syntax)?)
                        }
                        OperandType::F_64 => {
                            ConstantValue::F64(create_f64_from_decimal(bytes, syntax)?)
                        }
                        _ => {
                            return Err(CompileError::InvalidTypeBinding(type_id));
                        }
                    }
                }
                _ => {
                    let float = create_f32_from_decimal(bytes, syntax)?;

                    ConstantValue::F32(float)
                }
            },
        };

        self.create_emission_from_value(value, target)
    }

    fn visit_integer_expression(
        &mut self,
        syntax: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        trace!("Visiting integer expression");

        let type_id = {
            let raw = *self.context.get_type_binding(&(self.code_id, syntax.id))?;

            self.context.get_inferred_type_id(raw)?
        };
        let bytes = self
            .source
            .get_code(syntax.code_id())
            .get_bytes(syntax.node.span)?;
        let value = match type_id {
            TypeId::I_8 => ConstantValue::I8(create_i8_from_decimal(bytes, syntax)?),
            TypeId::I_16 => ConstantValue::I16(create_i16_from_decimal(bytes, syntax)?),
            TypeId::I_32 => ConstantValue::I32(create_i32_from_decimal(bytes, syntax)?),
            TypeId::I_64 => ConstantValue::I64(create_i64_from_decimal(bytes, syntax)?),
            TypeId::I_128 => ConstantValue::I128(create_i128_from_decimal(bytes, syntax)?),
            TypeId::I_SIZE => {
                if cfg!(target_pointer_width = "64") {
                    ConstantValue::I64(create_i64_from_decimal(bytes, syntax)?)
                } else {
                    ConstantValue::I32(create_i32_from_decimal(bytes, syntax)?)
                }
            }
            TypeId::U_8 => ConstantValue::U8(create_u8_from_decimal(bytes, syntax)?),
            TypeId::U_16 => ConstantValue::U16(create_u16_from_decimal(bytes, syntax)?),
            TypeId::U_32 => ConstantValue::U32(create_u32_from_decimal(bytes, syntax)?),
            TypeId::U_64 => ConstantValue::U64(create_u64_from_decimal(bytes, syntax)?),
            TypeId::U_128 => ConstantValue::U128(create_u128_from_decimal(bytes, syntax)?),
            TypeId::U_SIZE => {
                if cfg!(target_pointer_width = "64") {
                    ConstantValue::U64(create_u64_from_decimal(bytes, syntax)?)
                } else {
                    ConstantValue::U32(create_u32_from_decimal(bytes, syntax)?)
                }
            }
            _ => match &target {
                ExpressionTarget::ClaimedRegister(register_claims) => {
                    let register = register_claims.expect_single()?;

                    match register.operand_type {
                        OperandType::I_8 => {
                            ConstantValue::I8(create_i8_from_decimal(bytes, syntax)?)
                        }
                        OperandType::I_16 => {
                            ConstantValue::I16(create_i16_from_decimal(bytes, syntax)?)
                        }
                        OperandType::I_32 => {
                            ConstantValue::I32(create_i32_from_decimal(bytes, syntax)?)
                        }
                        OperandType::I_64 => {
                            ConstantValue::I64(create_i64_from_decimal(bytes, syntax)?)
                        }
                        OperandType::I_128 => {
                            ConstantValue::I128(create_i128_from_decimal(bytes, syntax)?)
                        }
                        OperandType::U_8 => {
                            ConstantValue::U8(create_u8_from_decimal(bytes, syntax)?)
                        }
                        OperandType::U_16 => {
                            ConstantValue::U16(create_u16_from_decimal(bytes, syntax)?)
                        }
                        OperandType::U_32 => {
                            ConstantValue::U32(create_u32_from_decimal(bytes, syntax)?)
                        }
                        OperandType::U_64 => {
                            ConstantValue::U64(create_u64_from_decimal(bytes, syntax)?)
                        }
                        OperandType::U_128 => {
                            ConstantValue::U128(create_u128_from_decimal(bytes, syntax)?)
                        }
                        _ => {
                            return Err(CompileError::InvalidTypeBinding(type_id));
                        }
                    }
                }
                _ => ConstantValue::I32(create_i32_from_decimal(bytes, syntax)?),
            },
        };

        self.create_emission_from_value(value, target)
    }

    fn visit_string_expression(
        &mut self,
        _: SyntaxReader,
        _: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        trace!("Visiting string expression");

        todo!()
    }

    fn visit_array_expression(
        &mut self,
        syntax: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        trace!("Visiting array expression");

        let ArrayExpression { elements } = syntax.as_component()?;

        let mut array_instructions = self.create_instruction_range();

        let array_registers = match target {
            ExpressionTarget::ClaimedRegister(registers) => registers,
            ExpressionTarget::UnclaimedRegister(register_kind) => {
                let type_id = *self.context.get_type_binding(&(self.code_id, syntax.id))?;

                self.claim_registers(type_id, register_kind)?
            }
            ExpressionTarget::Any => {
                let type_id = *self.context.get_type_binding(&(self.code_id, syntax.id))?;

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
            let element_emission = self.visit_expression(element, element_target)?;

            match element_emission {
                Emission::Instructions(instructions) => array_instructions.merge(instructions),
                _ => return Err(CompileError::InvalidEmission),
            }
        }

        array_instructions.target_registers = Some(array_registers);

        Ok(Emission::Instructions(array_instructions))
    }

    fn visit_array_repeat_expression(
        &mut self,
        syntax: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        trace!("Visiting array repeat expression");

        let ArrayRepeatExpression { element, .. } = syntax.as_component()?;

        let mut array_instructions = self.create_instruction_range();

        let type_id = *self.context.get_type_binding(&(self.code_id, syntax.id))?;
        let array_length = if let Type::Array { length, .. } = *self.context.types.get_type(type_id)
        {
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

        match self.visit_expression(element, first_element_target)? {
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
                    source_register.address(),
                );

                self.emit_instruction(move_instruction, &mut array_instructions);
            }
        }

        array_instructions.target_registers = Some(array_registers);

        Ok(Emission::Instructions(array_instructions))
    }

    fn visit_index_expression(
        &mut self,
        syntax: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        trace!("Visiting index expression");

        let IndexExpression { collection, index } = syntax.as_component()?;

        let mut index_instructions = self.create_instruction_range();

        let collection_emission = self.visit_expression(
            collection,
            ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
        )?;

        let (list_registers, list_instructions) =
            match collection_emission {
                Emission::Place(Place::Registers(registers)) => (registers, None),
                Emission::Instructions(instructions) => {
                    let registers = instructions.target_registers.clone().ok_or(
                        CompileError::ExpectedValue {
                            code_id: collection.code_id(),
                            syntax_id: collection.id,
                        },
                    )?;

                    (registers, Some(instructions))
                }
                _ => {
                    return Err(CompileError::ExpectedValue {
                        code_id: collection.code_id(),
                        syntax_id: collection.id,
                    });
                }
            };

        let list_type_id = *self
            .context
            .get_type_binding(&(self.code_id, collection.id))?;
        let list_type = *self.context.types.get_type(list_type_id);

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

        let element_operand_types = self.context.get_operand_types(element_type_id)?;
        let element_register_count = element_operand_types.len();

        if index.node.kind == SyntaxKind::IntegerExpression {
            let index_bytes = self
                .source
                .get_code(index.code_id())
                .get_bytes(index.node.span)?;
            let constant_index = create_usize_from_decimal(index_bytes, index)?;

            if constant_index >= array_length {
                return Err(CompileError::IndexOutOfBounds {
                    index: constant_index,
                    length: array_length,
                    position: index.position(),
                });
            }

            let register_offset = constant_index * element_register_count;

            let element_registers = list_registers
                .claims
                .iter()
                .skip(register_offset)
                .take(element_register_count)
                .copied()
                .collect::<RegisterClaim::SmallVec>();

            let element_allocation = RegisterClaims {
                claims: element_registers,
                kind: RegisterKind::Scoped,
            };

            if let Some(mut instructions) = list_instructions {
                instructions.target_registers = Some(element_allocation);

                return Ok(Emission::Instructions(instructions));
            }

            return self.create_emission_from_registers(element_allocation, target);
        }

        let index_emission = self.visit_expression(
            index,
            ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
        )?;

        let index_address = if let Emission::Value(constant) = &index_emission
            && let Some(encoded) = constant.to_encoded_u16()
        {
            Address::new(MemoryKind::ENCODED, encoded)
        } else {
            let index_place =
                self.place_emission(index_emission, &mut index_instructions, &index)?;

            match index_place {
                Place::Constant { index, .. } => Address::new(MemoryKind::CONSTANT, index),
                Place::Registers(allocation) if allocation.claims.len() == 1 => {
                    Address::new(MemoryKind::REGISTER, allocation.claims[0].index)
                }
                _ => {
                    return Err(CompileError::ExpectedIntegerIndex {
                        found: *self.context.get_type_binding(&(self.code_id, index.id))?,
                        position: index.position(),
                    });
                }
            }
        };

        let base_index = list_registers.base_index()?;

        let element_type_id = *self.context.get_type_binding(&(self.code_id, syntax.id))?;
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

        let get_instruction = Instruction::get(
            destination_register.index,
            destination_register.operand_type,
            base_index,
            index_address,
        );

        self.emit_instruction(get_instruction, &mut index_instructions);

        index_instructions.target_registers = Some(destination);

        Ok(Emission::Instructions(index_instructions))
    }

    fn visit_range_expression(
        &mut self,
        syntax: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        trace!("Visiting range expression");

        let RangeExpression { start, end } = syntax.as_component()?;

        let mut range_instructions = self.create_instruction_range();

        let target_registers = match target {
            ExpressionTarget::ClaimedRegister(registers) => registers,
            ExpressionTarget::UnclaimedRegister(register_kind) => {
                let type_id = *self.context.get_type_binding(&(self.code_id, syntax.id))?;

                self.claim_registers(type_id, register_kind)?
            }
            ExpressionTarget::Any => {
                let type_id = *self.context.get_type_binding(&(self.code_id, syntax.id))?;

                self.claim_registers(type_id, RegisterKind::Temporary)?
            }
        };

        for (field_expression, destination) in
            [start, end].into_iter().zip(&target_registers.claims)
        {
            let field_emission = self.visit_expression(
                field_expression,
                ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
            )?;

            if let Emission::Value(constant) = &field_emission
                && let Some(encoded) = constant.to_encoded_u16()
            {
                let move_instruction = Instruction::r#move(
                    destination.index,
                    constant.operand_type(),
                    Address::new(MemoryKind::ENCODED, encoded),
                );

                self.emit_instruction(move_instruction, &mut range_instructions);

                continue;
            }

            let field_place =
                self.place_emission(field_emission, &mut range_instructions, &field_expression)?;

            match field_place {
                place @ Place::Constant { operand_type, .. }
                | place @ Place::Encoded { operand_type, .. } => {
                    let move_instruction =
                        Instruction::r#move(destination.index, operand_type, place.address()?);

                    self.emit_instruction(move_instruction, &mut range_instructions);
                }
                Place::Registers(allocation) => {
                    for register in &allocation.claims {
                        if register.index == destination.index {
                            continue;
                        }

                        let move_instruction = Instruction::r#move(
                            destination.index,
                            register.operand_type,
                            register.address(),
                        );

                        self.emit_instruction(move_instruction, &mut range_instructions);
                    }
                }
            }
        }

        range_instructions.target_registers = Some(target_registers);

        Ok(Emission::Instructions(range_instructions))
    }

    fn visit_path_expression(
        &mut self,
        syntax: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        trace!("Visiting path expression");

        let declaration_id = *self
            .context
            .get_declaration_binding(&(self.code_id, syntax.id))?;

        if let Some(local) = self.locals.get(&declaration_id).cloned() {
            match local {
                Local::Place(Place::Registers(registers)) => {
                    return self.create_emission_from_registers(registers, target);
                }
                Local::Place(place) => match target {
                    ExpressionTarget::ClaimedRegister(register_claims) => {
                        let mut instructions = self.create_instruction_range();

                        for register in register_claims.claims {
                            let move_instruction = Instruction::r#move(
                                register.index,
                                register.operand_type,
                                place.address()?,
                            );

                            self.emit_instruction(move_instruction, &mut instructions);
                        }

                        return Ok(Emission::Instructions(instructions));
                    }
                    ExpressionTarget::UnclaimedRegister(register_kind) => {
                        let type_id = *self.context.get_type_binding(&(self.code_id, syntax.id))?;
                        let registers = self.claim_registers(type_id, register_kind)?;

                        let mut instructions = self.create_instruction_range();

                        for register in &registers.claims {
                            let move_instruction = Instruction::r#move(
                                register.index,
                                register.operand_type,
                                place.address()?,
                            );

                            self.emit_instruction(move_instruction, &mut instructions);
                        }

                        instructions.target_registers = Some(registers);

                        return Ok(Emission::Instructions(instructions));
                    }
                    ExpressionTarget::Any => {
                        return Ok(Emission::Place(place));
                    }
                },
                Local::Constant(value) => return self.create_emission_from_value(value, target),
            }
        }

        let declaration = self.context.declarations.get_declaration(declaration_id);

        match declaration.definition {
            Definition::Function { .. } => {
                let type_id = *self.context.get_type_binding(&(self.code_id, syntax.id))?;
                let callee_type = *self.context.types.get_type(type_id);
                let type_arguments =
                    if let Type::FunctionDefinition { type_arguments, .. } = callee_type {
                        type_arguments
                            .as_usize_range()
                            .map(|index| {
                                let type_id = *self.context.types.get_type_member(index)?;

                                self.context.get_inferred_type_id(type_id)
                            })
                            .try_collect::<TypeId::SmallVec>()?
                    } else {
                        SmallVec::new()
                    };
                let prototype_id = self
                    .prototypes
                    .monomorphize_function_to_prototype(declaration_id, type_arguments);

                Ok(Emission::Value(ConstantValue::Function {
                    prototype_id,
                    type_id,
                }))
            }
            Definition::Constant { type_id, .. } => {
                let value =
                    if let Some(value) = self.context.get_constant_item_value(&declaration_id) {
                        value
                    } else {
                        let (position, syntax_id) =
                            declaration.syntax.ok_or(CompileError::ExpectedValue {
                                code_id: syntax.code_id(),
                                syntax_id: syntax.id,
                            })?;
                        let tree = self.syntax.get_tree(position.code_id)?;
                        let const_syntax = tree.read_node(syntax_id)?;
                        let const_syntax: ConstItem = const_syntax.as_component()?;
                        let value_expression =
                            const_syntax.value.ok_or(CompileError::ExpectedValue {
                                code_id: syntax.code_id(),
                                syntax_id: syntax.id,
                            })?;

                        self.context
                            .add_type_binding(self.code_id, value_expression.id, type_id);

                        let expression_emission = self.visit_expression(
                            value_expression,
                            ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
                        )?;
                        let constant_value = match expression_emission {
                            Emission::Value(constant_value) => constant_value,
                            _ => {
                                return Err(CompileError::ExpectedValue {
                                    code_id: syntax.code_id(),
                                    syntax_id: syntax.id,
                                });
                            }
                        };

                        self.context
                            .add_constant_item_value(declaration_id, constant_value);

                        constant_value
                    };

                Ok(Emission::Value(value))
            }
            Definition::StructType { fields: None, .. } => Ok(Emission::None),
            Definition::Variant {
                discriminant,
                fields: None,
                ..
            } => {
                let mut instructions = self.create_instruction_range();
                let type_id = *self.context.get_type_binding(&(self.code_id, syntax.id))?;
                let registers = match target {
                    ExpressionTarget::ClaimedRegister(registers) => registers,
                    ExpressionTarget::UnclaimedRegister(register_kind) => {
                        self.claim_registers(type_id, register_kind)?
                    }
                    ExpressionTarget::Any => {
                        self.claim_registers(type_id, RegisterKind::Temporary)?
                    }
                };
                let base_register = registers.base()?;
                let move_instruction = Instruction::r#move(
                    base_register.index,
                    base_register.operand_type,
                    Address::new(MemoryKind::ENCODED, discriminant),
                );

                self.emit_instruction(move_instruction, &mut instructions);

                Ok(Emission::Instructions(instructions))
            }
            _ => Err(CompileError::ExpectedValue {
                code_id: syntax.code_id(),
                syntax_id: syntax.id,
            }),
        }
    }

    fn visit_struct_expression(
        &mut self,
        syntax: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        trace!("Visiting struct expression");

        let StructExpression {
            path: _,
            fields: struct_fields,
        } = syntax.as_component()?;
        let StructExpressionStructFields {
            name_expression_pairs,
        } = struct_fields.as_component()?;

        let mut struct_instructions = self.create_instruction_range();

        let type_id = *self.context.get_type_binding(&(self.code_id, syntax.id))?;
        let target_registers = match target {
            ExpressionTarget::ClaimedRegister(registers) => registers,
            ExpressionTarget::UnclaimedRegister(register_kind) => {
                self.claim_registers(type_id, register_kind)?
            }
            ExpressionTarget::Any => self.claim_registers(type_id, RegisterKind::Temporary)?,
        };

        for ((_, field_expression), destination) in
            name_expression_pairs.zip(&target_registers.claims)
        {
            let field_emission = self.visit_expression(
                field_expression,
                ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
            )?;

            if let Emission::Value(constant) = &field_emission
                && let Some(encoded) = constant.to_encoded_u16()
            {
                let move_instruction = Instruction::r#move(
                    destination.index,
                    constant.operand_type(),
                    Address::new(MemoryKind::ENCODED, encoded),
                );

                self.emit_instruction(move_instruction, &mut struct_instructions);

                continue;
            }

            let field_place =
                self.place_emission(field_emission, &mut struct_instructions, &field_expression)?;

            match field_place {
                place @ Place::Constant { operand_type, .. }
                | place @ Place::Encoded { operand_type, .. } => {
                    let move_instruction =
                        Instruction::r#move(destination.index, operand_type, place.address()?);

                    self.emit_instruction(move_instruction, &mut struct_instructions);
                }
                Place::Registers(ref allocation) => {
                    for register in &allocation.claims {
                        let move_instruction = Instruction::r#move(
                            destination.index,
                            register.operand_type,
                            register.address(),
                        );

                        self.emit_instruction(move_instruction, &mut struct_instructions);
                    }
                }
            }
        }

        struct_instructions.target_registers = Some(target_registers);

        Ok(Emission::Instructions(struct_instructions))
    }

    fn visit_grouped_expression(
        &mut self,
        syntax: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        trace!("Visiting grouped expression");

        let GroupedExpression { expression } = syntax.as_component()?;

        if let Some(expression) = expression {
            self.visit_expression(expression, target)
        } else {
            Ok(Emission::None)
        }
    }

    fn visit_block_expression(
        &mut self,
        syntax: SyntaxReader<'_>,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        trace!("Visiting block expression");

        let BlockExpression { children } = syntax.as_component()?;

        let mut block_instructions = self.create_instruction_range();

        let type_id = *self.context.get_type_binding(&(self.code_id, syntax.id))?;
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

        for (index, child) in children.enumerate() {
            if child.node.kind.is_statement() {
                if let Some(instructions) = self.visit_statement(child)? {
                    block_instructions.merge(instructions);
                }

                continue;
            }

            if index == child_count - 1 {
                last_emission = Some(self.visit_expression(
                    child,
                    ExpressionTarget::ClaimedRegister(target_registers),
                )?);

                break;
            }

            let expression_emission = self.visit_expression(
                child,
                ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
            )?;

            if let Emission::Instructions(expression_instructions) = expression_emission {
                block_instructions.merge(expression_instructions);
            }
        }

        let result_emission = if let Some(last_emission) = last_emission {
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

        self.register_tracker.restore(saved_register_tracker);

        let target_allocation = match &result_emission {
            Emission::Instructions(instructions) => instructions.target_registers.as_ref(),
            Emission::Place(Place::Registers(target_allocation)) => Some(target_allocation),
            _ => None,
        };

        if let Some(target_allocation) = target_allocation
            && let Some(last_register) = target_allocation.claims.last()
        {
            let target_end =
                last_register.index + last_register.operand_type.register_width().as_u16();

            match target_allocation.kind {
                RegisterKind::Scoped => {
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

    fn visit_if_expression(
        &mut self,
        syntax: SyntaxReader<'_>,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        trace!("Visiting if expression");

        let IfExpression {
            condition,
            then_branch,
            else_branch,
        } = syntax.as_component()?;

        let mut if_instructions = self.create_instruction_range();

        let condition_emission = self.visit_expression(
            condition,
            ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
        )?;

        self.handle_condition_emission(&mut if_instructions, condition_emission, &condition, true)?;

        let target_registers = match target {
            ExpressionTarget::ClaimedRegister(registers) => registers,
            ExpressionTarget::UnclaimedRegister(register_kind) => {
                let type_id = *self.context.get_type_binding(&(self.code_id, syntax.id))?;

                self.claim_registers(type_id, register_kind)?
            }
            ExpressionTarget::Any => {
                let type_id = *self.context.get_type_binding(&(self.code_id, syntax.id))?;

                self.claim_registers(type_id, RegisterKind::Temporary)?
            }
        };
        let jump_over_then_id = self.create_jump_id();
        let start_else_anchor_count = self.jump_over_branch_ids.len();

        self.jump_forward_from_current_instruction(jump_over_then_id, &mut if_instructions)?;

        {
            let saved_register_tracker = self.register_tracker;
            let then_emission = self.visit_block_expression(
                then_branch,
                ExpressionTarget::ClaimedRegister(target_registers.clone()),
            )?;
            let then_register_tracker = self.register_tracker;

            self.handle_branch_emission(then_emission, &mut if_instructions, &then_branch)?;

            if let Some(else_branch) = else_branch {
                let jump_over_else_id = self.create_jump_id();
                self.jump_over_branch_ids.push(jump_over_else_id);

                self.jump_forward_from_current_instruction(
                    jump_over_else_id,
                    &mut if_instructions,
                )?;
                self.jump_forward_to_next_instruction(jump_over_then_id)?;
                self.register_tracker.restore(saved_register_tracker);

                let else_emission = match else_branch.node.kind {
                    SyntaxKind::BlockExpression => self.visit_block_expression(
                        else_branch,
                        ExpressionTarget::ClaimedRegister(target_registers.clone()),
                    )?,
                    SyntaxKind::IfExpression => self.visit_if_expression(
                        else_branch,
                        ExpressionTarget::ClaimedRegister(target_registers.clone()),
                    )?,
                    _ => {
                        return Err(CompileError::ExpectedSyntax {
                            expected: &[SyntaxKind::BlockExpression, SyntaxKind::IfExpression],
                        });
                    }
                };
                let else_register_tracker = self.register_tracker;

                self.handle_branch_emission(else_emission, &mut if_instructions, &else_branch)?;

                self.register_tracker.restore(else_register_tracker);
                self.register_tracker.max =
                    self.register_tracker.max.max(then_register_tracker.max);

                if_instructions.target_registers = Some(target_registers);
            } else {
                self.register_tracker.restore(then_register_tracker);
                self.jump_forward_to_next_instruction(jump_over_then_id)?;

                if_instructions.target_registers = Some(target_registers);
            }
        }

        let end_else_anchor_count = self.jump_over_branch_ids.len();

        for index in start_else_anchor_count..end_else_anchor_count {
            let jump_id = self.jump_over_branch_ids[index];

            self.jump_forward_to_next_instruction(jump_id)?;
        }

        Ok(Emission::Instructions(if_instructions))
    }

    fn visit_math_expression(
        &mut self,
        syntax: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        trace!("Visiting math expression");

        let MathExpression { left, right } = syntax.as_component()?;

        let mut math_instructions = self.create_instruction_range();

        let left_emission = self.visit_expression(
            left,
            ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
        )?;
        let right_emission = self.visit_expression(
            right,
            ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
        )?;

        if let (Emission::Value(left_value), Emission::Value(right_value)) =
            (&left_emission, &right_emission)
        {
            let combined = match syntax.node.kind {
                SyntaxKind::AdditionExpression | SyntaxKind::AdditionAssignmentExpression => {
                    left_value.add(*right_value, &syntax)?
                }
                SyntaxKind::SubtractionExpression | SyntaxKind::SubtractionAssignmentExpression => {
                    left_value.subtract(*right_value, &syntax)?
                }
                SyntaxKind::MultiplicationExpression
                | SyntaxKind::MultiplicationAssignmentExpression => {
                    left_value.multiply(*right_value, &syntax)?
                }
                SyntaxKind::DivisionExpression | SyntaxKind::DivisionAssignmentExpression => {
                    left_value.divide(*right_value, &syntax)?
                }
                SyntaxKind::ModuloExpression | SyntaxKind::ModuloAssignmentExpression => {
                    left_value.modulo(*right_value, &syntax)?
                }
                SyntaxKind::ExponentExpression | SyntaxKind::ExponentAssignmentExpression => {
                    left_value.exponentiate(*right_value, &syntax)?
                }
                _ => {
                    return Err(CompileError::UnexpectedSyntax {
                        found: syntax.node.kind,
                    });
                }
            };

            return self.create_emission_from_value(combined, target);
        }

        let left_address =
            self.handle_operand_emission(left_emission, &mut math_instructions, &left)?;
        let right_address =
            self.handle_operand_emission(right_emission, &mut math_instructions, &right)?;

        let type_id = *self.context.get_type_binding(&(self.code_id, syntax.id))?;
        let is_assignment = matches!(
            syntax.node.kind,
            SyntaxKind::AdditionAssignmentExpression
                | SyntaxKind::SubtractionAssignmentExpression
                | SyntaxKind::MultiplicationAssignmentExpression
                | SyntaxKind::DivisionAssignmentExpression
                | SyntaxKind::ModuloAssignmentExpression
                | SyntaxKind::ExponentAssignmentExpression
        );
        let (destination, operand_type, registers) = if is_assignment {
            let operand_type = self
                .context
                .get_operand_types(type_id)?
                .first()
                .copied()
                .ok_or_else(|| CompileError::CannotApplyOperator {
                    operator: syntax.node.kind,
                    type_id,
                    operand_position: syntax.position(),
                })?;

            (left_address.index, operand_type, None)
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

        let math_instruction = match syntax.node.kind {
            SyntaxKind::AdditionExpression | SyntaxKind::AdditionAssignmentExpression => {
                Instruction::add(destination, operand_type, left_address, right_address)
            }
            SyntaxKind::SubtractionExpression | SyntaxKind::SubtractionAssignmentExpression => {
                Instruction::subtract(destination, operand_type, left_address, right_address)
            }
            SyntaxKind::MultiplicationExpression
            | SyntaxKind::MultiplicationAssignmentExpression => {
                Instruction::multiply(destination, operand_type, left_address, right_address)
            }
            SyntaxKind::DivisionExpression | SyntaxKind::DivisionAssignmentExpression => {
                Instruction::divide(destination, operand_type, left_address, right_address)
            }
            SyntaxKind::ModuloExpression | SyntaxKind::ModuloAssignmentExpression => {
                Instruction::modulo(destination, operand_type, left_address, right_address)
            }
            SyntaxKind::ExponentExpression | SyntaxKind::ExponentAssignmentExpression => {
                Instruction::power(destination, operand_type, left_address, right_address)
            }
            _ => {
                return Err(CompileError::UnexpectedSyntax {
                    found: syntax.node.kind,
                });
            }
        };

        self.emit_instruction(math_instruction, &mut math_instructions);

        math_instructions.target_registers = registers;

        Ok(Emission::Instructions(math_instructions))
    }

    fn visit_comparison_expression(
        &mut self,
        syntax: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        trace!("Visiting comparison expression");

        let ComparisonExpression { left, right } = syntax.as_component()?;

        let mut comparison_instructions = self.create_instruction_range();

        let left_emission = self.visit_expression(
            left,
            ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
        )?;
        let right_emission = self.visit_expression(
            right,
            ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
        )?;

        if let Emission::Value(left_constant) = left_emission
            && let Emission::Value(right_constant) = right_emission
        {
            let combined = match syntax.node.kind {
                SyntaxKind::EqualExpression => left_constant.equal(right_constant, &syntax)?,
                SyntaxKind::NotEqualExpression => {
                    left_constant.not_equal(right_constant, &syntax)?
                }
                SyntaxKind::LessThanExpression => {
                    left_constant.less_than(right_constant, &syntax)?
                }
                SyntaxKind::GreaterThanExpression => {
                    left_constant.greater_than(right_constant, &syntax)?
                }
                SyntaxKind::LessThanOrEqualExpression => {
                    left_constant.less_than_or_equal(right_constant, &syntax)?
                }
                SyntaxKind::GreaterThanOrEqualExpression => {
                    left_constant.greater_than_or_equal(right_constant, &syntax)?
                }
                _ => {
                    return Err(CompileError::UnexpectedSyntax {
                        found: syntax.node.kind,
                    });
                }
            };

            return self.create_emission_from_value(combined, target);
        }

        let left_address =
            self.handle_operand_emission(left_emission, &mut comparison_instructions, &left)?;
        let left_operand_types = self
            .context
            .get_operand_types(*self.context.get_type_binding(&(self.code_id, left.id))?)?;
        let left_operand_type = if left_operand_types.len() == 1 {
            left_operand_types[0]
        } else {
            return Err(CompileError::CannotApplyOperator {
                operator: syntax.node.kind,
                type_id: *self.context.get_type_binding(&(self.code_id, left.id))?,
                operand_position: left.position(),
            });
        };
        let right_address =
            self.handle_operand_emission(right_emission, &mut comparison_instructions, &right)?;

        let type_id = *self.context.get_type_binding(&(self.code_id, syntax.id))?;
        let target_registers = match target {
            ExpressionTarget::ClaimedRegister(registers) => registers,
            ExpressionTarget::UnclaimedRegister(register_kind) => {
                self.claim_registers(type_id, register_kind)?
            }
            ExpressionTarget::Any => self.claim_registers(type_id, RegisterKind::Temporary)?,
        };
        let register = target_registers.expect_single()?;

        let comparison_instruction = match syntax.node.kind {
            SyntaxKind::EqualExpression => {
                Instruction::equal(true, left_operand_type, left_address, right_address)
            }
            SyntaxKind::NotEqualExpression => {
                Instruction::equal(false, left_operand_type, left_address, right_address)
            }
            SyntaxKind::LessThanExpression => {
                Instruction::less(true, left_operand_type, left_address, right_address)
            }
            SyntaxKind::GreaterThanExpression => {
                Instruction::less_equal(false, left_operand_type, left_address, right_address)
            }
            SyntaxKind::LessThanOrEqualExpression => {
                Instruction::less_equal(true, left_operand_type, left_address, right_address)
            }
            SyntaxKind::GreaterThanOrEqualExpression => {
                Instruction::less(false, left_operand_type, left_address, right_address)
            }
            _ => {
                return Err(CompileError::UnexpectedSyntax {
                    found: syntax.node.kind,
                });
            }
        };
        let load_false_instruction = Instruction::move_with_jump(
            register.index,
            OperandType::BOOLEAN,
            Address::new(MemoryKind::ENCODED, false as u16),
            1,
            true,
        );
        let load_true_instruction = Instruction::r#move(
            register.index,
            OperandType::BOOLEAN,
            Address::new(MemoryKind::ENCODED, true as u16),
        );

        self.emit_instruction(comparison_instruction, &mut comparison_instructions);
        self.emit_instruction(load_false_instruction, &mut comparison_instructions);
        self.emit_instruction(load_true_instruction, &mut comparison_instructions);

        comparison_instructions.target_registers = Some(target_registers);

        Ok(Emission::Instructions(comparison_instructions))
    }

    fn visit_logic_expression(
        &mut self,
        syntax: SyntaxReader<'_>,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        trace!("Visiting logic expression");

        let LogicExpression { left, right } = syntax.as_component()?;

        let mut logic_instructions = self.create_instruction_range();

        let left_emission = self.visit_expression(
            left,
            ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
        )?;
        let right_emission = self.visit_expression(
            right,
            ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
        )?;

        if let Emission::Value(left_constant) = left_emission
            && let Emission::Value(right_constant) = right_emission
        {
            let combined = match syntax.node.kind {
                SyntaxKind::AndExpression => left_constant.and(right_constant, &syntax)?,
                SyntaxKind::OrExpression => left_constant.or(right_constant, &syntax)?,
                _ => {
                    return Err(CompileError::UnexpectedSyntax {
                        found: syntax.node.kind,
                    });
                }
            };

            return self.create_emission_from_value(combined, target);
        }

        let left_address =
            self.handle_operand_emission(left_emission, &mut logic_instructions, &left)?;
        let right_address =
            self.handle_operand_emission(right_emission, &mut logic_instructions, &right)?;

        let target_registers = match target {
            ExpressionTarget::ClaimedRegister(registers) => registers,
            ExpressionTarget::UnclaimedRegister(register_kind) => {
                let type_id = *self.context.get_type_binding(&(self.code_id, syntax.id))?;

                self.claim_registers(type_id, register_kind)?
            }
            ExpressionTarget::Any => {
                let type_id = *self.context.get_type_binding(&(self.code_id, syntax.id))?;

                self.claim_registers(type_id, RegisterKind::Temporary)?
            }
        };
        let register = target_registers.expect_single()?;

        let test_instruction = match syntax.node.kind {
            SyntaxKind::AndExpression => Instruction::test(false, left_address, 1),
            SyntaxKind::OrExpression => Instruction::test(true, left_address, 1),
            _ => {
                return Err(CompileError::UnexpectedSyntax {
                    found: syntax.node.kind,
                });
            }
        };
        let right_move_instruction = Instruction::move_with_jump(
            register.index,
            OperandType::BOOLEAN,
            right_address,
            1,
            true,
        );
        let left_move_instruction =
            Instruction::r#move(register.index, OperandType::BOOLEAN, left_address);

        self.emit_instruction(test_instruction, &mut logic_instructions);
        self.emit_instruction(right_move_instruction, &mut logic_instructions);
        self.emit_instruction(left_move_instruction, &mut logic_instructions);

        logic_instructions.target_registers = Some(target_registers);

        Ok(Emission::Instructions(logic_instructions))
    }

    fn visit_negation_expression(
        &mut self,
        syntax: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        trace!("Visiting negation expression");

        let NegationExpression { operand } = syntax.as_component()?;

        let mut negation_instructions = self.create_instruction_range();

        let expression_emission = self.visit_expression(
            operand,
            ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
        )?;

        if let Emission::Value(constant) = &expression_emission
            && operand.node.kind != SyntaxKind::PathExpression
        {
            let negated = constant.negate(&operand)?;

            return self.create_emission_from_value(negated, target);
        }

        let operand = self.handle_operand_emission(
            expression_emission,
            &mut negation_instructions,
            &operand,
        )?;
        let target_registers = match target {
            ExpressionTarget::ClaimedRegister(registers) => registers,
            ExpressionTarget::UnclaimedRegister(register_kind) => {
                let type_id = *self.context.get_type_binding(&(self.code_id, syntax.id))?;

                self.claim_registers(type_id, register_kind)?
            }
            ExpressionTarget::Any => {
                let type_id = *self.context.get_type_binding(&(self.code_id, syntax.id))?;

                self.claim_registers(type_id, RegisterKind::Temporary)?
            }
        };
        let register = target_registers.expect_single()?;

        let negate_instruction =
            Instruction::negate(register.index, register.operand_type, operand);

        self.emit_instruction(negate_instruction, &mut negation_instructions);

        negation_instructions.target_registers = Some(target_registers);

        Ok(Emission::Instructions(negation_instructions))
    }

    fn visit_not_expression(
        &mut self,
        syntax: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        trace!("Visiting not expression");

        let NotExpression { operand } = syntax.as_component()?;

        let mut not_instructions = self.create_instruction_range();

        let expression_emission = self.visit_expression(
            operand,
            ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
        )?;

        if let Emission::Value(constant) = &expression_emission
            && operand.node.kind != SyntaxKind::PathExpression
        {
            let negated = constant.negate(&operand)?;

            return Ok(Emission::Value(negated));
        }

        let operand =
            self.handle_operand_emission(expression_emission, &mut not_instructions, &operand)?;
        let target_registers = match target {
            ExpressionTarget::ClaimedRegister(registers) => registers,
            ExpressionTarget::UnclaimedRegister(register_kind) => {
                let type_id = *self.context.get_type_binding(&(self.code_id, syntax.id))?;

                self.claim_registers(type_id, register_kind)?
            }
            ExpressionTarget::Any => {
                let type_id = *self.context.get_type_binding(&(self.code_id, syntax.id))?;

                self.claim_registers(type_id, RegisterKind::Temporary)?
            }
        };
        let register = target_registers.expect_single()?;

        let negate_instruction =
            Instruction::negate(register.index, register.operand_type, operand);

        self.emit_instruction(negate_instruction, &mut not_instructions);

        not_instructions.target_registers = Some(target_registers);

        Ok(Emission::Instructions(not_instructions))
    }

    fn visit_while_expression(
        &mut self,
        syntax: SyntaxReader<'_>,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        trace!("Visiting while expression");

        let WhileExpression { condition, body } = syntax.as_component()?;

        let mut while_instructions = self.create_instruction_range();

        let loop_start_index = self.instructions.len();

        let condition_emission = self.visit_expression(
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

        self.jump_forward_from_current_instruction(jump_forward_id, &mut while_instructions)?;

        let body_emission = self.visit_block_expression(body, target)?;

        if let Emission::Instructions(instructions) = body_emission {
            while_instructions.merge(instructions);
        }

        self.end_loop_on_next_instruction(
            loop_start_index,
            jump_backward_id,
            &mut while_instructions,
        )?;

        for break_id in take(&mut self.break_ids) {
            self.jump_forward_to_next_instruction(break_id)?;
        }

        self.jump_forward_to_next_instruction(jump_forward_id)?;

        while_instructions.target_registers = None;

        Ok(Emission::Instructions(while_instructions))
    }

    fn visit_break_expression(
        &mut self,
        _: SyntaxReader<'_>,
        _: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        trace!("Visiting break expression");

        let mut break_instructions = self.create_instruction_range();
        let break_id = self.create_jump_id();

        self.break_ids.push(break_id);
        self.jump_forward_from_current_instruction(break_id, &mut break_instructions)?;

        Ok(Emission::Instructions(break_instructions))
    }

    fn visit_call_expression(
        &mut self,
        syntax: SyntaxReader<'_>,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        trace!("Visiting call expression");

        let CallExpression { callee, arguments } = syntax.as_component()?;

        let mut call_instructions = self.create_instruction_range();

        let callee_emission = self.visit_expression(
            callee,
            ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
        )?;
        let callee =
            self.handle_operand_emission(callee_emission, &mut call_instructions, &callee)?;
        let (destination, registers) = self.claim_register_for_target(target, syntax)?;
        let arguments_start =
            self.visit_value_arguments(arguments, destination, &mut call_instructions)?;
        let call_instruction = Instruction::call(destination, callee, arguments_start);

        self.emit_instruction(call_instruction, &mut call_instructions);

        call_instructions.target_registers = registers;

        Ok(Emission::Instructions(call_instructions))
    }

    fn visit_method_call_expression(
        &mut self,
        syntax: SyntaxReader<'_>,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        trace!("Visiting method call expression");

        let MethodCallExpression {
            method_parent,
            method,
            value_arguments,
            ..
        } = syntax.as_component()?;

        let mut call_instructions = self.create_instruction_range();

        let method_type_id = *self.context.get_type_binding(&(self.code_id, method.id))?;
        let parent_type_id = {
            let raw_type_id = self
                .context
                .get_type_binding(&(self.code_id, method_parent.id))?;

            self.context.get_inferred_type_id(*raw_type_id)?
        };

        let Type::FunctionDefinition {
            declaration_id: method_declaration_id,
            type_arguments,
        } = *self.context.types.get_type(method_type_id)
        else {
            return Err(CompileError::ExpectedFunctionDefinitionType(method_type_id));
        };
        let method_declaration = self
            .context
            .declarations
            .get_declaration(method_declaration_id);

        let mut type_argument_ids = TypeId::SmallVec::new();

        if matches!(
            method_declaration.definition,
            Definition::Function {
                parent_impl_or_trait: Some(_),
                ..
            }
        ) {
            type_argument_ids.push(parent_type_id);
        }

        for index in type_arguments.as_usize_range() {
            let raw_type_argument_id = *self.context.types.get_type_member(index)?;
            let type_argument_id = self.context.get_inferred_type_id(raw_type_argument_id)?;

            type_argument_ids.push(type_argument_id);
        }

        let prototype_id = self
            .prototypes
            .monomorphize_function_to_prototype(method_declaration_id, type_argument_ids);
        let parent_emission = self.visit_expression(
            method_parent,
            ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
        )?;
        let parent =
            self.handle_operand_emission(parent_emission, &mut call_instructions, &method_parent)?;
        let parent_registers = self.claim_registers(parent_type_id, RegisterKind::Temporary)?;

        for register in &parent_registers.claims {
            let move_instruction =
                Instruction::r#move(register.index, register.operand_type, parent);

            self.emit_instruction(move_instruction, &mut call_instructions);
        }

        let arguments_start_register = parent_registers.base_index()?;

        if let Some(value_arguments) = value_arguments {
            self.visit_value_arguments(
                value_arguments,
                arguments_start_register,
                &mut call_instructions,
            )?;
        }

        let (destination, registers) = self.claim_register_for_target(target, syntax)?;
        let call_instruction = Instruction::call(
            destination,
            Address::new(MemoryKind::CONSTANT, prototype_id.index()),
            arguments_start_register,
        );

        self.emit_instruction(call_instruction, &mut call_instructions);

        call_instructions.target_registers = registers;

        Ok(Emission::Instructions(call_instructions))
    }

    fn visit_value_arguments(
        &mut self,
        arguments: SyntaxReader,
        arguments_start: u16,
        instructions: &mut InstructionRange,
    ) -> Result<u16, CompileError> {
        trace!("Visiting value arguments");

        let mut next_argument_offset = 0;

        for argument in arguments.children() {
            let argument_emission = self.visit_expression(
                argument,
                ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
            )?;
            let argument_address =
                self.handle_operand_emission(argument_emission, instructions, &argument)?;
            let argument_type_id = *self
                .context
                .get_type_binding(&(self.code_id, argument.id))?;
            let argument_operand_types = self.context.get_operand_types(argument_type_id)?;
            let argument_start_register = arguments_start + next_argument_offset;

            if argument_address.memory == MemoryKind::REGISTER
                && argument_address.index == argument_start_register
            {
                next_argument_offset += argument_operand_types
                    .iter()
                    .map(|operand_type| operand_type.register_width().as_u16())
                    .sum::<u16>();

                continue;
            }

            let mut value_offset = 0;

            for operand_type in argument_operand_types {
                let value_address = Address::new(
                    argument_address.memory,
                    argument_address.index + value_offset,
                );
                let move_instruction = Instruction::r#move(
                    argument_start_register + value_offset,
                    operand_type,
                    value_address,
                );

                self.emit_instruction(move_instruction, instructions);

                value_offset += operand_type.register_width().as_u16();
            }

            next_argument_offset += value_offset;
        }

        Ok(arguments_start)
    }

    fn visit_field_access_expression(
        &mut self,
        syntax: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        trace!("Visiting field access expression");

        let FieldAccessExpression {
            struct_expression,
            field_name,
        } = syntax.as_component()?;

        let operand_emission = self.visit_expression(
            struct_expression,
            ExpressionTarget::UnclaimedRegister(RegisterKind::Temporary),
        )?;

        let struct_registers = match operand_emission {
            Emission::Place(Place::Registers(registers)) => registers,
            emission => {
                let mut field_access_instructions = self.create_instruction_range();
                let place = self.place_emission(
                    emission,
                    &mut field_access_instructions,
                    &struct_expression,
                )?;

                match place {
                    Place::Registers(registers) => registers,
                    _ => {
                        return Err(CompileError::ExpectedValue {
                            code_id: struct_expression.code_id(),
                            syntax_id: struct_expression.id,
                        });
                    }
                }
            }
        };

        let field_declaration_id = *self
            .context
            .get_declaration_binding(&(self.code_id, field_name.id))?;
        let field_declaration = self
            .context
            .declarations
            .get_declaration(field_declaration_id);

        let parent_struct = match field_declaration.definition {
            Definition::Field { parent_struct, .. } => parent_struct,
            _ => {
                return Err(CompileError::ExpectedValue {
                    code_id: field_name.code_id(),
                    syntax_id: field_name.id,
                });
            }
        };

        let struct_declaration = self.context.declarations.get_declaration(parent_struct);

        let fields = match struct_declaration.definition {
            Definition::StructType {
                fields: Some(fields),
                ..
            } => fields,
            _ => {
                return Err(CompileError::ExpectedValue {
                    code_id: field_name.code_id(),
                    syntax_id: field_name.id,
                });
            }
        };

        let field_entries = self.context.scopes.get_members(fields);

        let mut register_offset = 0usize;

        for &field_id in field_entries {
            if field_id == field_declaration_id {
                break;
            }

            let field_declaration = self.context.declarations.get_declaration(field_id);

            if let Definition::Field { type_id, .. } = field_declaration.definition {
                let operand_types = self.context.get_operand_types(type_id)?;
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
                code_id: field_name.code_id(),
                syntax_id: field_name.id,
            }),
        }
    }

    fn visit_self_expression(
        &mut self,
        _syntax: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        trace!("Visiting self expression");

        let Some(self_registers) = &self.self_registers else {
            return Err(CompileError::ExpectedAllocation);
        };

        self.create_emission_from_registers(self_registers.clone(), target)
    }

    fn visit_reference_expression(
        &mut self,
        syntax: SyntaxReader,
        target: ExpressionTarget,
    ) -> Result<Emission, CompileError> {
        trace!("Visiting reference expression");

        let ReferenceExpression { operand } = syntax.as_component()?;

        let mut reference_instructions = self.create_instruction_range();

        let operand_emission = self.visit_expression(
            operand,
            ExpressionTarget::UnclaimedRegister(RegisterKind::Scoped),
        )?;
        let type_id = *self.context.get_type_binding(&(self.code_id, syntax.id))?;
        let referenced_type_id = if let Type::Reference {
            referenced_type_id, ..
        } = self.context.types.get_type(type_id)
        {
            *referenced_type_id
        } else {
            return Err(CompileError::ExpectedReferenceType(type_id));
        };
        let start_register = match operand_emission {
            Emission::Value(value) => {
                let register = match target {
                    ExpressionTarget::ClaimedRegister(register_claims) => register_claims,
                    _ => self.claim_registers(referenced_type_id, RegisterKind::Scoped)?,
                };
                let base_index = register.base_index()?;
                let operand_address = self.materialize_value(value)?;
                let move_instruction =
                    Instruction::r#move(base_index, value.operand_type(), operand_address);

                self.emit_instruction(move_instruction, &mut reference_instructions);

                base_index
            }
            Emission::Place(Place::Constant {
                index,
                operand_type,
            }) => {
                let register = self.claim_registers(referenced_type_id, RegisterKind::Scoped)?;
                let base_index = register.base_index()?;
                let operand_address = Address::new(MemoryKind::CONSTANT, index);
                let move_instruction =
                    Instruction::r#move(base_index, operand_type, operand_address);

                self.emit_instruction(move_instruction, &mut reference_instructions);

                base_index
            }
            Emission::Place(Place::Registers(registers)) => registers.base_index()?,
            Emission::Instructions(instructions) => {
                let base_index = instructions
                    .target_registers
                    .as_ref()
                    .ok_or(CompileError::ExpectedAllocation)?
                    .base_index()?;

                reference_instructions.merge(instructions);

                base_index
            }
            _ => {
                return Err(CompileError::ExpectedValue {
                    code_id: operand.code_id(),
                    syntax_id: operand.id,
                });
            }
        };
        let destination = self
            .register_tracker
            .allocate_next_local(OperandType::HEAP_POINTER);
        let end_register = self
            .context
            .get_operand_types(type_id)?
            .iter()
            .fold(start_register, |current, operand_type| {
                current + operand_type.register_width().as_u16()
            });
        let reference_instruction =
            Instruction::reference(destination, start_register, end_register);

        self.emit_instruction(reference_instruction, &mut reference_instructions);

        reference_instructions.target_registers = Some(RegisterClaims {
            claims: smallvec![RegisterClaim {
                index: destination,
                operand_type: OperandType::HEAP_POINTER
            }],
            kind: RegisterKind::Scoped,
        });

        Ok(Emission::Instructions(reference_instructions))
    }
}

#[derive(Clone, Debug)]
pub enum Emission {
    Value(ConstantValue),
    Place(Place),
    Instructions(InstructionRange),
    NativeFunction(NativeFunction),
    Never,
    None,
}

impl From<ConstantValue> for Emission {
    fn from(value: ConstantValue) -> Self {
        Emission::Value(value)
    }
}

#[derive(Clone, Debug, Default)]
pub struct InstructionRange {
    instructions: Range<u32>,
    drops: Range<u32>,
    target_registers: Option<RegisterClaims>,
}

impl InstructionRange {
    fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }

    fn merge(&mut self, other: InstructionRange) {
        debug_assert_eq!(self.instructions.end, other.instructions.start);
        debug_assert_eq!(self.drops.end, other.drops.start);

        self.instructions.end = other.instructions.end;
        self.drops.end = other.drops.end;
        self.target_registers = other.target_registers;
    }

    fn extend(&mut self, other: InstructionRange) -> Option<RegisterClaims> {
        debug_assert_eq!(self.instructions.end, other.instructions.start);
        debug_assert_eq!(self.drops.end, other.drops.start);

        self.instructions.end = other.instructions.end;
        self.drops.end = other.drops.end;

        other.target_registers
    }
}

#[derive(Clone, Debug)]
pub enum Place {
    Constant {
        index: u16,
        operand_type: OperandType,
    },
    Encoded {
        index: u16,
        operand_type: OperandType,
    },
    Registers(RegisterClaims),
}

impl Place {
    fn address(&self) -> Result<Address, CompileError> {
        match self {
            Place::Constant { index, .. } => Ok(Address::new(MemoryKind::CONSTANT, *index)),
            Place::Encoded { index, .. } => Ok(Address::new(MemoryKind::ENCODED, *index)),
            Place::Registers(registers) => registers
                .base_index()
                .map(|index| Address::new(MemoryKind::REGISTER, index)),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Local {
    Place(Place),
    Constant(ConstantValue),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegisterClaims {
    claims: RegisterClaim::SmallVec,
    kind: RegisterKind,
}

impl RegisterClaims {
    fn base(&self) -> Result<&RegisterClaim, CompileError> {
        self.claims.first().ok_or(CompileError::ExpectedAllocation)
    }

    fn base_index(&self) -> Result<u16, CompileError> {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RegisterClaim {
    index: u16,
    operand_type: OperandType,
}

impl RegisterClaim {
    type SmallVec = SmallVec<[Self; optimize_inline_capacity::<Self, 4>()]>;

    fn end(&self) -> u16 {
        self.index + self.operand_type.register_width().as_u16()
    }

    fn address(self) -> Address {
        Address::new(MemoryKind::REGISTER, self.index)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RegisterKind {
    Scoped,
    Temporary,
    Reserved,
}

#[derive(Debug)]
enum ExpressionTarget {
    ClaimedRegister(RegisterClaims),
    UnclaimedRegister(RegisterKind),
    Any,
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

    fn restore(&mut self, other: Self) {
        self.reserved = other.reserved;
        self.next_local = other.next_local;
        self.next_temporary = other.next_temporary;
        self.next_reserved = other.next_reserved;
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
        let next = self.next_reserved;
        self.next_reserved += operand_type.register_width().as_u16();

        next
    }

    fn deallocate(&mut self, allocation: &RegisterClaims) {
        let (Some(first_claim), Some(last_claim)) =
            (allocation.claims.first(), allocation.claims.last())
        else {
            return;
        };

        if self.max == last_claim.end() {
            self.max = first_claim.index;
        }

        match allocation.kind {
            RegisterKind::Scoped => {
                self.next_local = self.next_local.min(last_claim.index);
            }
            RegisterKind::Temporary => {
                self.next_temporary = self.next_temporary.min(last_claim.index);
            }
            RegisterKind::Reserved => {
                self.next_reserved = 0;
            }
        }
    }
}
