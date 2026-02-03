use std::collections::HashMap;

use rustc_hash::FxBuildHasher;
use smallvec::SmallVec;
use tracing::{debug, trace};

use crate::{
    compiler::{
        CompileContext, CompileError,
        context::{Declaration, DeclarationId, DeclarationKind, ScopeId, TypeId, TypeNode},
    },
    instruction::{Address, Drop, Instruction, MemoryKind, Move, OperandType, Operation, Test},
    native_function::NativeFunction,
    prototype::Prototype,
    source::{Position, Source, SourceFileId, Span},
    syntax::{Syntax, SyntaxId, SyntaxKind, SyntaxNode, SyntaxReader, SyntaxVisitor},
    r#type::Type,
};

#[derive(Debug)]
pub struct Emitter<'a> {
    declaration_id: Option<DeclarationId>,

    prototype_index: u16,

    file_id: SourceFileId,

    function_type_id: TypeId,

    source: &'a Source,

    syntax: &'a Syntax,

    context: &'a mut CompileContext,

    /// Bytecode instruction list that is filled during compilation.
    instructions: Vec<Instruction>,

    /// Local variables declared in the function being compiled.
    locals: HashMap<DeclarationId, Target, FxBuildHasher>,

    /// Concatenated list of arguments referenced by CALL instructions.
    call_arguments: Vec<(Address, OperandType)>,

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

    maximum_register: u16,
}

impl<'a> Emitter<'a> {
    pub fn new(
        declaration_info: Option<(DeclarationId, Declaration)>,
        prototype_index: u16,
        file_id: SourceFileId,
        function_type_id: TypeId,
        source: &'a Source,
        syntax: &'a Syntax,
        context: &'a mut CompileContext,
        starting_scope_id: ScopeId,
    ) -> Self {
        let mut emitter = Self {
            declaration_id: declaration_info.map(|(id, _)| id),
            prototype_index,
            file_id,
            function_type_id,
            source,
            syntax,
            context,
            instructions: Vec::new(),
            locals: HashMap::default(),
            call_arguments: Vec::new(),
            drop_lists: Vec::new(),
            pending_drops: vec![SmallVec::new()],
            jump_placements: HashMap::new(),
            jump_over_else_anchor_ids: Vec::new(),
            current_scope_id: starting_scope_id,
            next_jump_id: 0,
            next_local_register: 0,
            next_temporary_register: 0,
            maximum_register: 0,
        };

        if let Some((declaration_id, declaration)) = &declaration_info
            && let DeclarationKind::Function { parameters, .. } = &declaration.kind
        {
            emitter.locals.insert(
                *declaration_id,
                Target::Constant {
                    index: prototype_index,
                },
            );

            let (start, count) = *parameters;

            emitter.locals.reserve(count as usize);

            for index in 0..count {
                let current_parameter_index = start + index;
                if let Some(parameter_id) = emitter
                    .context
                    .get_declaration_member(current_parameter_index)
                {
                    let target = emitter.allocate_local_register();

                    emitter.locals.insert(parameter_id, target);
                }
            }
        }

        emitter
    }

    pub fn emit_main(self) -> Result<Prototype, CompileError> {
        let root = self
            .syntax
            .get_tree(self.file_id)
            .ok_or(CompileError::MissingSyntaxTree {
                file_id: self.file_id,
            })?
            .root()
            .ok_or(CompileError::MissingSyntaxNode {
                syntax_id: SyntaxId::ROOT,
            })?;

        self.emit(root)
    }

    pub fn emit(mut self, node: SyntaxReader) -> Result<Prototype, CompileError> {
        match node.kind() {
            SyntaxKind::MainFunctionItem | SyntaxKind::BlockExpression => {
                let children = node
                    .multiple_children()
                    .ok_or(CompileError::MissingChildren {
                        parent_kind: node.kind(),
                        start_index: node.inner().children.0,
                        count: node.inner().children.1,
                    })?;
                let last_index = children.len() - 1;

                for (index, child) in children.into_iter().enumerate() {
                    let child_emission = if index == last_index {
                        self.handle_implicit_return(child, None)?
                    } else {
                        self.visit(child, None)?
                    };

                    self.handle_top_emission(child_emission, child)?;
                }
            }
            SyntaxKind::ExpressionStatement => {
                let expression_node = node.left_child().ok_or(CompileError::MissingChild {
                    parent_kind: node.kind(),
                    child_index: 0,
                })?;

                let mut expression_emission = self.handle_implicit_return(expression_node, None)?;

                if let Emission::Instructions(instructions) = &mut expression_emission {
                    instructions.set_target(None);
                }

                self.handle_top_emission(expression_emission, expression_node)?;
            }
            _ => return Err(CompileError::InvalidSyntaxNode { kind: node.kind() }),
        }

        self.finish()
    }

    pub fn finish(mut self) -> Result<Prototype, CompileError> {
        // self.context.constants.finalize_string_pool();

        let name_position = if let Some(declaration_id) = self.declaration_id {
            let declaration = self
                .context
                .get_declaration(declaration_id)
                .ok_or(CompileError::MissingDeclaration { declaration_id })?;

            Some(declaration.position)
        } else {
            None
        };
        let register_count = self.maximum_register;
        let function_type = self
            .context
            .get_full_type(self.function_type_id, self.source)
            .ok_or(CompileError::MissingType {
                type_id: self.function_type_id,
            })?
            .into_function_type()
            .ok_or(CompileError::ExpectedFunctionType {
                type_id: self.function_type_id,
            })?;

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
                            r#type,
                            jump_distance,
                            ..
                        } = Move::from(&*instruction);
                        let total_distance = jump_distance + distance;

