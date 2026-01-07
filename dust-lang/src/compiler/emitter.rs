use std::collections::HashMap;

use rustc_hash::FxBuildHasher;
use smallvec::SmallVec;
use tracing::{Level, debug, info, span, trace};

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

    source: Source,

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

    /// List of register indices that need to be dropped at the end of the current scope.
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
        source: Source,
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
                        address: Address::register(register),
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
        let span = span!(Level::INFO, "emit");
        let _enter = span.enter();

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
                        self.handle_implicit_return(child)?
                    } else {
                        self.visit(child)?
                    };

                    Self::handle_top_emission(&mut self, child_emission)?;
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

        println!("{:#?}", self.instructions);

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

    fn syntax_tree(&self) -> Result<&SyntaxTree, CompileError> {
        self.syntax
            .get_tree(self.file_id)
            .ok_or(CompileError::MissingSyntaxTree {
                file_id: self.file_id,
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

    fn allocate_temporary_register(&mut self) -> u16 {
        let register = self.next_temporary_register;

        trace!("Allocating temporary reg_{}", register);

        self.next_temporary_register += 1;

        if self.next_temporary_register > self.maximum_register {
            self.maximum_register = self.next_temporary_register;
        }

        register
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

    fn allocate_local_register(&mut self) -> u16 {
        let register = self.next_local_register;

        trace!("Allocating local reg_{}", register);

        self.next_local_register += 1;
        self.next_temporary_register = self.next_local_register;

        if self.next_local_register > self.maximum_register {
            self.maximum_register = self.next_local_register;
        }

        register
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

    fn handle_top_emission(compiler: &mut Emitter, emission: Emission) -> Result<(), CompileError> {
        match emission {
            Emission::Constant(constant, type_id) => {
                let destination = compiler.allocate_temporary_register();
                let address = compiler.get_constant_address(constant);
                let operand_type = compiler
                    .context
                    .types
                    .get_operand_type(type_id)
                    .ok_or(CompileError::MissingType { type_id })?;
                let move_instruction = Instruction::r#move(destination, address, operand_type);

                compiler.emit_instruction(move_instruction);
            }
            Emission::Function(function_address, type_id) => {
                let destination = compiler.allocate_temporary_register();
                let operand_type = compiler
                    .context
                    .types
                    .get_operand_type(type_id)
                    .ok_or(CompileError::MissingType { type_id })?;
                let move_instruction =
                    Instruction::r#move(destination, function_address, operand_type);

                compiler.emit_instruction(move_instruction);
            }
            Emission::Local(Local { address, type_id }) => {
                let destination = compiler.allocate_temporary_register();
                let operand_type = compiler
                    .context
                    .types
                    .get_operand_type(type_id)
                    .ok_or(CompileError::MissingType { type_id })?;
                let move_instruction = Instruction::r#move(destination, address, operand_type);

                compiler.emit_instruction(move_instruction);
            }
            Emission::Instructions(InstructionsEmission { instructions, .. }) => {
                for (instruction, mut jump_anchors) in instructions {
                    compiler.emit_instruction(instruction);

                    jump_anchors.sort();

                    for anchor in jump_anchors {
                        match anchor {
                            JumpAnchor::ForwardFromHere { id } => {
                                let coalesce = if instruction.is_coallescible_with_jump(true) {
                                    true
                                } else {
                                    compiler.emit_instruction(Instruction::no_op());

                                    false
                                };

                                compiler.jump_placements.insert(
                                    id,
                                    JumpPlacement {
                                        index: compiler.instructions.len() - 1,
                                        distance: 0,
                                        forward: true,
                                        coalesce,
                                    },
                                );
                            }
                            JumpAnchor::ForwardToNext { id } => {
                                let placement = compiler.jump_placements.get_mut(&id).unwrap();
                                let next_index = compiler.instructions.len();

                                placement.distance = (next_index - placement.index - 1) as u16;
                            }
                            JumpAnchor::LoopStartHere { forward_id } => {
                                let coalesce = if instruction.is_coallescible_with_jump(false) {
                                    true
                                } else {
                                    compiler.emit_instruction(Instruction::no_op());

                                    false
                                };

                                compiler.jump_placements.insert(
                                    forward_id,
                                    JumpPlacement {
                                        index: compiler.instructions.len() - 1,
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
                                let next_index = compiler.instructions.len();
                                let forward_index =
                                    compiler.jump_placements.get(&forward_id).unwrap().index;
                                let coalesce = if instruction.is_coallescible_with_jump(false) {
                                    true
                                } else {
                                    compiler.emit_instruction(Instruction::no_op());

                                    false
                                };

                                let forward_distance = if coalesce {
                                    (next_index - forward_index - 1) as u16
                                } else {
                                    (next_index - forward_index) as u16
                                };

                                compiler
                                    .jump_placements
                                    .get_mut(&forward_id)
                                    .unwrap()
                                    .distance = forward_distance;

                                let backward_distance =
                                    (compiler.instructions.len() - forward_index - 1) as u16;
                                let backward_placement = JumpPlacement {
                                    index: compiler.instructions.len() - 1,
                                    distance: backward_distance,
                                    forward: false,
                                    coalesce,
                                };

                                compiler
                                    .jump_placements
                                    .insert(backward_id, backward_placement);
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
        node: &SyntaxNode,
    ) -> Result<(Address, TypeId), CompileError> {
        match emission {
            Emission::Constant(constant, type_id) => {
                let address = self.get_constant_address(constant);

                Ok((address, type_id))
            }
            Emission::Function(address, type_id) => Ok((address, type_id)),
            Emission::Local(Local { address, type_id }) => Ok((address, type_id)),
            Emission::Instructions(operand_instructions) => {
                let destination = operand_instructions
                    .target_register
                    .ok_or(CompileError::ExpectedExpression {
                        node_kind: node.kind,
                        position: Position::new(self.file_id, node.span),
                    })?
                    .register;
                let type_id = operand_instructions.type_id;

                instructions.merge(operand_instructions);

                Ok((Address::register(destination), type_id))
            }
            Emission::None => Err(CompileError::ExpectedExpression {
                node_kind: node.kind,
                position: Position::new(self.file_id, node.span),
            }),
        }
    }

    fn handle_condition_emission(
        &mut self,
        instructions: &mut InstructionsEmission,
        emission: Emission,
        node: &SyntaxNode,
    ) -> Result<(), CompileError> {
        let type_id = match emission {
            Emission::Constant(constant, type_id) => {
                let address = self.get_constant_address(constant);
                let test_instruction = Instruction::test(address, true, 1);

                instructions.push(test_instruction);

                type_id
            }
            Emission::Function(_, _) => {
                return Err(CompileError::ExpectedBooleanExpression {
                    node_kind: node.kind,
                    position: Position::new(self.file_id, node.span),
                });
            }
            Emission::Local(Local { type_id, address }) => {
                let test_instruction = Instruction::test(address, true, 1);

                instructions.push(test_instruction);

                type_id
            }
            Emission::Instructions(mut condition_instructions) => {
                let length = condition_instructions.length();
                let type_id = condition_instructions.type_id;

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

                type_id
            }
            Emission::None => TypeId::NONE,
        };

        if type_id == TypeId::BOOLEAN {
            Ok(())
        } else {
            Err(CompileError::ExpectedBooleanExpression {
                node_kind: node.kind,
                position: Position::new(self.file_id, node.span),
            })
        }
    }

    fn handle_branch_emission(
        &mut self,
        instructions_emission: &mut InstructionsEmission,
        emission: Emission,
        destination_register: u16,
    ) -> Result<TypeId, CompileError> {
        match emission {
            Emission::Constant(constant, type_id) => {
                let address = self.get_constant_address(constant);
                let operand_type = self.get_operand_type(type_id)?;
                let move_instruction =
                    Instruction::r#move(destination_register, address, operand_type);

                instructions_emission.push(move_instruction);

                Ok(type_id)
            }
            Emission::Function(address, type_id) => {
                let operand_type = self.get_operand_type(type_id)?;
                let move_instruction =
                    Instruction::r#move(destination_register, address, operand_type);

                instructions_emission.push(move_instruction);

                Ok(type_id)
            }
            Emission::Local(Local { address, type_id }) => {
                let operand_type = self.get_operand_type(type_id)?;
                let move_instruction =
                    Instruction::r#move(destination_register, address, operand_type);

                instructions_emission.push(move_instruction);

                Ok(type_id)
            }
            Emission::Instructions(branch_instructions) => {
                let type_id = branch_instructions.type_id;

                instructions_emission.merge(branch_instructions);

                Ok(type_id)
            }
            Emission::None => Ok(TypeId::NONE),
        }
    }

    fn handle_return_emission(
        &mut self,
        return_instructions: &mut InstructionsEmission,
        emission: Emission,
        node: SyntaxReader,
    ) -> Result<(), CompileError> {
        let (address, type_id) = match emission {
            Emission::Constant(constant, type_id) => {
                let address = self.get_constant_address(constant);

                (address, type_id)
            }
            Emission::Function(address, type_id) => (address, type_id),
            Emission::Local(Local { address, type_id }) => (address, type_id),
            Emission::Instructions(instructions) => {
                if let Some(target) = instructions.target_register {
                    let type_id = instructions.type_id;

                    return_instructions.merge(instructions);

                    (Address::register(target.register), type_id)
                } else if instructions.type_id == TypeId::NONE {
                    return_instructions.merge(instructions);

                    (Address::default(), TypeId::NONE)
                } else {
                    return Err(CompileError::ExpectedExpression {
                        node_kind: node.kind(),
                        position: Position::new(self.file_id, node.span()),
                    });
                }
            }
            Emission::None => (Address::default(), TypeId::NONE),
        };
        let operand_type = self
            .context
            .types
            .get_operand_type(type_id)
            .ok_or(CompileError::MissingType { type_id })?;
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

    fn handle_implicit_return(&mut self, node: SyntaxReader<'_>) -> Result<Emission, CompileError> {
        let mut return_emission = InstructionsEmission::new();
        let emission = self.visit(node)?;

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
    type Output = Emission;

    type Error = CompileError;

    fn visit_item(&mut self, node: SyntaxReader<'_>) -> Result<Self::Output, Self::Error> {
        match node.kind() {
            SyntaxKind::MainFunctionItem => self.visit_main_function_item(node),
            SyntaxKind::ModuleItem => self.visit_module_item(node),
            SyntaxKind::FunctionItem => self.visit_function_item(node),
            SyntaxKind::UseItem => self.visit_use_item(node),
            _ => Err(CompileError::ExpectedItem {
                node_kind: node.kind(),
                position: Position::new(self.file_id, node.span()),
            }),
        }
    }

    fn visit_statement(&mut self, _: SyntaxReader<'_>) -> Result<Self::Output, Self::Error> {
        todo!()
    }

    fn visit_expression(&mut self, _: SyntaxReader<'_>) -> Result<Self::Output, Self::Error> {
        todo!()
    }

    fn visit_main_function_item(
        &mut self,
        node: SyntaxReader<'_>,
    ) -> Result<Self::Output, Self::Error> {
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
            let child_emission = self.visit(child)?;

            if index == last_child {
                final_emission = self.handle_implicit_return(child)?;
            } else {
                Self::handle_top_emission(self, child_emission)?;
            }
        }

        Ok(final_emission)
    }

    fn visit_module_item(&mut self, _: SyntaxReader<'_>) -> Result<Self::Output, Self::Error> {
        todo!()
    }

    fn visit_function_item(&mut self, _: SyntaxReader<'_>) -> Result<Self::Output, Self::Error> {
        todo!()
    }

    fn visit_use_item(&mut self, _: SyntaxReader<'_>) -> Result<Self::Output, Self::Error> {
        todo!()
    }

    fn visit_expression_statement(
        &mut self,
        _: SyntaxReader<'_>,
    ) -> Result<Self::Output, Self::Error> {
        todo!()
    }

    fn visit_reassignment_statement(
        &mut self,
        _: SyntaxReader<'_>,
    ) -> Result<Self::Output, Self::Error> {
        todo!()
    }

    fn visit_integer_expression(
        &mut self,
        node: SyntaxReader<'_>,
    ) -> Result<Self::Output, Self::Error> {
        info!("Emitting integer expression");

        Ok(Emission::Constant(
            Constant::Integer(node.decode_integer()),
            TypeId::INTEGER,
        ))
    }

    fn visit_path_expression(&mut self, _: SyntaxReader<'_>) -> Result<Self::Output, Self::Error> {
        todo!()
    }

    fn visit_block_expression(&mut self, _: SyntaxReader<'_>) -> Result<Self::Output, Self::Error> {
        todo!()
    }

    fn visit_if_expression(&mut self, _: SyntaxReader<'_>) -> Result<Self::Output, Self::Error> {
        todo!()
    }

    fn visit_let_statement(&mut self, _: SyntaxReader<'_>) -> Result<Self::Output, Self::Error> {
        todo!()
    }

    fn visit_math_expression(&mut self, _: SyntaxReader<'_>) -> Result<Self::Output, Self::Error> {
        todo!()
    }

    fn visit_while_expression(&mut self, _: SyntaxReader<'_>) -> Result<Self::Output, Self::Error> {
        todo!()
    }

    fn visit_function_expression(
        &mut self,
        _: SyntaxReader<'_>,
    ) -> Result<Self::Output, Self::Error> {
        todo!()
    }

    fn visit_call_expression(&mut self, _: SyntaxReader<'_>) -> Result<Self::Output, Self::Error> {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub enum Emission {
    Constant(Constant, TypeId),
    Function(Address, TypeId),
    Local(Local),
    Instructions(InstructionsEmission),
    None,
}

impl Emission {
    fn set_type(&mut self, type_id: TypeId) {
        match self {
            Emission::Constant(_, existing_type) => *existing_type = type_id,
            Emission::Function(_, existing_type) => *existing_type = type_id,
            Emission::Local(local) => local.type_id = type_id,
            Emission::Instructions(emission) => emission.set_type(type_id),
            Emission::None => {}
        }
    }

    fn target_register(&self) -> Option<TargetRegister> {
        match self {
            Emission::Local(Local { address, .. }) => Some(TargetRegister {
                register: address.index,
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
    type_id: TypeId,
    target_register: Option<TargetRegister>,
}

impl InstructionsEmission {
    fn new() -> Self {
        Self {
            instructions: Vec::new(),
            type_id: TypeId::NONE,
            target_register: None,
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        Self {
            instructions: Vec::with_capacity(capacity),
            type_id: TypeId::NONE,
            target_register: None,
        }
    }

    fn with_instruction(instruction: Instruction) -> Self {
        Self {
            instructions: vec![(instruction, Vec::new())],
            type_id: TypeId::NONE,
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

    fn set_type(&mut self, type_id: TypeId) {
        self.type_id = type_id;
    }

    fn set_target(&mut self, target: Option<TargetRegister>) {
        self.target_register = target;
    }

    fn add_drop(&mut self, compiler: &mut Emitter, target_register: Option<TargetRegister>) {
        let start = compiler.drop_lists.len() as u16;
        let mut pending_drops_for_scope = compiler.pending_drops.pop().unwrap();

        for register in pending_drops_for_scope.drain(..) {
            if let Some(target_register) = target_register
                && register == target_register.register
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
        self.type_id = other.type_id;
        self.target_register = other.target_register;
    }
}

#[derive(Clone, Copy, Debug)]
struct TargetRegister {
    register: u16,
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
