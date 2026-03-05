use std::{collections::HashMap, marker::PhantomData};

use rustc_hash::FxBuildHasher;
use smallvec::SmallVec;
use tracing::{debug, trace};

use crate::{
    compiler::error::CompileError,
    constant_table::{ConstantId, ConstantTable},
    dust_error::{ErrorKind, InternalError},
    instruction::{
        Add, Address, CallArgument, Drop, Instruction, MemoryKind, Operation, SmallType, Test,
    },
    native_function::NativeFunction,
    prototype::{Prototype, PrototypeId, PrototypeList},
    register::RegisterClass,
    resolver::{
        Resolver,
        declaration_graph::{DeclarationId, DeclarationKind, Visibility},
        scope_graph::ScopeId,
        type_graph::{TypeId, TypeNode},
    },
    source::{Position, Source, Span},
    syntax::{Syntax, SyntaxKind, SyntaxReader, SyntaxReaderIterator, SyntaxVisitor},
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
            integer_32_tracker: RegisterTracker::default(),
            integer_64_tracker: RegisterTracker::default(),
            float_64_tracker: RegisterTracker::default(),
            pointer_tracker: RegisterTracker::default(),
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

            for (parameter, expected_type) in parameters
                .into_iter()
                .zip(value_parameter_types.into_iter())
            {
                let parameter_id = *emitter.resolver.get_declaration_binding(&parameter.id)?;
                let register_classes = emitter
                    .resolver
                    .get_register_classes(expected_type, &parameter)?;
                let target = emitter.allocate_local_registers(register_classes);

                emitter.locals.insert(parameter_id, Place::Register(target));
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
                            operand,
                            jump_distance,
                            ..
                        } = Test::from(&*instruction);
                        let total_distance = jump_distance + distance;

                        *instruction = Instruction::test(operand, comparator, total_distance);
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
            register_count: self.integer_32_tracker.next_temporary
                + self.integer_64_tracker.next_temporary
                + self.float_64_tracker.next_temporary
                + self.pointer_tracker.next_temporary,
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

    fn allocate_temporary_registers(
        &mut self,
        classes: SmallVec<[RegisterClass; 8]>,
    ) -> RegisterAllocation {
        let tracker = match classes[0] {
            RegisterClass::Integer32 => &mut self.integer_32_tracker,
            RegisterClass::Integer64 => &mut self.integer_64_tracker,
            RegisterClass::Float64 => &mut self.float_64_tracker,
            RegisterClass::Pointer => &mut self.pointer_tracker,
        };
        let index = tracker.next_temporary;
        tracker.next_temporary += 1;

        trace!("Allocating temporary reg_{}_{index}", classes[0]);

        RegisterAllocation {
            classes,
            start_index: index,
            is_temporary: true,
        }
    }

    fn allocate_local_registers(
        &mut self,
        classes: SmallVec<[RegisterClass; 8]>,
    ) -> RegisterAllocation {
        let tracker = match classes[0] {
            RegisterClass::Integer32 => &mut self.integer_32_tracker,
            RegisterClass::Integer64 => &mut self.integer_64_tracker,
            RegisterClass::Float64 => &mut self.float_64_tracker,
            RegisterClass::Pointer => &mut self.pointer_tracker,
        };
        let index = tracker.next_local;
        tracker.next_local += 1;

        trace!("Allocating local reg_{}_{index}", classes[0]);

        RegisterAllocation {
            classes,
            start_index: index,
            is_temporary: false,
        }
    }

    fn free_temporary_registers(&mut self, target: &RegisterAllocation) {
        let tracker = match target.classes[0] {
            RegisterClass::Integer32 => &mut self.integer_32_tracker,
            RegisterClass::Integer64 => &mut self.integer_64_tracker,
            RegisterClass::Float64 => &mut self.float_64_tracker,
            RegisterClass::Pointer => &mut self.pointer_tracker,
        };
        let count = target.classes.len() as u16;

        debug_assert!(target.is_temporary);
        debug_assert!(tracker.next_temporary >= tracker.next_local + count);
        trace!(
            "Freeing temporary registers: {}",
            ((tracker.next_temporary - count)..(tracker.next_temporary))
                .map(|register| format!("reg_{register}"))
                .collect::<String>()
        );

        tracker.next_temporary -= count;
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
            if let Some(target) = &instructions.target
                && register == target.start_index
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
            ConstantEmission::Character(character) => self.constants.add_character(character),
            ConstantEmission::String {
                pool_start,
                pool_end,
            } => self.constants.add_pooled_string(pool_start, pool_end),
            ConstantEmission::U8(integer) => return Address::encoded_u8(integer),
            ConstantEmission::I8(integer) => return Address::encoded_i8(integer),
            ConstantEmission::U16(integer) => return Address::encoded_u16(integer),
            ConstantEmission::I16(integer) => return Address::encoded_i16(integer),
            ConstantEmission::U32(integer) => self.constants.add_u32(integer),
            ConstantEmission::I32(integer) => self.constants.add_i32(integer),
            ConstantEmission::U64(integer) => self.constants.add_u64(integer),
            ConstantEmission::I64(integer) => self.constants.add_i64(integer),
            ConstantEmission::U128(integer) => self.constants.add_u128(integer),
            ConstantEmission::I128(integer) => self.constants.add_i128(integer),
            ConstantEmission::F32(float) => self.constants.add_f32(float),
            ConstantEmission::F64(float) => self.constants.add_f64(float),
        };

        Address::constant(constant_id.0)
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
            let left_type = left_constant.small_type();
            let right_type = right_constant.small_type();

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
                let register_classes = self.resolver.get_register_classes(type_id, &node)?;
                let destination = self.allocate_temporary_registers(register_classes);
                let operand_type = constant.small_type();
                let operand = self.get_constant_address(constant);
                let move_instruction =
                    Instruction::r#move(destination.start_index, operand_type, operand, 0);

                self.emit_instruction(move_instruction);
            }
            Emission::Place(place) => {
                let type_id = *self.resolver.get_type_binding(&node.id)?;
                let register_classes = self.resolver.get_register_classes(type_id, &node)?;
                let destination = self.allocate_temporary_registers(register_classes);
                let operand_type = self
                    .resolver
                    .get_type_binding(&node.id)
                    .and_then(|type_id| self.resolver.get_small_type(*type_id, &node))?;
                let (operand, secondary_index) = place.address_and_register_size();
                let move_instruction = Instruction::r#move(
                    destination.start_index,
                    operand_type,
                    operand,
                    secondary_index,
                );

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
    ) -> Result<Address, ErrorKind> {
        let (operand, secondary_operand) = match emission {
            Emission::Constant(constant) => (self.get_constant_address(constant), 0),
            Emission::Place(place) => place.address_and_register_size(),
            Emission::Instructions(operand_instructions) => {
                let destination = operand_instructions
                    .target
                    .as_ref()
                    .ok_or(ErrorKind::Compile(CompileError::ExpectedValue {
                        node_kind: node.kind(),
                        position: node.position(),
                    }))?
                    .start_index;

                instructions.merge(operand_instructions);

                return Ok(Address::register(destination));
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
                    node_kind: node.kind(),
                    position: node.position(),
                }));
            }
        };

        if secondary_operand != 0 {
            let type_id = *self.resolver.get_type_binding(&node.id)?;
            let operand_type = self.resolver.get_small_type(type_id, node)?;
            let register_size = self.resolver.get_register_classes(type_id, node)?;
            let temporary_target = self.allocate_temporary_registers(register_size);
            let move_instruction = Instruction::r#move(
                temporary_target.start_index,
                operand_type,
                operand,
                secondary_operand,
            );

            instructions.push(move_instruction);

            Ok(Address::register(temporary_target.start_index))
        } else {
            Ok(operand)
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
                let operand = self.get_constant_address(constant);
                let test_instruction = Instruction::test(operand, true, 1);

                instructions.push(test_instruction);
            }
            Emission::Place(place) => {
                let (operand, _) = place.address_and_register_size();
                let test_instruction = Instruction::test(operand, true, 1);

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

                            if let Some(target) = &condition_instructions.target
                                && target.is_temporary
                            {
                                self.free_temporary_registers(target);
                            }
                        }
                        Operation::TEST => {
                            let first_move_instruction =
                                condition_instructions.instructions[length - 2].0;
                            let new_test_instruction =
                                Instruction::test(first_move_instruction.b_address(), false, 0);

                            condition_instructions.instructions.truncate(length - 2);
                            condition_instructions.push(new_test_instruction);

                            if let Some(target) = &condition_instructions.target
                                && target.is_temporary
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
        target: &RegisterAllocation,
        node: SyntaxReader,
    ) -> Result<(), ErrorKind> {
        match emission {
            Emission::Constant(constant) => {
                let operand_type = constant.small_type();
                let operand = self.get_constant_address(constant);
                let move_instruction =
                    Instruction::r#move(target.start_index, operand_type, operand, 0);

                instructions_emission.push(move_instruction);
            }
            Emission::Place(place) => {
                let operand_type = self
                    .resolver
                    .get_type_binding(&node.id)
                    .and_then(|type_id| self.resolver.get_small_type(*type_id, &node))?;
                let (operand, secondary_operand) = place.address_and_register_size();
                let move_instruction = Instruction::r#move(
                    target.start_index,
                    operand_type,
                    operand,
                    secondary_operand,
                );

                instructions_emission.push(move_instruction);
            }
            Emission::Instructions(branch_instructions) => {
                instructions_emission.merge(branch_instructions);
                instructions_emission.set_target(Some(target.clone()));
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
        let operand_type = self
            .resolver
            .get_type_binding(&node.id)
            .and_then(|type_id| self.resolver.get_small_type(*type_id, &node))?;
        let (operand, register_size) = match emission {
            Emission::Constant(constant) => (self.get_constant_address(constant), 0),
            Emission::Place(place) => place.address_and_register_size(),
            Emission::Instructions(instructions) => {
                return_instructions.merge(instructions);

                if let Some(target) = &return_instructions.target {
                    target.address_and_register_size()
                } else {
                    debug_assert_eq!(operand_type, SmallType::UNIT);

                    (Address::default(), 0)
                }
            }
            Emission::NativeFunction(_) => {
                return Err(ErrorKind::Compile(
                    CompileError::ExpectedNativeFunctionCall {
                        position: node.position(),
                    },
                ));
            }
            Emission::None => (Address::default(), 0),
        };
        let return_instruction = Instruction::r#return(operand_type, operand, register_size);

        return_instructions.push(return_instruction);

        Ok(())
    }

    fn handle_implicit_return(
        &mut self,
        node: SyntaxReader,
        target: Option<&RegisterAllocation>,
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

            let return_instruction = Instruction::r#return(SmallType::UNIT, Address::default(), 0);

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

        let register_size = self.resolver.get_register_classes(type_id, &expression)?;
        let target = self.allocate_local_registers(register_size);
        let destination = target.start_index;
        let expression_emission = self.visit_expression(expression, Some(&target))?;

        match expression_emission {
            Emission::Constant(constant) => {
                let operand_type = constant.small_type();
                let operand = self.get_constant_address(constant);
                let move_instruction = Instruction::r#move(destination, operand_type, operand, 0);

                let_statement_instructions.push(move_instruction);
            }
            Emission::Place(place) => {
                let operand_type = self
                    .resolver
                    .get_type_binding(&expression.id)
                    .and_then(|type_id| self.resolver.get_small_type(*type_id, &expression))?;
                let (operand, secondary_operand) = place.address_and_register_size();
                let move_instruction =
                    Instruction::r#move(destination, operand_type, operand, secondary_operand);

                let_statement_instructions.push(move_instruction);
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

        if type_id == TypeId::STRING {
            self.add_drop(destination);
        }

        self.locals.insert(declaration_id, Place::Register(target));
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

        let target = local.expect_register(&path)?.clone();
        let destination = target.start_index;

        let mut reassignment_instructions = InstructionsEmission::new();
        let expression_emission = self.visit_expression(expression, Some(&target))?;

        match expression_emission {
            Emission::Constant(constant) => {
                let operand_type = constant.small_type();
                let operand = self.get_constant_address(constant);
                let move_instruction = Instruction::r#move(destination, operand_type, operand, 0);

                reassignment_instructions.push(move_instruction);
            }
            Emission::Place(place) => {
                let operand_type = self
                    .resolver
                    .get_type_binding(&expression.id)
                    .and_then(|type_id| self.resolver.get_small_type(*type_id, &expression))?;
                let (address, secondary_address) = place.address_and_register_size();
                let move_instruction =
                    Instruction::r#move(destination, operand_type, address, secondary_address);

                reassignment_instructions.push(move_instruction);
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
            instructions: &mut InstructionsEmission,
            element_emission: Emission,
            element_node: &SyntaxReader,
        ) -> Result<Address, ErrorKind> {
            match element_emission {
                Emission::Place(place) => {
                    let operand_type = emitter
                        .resolver
                        .get_type_binding(&element_node.id)
                        .and_then(|type_id| {
                            emitter.resolver.get_small_type(*type_id, &element_node)
                        })?;
                    let (operand, size) = place.address_and_register_size();

                    if size != 0 {
                        let type_id = *emitter.resolver.get_type_binding(&element_node.id)?;
                        let register_classes = emitter
                            .resolver
                            .get_register_classes(type_id, &element_node)?;
                        let target = emitter.allocate_temporary_registers(register_classes);
                        let move_instruction =
                            Instruction::r#move(target.start_index, operand_type, operand, size);

                        instructions.push(move_instruction);

                        Ok(Address::register(target.start_index))
                    } else {
                        Ok(operand)
                    }
                }
                Emission::Constant(constant) => {
                    let type_id = *emitter.resolver.get_type_binding(&element_node.id)?;
                    let register_classes = emitter
                        .resolver
                        .get_register_classes(type_id, &element_node)?;
                    let operand_type = constant.small_type();
                    let operand = emitter.get_constant_address(constant);

                    let target = emitter.allocate_temporary_registers(register_classes);
                    let move_instruction =
                        Instruction::r#move(target.start_index, operand_type, operand, 0);

                    instructions.push(move_instruction);

                    Ok(Address::register(target.start_index))
                }
                Emission::Instructions(InstructionsEmission {
                    instructions: element_instructions,
                    target,
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
            self.get_constant_address(ConstantEmission::U64(elements.len() as u64));

        let target = if let Some(target) = target {
            target.clone()
        } else {
            let type_id = *self.resolver.get_type_binding(&node.id)?;
            let register_classes = self.resolver.get_register_classes(type_id, &node)?;

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
            let index_address = self.get_constant_address(ConstantEmission::U64(index as u64));
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
            let register_classes = self.resolver.get_register_classes(type_id, &node)?;

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
            let register_classes = self.resolver.get_register_classes(type_id, &node)?;

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
                                self.resolver.get_register_classes(type_id, &child)?;

                            self.allocate_temporary_registers(register_classes)
                        };
                        let operand_type = constant.small_type();
                        let operand = self.get_constant_address(constant);
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
                                    self.resolver.get_register_classes(type_id, &child)?;

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
            let register_classes = self.resolver.get_register_classes(type_id, &node)?;

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
        let mut get_target = || -> Result<RegisterAllocation, ErrorKind> {
            if let Some(target) = target {
                Ok(target.clone())
            } else {
                let register_classes = self.resolver.get_register_classes(type_id, &node)?;

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
            let register_classes = self.resolver.get_register_classes(TypeId::BOOLEAN, &node)?;

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
            SmallType::BOOLEAN,
            Address::encoded_boolean(false),
            1,
        );
        let load_true_instruction = Instruction::r#move(
            target.start_index,
            SmallType::BOOLEAN,
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
            let register_classes = self.resolver.get_register_classes(TypeId::BOOLEAN, &node)?;

            self.allocate_temporary_registers(register_classes)
        };

        let test_instruction = match node.kind() {
            SyntaxKind::AndExpression => Instruction::test(left_address, false, 1),
            SyntaxKind::OrExpression => Instruction::test(left_address, true, 1),
            _ => unreachable!("Expected logical expression, found {}", node.kind()),
        };
        let right_move_instruction =
            Instruction::r#move(target.start_index, SmallType::BOOLEAN, right_address, 1);
        let left_move_instruction =
            Instruction::r#move(target.start_index, SmallType::BOOLEAN, left_address, 0);

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
                    operand_type: constant.small_type(),
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
            let register_classes = self.resolver.get_register_classes(type_id, &node)?;

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
                prototype_id: *prototype_index,
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

        Ok(Emission::Place(Place::Prototype { prototype_id }))
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
                r#type: SmallType::UNIT,
            });
            argument_count += 1;
        }

        let callee_emission = self.visit_expression(callee, None)?;
        let callee_address =
            self.handle_operand_emission(&mut call_emission, callee_emission, &callee)?;

        let return_type_id = *self.resolver.get_type_binding(&node.id)?;
        let return_operand_type = self.resolver.get_small_type(return_type_id, &node)?;

        let register_count = self.resolver.get_register_classes(return_type_id, &node)?;
        let target = if let Some(target) = target {
            Some(target.clone())
        } else if return_operand_type != SmallType::UNIT {
            Some(self.allocate_temporary_registers(register_count))
        } else {
            None
        };
        let destination = target.as_ref().map(|target| target.start_index);

        let call_instruction =
            Instruction::call(destination, callee_address, arguments_start, argument_count);

        call_emission.push(call_instruction);

        if return_operand_type != SmallType::UNIT {
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
    fn target(&self) -> Option<&RegisterAllocation> {
        match self {
            Emission::Place(Place::Register(target)) => Some(target),
            Emission::Instructions(emission) => emission.target.as_ref(),
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
    Constant { id: ConstantId },
    Prototype { prototype_id: PrototypeId },
    Register(RegisterAllocation),
}

impl Place {
    fn expect_register(&self, node: &SyntaxReader) -> Result<&RegisterAllocation, ErrorKind> {
        match self {
            Place::Register(target) => Ok(target),
            _ => Err(ErrorKind::Compile(CompileError::CannotMutate {
                position: node.position(),
            })),
        }
    }

    fn address_and_register_size(&self) -> (Address, u16) {
        match self {
            Place::Constant { id } => (Address::constant(id.0), 0),
            Place::Prototype { prototype_id } => (Address::constant(prototype_id.index()), 0),
            Place::Register(register) => register.address_and_register_size(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RegisterAllocation {
    classes: SmallVec<[RegisterClass; 8]>,
    start_index: u16,
    is_temporary: bool,
}

impl RegisterAllocation {
    fn address_and_register_size(&self) -> (Address, u16) {
        let address = Address::register(self.start_index);
        let register_size = self.classes.len() as u16;

        (address, register_size)
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
    fn small_type(&self) -> SmallType {
        match self {
            ConstantEmission::Boolean(_) => SmallType::BOOLEAN,
            ConstantEmission::Character(_) => SmallType::CHARACTER,
            ConstantEmission::String { .. } => SmallType::STRING,
            ConstantEmission::U8(_) => SmallType::U_8,
            ConstantEmission::I8(_) => SmallType::I_8,
            ConstantEmission::U16(_) => SmallType::U_16,
            ConstantEmission::I16(_) => SmallType::I_16,
            ConstantEmission::U32(_) => SmallType::U_32,
            ConstantEmission::I32(_) => SmallType::I_32,
            ConstantEmission::U64(_) => SmallType::U_64,
            ConstantEmission::I64(_) => SmallType::I_64,
            ConstantEmission::U128(_) => SmallType::U_128,
            ConstantEmission::I128(_) => SmallType::I_128,
            ConstantEmission::F32(_) => SmallType::F_32,
            ConstantEmission::F64(_) => SmallType::F_64,
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
}