                        *instruction = Instruction::move_with_jump(
                            destination,
                            operand,
                            r#type,
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
                        panic!("Expected MOVE or DROP instruction for coalesced jump");
                    }
                }
            } else {
                let jump_instruction = Instruction::jump(distance, forward);

                self.instructions[index] = jump_instruction;
            }
        }

        Ok(Prototype {
            index: self.prototype_index,
            name_position,
            function_type,
            instructions: self.instructions,
            call_arguments: self.call_arguments,
            drop_lists: self.drop_lists,
            register_count,
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

    fn allocate_temporary_register(&mut self) -> Target {
        let register = self.next_temporary_register;

        trace!("Allocating temporary reg_{}", register);

        self.next_temporary_register += 1;

        if self.next_temporary_register > self.maximum_register {
            self.maximum_register = self.next_temporary_register;
        }

        Target::Register {
            index: register,
            is_temporary: true,
        }
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

        if self.maximum_register > self.next_temporary_register {
            self.maximum_register = self.next_temporary_register;
        }
    }

    fn allocate_local_register(&mut self) -> Target {
        let register = self.next_local_register;

        trace!("Allocating local reg_{}", register);

        self.next_local_register += 1;
        self.next_temporary_register = self.next_local_register;

        if self.next_local_register > self.maximum_register {
            self.maximum_register = self.next_local_register;
        }

        Target::Register {
            index: register,
            is_temporary: false,
        }
    }

    fn enter_child_scope(&mut self, child_scope_id: ScopeId) {
        self.current_scope_id = child_scope_id;

        self.pending_drops.push(SmallVec::new());
    }

    fn enter_parent_scope(&mut self, parent_scope_id: ScopeId, next_local_register: u16) {
        self.current_scope_id = parent_scope_id;
        self.next_local_register = next_local_register;
        self.next_temporary_register = next_local_register;
    }

    fn add_drop(&mut self, register: u16) {
        self.pending_drops.last_mut().unwrap().push(register);
    }

    fn get_constant_address(&mut self, constant: Constant) -> Address {
        let index = match constant {
            Constant::Boolean(boolean) => return Address::encoded(boolean as u16),
            Constant::Byte(byte) => return Address::encoded(byte as u16),
            Constant::Character(character) => self.context.constants.add_character(character),
            Constant::Float(float) => self.context.constants.add_float(float),
            Constant::Integer(integer) => self.context.constants.add_integer(integer),
            Constant::String {
                pool_start,
                pool_end,
            } => self
                .context
                .constants
                .add_pooled_string(pool_start, pool_end),
        };

        Address::constant(index)
    }

    fn combine_constants(
        &mut self,
        operation: SyntaxKind,
        left: Constant,
        left_node: &SyntaxNode,
        right: Constant,
        right_node: &SyntaxNode,
    ) -> Result<Constant, CompileError> {
        debug!(
            "Combining constants: {:?} {:?} {:?}",
            left, right, operation
        );

        let check_for_division_by_zero = || {
            if matches!(
                right,
                Constant::Byte(0) | Constant::Integer(0) | Constant::Float(0.0)
            ) {
                Err(CompileError::DivisionByZero {
                    position: Position::new(
                        self.file_id,
                        Span::new(left_node.span.0, right_node.span.1),
                    ),
                })
            } else {
                Ok(())
            }
        };

        let combined = match (left, right) {
            (Constant::Boolean(left), Constant::Boolean(right)) => match operation {
                SyntaxKind::AndExpression => Constant::Boolean(left && right),
                SyntaxKind::OrExpression => Constant::Boolean(left || right),
                SyntaxKind::GreaterThanExpression => Constant::Boolean(left || right),
                SyntaxKind::GreaterThanOrEqualExpression => Constant::Boolean(left >= right),
                SyntaxKind::LessThanExpression => Constant::Boolean(left || right),
                SyntaxKind::LessThanOrEqualExpression => Constant::Boolean(left <= right),
                SyntaxKind::EqualExpression => Constant::Boolean(left == right),
                SyntaxKind::NotEqualExpression => Constant::Boolean(left != right),
                _ => todo!(),
            },
            (Constant::Byte(left), Constant::Byte(right)) => match operation {
                SyntaxKind::AdditionExpression => Constant::Byte(left.saturating_add(right)),
                SyntaxKind::SubtractionExpression => Constant::Byte(left.saturating_sub(right)),
                SyntaxKind::MultiplicationExpression => Constant::Byte(left.saturating_mul(right)),
                SyntaxKind::DivisionExpression => {
                    check_for_division_by_zero()?;

                    Constant::Byte(left.saturating_div(right))
                }
                SyntaxKind::ModuloExpression => {
                    check_for_division_by_zero()?;

                    Constant::Byte(left % right)
                }
                SyntaxKind::ExponentExpression => Constant::Byte(left.saturating_pow(right as u32)),
                SyntaxKind::GreaterThanExpression => Constant::Boolean(left > right),
                SyntaxKind::GreaterThanOrEqualExpression => Constant::Boolean(left >= right),
                SyntaxKind::LessThanExpression => Constant::Boolean(left < right),
                SyntaxKind::LessThanOrEqualExpression => Constant::Boolean(left <= right),
                SyntaxKind::EqualExpression => Constant::Boolean(left == right),
                SyntaxKind::NotEqualExpression => Constant::Boolean(left != right),
                _ => todo!(),
            },
            (Constant::Float(left), Constant::Float(right)) => match operation {
                SyntaxKind::AdditionExpression => Constant::Float(left + right),
                SyntaxKind::SubtractionExpression => Constant::Float(left - right),
                SyntaxKind::MultiplicationExpression => Constant::Float(left * right),
                SyntaxKind::DivisionExpression => {
                    check_for_division_by_zero()?;

                    Constant::Float(left / right)
                }
                SyntaxKind::ModuloExpression => {
                    check_for_division_by_zero()?;

                    Constant::Float(left % right)
                }
                SyntaxKind::ExponentExpression => Constant::Float(left.powf(right)),
                SyntaxKind::GreaterThanExpression => Constant::Boolean(left > right),
                SyntaxKind::GreaterThanOrEqualExpression => Constant::Boolean(left >= right),
                SyntaxKind::LessThanExpression => Constant::Boolean(left < right),
                SyntaxKind::LessThanOrEqualExpression => Constant::Boolean(left <= right),
                SyntaxKind::EqualExpression => Constant::Boolean(left == right),
                SyntaxKind::NotEqualExpression => Constant::Boolean(left != right),
                _ => todo!(),
            },
            (Constant::Integer(left), Constant::Integer(right)) => match operation {
                SyntaxKind::AdditionExpression => Constant::Integer(left.saturating_add(right)),
                SyntaxKind::SubtractionExpression => Constant::Integer(left.saturating_sub(right)),
                SyntaxKind::MultiplicationExpression => {
                    Constant::Integer(left.saturating_mul(right))
                }
                SyntaxKind::DivisionExpression => {
                    check_for_division_by_zero()?;

                    Constant::Integer(left.saturating_div(right))
                }
                SyntaxKind::ModuloExpression => {
                    check_for_division_by_zero()?;

                    Constant::Integer(left % right)
                }
                SyntaxKind::ExponentExpression => {
                    Constant::Integer(left.saturating_pow(right as u32))
                }
                SyntaxKind::GreaterThanExpression => Constant::Boolean(left > right),
                SyntaxKind::GreaterThanOrEqualExpression => Constant::Boolean(left >= right),
                SyntaxKind::LessThanExpression => Constant::Boolean(left < right),
                SyntaxKind::LessThanOrEqualExpression => Constant::Boolean(left <= right),
                SyntaxKind::EqualExpression => Constant::Boolean(left == right),
                SyntaxKind::NotEqualExpression => Constant::Boolean(left != right),
                _ => todo!(),
            },
            (Constant::Character(left), Constant::Character(right)) => match operation {
                SyntaxKind::AdditionExpression => {
                    let mut string = String::with_capacity(2);

                    string.push(left);
                    string.push(right);

                    let combined = self
                        .context
                        .constants
                        .push_str_to_string_pool(string.as_bytes());

                    Constant::String {
                        pool_start: combined.0,
                        pool_end: combined.1,
                    }
                }
                SyntaxKind::GreaterThanExpression => Constant::Boolean(left > right),
                SyntaxKind::GreaterThanOrEqualExpression => Constant::Boolean(left >= right),
                SyntaxKind::LessThanExpression => Constant::Boolean(left < right),
                SyntaxKind::LessThanOrEqualExpression => Constant::Boolean(left <= right),
                SyntaxKind::EqualExpression => Constant::Boolean(left == right),
                SyntaxKind::NotEqualExpression => Constant::Boolean(left != right),
                _ => todo!("Error"),
            },
            (
                Constant::String {
                    pool_start: left_pool_start,
                    pool_end: left_pool_end,
                },
                Constant::String {
                    pool_start: right_pool_start,
                    pool_end: right_pool_end,
                },
            ) => {
                let left = self
                    .context
                    .constants
                    .get_string_pool_range(left_pool_start as usize..left_pool_end as usize);
                let right = self
                    .context
                    .constants
                    .get_string_pool_range(right_pool_start as usize..right_pool_end as usize);

                match operation {
                    SyntaxKind::AdditionExpression => {
                        if left_pool_end == right_pool_start {
                            return Ok(Constant::String {
                                pool_start: left_pool_start,
                                pool_end: right_pool_end,
                            });
                        }

                        let mut string = String::with_capacity(left.len() + right.len());

                        string.push_str(left);
                        string.push_str(right);

                        let combined = self
                            .context
                            .constants
                            .push_str_to_string_pool(string.as_bytes());

                        Constant::String {
                            pool_start: combined.0,
                            pool_end: combined.1,
                        }
                    }
                    SyntaxKind::GreaterThanExpression => Constant::Boolean(left > right),
                    SyntaxKind::GreaterThanOrEqualExpression => Constant::Boolean(left >= right),
                    SyntaxKind::LessThanExpression => Constant::Boolean(left < right),
                    SyntaxKind::LessThanOrEqualExpression => Constant::Boolean(left <= right),
                    SyntaxKind::EqualExpression => Constant::Boolean(left == right),
                    SyntaxKind::NotEqualExpression => Constant::Boolean(left != right),
                    _ => todo!("Error"),
                }
            }
            (
                Constant::Character(left),
                Constant::String {
                    pool_start,
                    pool_end,
                },
            ) => {
                let right = self
                    .context
                    .constants
                    .get_string_pool_range(pool_start as usize..pool_end as usize);
                let mut string = String::with_capacity(1 + right.len());

                string.push(left);
                string.push_str(right);

                let combined = match operation {
                    SyntaxKind::AdditionExpression => self
                        .context
                        .constants
                        .push_str_to_string_pool(string.as_bytes()),
                    _ => todo!("Error"),
                };

                Constant::String {
                    pool_start: combined.0,
                    pool_end: combined.1,
                }
            }
            (
                Constant::String {
                    pool_start,
                    pool_end,
                },
                Constant::Character(right),
            ) => {
                let left = self
                    .context
                    .constants
                    .get_string_pool_range(pool_start as usize..pool_end as usize);
                let mut string = String::with_capacity(left.len() + 1);

                string.push_str(left);
                string.push(right);

                let combined = match operation {
                    SyntaxKind::AdditionExpression => self
                        .context
                        .constants
                        .push_str_to_string_pool(string.as_bytes()),
                    _ => todo!("Error"),
                };

                Constant::String {
                    pool_start: combined.0,
                    pool_end: combined.1,
                }
            }
            _ => {
                return Err(CompileError::TypeConflict {
                    expected: left.full_type(),
                    found: right.full_type(),
                    position: Position::new(self.file_id, right_node.span),
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
        let type_id = *self
            .context
            .get_type_binding(&node.id)
            .ok_or(CompileError::MissingTypeBinding { syntax_id: node.id })?;

        match emission {
            Emission::Constant(constant) => {
                let destination = self.allocate_temporary_register();
                let address = self.get_constant_address(constant);
                let operand_type = self
                    .context
                    .get_operand_type(type_id)
                    .ok_or(CompileError::MissingType { type_id })?;
                let move_instruction =
                    Instruction::r#move(destination.index(), address, operand_type);

                self.emit_instruction(move_instruction);
            }
            Emission::Target(target) => {
                let destination = self.allocate_temporary_register();
                let type_id = *self
                    .context
                    .get_type_binding(&node.id)
                    .ok_or(CompileError::MissingTypeBinding { syntax_id: node.id })?;
                let operand_type = self.context.get_operand_type(type_id).ok_or(
                    CompileError::CannotInferType {
                        position: Position::new(self.file_id, node.span()),
                    },
                )?;
                let operand_address = Address::register(target.index());
                let move_instruction =
                    Instruction::r#move(destination.index(), operand_address, operand_type);

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
            Emission::Target(target) => Ok(target.address()),
            Emission::Instructions(operand_instructions) => {
                let destination = operand_instructions
                    .target
                    .ok_or(CompileError::ExpectedExpression {
                        node_kind: node.kind(),
                        position: Position::new(self.file_id, node.span()),
                    })?
                    .index();

                instructions.merge(operand_instructions);

                Ok(Address::register(destination))
            }
            Emission::None => Err(CompileError::ExpectedExpression {
                node_kind: node.kind(),
                position: Position::new(self.file_id, node.span()),
            }),
        }
    }

    fn handle_condition_emission(
        &mut self,
        instructions: &mut InstructionsEmission,
        emission: Emission,
        node: &SyntaxNode,
    ) -> Result<(), CompileError> {
        match emission {
            Emission::Constant(constant) => {
                let address = self.get_constant_address(constant);
                let test_instruction = Instruction::test(address, true, 1);

                instructions.push(test_instruction);
            }
            Emission::Target(target) => {
                let address = Address::register(target.index());
                let test_instruction = Instruction::test(address, true, 1);

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
                                self.free_temporary_registers(1);
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
                                self.free_temporary_registers(1);
                            }
                        }
                        _ => {}
                    }
                }

                instructions.merge(condition_instructions);
            }
            _ => {
                return Err(CompileError::ExpectedBooleanExpression {
                    node_kind: node.kind,
                    position: Position::new(self.file_id, node.span),
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
                let operand_type = constant.operand_type();
                let move_instruction =
                    Instruction::r#move(destination_register, address, operand_type);

                instructions_emission.push(move_instruction);
            }
            Emission::Target(target) => {
                let type_id = *self
                    .context
                    .get_type_binding(&node.id)
                    .ok_or(CompileError::MissingTypeBinding { syntax_id: node.id })?;
                let operand_type = self
                    .context
                    .get_operand_type(type_id)
                    .ok_or(CompileError::MissingType { type_id })?;
                let move_instruction =
                    Instruction::r#move(destination_register, target.address(), operand_type);

                instructions_emission.push(move_instruction);
            }
            Emission::Instructions(branch_instructions) => {
                instructions_emission.merge(branch_instructions);
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
        let type_id = *self
            .context
            .get_type_binding(&node.id)
            .ok_or(CompileError::MissingTypeBinding { syntax_id: node.id })?;
        let operand_type =
            self.context
                .get_operand_type(type_id)
                .ok_or(CompileError::CannotInferType {
                    position: Position::new(self.file_id, node.span()),
                })?;
        let address = match emission {
            Emission::Constant(constant) => self.get_constant_address(constant),
            Emission::Target(target) => target.address(),
            Emission::Instructions(instructions) => {
                if let Some(target) = instructions.target {
                    return_instructions.merge(instructions);

                    Address::register(target.index())
                } else if type_id == TypeId::NONE {
                    return_instructions.merge(instructions);

                    Address::default()
                } else {
                    return Err(CompileError::ExpectedExpression {
                        node_kind: node.kind(),
                        position: Position::new(self.file_id, node.span()),
                    });
                }
            }
            Emission::None => Address::default(),
        };
        let return_instruction = Instruction::r#return(address, operand_type);

        return_instructions.push(return_instruction);

        Ok(())
    }

    fn handle_implicit_return(
        &mut self,
        node: SyntaxReader,
        input: Option<Target>,
    ) -> Result<Emission, CompileError> {
        let mut return_emission = InstructionsEmission::new();
        let emission = self.visit(node, input)?;

        if node.kind().is_item() || node.kind().is_statement() {
            if let Emission::Instructions(instructions) = emission {
                return_emission.merge(instructions);
            }

            let function_type_node = *self.context.get_type_mut(self.function_type_id).ok_or(
                CompileError::CannotInferType {
                    position: Position::new(self.file_id, node.span()),
                },
            )?;

            if let TypeNode::Function { return_type_id, .. } = function_type_node {
                self.context.unify_types(return_type_id, TypeId::NONE)?;
            } else {
                return Err(CompileError::ExpectedFunctionType {
                    type_id: self.function_type_id,
                });
            }

            let return_instruction = Instruction::r#return(Address::default(), OperandType::NONE);

            return_emission.push(return_instruction);
        } else {
            self.handle_return_emission(&mut return_emission, emission, node)?;
        }

        Ok(Emission::Instructions(return_emission))
    }
}

impl<'a> SyntaxVisitor for Emitter<'a> {
    type Input = Option<Target>;

    type Output = Emission;

    fn file_id(&self) -> SourceFileId {
        self.file_id
    }

    fn visit_main_function_item(
        &mut self,
        node: SyntaxReader,
        target: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Emitting main function item");

        let children = node
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.kind(),
                start_index: node.inner().children.0,
                count: node.inner().children.1,
            })?;
        let last_child = children.len() - 1;
        let mut final_emission = Emission::None;

        for (index, child) in children.into_iter().enumerate() {
            let child_emission = self.visit(child, target)?;

            if index == last_child {
                final_emission = self.handle_implicit_return(child, target)?;
            } else {
                self.handle_top_emission(child_emission, child)?;
            }
        }

        Ok(final_emission)
    }

    fn visit_module_item(
        &mut self,
        _: SyntaxReader<'_>,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        todo!()
    }

    fn visit_function_item(
        &mut self,
        node: SyntaxReader<'_>,
        _target: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Emitting function item");

        let function_expression = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.inner().kind,
            child_index: 1,
        })?;

        let emission = self.visit_function_expression(function_expression, None)?;

        if let Emission::Target(target) = emission {
            let declaration_id = *self
                .context
                .get_declaration_binding(&function_expression.id)
                .ok_or(CompileError::MissingDeclarationBinding {
                    syntax_id: function_expression.id,
                })?;

            self.locals.insert(declaration_id, target);
        }

        Ok(Emission::None)
    }

    fn visit_use_item(
        &mut self,
        _: SyntaxReader<'_>,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        todo!()
    }

    fn visit_struct_item(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        Ok(Emission::None)
    }

    fn visit_expression_statement(
        &mut self,
        node: SyntaxReader<'_>,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Emitting expression statement");

        let child = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;

        let emission = self.visit_expression(child, input)?;

        if input.is_some() {
            return Ok(emission);
        }

        match emission {
            Emission::Instructions(mut instructions) => {
                instructions.set_target(None);

                Ok(Emission::Instructions(instructions))
            }
            _ => Ok(Emission::None),
        }
    }

    fn visit_let_statement(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Emitting let statement");

        let mut children = node
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.kind(),
                start_index: node.inner().children.0,
                count: node.inner().children.1,
            })?;
        let path = children.next().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;
        let expression_statement = children.next().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 1,
        })?;
        let expression = expression_statement
            .left_child()
            .ok_or(CompileError::MissingChild {
                parent_kind: expression_statement.kind(),
                child_index: 0,
            })?;

        let mut let_statement_emission = InstructionsEmission::new();
        let target = self.allocate_local_register();
        let expression_emission = self.visit_expression(expression, Some(target))?;
        let type_id = *self
            .context
            .get_type_binding(&expression.id)
            .ok_or(CompileError::MissingTypeBinding { syntax_id: node.id })?;

        match expression_emission {
            Emission::Constant(constant) => {
                let address = self.get_constant_address(constant);
                let operand_type = constant.operand_type();
                let move_instruction = Instruction::r#move(target.index(), address, operand_type);

                let_statement_emission.push(move_instruction);
            }
            Emission::Target(expression_target) => {
                let operand_type = self
                    .context
                    .get_operand_type(type_id)
                    .ok_or(CompileError::MissingType { type_id })?;
                let move_instruction =
                    Instruction::r#move(target.index(), expression_target.address(), operand_type);

                let_statement_emission.push(move_instruction);
            }
            Emission::Instructions(expression_instructions) => {
                let_statement_emission.merge(expression_instructions);
            }
            Emission::None => {
                return Err(CompileError::ExpectedExpression {
                    node_kind: expression.kind(),
                    position: Position::new(self.file_id, expression.span()),
                });
            }
        };

        let declaration_id = *self
            .context
            .get_declaration_binding(&path.id)
            .ok_or(CompileError::MissingDeclarationBinding { syntax_id: node.id })?;

        if type_id == TypeId::STRING {
            self.add_drop(target.index());
        }

        self.locals.insert(declaration_id, target);
        let_statement_emission.set_target(None);

        Ok(Emission::Instructions(let_statement_emission))
    }

    fn visit_binary_assignment_statement(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Emitting binary assignment statement");

        let mut emission = self.visit_math_binary_expression(node, input)?;

        if let Emission::Instructions(instructions_emission) = &mut emission {
            instructions_emission.set_target(None);
        }

        Ok(emission)
    }

    fn visit_reassignment_statement(
        &mut self,
        node: SyntaxReader<'_>,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Emitting reassignment statement");

        let path = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;
        let expression_statement = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 1,
        })?;
        let expression = expression_statement
            .left_child()
            .ok_or(CompileError::MissingChild {
                parent_kind: expression_statement.kind(),
                child_index: 0,
            })?;

        let declaration_id = self
            .context
            .get_declaration_binding(&path.id)
            .ok_or(CompileError::MissingDeclarationBinding { syntax_id: path.id })?;
        let target = *self
            .locals
            .get(declaration_id)
            .ok_or(CompileError::MissingLocal {
                declaration_id: *declaration_id,
            })?;

        let mut reassignment_emission = InstructionsEmission::new();
        let expression_emission = self.visit_expression(expression, Some(target))?;

        match expression_emission {
            Emission::Constant(constant) => {
                let address = self.get_constant_address(constant);
                let operand_type = constant.operand_type();
                let move_instruction = Instruction::r#move(target.index(), address, operand_type);

                reassignment_emission.push(move_instruction);
            }
            Emission::Target(expression_target) => {
                let type_id = *self
                    .context
                    .get_type_binding(&node.id)
                    .ok_or(CompileError::MissingTypeBinding { syntax_id: node.id })?;
                let operand_type = self.context.get_operand_type(type_id).ok_or(
                    CompileError::CannotInferType {
                        position: Position::new(self.file_id, expression.span()),
                    },
                )?;
                let move_instruction =
                    Instruction::r#move(target.index(), expression_target.address(), operand_type);

                reassignment_emission.push(move_instruction);
            }
            Emission::Instructions(instructions_emission) => {
                reassignment_emission.merge(instructions_emission);
                reassignment_emission.set_target(None);
            }
            Emission::None => {
                return Err(CompileError::ExpectedExpression {
                    node_kind: expression.kind(),
                    position: Position::new(self.file_id, expression.span()),
                });
            }
        }

        Ok(Emission::Instructions(reassignment_emission))
    }

    fn visit_boolean_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Emitting boolean expression");

        Ok(Emission::Constant(Constant::Boolean(
            node.inner().children.0 != 0,
        )))
    }

    fn visit_byte_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Emitting byte expression");

        Ok(Emission::Constant(Constant::Byte(
            node.inner().children.0 as u8,
        )))
    }

    fn visit_character_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Emitting character expression");

        Ok(Emission::Constant(Constant::Character(
            node.inner().decode_character(),
        )))
    }

    fn visit_float_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Emitting float expression");

        Ok(Emission::Constant(Constant::Float(
            node.inner().decode_float(),
        )))
    }

    fn visit_integer_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Emitting integer expression");

        Ok(Emission::Constant(Constant::Integer(
            node.inner().decode_integer(),
        )))
    }

    fn visit_string_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Emitting string expression");

        let span_without_quotes = node.span().shrink(1);
        let bytes = self
            .source
            .get_file(self.file_id)
            .ok_or(CompileError::MissingSourceFile {
                file_id: self.file_id,
            })?
            .source_code
            .get_span_bytes(span_without_quotes);
        let (pool_start, pool_end) = self.context.constants.push_str_to_string_pool(bytes);

        self.context.set_type_binding(node.id, TypeId::STRING);

        Ok(Emission::Constant(Constant::String {
            pool_start,
            pool_end,
        }))
    }

    fn visit_list_expression(
        &mut self,
        node: SyntaxReader,
        target: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        fn handle_element_emission(
            emitter: &mut Emitter,
            instructions: &mut InstructionsEmission,
            element_emission: Emission,
            element_node: &SyntaxNode,
        ) -> Result<Address, CompileError> {
            match element_emission {
                Emission::Constant(constant) => Ok(emitter.get_constant_address(constant)),
                Emission::Target(target) => Ok(target.address()),
                Emission::Instructions(InstructionsEmission {
                    instructions: element_instructions,
                    target,
                    ..
                }) => {
                    let target = target.ok_or(CompileError::ExpectedExpression {
                        node_kind: element_node.kind,
                        position: Position::new(emitter.file_id, element_node.span),
                    })?;

                    instructions.instructions.extend(element_instructions);

                    Ok(Address::register(target.index()))
                }
                Emission::None => Err(CompileError::ExpectedExpression {
                    node_kind: element_node.kind,
                    position: Position::new(emitter.file_id, element_node.span),
                }),
            }
        }
        debug!("Emitting list expression");

        let children = node
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.kind(),
                start_index: node.inner().children.0,
                count: node.inner().children.1,
            })?;
        let child_count_address =
            self.get_constant_address(Constant::Integer(children.len() as i64));

        let target = target.unwrap_or_else(|| self.allocate_temporary_register());
        let mut list_emission = {
            let mut emission = InstructionsEmission::with_capacity(children.len());

            emission.push(Instruction::no_op()); // Placeholder for NEW_LIST

            emission
        };
        let mut operand_type = None;

        for (index, child) in children.enumerate() {
            let element_emission = self.visit_expression(child, None)?;
            let element_address =
                handle_element_emission(self, &mut list_emission, element_emission, child.inner())?;
            let index_address = self.get_constant_address(Constant::Integer(index as i64));
            let operand_type = if let Some(operand_type) = operand_type {
                operand_type
            } else {
                let type_id = *self.context.get_type_binding(&child.id).ok_or(
                    CompileError::MissingTypeBinding {
                        syntax_id: child.id,
                    },
                )?;
                let element_operand_type = self
                    .context
                    .get_operand_type(type_id)
                    .ok_or(CompileError::MissingType { type_id })?;

                operand_type = Some(element_operand_type);

                element_operand_type
            };
            let set_list_instruction =
                Instruction::set_list(target.index(), element_address, index_address, operand_type);

            list_emission.push(set_list_instruction);
        }

        let list_type = *self
            .context
            .get_type_binding(&node.id)
            .ok_or(CompileError::MissingTypeBinding { syntax_id: node.id })?;
        let operand_type = self
            .context
            .get_operand_type(list_type)
            .ok_or(CompileError::MissingType { type_id: list_type })?;
        let new_list_instruction =
            Instruction::new_list(target.index(), child_count_address, operand_type);

        list_emission.instructions[0] = (new_list_instruction, Vec::new());

        list_emission.set_target(Some(target));

        Ok(Emission::Instructions(list_emission))
    }

    fn visit_index_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Emitting index expression");

        let left_child = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;
        let right_child = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 1,
        })?;

        let left_emission = self.visit_expression(left_child, None)?;
        let right_emission = self.visit_expression(right_child, None)?;

        let mut index_emission = InstructionsEmission::new();

        let list_address =
            self.handle_operand_emission(&mut index_emission, left_emission, &left_child)?;
        let index_address =
            self.handle_operand_emission(&mut index_emission, right_emission, &right_child)?;

        let list_type_id = *self.context.get_type_binding(&left_child.id).ok_or(
            CompileError::MissingTypeBinding {
                syntax_id: left_child.id,
            },
        )?;

        let target = input.unwrap_or_else(|| self.allocate_temporary_register());
        let element_type_id =
            *self
                .context
                .get_type_binding(&node.id)
                .ok_or(CompileError::MissingType {
                    type_id: list_type_id,
                })?;
        let operand_type =
            self.context
                .get_operand_type(element_type_id)
                .ok_or(CompileError::MissingType {
                    type_id: list_type_id,
                })?;
        let get_list_instruction =
            Instruction::get_list(target.index(), list_address, index_address, operand_type);

        index_emission.push(get_list_instruction);
        index_emission.set_target(Some(target));

        Ok(Emission::Instructions(index_emission))
    }

    fn visit_path_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Emitting path expression");

        let declaration_id = self
            .context
            .get_declaration_binding(&node.id)
            .ok_or(CompileError::MissingDeclarationBinding { syntax_id: node.id })?;
        let local = *self
            .locals
            .get(declaration_id)
            .ok_or(CompileError::MissingLocal {
                declaration_id: *declaration_id,
            })?;

        Ok(Emission::Target(local))
    }

    fn visit_struct_expression(
        &mut self,
        node: SyntaxReader,
        target: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Emitting struct expression");

        let fields = node
            .right_child()
            .ok_or(CompileError::MissingChild {
                parent_kind: node.kind(),
                child_index: 1,
            })?
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.kind(),
                start_index: node.inner().children.0,
                count: node.inner().children.1,
            })?;

        let base_target = target.unwrap_or_else(|| self.allocate_temporary_register());
        let field_count = fields.len() as u16;
        let struct_target = Target::Struct {
            base_index: base_target.index(),
            field_count,
            is_temporary: base_target.is_temporary(),
        };
        let mut field_targets = vec![0; fields.len()];

        if struct_target.is_temporary() {
            field_targets.fill_with(|| self.allocate_temporary_register().index());
        } else {
            field_targets.fill_with(|| self.allocate_local_register().index());
        }

        let mut struct_emission = InstructionsEmission::new();
        let mut base_register = None;

        for (field, destination) in fields.into_iter().zip(field_targets.into_iter()) {
            let field_expression = field.right_child().ok_or(CompileError::MissingChild {
                parent_kind: field.kind(),
                child_index: 1,
            })?;
            let field_emission = self.visit_expression(field_expression, None)?;
            let field_address = self.handle_operand_emission(
                &mut struct_emission,
                field_emission,
                &field_expression,
            )?;
            let field_type_id = *self.context.get_type_binding(&field_expression.id).ok_or(
                CompileError::MissingTypeBinding {
                    syntax_id: field_expression.id,
                },
            )?;
            let operand_type =
                self.context
                    .get_operand_type(field_type_id)
                    .ok_or(CompileError::MissingType {
                        type_id: field_type_id,
                    })?;
            let field_move_instruction =
                Instruction::r#move(destination, field_address, operand_type);

            struct_emission.push(field_move_instruction);

            if base_register.is_none() {
                base_register = Some(destination);
            }
        }

        let struct_reference_instruction =
            Instruction::reference(base_target.index(), base_register.unwrap_or(0), field_count);

        struct_emission.push(struct_reference_instruction);
        struct_emission.set_target(Some(struct_target));

        Ok(Emission::Instructions(struct_emission))
    }

    fn visit_block_expression(
        &mut self,
        node: SyntaxReader<'_>,
        target: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Emitting block expression");

        let children = node
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.kind(),
                start_index: node.inner().children.0,
                count: node.inner().children.1,
            })?;

        let block_scope_id = *self
            .context
            .get_scope_binding(&node.id)
            .ok_or(CompileError::MissingScopeBinding { syntax_id: node.id })?;
        let parent_scope_id = self.current_scope_id;
        let parent_scope_next_local_register = self.next_local_register;

        self.enter_child_scope(block_scope_id);

        let child_count = children.len();
        let mut block_emission = InstructionsEmission::new();

        for (index, child) in children.into_iter().enumerate() {
            let is_last = index == child_count - 1;
            let child_emission = if is_last {
                self.visit(child, target)?
            } else {
                self.visit(child, None)?
            };
            let child_target = child_emission.target();

            if is_last {
                match child_emission {
                    Emission::Constant(constant) => {
                        if block_emission.is_empty() {
                            self.enter_parent_scope(
                                parent_scope_id,
                                parent_scope_next_local_register,
                            );
                            block_emission.add_drop(self, child_target);

                            return Ok(Emission::Constant(constant));
                        }

                        let target = target.unwrap_or_else(|| self.allocate_temporary_register());
                        let address = self.get_constant_address(constant);
                        let operand_type = constant.operand_type();
                        let move_instruction =
                            Instruction::r#move(target.index(), address, operand_type);

                        block_emission.push(move_instruction);
                        block_emission.set_target(Some(target));
                    }
                    Emission::Target(final_target) => {
                        if block_emission.is_empty() {
                            self.enter_parent_scope(
                                parent_scope_id,
                                parent_scope_next_local_register,
                            );
                            block_emission.add_drop(self, child_target);

                            return Ok(Emission::Target(final_target));
                        }

                        if let Some(block_target) = target {
                            let type_id = *self
                                .context
                                .get_type_binding(&node.id)
                                .ok_or(CompileError::MissingTypeBinding { syntax_id: node.id })?;
                            let operand_type = self
                                .context
                                .get_operand_type(type_id)
                                .ok_or(CompileError::MissingType { type_id })?;
                            let move_instruction = Instruction::r#move(
                                block_target.index(),
                                final_target.address(),
                                operand_type,
                            );

                            block_emission.push(move_instruction);
                            block_emission.set_target(Some(block_target));
                        } else if final_target.address().memory == MemoryKind::REGISTER {
                            block_emission.set_target(Some(final_target));
                        } else {
                            let target = self.allocate_temporary_register();
                            let type_id = *self
                                .context
                                .get_type_binding(&node.id)
                                .ok_or(CompileError::MissingTypeBinding { syntax_id: node.id })?;
                            let operand_type = self
                                .context
                                .get_operand_type(type_id)
                                .ok_or(CompileError::MissingType { type_id })?;
                            let move_instruction = Instruction::r#move(
                                target.index(),
                                final_target.address(),
                                operand_type,
                            );

                            block_emission.push(move_instruction);
                            block_emission.set_target(Some(target));
                        }
                    }
                    Emission::Instructions(instructions) => {
                        block_emission.merge(instructions);
                    }
                    Emission::None => {}
                }
            } else if let Emission::Instructions(child_instructions) = child_emission {
                block_emission.merge(child_instructions);
            }
        }

        self.enter_parent_scope(parent_scope_id, parent_scope_next_local_register);
        block_emission.add_drop(self, block_emission.target);

        Ok(Emission::Instructions(block_emission))
    }

    fn visit_if_expression(
        &mut self,
        node: SyntaxReader<'_>,
        target: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Emitting if expression");

        let mut children = node
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.kind(),
                start_index: node.inner().children.0,
                count: node.inner().children.1,
            })?;

        let mut if_emission = InstructionsEmission::new();

        let condition = children.next().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;
        let condition_emission = self.visit_expression(condition, None)?;

        self.handle_condition_emission(&mut if_emission, condition_emission, condition.inner())?;

        let target = target.unwrap_or_else(|| self.allocate_temporary_register());
        let jump_over_then_id = self.create_jump_id();
        let start_else_anchor_count = self.jump_over_else_anchor_ids.len();

        if_emission.push_drop_anchor(JumpAnchor::ForwardFromHere {
            id: jump_over_then_id,
        });

        let then_expression = children.next().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 1,
        })?;
        let then_emission = self.visit_expression(then_expression, Some(target))?;

        self.handle_branch_emission(
            &mut if_emission,
            then_emission,
            target.index(),
            then_expression,
        )?;

        if_emission.push_drop_anchor(JumpAnchor::ForwardToNext {
            id: jump_over_then_id,
        });

        let else_expression = children.next();

        if let Some(else_expression) = else_expression {
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
        target: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Emitting else expression");

        let child = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;

        self.visit_expression(child, target)
    }

    fn visit_math_binary_expression(
        &mut self,
        node: SyntaxReader,
        target: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Emitting math binary expression");

        let left_child = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;
        let right_child = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 1,
        })?;

        let left_emission = self.visit_expression(left_child, None)?;
        let right_emission = self.visit_expression(right_child, None)?;

        if target.is_none()
            && let (Emission::Constant(left_value), Emission::Constant(right_value)) =
                (&left_emission, &right_emission)
        {
            let combined = self.combine_constants(
                node.kind(),
                *left_value,
                left_child.inner(),
                *right_value,
                right_child.inner(),
            )?;

            return Ok(Emission::Constant(combined));
        }

        let mut math_emission = InstructionsEmission::new();

        let left_target = left_emission.target();
        let left_address =
            self.handle_operand_emission(&mut math_emission, left_emission, &left_child)?;
        let right_address =
            self.handle_operand_emission(&mut math_emission, right_emission, &right_child)?;

        let left_type = *self.context.get_type_binding(&left_child.id).ok_or(
            CompileError::MissingTypeBinding {
                syntax_id: left_child.id,
            },
        )?;
        let right_type = *self.context.get_type_binding(&right_child.id).ok_or(
            CompileError::MissingTypeBinding {
                syntax_id: right_child.id,
            },
        )?;
        let math_expression_type = *self
            .context
            .get_type_binding(&node.id)
            .ok_or(CompileError::MissingTypeBinding { syntax_id: node.id })?;
        let operand_type = match (left_type, right_type) {
            (TypeId::STRING, TypeId::CHARACTER) => OperandType::STRING_CHARACTER,
            (TypeId::CHARACTER, TypeId::STRING) => OperandType::CHARACTER_STRING,
            (TypeId::CHARACTER, TypeId::CHARACTER) => OperandType::CHARACTER,
            _ if math_expression_type == TypeId::NONE => {
                self.context
                    .get_operand_type(left_type)
                    .ok_or(CompileError::MissingType { type_id: left_type })?
            }
            _ => self.context.get_operand_type(math_expression_type).ok_or(
                CompileError::MissingType {
                    type_id: math_expression_type,
                },
            )?,
        };

        let math_instruction = match node.kind() {
            SyntaxKind::AdditionExpression => {
                let target = target.unwrap_or_else(|| self.allocate_temporary_register());

                math_emission.set_target(Some(target));

                if (matches!(
                    operand_type,
                    OperandType::STRING
                        | OperandType::CHARACTER_STRING
                        | OperandType::STRING_CHARACTER
                ) || (operand_type == OperandType::CHARACTER
                    && math_expression_type == TypeId::STRING))
                    && target.is_temporary()
                {
                    self.pending_drops.last_mut().unwrap().push(target.index());
                }

                Instruction::add(target.index(), left_address, right_address, operand_type)
            }
            SyntaxKind::AdditionAssignmentStatement => {
                math_emission.set_target(left_target);

                Instruction::add(
                    left_address.index,
                    left_address,
                    right_address,
                    operand_type,
                )
            }
            SyntaxKind::SubtractionExpression => {
                let target = target.unwrap_or_else(|| self.allocate_temporary_register());

                math_emission.set_target(Some(target));

                Instruction::subtract(target.index(), left_address, right_address, operand_type)
            }
            SyntaxKind::SubtractionAssignmentStatement => {
                math_emission.set_target(left_target);

                Instruction::subtract(
                    left_address.index,
                    left_address,
                    right_address,
                    operand_type,
                )
            }
            SyntaxKind::MultiplicationExpression => {
                let target = target.unwrap_or_else(|| self.allocate_temporary_register());

                math_emission.set_target(Some(target));

                Instruction::multiply(target.index(), left_address, right_address, operand_type)
            }
            SyntaxKind::MultiplicationAssignmentStatement => {
                math_emission.set_target(left_target);

                Instruction::multiply(
                    left_address.index,
                    left_address,
                    right_address,
                    operand_type,
                )
            }
            SyntaxKind::DivisionExpression => {
                let target = target.unwrap_or_else(|| self.allocate_temporary_register());

                math_emission.set_target(Some(target));

                Instruction::divide(target.index(), left_address, right_address, operand_type)
            }
            SyntaxKind::DivisionAssignmentStatement => {
                math_emission.set_target(left_target);

                Instruction::divide(
                    left_address.index,
                    left_address,
                    right_address,
                    operand_type,
                )
            }
            SyntaxKind::ModuloExpression => {
                let target = target.unwrap_or_else(|| self.allocate_temporary_register());

                math_emission.set_target(Some(target));

                Instruction::modulo(target.index(), left_address, right_address, operand_type)
            }
            SyntaxKind::ModuloAssignmentStatement => {
                math_emission.set_target(left_target);

                Instruction::modulo(
                    left_address.index,
                    left_address,
                    right_address,
                    operand_type,
                )
            }
            SyntaxKind::ExponentExpression => {
                let target = target.unwrap_or_else(|| self.allocate_temporary_register());

                math_emission.set_target(Some(target));

                Instruction::power(target.index(), left_address, right_address, operand_type)
            }
            SyntaxKind::ExponentAssignmentStatement => {
                math_emission.set_target(left_target);

                Instruction::power(
                    left_address.index,
                    left_address,
                    right_address,
                    operand_type,
                )
            }
            _ => unreachable!("Expected binary expression, found {}", node.kind()),
        };

        math_emission.push(math_instruction);

        Ok(Emission::Instructions(math_emission))
    }

    fn visit_comparison_binary_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Emitting comparison binary expression");

        let left_child = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;
        let right_child = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 1,
        })?;

        let left_emission = self.visit_expression(left_child, None)?;
        let right_emission = self.visit_expression(right_child, None)?;

        if let Emission::Constant(left_constant) = left_emission
            && let Emission::Constant(right_constant) = right_emission
        {
            let combined = self.combine_constants(
                node.kind(),
                left_constant,
                left_child.inner(),
                right_constant,
                right_child.inner(),
            )?;

            return Ok(Emission::Constant(combined));
        }

        let mut comparison_emission = InstructionsEmission::new();

        let left_address =
            self.handle_operand_emission(&mut comparison_emission, left_emission, &left_child)?;
        let right_address =
            self.handle_operand_emission(&mut comparison_emission, right_emission, &right_child)?;

        let target = input.unwrap_or_else(|| self.allocate_temporary_register());

        let type_id = *self
            .context
            .get_type_binding(&left_child.id)
            .ok_or(CompileError::MissingTypeBinding { syntax_id: node.id })?;
        let operand_type = self
            .context
            .get_operand_type(type_id)
            .ok_or(CompileError::MissingType { type_id })?;

        let comparison_instruction = match node.kind() {
            SyntaxKind::EqualExpression => {
                Instruction::equal(true, left_address, right_address, operand_type)
            }
            SyntaxKind::NotEqualExpression => {
                Instruction::equal(false, left_address, right_address, operand_type)
            }
            SyntaxKind::LessThanExpression => {
                Instruction::less(true, left_address, right_address, operand_type)
            }
            SyntaxKind::GreaterThanExpression => {
                Instruction::less_equal(false, left_address, right_address, operand_type)
            }
            SyntaxKind::LessThanOrEqualExpression => {
                Instruction::less_equal(true, left_address, right_address, operand_type)
            }
            SyntaxKind::GreaterThanOrEqualExpression => {
                Instruction::less(false, left_address, right_address, operand_type)
            }
            _ => unreachable!("Expected comparison expression, found {}", node.kind()),
        };
        let load_false_instruction = Instruction::move_with_jump(
            target.index(),
            Address::encoded(false as u16),
            OperandType::BOOLEAN,
            1,
            true,
        );
        let load_true_instruction = Instruction::r#move(
            target.index(),
            Address::encoded(true as u16),
            OperandType::BOOLEAN,
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
        target: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Emitting logical binary expression");

        let left_child = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;
        let right_child = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 1,
        })?;

        let left_emission = self.visit_expression(left_child, None)?;
        let right_emission = self.visit_expression(right_child, None)?;

        if let Emission::Constant(left_constant) = left_emission
            && let Emission::Constant(right_constant) = right_emission
        {
            let combined = self.combine_constants(
                node.kind(),
                left_constant,
                left_child.inner(),
                right_constant,
                right_child.inner(),
            )?;

            return Ok(Emission::Constant(combined));
        }

        let mut logical_emission = InstructionsEmission::new();

        let left_address =
            self.handle_operand_emission(&mut logical_emission, left_emission, &left_child)?;
        let right_address =
            self.handle_operand_emission(&mut logical_emission, right_emission, &right_child)?;

        let target = target.unwrap_or_else(|| self.allocate_temporary_register());

        let test_instruction = match node.kind() {
            SyntaxKind::AndExpression => Instruction::test(left_address, false, 1),
            SyntaxKind::OrExpression => Instruction::test(left_address, true, 1),
            _ => unreachable!("Expected logical expression, found {}", node.kind()),
        };
        let right_move_instruction = Instruction::move_with_jump(
            target.index(),
            right_address,
            OperandType::BOOLEAN,
            1,
            true,
        );
        let left_move_instruction =
            Instruction::r#move(target.index(), left_address, OperandType::BOOLEAN);

        logical_emission.push(test_instruction);
        logical_emission.push(right_move_instruction);
        logical_emission.push(left_move_instruction);
        logical_emission.set_target(Some(target));

        Ok(Emission::Instructions(logical_emission))
    }

    fn visit_unary_negation_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Emitting unary negation expression");

        let child = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;

        let child_emission = self.visit_expression(child, None)?;

        if let Emission::Constant(constant) = child_emission {
            let negated = match constant {
                Constant::Boolean(boolean) => Constant::Boolean(!boolean),
                Constant::Byte(byte) => Constant::Byte(!byte),
                Constant::Integer(integer) => Constant::Integer(-integer),
                Constant::Float(float) => Constant::Float(-float),
                _ => unreachable!(
                    "Expected constant suitable for negation, found {:?}",
                    constant
                ),
            };

            return Ok(Emission::Constant(negated));
        }

        let mut negation_emission = InstructionsEmission::new();

        let child_address =
            self.handle_operand_emission(&mut negation_emission, child_emission, &child)?;
        let target = input.unwrap_or_else(|| self.allocate_temporary_register());
        let operand_type = match node.kind() {
            SyntaxKind::NegationExpression => {
                let type_id = *self
                    .context
                    .get_type_binding(&node.id)
                    .ok_or(CompileError::MissingTypeBinding { syntax_id: node.id })?;
                self.context
                    .get_operand_type(type_id)
                    .ok_or(CompileError::MissingType { type_id })?
            }
            SyntaxKind::NotExpression => OperandType::BOOLEAN,
            _ => unreachable!("Expected unary negation expression, found {}", node.kind()),
        };

        let negation_instruction = Instruction::negate(target.index(), child_address, operand_type);

        negation_emission.push(negation_instruction);
        negation_emission.set_target(Some(target));

        Ok(Emission::Instructions(negation_emission))
    }

    fn visit_while_expression(
        &mut self,
        node: SyntaxReader<'_>,
        _target: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Emitting while expression");

        let condition = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;
        let body = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 1,
        })?;

        let mut while_emission = InstructionsEmission::new();
        let condition_emission = self.visit_expression(condition, None)?;

        self.handle_condition_emission(&mut while_emission, condition_emission, condition.inner())?;

        let jump_forward_id = self.create_jump_id();
        let jump_backward_id = self.create_jump_id();

        while_emission.push_drop_anchor(JumpAnchor::LoopStartHere {
            forward_id: jump_forward_id,
        });

        let body_emission = self.visit(body, None)?;

        match body_emission {
            Emission::Target(operand) => {
                let destination = self.allocate_temporary_register();
                let type_id = *self
                    .context
                    .get_type_binding(&body.id)
                    .ok_or(CompileError::MissingTypeBinding { syntax_id: body.id })?;
                let operand_type = self
                    .context
                    .get_operand_type(type_id)
                    .ok_or(CompileError::MissingType { type_id })?;
                let move_instruction =
                    Instruction::r#move(destination.index(), operand.address(), operand_type);

                while_emission.push(move_instruction);
            }
            Emission::Constant(constant) => {
                let destination = self.allocate_temporary_register();
                let address = self.get_constant_address(constant);
                let operand_type = constant.operand_type();
                let move_instruction =
                    Instruction::r#move(destination.index(), address, operand_type);

                while_emission.push(move_instruction);
            }
            Emission::Instructions(InstructionsEmission { instructions, .. }) => {
                while_emission.instructions.extend(instructions);
            }
            Emission::None => {}
        }

        while_emission.push_drop_anchor(JumpAnchor::LoopEndOnNext {
            forward_id: jump_forward_id,
            backward_id: jump_backward_id,
        });

        Ok(Emission::Instructions(while_emission))
    }

    fn visit_function_expression(
        &mut self,
        node: SyntaxReader<'_>,
        _target: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Emitting function expression");

        let declaration_id = self.context.get_declaration_binding(&node.id).copied();
        let declaration_info = if let Some(declaration_id) = declaration_id {
            let declaration = *self
                .context
                .get_declaration(declaration_id)
                .ok_or(CompileError::MissingDeclaration { declaration_id })?;

            Some((declaration_id, declaration))
        } else {
            None
        };

        let function_type = *self
            .context
            .get_type_binding(&node.id)
            .ok_or(CompileError::MissingTypeBinding { syntax_id: node.id })?;
        let body = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;

        let prototype_index = self.context.prototypes.len();

        if let Some(declaration_id) = declaration_id
            && let Some(declaration) = self.context.get_declaration_mut(&declaration_id)
            && let DeclarationKind::Function {
                prototype_index: declaration_prototype_index,
                ..
            } = &mut declaration.kind
        {
            *declaration_prototype_index = Some(prototype_index as u16);
        }

        self.context.prototypes.push(Prototype::default());

        let function_scope_id = *self
            .context
            .get_scope_binding(&body.id)
            .ok_or(CompileError::MissingScopeBinding { syntax_id: node.id })?;

        let function_emitter = Emitter::new(
            declaration_info,
            prototype_index as u16,
            self.file_id,
            function_type,
            self.source,
            self.syntax,
            self.context,
            function_scope_id,
        );

        self.context.prototypes[prototype_index] = function_emitter.emit(body)?;

        Ok(Emission::Target(Target::Constant {
            index: prototype_index as u16,
        }))
    }

    fn visit_call_expression(
        &mut self,
        node: SyntaxReader<'_>,
        target: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        debug!("Emitting call expression");

        let callee = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 0,
        })?;
        let arguments = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.kind(),
            child_index: 1,
        })?;

        let mut call_emission = InstructionsEmission::new();

        let callee_emission = self.visit_expression(callee, None)?;
        let callee_address =
            self.handle_operand_emission(&mut call_emission, callee_emission, &callee)?;

        let arguments_start = self.call_arguments.len() as u16;
        let mut argument_count = 0u16;

        if let Some(argument_nodes) = arguments.multiple_children() {
            for argument in argument_nodes {
                let argument_emission = self.visit_expression(argument, None)?;
                let argument_address =
                    self.handle_operand_emission(&mut call_emission, argument_emission, &argument)?;
                let argument_type_id = *self.context.get_type_binding(&argument.id).ok_or(
                    CompileError::MissingTypeBinding {
                        syntax_id: argument.id,
                    },
                )?;
                let argument_operand_type = self.context.get_operand_type(argument_type_id).ok_or(
                    CompileError::MissingType {
                        type_id: argument_type_id,
                    },
                )?;

                self.call_arguments
                    .push((argument_address, argument_operand_type));
                argument_count += 1;
            }
        }

        let return_type_id = *self
            .context
            .get_type_binding(&node.id)
            .ok_or(CompileError::MissingTypeBinding { syntax_id: node.id })?;
        let return_operand_type =
            self.context
                .get_operand_type(return_type_id)
                .ok_or(CompileError::MissingType {
                    type_id: return_type_id,
                })?;

        let destination = if return_operand_type == OperandType::NONE {
            None
        } else {
            Some(target.unwrap_or_else(|| self.allocate_temporary_register()))
        };

        let is_native = self
            .context
            .get_declaration_binding(&callee.id)
            .and_then(|id| self.context.get_declaration(*id))
            .is_some_and(|declaration| matches!(declaration.kind, DeclarationKind::NativeFunction));

        let call_instruction = if is_native {
            let source_file = self.source.files().get(self.file_id.0 as usize).ok_or(
                CompileError::MissingSourceFile {
                    file_id: self.file_id,
                },
            )?;
            let function_name = source_file.source_code.get_span(callee.span());
            let native_function = NativeFunction::from_str(function_name).ok_or(
                CompileError::InvalidNativeFunction {
                    name: function_name.to_string(),
                    position: Position::new(self.file_id, callee.span()),
                },
            )?;
            let destination_register = destination.map(|target| target.index()).unwrap_or(u16::MAX);

            Instruction::call_native(
                destination_register,
                native_function,
                arguments_start,
                return_operand_type,
            )
        } else {
            Instruction::call(
                destination.map(|target| target.index()),
                callee_address,
                arguments_start,
                argument_count,
            )
        };

        call_emission.push(call_instruction);

        if return_operand_type != OperandType::NONE {
            call_emission.set_target(destination);
        }

        Ok(Emission::Instructions(call_emission))
    }

    fn visit_type(
        &mut self,
        _: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        Ok(Emission::None)
    }
}

