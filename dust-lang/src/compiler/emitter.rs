use std::collections::HashMap;

use rustc_hash::FxBuildHasher;
use smallvec::SmallVec;
use tracing::{debug, info, trace};

use crate::{
    compiler::{
        CompileContext, CompileError,
        context::{Declaration, DeclarationId, DeclarationKind, Scope, ScopeId, ScopeKind},
        type_graph::{TypeId, TypeNode},
    },
    instruction::{Address, Drop, Instruction, Move, OperandType, Operation, Test},
    native_function::NativeFunction,
    prototype::Prototype,
    source::{Position, Source, SourceFileId, Span},
    syntax::{Syntax, SyntaxId, SyntaxKind, SyntaxNode, SyntaxReader, SyntaxTree, SyntaxVisitor},
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
    locals: HashMap<DeclarationId, Local, FxBuildHasher>,

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
        let mut prototype_compiler = Self {
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
            prototype_compiler.locals.insert(
                *declaration_id,
                Local {
                    address: Address::constant(prototype_index),
                    type_id: declaration.type_id,
                },
            );

            let (start, count) = *parameters;

            prototype_compiler.locals.reserve(count as usize);

            for index in 0..count {
                let current_parameter_index = start + index;
                if let Some(parameter_id) = prototype_compiler
                    .context
                    .get_parameter(current_parameter_index)
                    && let Some(parameter_declaration) = prototype_compiler
                        .context
                        .get_declaration(parameter_id)
                        .copied()
                {
                    let register = prototype_compiler.allocate_local_register();
                    let parameter_local = Local {
                        address: Address::register(register.index),
                        type_id: parameter_declaration.type_id,
                    };

                    prototype_compiler
                        .locals
                        .insert(parameter_id, parameter_local);
                }
            }
        }

        prototype_compiler
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
                        start_index: node.node.children.0,
                        count: node.node.children.1,
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
            .types
            .get_full_type(self.function_type_id)
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

    fn allocate_temporary_register(&mut self) -> TargetRegister {
        let register = self.next_temporary_register;

        trace!("Allocating temporary reg_{}", register);

        self.next_temporary_register += 1;

        if self.next_temporary_register > self.maximum_register {
            self.maximum_register = self.next_temporary_register;
        }

        TargetRegister {
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

    fn allocate_local_register(&mut self) -> TargetRegister {
        let register = self.next_local_register;

        trace!("Allocating local reg_{}", register);

        self.next_local_register += 1;
        self.next_temporary_register = self.next_local_register;

        if self.next_local_register > self.maximum_register {
            self.maximum_register = self.next_local_register;
        }

        TargetRegister {
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

    fn get_operand_type(&mut self, type_id: TypeId) -> Result<OperandType, CompileError> {
        self.context
            .types
            .get_operand_type(type_id)
            .ok_or(CompileError::MissingType { type_id })
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
                return Err(CompileError::TypeMismatch {
                    expected: left.full_type(),
                    expected_position: Position::new(self.file_id, left_node.span),
                    found: right.full_type(),
                    found_position: Position::new(self.file_id, right_node.span),
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
                    .types
                    .get_operand_type(type_id)
                    .ok_or(CompileError::MissingType { type_id })?;
                let move_instruction =
                    Instruction::r#move(destination.index, address, operand_type);

                self.emit_instruction(move_instruction);
            }
            Emission::Function(function_address) => {
                let destination = self.allocate_temporary_register();
                let operand_type = self
                    .context
                    .types
                    .get_operand_type(type_id)
                    .ok_or(CompileError::MissingType { type_id })?;
                let move_instruction =
                    Instruction::r#move(destination.index, function_address, operand_type);

                self.emit_instruction(move_instruction);
            }
            Emission::Local(Local { address, type_id }) => {
                let destination = self.allocate_temporary_register();
                let operand_type = self
                    .context
                    .types
                    .get_operand_type(type_id)
                    .ok_or(CompileError::MissingType { type_id })?;
                let move_instruction =
                    Instruction::r#move(destination.index, address, operand_type);

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
            Emission::Constant(constant) => {
                let address = self.get_constant_address(constant);

                Ok(address)
            }
            Emission::Function(address) => Ok(address),
            Emission::Local(Local { address, .. }) => Ok(address),
            Emission::Instructions(operand_instructions) => {
                let destination = operand_instructions
                    .target_register
                    .ok_or(CompileError::ExpectedExpression {
                        node_kind: node.kind(),
                        position: Position::new(self.file_id, node.span()),
                    })?
                    .index;

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
            Emission::Local(Local { address, .. }) => {
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

                            if let Some(target) = condition_instructions.target_register
                                && target.is_temporary
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

                            if let Some(target) = condition_instructions.target_register
                                && target.is_temporary
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
    ) -> Result<(), CompileError> {
        match emission {
            Emission::Constant(constant) => {
                let address = self.get_constant_address(constant);
                let operand_type = constant.operand_type();
                let move_instruction =
                    Instruction::r#move(destination_register, address, operand_type);

                instructions_emission.push(move_instruction);
            }
            Emission::Function(address) => {
                let move_instruction =
                    Instruction::r#move(destination_register, address, OperandType::FUNCTION);

                instructions_emission.push(move_instruction);
            }
            Emission::Local(Local { address, type_id }) => {
                let operand_type = self.get_operand_type(type_id)?;
                let move_instruction =
                    Instruction::r#move(destination_register, address, operand_type);

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
        let operand_type = self
            .context
            .types
            .get_operand_type(type_id)
            .ok_or(CompileError::MissingType { type_id })?;
        let address = match emission {
            Emission::Constant(constant) => self.get_constant_address(constant),
            Emission::Function(address) => address,
            Emission::Local(Local { address, .. }) => address,
            Emission::Instructions(instructions) => {
                if let Some(target) = instructions.target_register {
                    return_instructions.merge(instructions);

                    Address::register(target.index)
                } else if type_id == TypeId::NONE {
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

        let function_type_node = *self
            .context
            .types
            .get_type_mut(self.function_type_id)
            .ok_or(CompileError::MissingType { type_id })?;

        if let TypeNode::Function { return_type_id, .. } = function_type_node {
            self.context.types.unify_types(return_type_id, type_id);
        } else {
            return Err(CompileError::ExpectedFunctionType {
                type_id: self.function_type_id,
            });
        }

        return_instructions.push(return_instruction);

        Ok(())
    }

    fn handle_implicit_return(
        &mut self,
        node: SyntaxReader,
        input: Option<TargetRegister>,
    ) -> Result<Emission, CompileError> {
        let mut return_emission = InstructionsEmission::new();
        let emission = self.visit(node, input)?;

        if node.kind().is_item() {
            let return_instruction = Instruction::r#return(Address::default(), OperandType::NONE);

            return_emission.push(return_instruction);
        } else {
            self.handle_return_emission(&mut return_emission, emission, node)?;
        }

        Ok(Emission::Instructions(return_emission))
    }
}

impl<'a> SyntaxVisitor for Emitter<'a> {
    type Input = Option<TargetRegister>;

    type Output = Emission;

    fn file_id(&self) -> SourceFileId {
        self.file_id
    }

    fn visit_main_function_item(
        &mut self,
        node: SyntaxReader,
        target: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        info!("Emitting main function item");

        let children = node
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.kind(),
                start_index: node.node.children.0,
                count: node.node.children.1,
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
        info!("Emitting module item");

        todo!()
    }

    fn visit_function_item(
        &mut self,
        _: SyntaxReader<'_>,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        todo!()
    }

    fn visit_use_item(
        &mut self,
        _: SyntaxReader<'_>,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        todo!()
    }

    fn visit_expression_statement(
        &mut self,
        _: SyntaxReader<'_>,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        todo!()
    }

    fn visit_let_statement(
        &mut self,
        node: SyntaxReader,
        target: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        let mut children = node
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.node.kind,
                start_index: node.node.children.0,
                count: node.node.children.1,
            })?;
        let expression_statement = children.nth(1).ok_or(CompileError::MissingChild {
            parent_kind: node.node.kind,
            child_index: 1,
        })?;
        let expression = expression_statement
            .left_child()
            .ok_or(CompileError::MissingChild {
                parent_kind: expression_statement.node.kind,
                child_index: 0,
            })?;

        let mut let_statement_emission = InstructionsEmission::new();
        let local_target = self.allocate_local_register();
        let expression_emission = self.visit_expression(expression, Some(local_target))?;

        match expression_emission {
            Emission::Constant(constant) => {
                let address = self.get_constant_address(constant);
                let operand_type = constant.operand_type();
                let move_instruction =
                    Instruction::r#move(local_target.index, address, operand_type);

                let_statement_emission.push(move_instruction);
            }
            Emission::Function(address) => {
                let move_instruction =
                    Instruction::r#move(local_target.index, address, OperandType::FUNCTION);

                let_statement_emission.push(move_instruction);
            }
            Emission::Local(Local { address, type_id }) => {
                let operand_type = self.get_operand_type(type_id)?;
                let move_instruction =
                    Instruction::r#move(local_target.index, address, operand_type);

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
            .get_declaration_binding(&node.id)
            .ok_or(CompileError::MissingDeclarationBinding { syntax_id: node.id })?;
        let expression_type = *self.context.get_type_binding(&expression.id).ok_or(
            CompileError::MissingTypeBinding {
                syntax_id: expression.id,
            },
        )?;

        if expression_type == TypeId::STRING {
            self.add_drop(local_target.index);
        }

        self.locals.insert(
            declaration_id,
            Local {
                address: Address::register(local_target.index),
                type_id: expression_type,
            },
        );
        let_statement_emission.set_target(Some(local_target));

        Ok(Emission::Instructions(let_statement_emission))
    }

    fn visit_reassignment_statement(
        &mut self,
        _: SyntaxReader<'_>,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        todo!()
    }

    fn visit_integer_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        Ok(Emission::Constant(Constant::Integer(node.decode_integer())))
    }

    fn visit_string_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
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

        self.context.add_type_binding(node.id, TypeId::STRING);

        Ok(Emission::Constant(Constant::String {
            pool_start,
            pool_end,
        }))
    }

    fn visit_path_expression(
        &mut self,
        node: SyntaxReader,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
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

        Ok(Emission::Local(local))
    }

    fn visit_block_expression(
        &mut self,
        _: SyntaxReader<'_>,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        todo!()
    }

    fn visit_if_expression(
        &mut self,
        _: SyntaxReader<'_>,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        todo!()
    }

    fn visit_math_expression(
        &mut self,
        node: SyntaxReader,
        target: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        let left_child = node.left_child().ok_or(CompileError::MissingChild {
            parent_kind: node.node.kind,
            child_index: 0,
        })?;
        let right_child = node.right_child().ok_or(CompileError::MissingChild {
            parent_kind: node.node.kind,
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
                left_child.node,
                *right_value,
                right_child.node,
            )?;

            return Ok(Emission::Constant(combined));
        }

        let mut math_emission = InstructionsEmission::new();

        let left_target = left_emission.target_register();
        let left_address =
            self.handle_operand_emission(&mut math_emission, left_emission, &left_child)?;
        let right_address =
            self.handle_operand_emission(&mut math_emission, right_emission, &right_child)?;

        let type_id = *self
            .context
            .get_type_binding(&node.id)
            .ok_or(CompileError::MissingTypeBinding { syntax_id: node.id })?;
        let operand_type = self
            .context
            .types
            .get_operand_type(type_id)
            .ok_or(CompileError::MissingType { type_id })?;

        let math_instruction = match node.kind() {
            SyntaxKind::AdditionExpression => {
                let target = target.unwrap_or_else(|| self.allocate_temporary_register());

                math_emission.set_target(Some(target));

                if type_id == TypeId::STRING && target.is_temporary {
                    self.pending_drops.last_mut().unwrap().push(target.index);
                }

                Instruction::add(target.index, left_address, right_address, operand_type)
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

                Instruction::subtract(target.index, left_address, right_address, operand_type)
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

                Instruction::multiply(target.index, left_address, right_address, operand_type)
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

                Instruction::divide(target.index, left_address, right_address, operand_type)
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

                Instruction::modulo(target.index, left_address, right_address, operand_type)
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

                Instruction::power(target.index, left_address, right_address, operand_type)
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
                Emission::Constant(constant) => {
                    let address = emitter.get_constant_address(constant);

                    Ok(address)
                }
                Emission::Function(address) => Ok(address),
                Emission::Local(Local { address, .. }) => Ok(address),
                Emission::Instructions(InstructionsEmission {
                    instructions: element_instructions,
                    target_register: target,
                    ..
                }) => {
                    let target = target.ok_or(CompileError::ExpectedExpression {
                        node_kind: element_node.kind,
                        position: Position::new(emitter.file_id, element_node.span),
                    })?;

                    instructions.instructions.extend(element_instructions);

                    Ok(Address::register(target.index))
                }
                Emission::None => Err(CompileError::ExpectedExpression {
                    node_kind: element_node.kind,
                    position: Position::new(emitter.file_id, element_node.span),
                }),
            }
        }

        let children = node
            .multiple_children()
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.node.kind,
                start_index: node.node.children.0,
                count: node.node.children.1,
            })?;
        let child_count_address =
            self.get_constant_address(Constant::Integer(children.len() as i64));

        let target = target.unwrap_or_else(|| self.allocate_temporary_register());
        let mut list_emission = {
            let mut emission = InstructionsEmission::with_capacity(children.len());

            emission.push(Instruction::no_op()); // Placeholder for NEW_LIST

            emission
        };
        let mut element_type = None;

        for (index, child) in children.enumerate() {
            let element_emission = self.visit_expression(child, None)?;
            let element_address =
                handle_element_emission(self, &mut list_emission, element_emission, &child.node)?;
            let index_address = self.get_constant_address(Constant::Integer(index as i64));
            let new_element_type = *self.context.get_type_binding(&child.id).ok_or(
                CompileError::MissingTypeBinding {
                    syntax_id: child.id,
                },
            )?;
            let operand_type = self
                .context
                .types
                .get_operand_type(new_element_type)
                .ok_or(CompileError::MissingType {
                    type_id: new_element_type,
                })?;
            let set_list_instruction =
                Instruction::set_list(target.index, element_address, index_address, operand_type);

            list_emission.push(set_list_instruction);

            if let Some(established) = element_type {
                let unified = self
                    .context
                    .types
                    .unify_types(established, new_element_type)?;

                if !unified {
                    let expected = self.context.types.get_full_type(established).ok_or(
                        CompileError::MissingType {
                            type_id: established,
                        },
                    )?;
                    let found = self.context.types.get_full_type(new_element_type).ok_or(
                        CompileError::MissingType {
                            type_id: new_element_type,
                        },
                    )?;

                    return Err(CompileError::TypeMismatch {
                        expected,
                        expected_position: Position::new(self.file_id, child.span()),
                        found,
                        found_position: Position::new(self.file_id, child.span()),
                    });
                }
            } else {
                element_type = Some(new_element_type);
            }
        }

        let element_type =
            element_type.unwrap_or_else(|| self.context.types.create_inferred_type());
        let list_type = self.context.types.add_type(TypeNode::List { element_type });
        let operand_type = self
            .context
            .types
            .get_operand_type(list_type)
            .ok_or(CompileError::MissingType { type_id: list_type })?;
        let new_list_instruction =
            Instruction::new_list(target.index, child_count_address, operand_type);

        list_emission.instructions[0] = (new_list_instruction, Vec::new());

        list_emission.set_target(Some(target));

        Ok(Emission::Instructions(list_emission))
    }

    fn visit_while_expression(
        &mut self,
        _: SyntaxReader<'_>,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        todo!()
    }

    fn visit_function_expression(
        &mut self,
        _: SyntaxReader<'_>,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        todo!()
    }

    fn visit_call_expression(
        &mut self,
        _: SyntaxReader<'_>,
        _: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub enum Emission {
    Constant(Constant),
    Function(Address),
    Local(Local),
    Instructions(InstructionsEmission),
    None,
}

impl Emission {
    fn target_register(&self) -> Option<TargetRegister> {
        match self {
            Emission::Local(Local { address, .. }) => Some(TargetRegister {
                index: address.index,
                is_temporary: false,
            }),
            Emission::Instructions(emission) => emission.target_register,
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct InstructionsEmission {
    instructions: Vec<(Instruction, Vec<JumpAnchor>)>,
    target_register: Option<TargetRegister>,
}

impl InstructionsEmission {
    fn new() -> Self {
        Self {
            instructions: Vec::new(),
            target_register: None,
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        Self {
            instructions: Vec::with_capacity(capacity),
            target_register: None,
        }
    }

    fn with_instruction(instruction: Instruction) -> Self {
        Self {
            instructions: vec![(instruction, Vec::new())],
            target_register: None,
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
        self.target_register = target;
    }

    fn add_drop(&mut self, compiler: &mut Emitter, target_register: Option<TargetRegister>) {
        let start = compiler.drop_lists.len() as u16;
        let mut pending_drops_for_scope = compiler.pending_drops.pop().unwrap();

        for register in pending_drops_for_scope.drain(..) {
            if let Some(target_register) = target_register
                && register == target_register.index
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
        self.target_register = other.target_register;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TargetRegister {
    index: u16,
    is_temporary: bool,
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
    fn type_id(&self) -> TypeId {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Local {
    address: Address,
    type_id: TypeId,
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