#[derive(Clone, Debug)]
pub enum Emission {
    Instructions(InstructionsEmission),
    Constant(Constant),
    Target(Target),
    None,
}

impl Emission {
    fn target(&self) -> Option<Target> {
        match self {
            Emission::Target(target) => Some(*target),
            Emission::Instructions(emission) => emission.target,
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct InstructionsEmission {
    instructions: Vec<(Instruction, Vec<JumpAnchor>)>,
    target: Option<Target>,
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

    fn set_target(&mut self, target: Option<Target>) {
        self.target = target;
    }

    fn push_drop_anchor(&mut self, anchor: JumpAnchor) {
        if let Some((_, anchors)) = self.instructions.last_mut() {
            anchors.push(anchor);
        }
    }

    fn add_drop(&mut self, compiler: &mut Emitter, target: Option<Target>) {
        let start = compiler.drop_lists.len() as u16;
        let mut pending_drops_for_scope = compiler.pending_drops.pop().unwrap();

        for register in pending_drops_for_scope.drain(..) {
            if let Some(target) = target
                && register == target.index()
            {
                continue;
            }

            compiler.drop_lists.push(register);
        }

        let end = compiler.drop_lists.len() as u16;

        if start == end {
            return;
        }

        if let Some((last_instruction, _)) = self.instructions.last_mut() {
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

                    self.push(drop_instruction);
                }
            }
        }
    }

    fn merge(&mut self, other: InstructionsEmission) {
        self.instructions.extend(other.instructions);
        self.target = other.target;
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Target {
    Constant {
        index: u16,
    },
    Register {
        index: u16,
        is_temporary: bool,
    },
    Struct {
        base_index: u16,
        field_count: u16,
        is_temporary: bool,
    },
}

impl Target {
    fn address(&self) -> Address {
        match self {
            Target::Constant { index } => Address::constant(*index),
            Target::Register { index, .. } => Address::register(*index),
            Target::Struct { base_index, .. } => Address::register(*base_index),
        }
    }

    fn index(&self) -> u16 {
        match self {
            Target::Constant { index } => *index,
            Target::Register { index, .. } => *index,
            Target::Struct { base_index, .. } => *base_index,
        }
    }

    fn is_temporary(&self) -> bool {
        match self {
            Target::Register { is_temporary, .. } | Target::Struct { is_temporary, .. } => {
                *is_temporary
            }
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Constant {
    Boolean(bool),
    Byte(u8),
    Character(char),
    Float(f64),
    Integer(i64),
    String { pool_start: u32, pool_end: u32 },
}

impl Constant {
    fn _type_id(&self) -> TypeId {
        match self {
            Constant::Boolean(_) => TypeId::BOOLEAN,
            Constant::Byte(_) => TypeId::BYTE,
            Constant::Character(_) => TypeId::CHARACTER,
            Constant::Float(_) => TypeId::FLOAT,
            Constant::Integer(_) => TypeId::INTEGER,
            Constant::String { .. } => TypeId::STRING,
        }
    }

    fn operand_type(&self) -> OperandType {
        match self {
            Constant::Boolean(_) => OperandType::BOOLEAN,
            Constant::Byte(_) => OperandType::BYTE,
            Constant::Character(_) => OperandType::CHARACTER,
            Constant::Float(_) => OperandType::FLOAT,
            Constant::Integer(_) => OperandType::INTEGER,
            Constant::String { .. } => OperandType::STRING,
        }
    }

    fn full_type(&self) -> Type {
        match self {
            Constant::Boolean(_) => Type::Boolean,
            Constant::Byte(_) => Type::Byte,
            Constant::Character(_) => Type::Character,
            Constant::Float(_) => Type::Float,
            Constant::Integer(_) => Type::Integer,
            Constant::String { .. } => Type::String,
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
