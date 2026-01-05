use std::collections::HashMap;

use rustc_hash::FxBuildHasher;
use smallvec::SmallVec;
use tracing::{debug, info, trace};

use crate::{
    compiler::CompileContext,
    instruction::{Address, Drop, Instruction, Move, OperandType, Operation, Test},
    native_function::NativeFunction,
    prototype::Prototype,
    resolver::{
        Declaration, DeclarationId, DeclarationKind, FunctionTypeNode, Scope, ScopeId, ScopeKind,
        TypeId, TypeNode,
    },
    source::{Position, SourceFileId},
    syntax_tree::{SyntaxId, SyntaxKind, SyntaxNode, SyntaxTree},
    r#type::Type,
};

use super::CompileError;

#[derive(Debug)]
pub struct PrototypeCompiler<'a> {
    declaration_id: Option<DeclarationId>,

    prototype_index: u16,

    file_id: SourceFileId,

    function_type_node: FunctionTypeNode,

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

impl<'a> PrototypeCompiler<'a> {
    pub fn new(
        declaration_info: Option<(DeclarationId, Declaration)>,
        prototype_index: u16,
        file_id: SourceFileId,
        function_type: FunctionTypeNode,
        context: &'a mut CompileContext,
        starting_scope_id: ScopeId,
    ) -> Self {
        let mut prototype_compiler = Self {
            declaration_id: declaration_info.map(|(id, _)| id),
            prototype_index,
            file_id,
            function_type_node: function_type,
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
                    .resolver
                    .get_parameter(current_parameter_index)
                    && let Some(parameter_declaration) = prototype_compiler
                        .context
                        .resolver
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

    pub fn compile_main(mut self) -> Result<Prototype, CompileError> {
        let root_node =
            *self
                .syntax_tree()?
                .get_node(SyntaxId(0))
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: SyntaxId(0),
                })?;

        self.compile_item(&root_node)?;
        self.finish(0)
    }

    pub fn finish(mut self, index: u16) -> Result<Prototype, CompileError> {
        // self.context.constants.finalize_string_pool();

        let name_position = if let Some(declaration_id) = self.declaration_id {
            let declaration = self
                .context
                .resolver
                .get_declaration(declaration_id)
                .ok_or(CompileError::MissingDeclaration { declaration_id })?;

            Some(declaration.position)
        } else {
            None
        };
        let register_count = self.maximum_register;
        let type_id = self
            .context
            .resolver
            .add_type_node(TypeNode::Function(self.function_type_node));
        let function_type = self
            .context
            .resolver
            .resolve_function_type(&self.function_type_node)
            .ok_or(CompileError::MissingType { type_id })?;

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
            index,
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

    fn syntax_tree(&self) -> Result<&SyntaxTree, CompileError> {
        self.context.file_trees.get(self.file_id.0 as usize).ok_or(
            CompileError::MissingSyntaxTree {
                file_id: self.file_id,
            },
        )
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

    fn get_type(&mut self, type_id: TypeId) -> Result<Type, CompileError> {
        self.context
            .resolver
            .resolve_type(type_id)
            .ok_or(CompileError::MissingType { type_id })
    }

    fn get_operand_type(&mut self, type_id: TypeId) -> Result<OperandType, CompileError> {
        self.context
            .resolver
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
        left: Constant,
        right: Constant,
        operation: SyntaxKind,
        position: Position,
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
                Err(CompileError::DivisionByZero { position })
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
                return Err(CompileError::MismatchedConstantTypes {
                    left: self.get_type(left.type_id())?,
                    right: self.get_type(right.type_id())?,
                });
            }
        };

        Ok(combined)
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
        node: &SyntaxNode,
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
                        node_kind: node.kind,
                        position: Position::new(self.file_id, node.span),
                    });
                }
            }
            Emission::None => (Address::default(), TypeId::NONE),
        };
        let operand_type = self
            .context
            .resolver
            .get_operand_type(type_id)
            .ok_or(CompileError::MissingType { type_id })?;
        let return_instruction = Instruction::r#return(address, operand_type);
        self.function_type_node.return_type = type_id;

        return_instructions.push(return_instruction);

        Ok(())
    }

    fn compile_item(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        match node.kind {
            SyntaxKind::MainFunctionItem => self.compile_function(node),
            SyntaxKind::ModuleItem => self.compile_module_item(node),
            SyntaxKind::FunctionItem => self.compile_function_item(node),
            SyntaxKind::UseItem => self.compile_use_item(node),
            _ => Err(CompileError::ExpectedItem {
                node_kind: node.kind,
                position: Position::new(self.file_id, node.span),
            }),
        }
    }

    fn compile_statement(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        match node.kind {
            SyntaxKind::ExpressionStatement => self.compile_expression_statement(node),
            SyntaxKind::LetStatement | SyntaxKind::LetMutStatement => {
                self.compile_let_statement(node)
            }
            SyntaxKind::ReassignmentStatement => self.compile_reassignment_statement(node),
            SyntaxKind::AdditionAssignmentStatement
            | SyntaxKind::SubtractionAssignmentStatement
            | SyntaxKind::MultiplicationAssignmentStatement
            | SyntaxKind::DivisionAssignmentStatement
            | SyntaxKind::ModuloAssignmentStatement
            | SyntaxKind::ExponentAssignmentStatement => self.compile_math_binary(node, None),
            _ => Err(CompileError::ExpectedStatement {
                node_kind: node.kind,
                position: Position::new(self.file_id, node.span),
            }),
        }
    }

    fn compile_function(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Compiling function");

        fn handle_emission(
            compiler: &mut PrototypeCompiler,
            emission: Emission,
        ) -> Result<(), CompileError> {
            match emission {
                Emission::Constant(constant, type_id) => {
                    let destination = compiler.allocate_temporary_register();
                    let address = compiler.get_constant_address(constant);
                    let operand_type = compiler
                        .context
                        .resolver
                        .get_operand_type(type_id)
                        .ok_or(CompileError::MissingType { type_id })?;
                    let move_instruction = Instruction::r#move(destination, address, operand_type);

                    compiler.emit_instruction(move_instruction);
                }
                Emission::Function(function_address, type_id) => {
                    let destination = compiler.allocate_temporary_register();
                    let operand_type = compiler
                        .context
                        .resolver
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
                        .resolver
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

        let (start_children, child_count) = match node.kind {
            SyntaxKind::MainFunctionItem => (node.children.0 as usize, node.children.1 as usize),
            SyntaxKind::ExpressionStatement => {
                let expression_id = SyntaxId(node.children.0);
                let expression_node = *self.syntax_tree()?.get_node(expression_id).ok_or(
                    CompileError::MissingSyntaxNode {
                        syntax_id: expression_id,
                    },
                )?;

                debug_assert!(
                    expression_node.kind == SyntaxKind::BlockExpression,
                    "Expected block expression"
                );

                (
                    expression_node.children.0 as usize,
                    expression_node.children.1 as usize,
                )
            }
            SyntaxKind::BlockExpression => (node.children.0 as usize, node.children.1 as usize),
            _ => todo!("Error: {}", node.kind),
        };
        let end_children = start_children + child_count;

        if child_count == 0 {
            let return_instruction = Instruction::r#return(Address::default(), OperandType::NONE);

            self.emit_instruction(return_instruction);

            return Ok(());
        }

        let mut current_child_index = start_children;

        while current_child_index < end_children {
            let child_id = *self
                .syntax_tree()?
                .children
                .get(current_child_index)
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: current_child_index as u32,
                })?;
            let child_node =
                *self
                    .syntax_tree()?
                    .get_node(child_id)
                    .ok_or(CompileError::MissingSyntaxNode {
                        syntax_id: child_id,
                    })?;
            current_child_index += 1;

            if current_child_index == end_children {
                let final_emission = self.compile_implicit_return(child_id, &child_node)?;

                handle_emission(self, final_emission)?;
            } else if child_node.kind.is_statement() {
                let statement_emission = self.compile_statement(&child_node)?;

                handle_emission(self, statement_emission)?;
            } else if child_node.kind.is_item() {
                self.compile_item(&child_node)?;
            } else {
                return Err(CompileError::ExpectedStatement {
                    node_kind: child_node.kind,
                    position: Position::new(self.file_id, child_node.span),
                });
            }
        }

        Ok(())
    }

    fn compile_module_item(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Compiling module item");

        let (start_children, child_count) = (node.children.0 as usize, node.children.1 as usize);
        let end_children = start_children + child_count;

        if child_count == 0 {
            return Ok(());
        }

        let mut current_child_index = start_children;

        while current_child_index < end_children {
            let child_id = *self
                .syntax_tree()?
                .children
                .get(current_child_index)
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: current_child_index as u32,
                })?;
            let child_node =
                *self
                    .syntax_tree()?
                    .get_node(child_id)
                    .ok_or(CompileError::MissingSyntaxNode {
                        syntax_id: child_id,
                    })?;
            current_child_index += 1;

            self.compile_item(&child_node)?;
        }

        Ok(())
    }

    fn compile_use_item(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Compiling use item");

        let syntax_tree = self.syntax_tree()?;

        let path_id = SyntaxId(node.children.0);
        let path_node = syntax_tree
            .get_node(path_id)
            .ok_or(CompileError::MissingSyntaxNode { syntax_id: path_id })?;
        let path_segments_node_ids = syntax_tree
            .get_children(path_node.children.0, path_node.children.1)
            .ok_or(CompileError::MissingChildren {
                parent_kind: path_node.kind,
                start_index: path_node.children.0,
                count: path_node.children.1,
            })?;
        let path_segments_nodes: SmallVec<[_; 4]> = path_segments_node_ids
            .iter()
            .map(|id| {
                syntax_tree
                    .get_node(*id)
                    .ok_or(CompileError::MissingSyntaxNode { syntax_id: *id })
            })
            .collect::<Result<_, _>>()?;

        let files = self.context.source.read_files();
        let file = files
            .get(self.file_id.0 as usize)
            .ok_or(CompileError::MissingSourceFile {
                file_id: self.file_id,
            })?;

        let module_name_node = path_segments_nodes.first().unwrap();
        let module_name_bytes = &file.source_code.as_ref()
            [module_name_node.span.0 as usize..module_name_node.span.1 as usize];
        let module_name = unsafe { std::str::from_utf8_unchecked(module_name_bytes) };
        let (module_import_id, module_import) = self
            .context
            .resolver
            .find_declarations(module_name)
            .ok_or(CompileError::MissingDeclarations {
                name: module_name.to_string(),
            })?
            .into_iter()
            .find(|(_id, declaration)| matches!(declaration.kind, DeclarationKind::Module { .. }))
            .ok_or(CompileError::UndeclaredVariable {
                name: module_name.to_string(),
                position: Position::new(self.file_id, module_name_node.span),
            })?;

        let (final_declaration_id, final_declaration) = if path_segments_nodes.len() > 1 {
            let mut current_scope_id =
                if let DeclarationKind::Module { inner_scope_id, .. } = &module_import.kind {
                    *inner_scope_id
                } else {
                    unreachable!("Expected module declaration");
                };
            let mut current_declaration_id = module_import_id;
            let mut current_declaration = module_import;

            for segment_node in path_segments_nodes.iter().skip(1) {
                let segment_bytes = &file.source_code.as_ref()
                    [segment_node.span.0 as usize..segment_node.span.1 as usize];
                let segment_name = unsafe { std::str::from_utf8_unchecked(segment_bytes) };
                let (declaration_id, declaration) = self
                    .context
                    .resolver
                    .find_declaration_in_scope(segment_name, current_scope_id)
                    .ok_or(CompileError::UndeclaredVariable {
                        name: segment_name.to_string(),
                        position: Position::new(self.file_id, segment_node.span),
                    })?;

                current_scope_id = match &declaration.kind {
                    DeclarationKind::Module { inner_scope_id, .. } => *inner_scope_id,
                    DeclarationKind::Function { inner_scope_id, .. } => *inner_scope_id,
                    _ => {
                        return Err(CompileError::CannotImport {
                            name: segment_name.to_string(),
                            position: Position::new(self.file_id, segment_node.span),
                        });
                    }
                };
                current_declaration_id = declaration_id;
                current_declaration = declaration;
            }

            (current_declaration_id, current_declaration)
        } else {
            (module_import_id, module_import)
        };

        drop(path_segments_nodes);
        drop(files);

        match final_declaration.kind {
            DeclarationKind::Function {
                prototype_index,
                file_id,
                inner_scope_id,
                syntax_id,
                ..
            } => {
                let prototype_index = if let Some(prototype_index) = prototype_index {
                    prototype_index
                } else {
                    let prototype_index_usize = self.context.prototypes.len();
                    let prototype_index_u16 = prototype_index_usize as u16;

                    self.context.prototypes.push(Prototype::default());

                    if let DeclarationKind::Function {
                        prototype_index: old_index,
                        ..
                    } = &mut self
                        .context
                        .resolver
                        .get_declaration_mut(&final_declaration_id)
                        .unwrap()
                        .kind
                    {
                        *old_index = Some(prototype_index_u16);
                    }

                    let function_type_node = self
                        .context
                        .resolver
                        .get_type_node(final_declaration.type_id)
                        .ok_or(CompileError::MissingType {
                            type_id: final_declaration.type_id,
                        })?
                        .into_function_type()
                        .ok_or(CompileError::ExpectedFunctionType {
                            type_id: final_declaration.type_id,
                        })?;
                    let function_file = self
                        .context
                        .file_trees
                        .get(file_id.0 as usize)
                        .ok_or(CompileError::MissingSourceFile { file_id })?;
                    let function_item_node = *function_file
                        .get_node(syntax_id)
                        .ok_or(CompileError::MissingSyntaxNode { syntax_id })?;
                    let function_expression_node_id = SyntaxId(function_item_node.children.1);
                    let function_expression_node = *function_file
                        .get_node(function_expression_node_id)
                        .ok_or(CompileError::MissingSyntaxNode {
                            syntax_id: function_expression_node_id,
                        })?;
                    let function_body_node_id = SyntaxId(function_expression_node.children.1);
                    let function_body_node = *function_file.get_node(function_body_node_id).ok_or(
                        CompileError::MissingSyntaxNode {
                            syntax_id: function_body_node_id,
                        },
                    )?;
                    let mut importer = PrototypeCompiler::new(
                        Some((final_declaration_id, final_declaration)),
                        prototype_index_usize as u16,
                        file_id,
                        function_type_node,
                        self.context,
                        inner_scope_id,
                    );

                    importer.compile_function(&function_body_node)?;

                    let prototype = importer.finish(prototype_index_u16)?;
                    self.context.prototypes[prototype_index_usize] = prototype;

                    prototype_index_u16
                };

                self.locals.insert(
                    final_declaration_id,
                    Local {
                        type_id: final_declaration.type_id,
                        address: Address::constant(prototype_index),
                    },
                );
            }
            _ => {
                let files = self.context.source.read_files();
                let file =
                    files
                        .get(self.file_id.0 as usize)
                        .ok_or(CompileError::MissingSourceFile {
                            file_id: self.file_id,
                        })?;
                let path_segments_nodes: SmallVec<[_; 4]> = path_segments_node_ids
                    .iter()
                    .map(|id| {
                        syntax_tree
                            .get_node(*id)
                            .ok_or(CompileError::MissingSyntaxNode { syntax_id: *id })
                    })
                    .collect::<Result<_, _>>()?;
                let module_name_node = path_segments_nodes.first().unwrap();
                let module_name_bytes = &file.source_code.as_ref()
                    [module_name_node.span.0 as usize..module_name_node.span.1 as usize];
                let module_name = unsafe { std::str::from_utf8_unchecked(module_name_bytes) };

                return Err(CompileError::CannotImport {
                    name: module_name.to_string(),
                    position: Position::new(self.file_id, path_node.span),
                });
            }
        }

        Ok(())
    }

    fn compile_expression_statement(
        &mut self,
        node: &SyntaxNode,
    ) -> Result<Emission, CompileError> {
        info!("Compiling expression statement");

        let expression_id = SyntaxId(node.children.0);
        let expression_node = *self.syntax_tree()?.get_node(expression_id).ok_or(
            CompileError::MissingSyntaxNode {
                syntax_id: expression_id,
            },
        )?;

        if expression_node.kind.is_expression() {
            let mut emission = self.compile_expression(expression_id, &expression_node, None)?;

            emission.set_type(TypeId::NONE);

            if let Emission::Instructions(instructions_emission) = &mut emission {
                instructions_emission.set_target(None);
            }

            Ok(emission)
        } else {
            self.compile_statement(&expression_node)
        }
    }

    fn compile_let_statement(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling let statement");

        let path_id = SyntaxId(node.children.0);
        let path_node = *self
            .syntax_tree()?
            .get_node(path_id)
            .ok_or(CompileError::MissingSyntaxNode { syntax_id: path_id })?;

        let expression_statement_id = SyntaxId(node.children.1);
        let expression_statement = *self
            .syntax_tree()?
            .get_node(expression_statement_id)
            .ok_or(CompileError::MissingSyntaxNode {
                syntax_id: expression_statement_id,
            })?;

        let expression_id = SyntaxId(expression_statement.children.0);
        let expression_node = *self.syntax_tree()?.get_node(expression_id).ok_or(
            CompileError::MissingSyntaxNode {
                syntax_id: SyntaxId(expression_statement.children.0),
            },
        )?;

        let mut let_statement_emission = InstructionsEmission::new();
        let local_register = self.allocate_local_register();
        let local_target = TargetRegister {
            register: local_register,
            is_temporary: false,
        };

        let expression_emission =
            self.compile_expression(expression_id, &expression_node, Some(local_target))?;
        let local_type_id = match expression_emission {
            Emission::Constant(constant, type_id) => {
                let address = self.get_constant_address(constant);
                let operand_type = self.get_operand_type(type_id)?;
                let move_instruction = Instruction::r#move(local_register, address, operand_type);

                let_statement_emission.push(move_instruction);

                type_id
            }
            Emission::Function(address, type_id) => {
                let operand_type = self.get_operand_type(type_id)?;
                let move_instruction = Instruction::r#move(local_register, address, operand_type);

                let_statement_emission.push(move_instruction);

                type_id
            }
            Emission::Local(Local { address, type_id }) => {
                let operand_type = self.get_operand_type(type_id)?;
                let move_instruction = Instruction::r#move(local_register, address, operand_type);

                let_statement_emission.push(move_instruction);

                type_id
            }
            Emission::Instructions(expression_instructions) => {
                let type_id = expression_instructions.type_id;

                let_statement_emission.merge(expression_instructions);

                type_id
            }
            Emission::None => {
                return Err(CompileError::ExpectedExpression {
                    node_kind: expression_node.kind,
                    position: Position::new(self.file_id, expression_node.span),
                });
            }
        };

        let files = self.context.source.read_files();
        let source_file =
            files
                .get(self.file_id.0 as usize)
                .ok_or(CompileError::MissingSourceFile {
                    file_id: self.file_id,
                })?;
        let variable_name = source_file
            .source_code
            .get(path_node.span.0 as usize, path_node.span.1 as usize);

        let (declaration_id, _) = self
            .context
            .resolver
            .find_declaration_in_scope(variable_name, self.current_scope_id)
            .ok_or(CompileError::UndeclaredVariable {
                name: variable_name.to_string(),
                position: Position::new(self.file_id, path_node.span),
            })?;

        drop(files);

        if local_type_id == TypeId::STRING {
            self.pending_drops.last_mut().unwrap().push(local_register);
        }

        self.locals.insert(
            declaration_id,
            Local {
                type_id: local_type_id,
                address: Address::register(local_register),
            },
        );
        let_statement_emission.set_target(None);

        Ok(Emission::Instructions(let_statement_emission))
    }

    fn compile_reassignment_statement(
        &mut self,
        node: &SyntaxNode,
    ) -> Result<Emission, CompileError> {
        info!("Compiling reassignment statement");

        let path_id = SyntaxId(node.children.0);
        let path_node = *self
            .syntax_tree()?
            .get_node(path_id)
            .ok_or(CompileError::MissingSyntaxNode { syntax_id: path_id })?;

        let expression_statement_id = SyntaxId(node.children.1);
        let expression_statement_node = *self
            .syntax_tree()?
            .get_node(expression_statement_id)
            .ok_or(CompileError::MissingSyntaxNode {
                syntax_id: expression_statement_id,
            })?;

        let expression_id = SyntaxId(expression_statement_node.children.0);
        let expression_node = *self.syntax_tree()?.get_node(expression_id).ok_or(
            CompileError::MissingSyntaxNode {
                syntax_id: expression_id,
            },
        )?;

        let files = self.context.source.read_files();
        let source_file =
            files
                .get(self.file_id.0 as usize)
                .ok_or(CompileError::MissingSourceFile {
                    file_id: self.file_id,
                })?;
        let variable_name_bytes =
            &source_file.source_code.as_ref()[path_node.span.0 as usize..path_node.span.1 as usize];
        let variable_name = unsafe { str::from_utf8_unchecked(variable_name_bytes) };

        let (declaration_id, declaration) = self
            .context
            .resolver
            .find_declaration_in_scope(variable_name, self.current_scope_id)
            .ok_or(CompileError::UndeclaredVariable {
                name: variable_name.to_string(),
                position: Position::new(self.file_id, path_node.span),
            })?;

        if !matches!(declaration.kind, DeclarationKind::LocalMutable { .. }) {
            return Err(CompileError::CannotMutate {
                name: variable_name.to_string(),
                position: Position::new(self.file_id, path_node.span),
            });
        }

        let local_register = self
            .locals
            .get(&declaration_id)
            .ok_or(CompileError::UndeclaredVariable {
                name: variable_name.to_string(),
                position: Position::new(self.file_id, path_node.span),
            })?
            .address
            .index;

        drop(files);

        let mut reassignment_emission = InstructionsEmission::new();
        let expression_emission = self.compile_expression(
            expression_id,
            &expression_node,
            Some(TargetRegister {
                register: local_register,
                is_temporary: false,
            }),
        )?;

        match expression_emission {
            Emission::Constant(constant, type_id) => {
                let address = self.get_constant_address(constant);
                let operand_type = self.get_operand_type(type_id)?;
                let move_instruction = Instruction::r#move(local_register, address, operand_type);

                reassignment_emission.push(move_instruction);
                reassignment_emission.set_target(None);

                Ok(Emission::Instructions(reassignment_emission))
            }
            Emission::Function(address, type_id) => {
                let operand_type = self.get_operand_type(type_id)?;
                let move_instruction = Instruction::r#move(local_register, address, operand_type);

                reassignment_emission.push(move_instruction);
                reassignment_emission.set_target(None);

                Ok(Emission::Instructions(reassignment_emission))
            }
            Emission::Local(local) => {
                let operand_type = self.get_operand_type(local.type_id)?;
                let move_instruction =
                    Instruction::r#move(local_register, local.address, operand_type);

                reassignment_emission.push(move_instruction);
                reassignment_emission.set_target(None);

                Ok(Emission::Instructions(reassignment_emission))
            }
            Emission::Instructions(instructions_emission) => {
                reassignment_emission.merge(instructions_emission);
                reassignment_emission.set_target(None);

                Ok(Emission::Instructions(reassignment_emission))
            }
            Emission::None => todo!(),
        }
    }

    fn compile_function_item(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Compiling function item");

        let path_id = SyntaxId(node.children.0);
        let path_node = *self
            .syntax_tree()?
            .get_node(path_id)
            .ok_or(CompileError::MissingSyntaxNode { syntax_id: path_id })?;

        let function_expression_id = SyntaxId(node.children.1);
        let function_expression_node = *self
            .syntax_tree()?
            .get_node(function_expression_id)
            .ok_or(CompileError::MissingSyntaxNode {
                syntax_id: function_expression_id,
            })?;

        let files = self.context.source.read_files();
        let source_file =
            files
                .get(self.file_id.0 as usize)
                .ok_or(CompileError::MissingSourceFile {
                    file_id: self.file_id,
                })?;

        let path_bytes =
            &source_file.source_code.as_ref()[path_node.span.0 as usize..path_node.span.1 as usize];
        let path_str = unsafe { str::from_utf8_unchecked(path_bytes) };
        let function_name = path_str.split("::").last().unwrap_or(path_str);

        let (declaration_id, declaration) = self
            .context
            .resolver
            .find_declaration_in_scope(function_name, self.current_scope_id)
            .ok_or(CompileError::UndeclaredVariable {
                name: function_name.to_string(),
                position: Position::new(self.file_id, path_node.span),
            })?;
        let function_type = self
            .context
            .resolver
            .get_type_node(declaration.type_id)
            .ok_or(CompileError::MissingType {
                type_id: declaration.type_id,
            })?
            .into_function_type()
            .ok_or(CompileError::ExpectedFunctionType {
                type_id: declaration.type_id,
            })?;

        drop(files);

        let function_emission = self.compile_function_expression(
            &function_expression_node,
            Some((declaration_id, declaration)),
            Some(function_type),
        )?;
        let Emission::Function(prototype_address, _) = function_emission else {
            return Err(CompileError::ExpectedFunction {
                node_kind: node.kind,
                position: Position::new(self.file_id, node.span),
            });
        };
        let local = Local {
            address: prototype_address,
            type_id: declaration.type_id,
        };

        self.locals.insert(declaration_id, local);

        Ok(())
    }

    fn compile_expression(
        &mut self,
        node_id: SyntaxId,
        node: &SyntaxNode,
        target: Option<TargetRegister>,
    ) -> Result<Emission, CompileError> {
        match node.kind {
            SyntaxKind::BooleanExpression => self.compile_boolean_expression(node),
            SyntaxKind::ByteExpression => self.compile_byte_expression(node),
            SyntaxKind::CharacterExpression => self.compile_character_expression(node),
            SyntaxKind::FloatExpression => self.compile_float_expression(node),
            SyntaxKind::IntegerExpression => self.compile_integer_expression(node),
            SyntaxKind::StringExpression => self.compile_string_expression(node),
            SyntaxKind::ListExpression => self.compile_list_expression(node, target),
            SyntaxKind::IndexExpression => self.compile_index_expression(node, target),
            SyntaxKind::PathExpression => self.compile_path_expression(node, target),
            SyntaxKind::AdditionExpression
            | SyntaxKind::SubtractionExpression
            | SyntaxKind::MultiplicationExpression
            | SyntaxKind::DivisionExpression
            | SyntaxKind::ModuloExpression
            | SyntaxKind::ExponentExpression => self.compile_math_binary(node, target),
            SyntaxKind::GreaterThanExpression
            | SyntaxKind::GreaterThanOrEqualExpression
            | SyntaxKind::LessThanExpression
            | SyntaxKind::LessThanOrEqualExpression
            | SyntaxKind::EqualExpression
            | SyntaxKind::NotEqualExpression => self.compile_comparison_expression(node, target),
            SyntaxKind::AndExpression | SyntaxKind::OrExpression => {
                self.compile_logic_expression(node, target)
            }
            SyntaxKind::NotExpression | SyntaxKind::NegationExpression => {
                self.compile_unary_expression(node, target)
            }
            SyntaxKind::GroupedExpression => self.compile_grouped_expression(node, target),
            SyntaxKind::BlockExpression => self.compile_block_expression(node_id, node, target),
            SyntaxKind::WhileExpression => self.compile_while_expression(node),
            SyntaxKind::FunctionExpression => self.compile_function_expression(node, None, None),
            SyntaxKind::CallExpression => self.compile_call_expression(node, target),
            SyntaxKind::AsExpression => self.compile_as_expression(node, target),
            SyntaxKind::IfExpression => self.compile_if_expression(node, target),
            _ => Err(CompileError::ExpectedExpression {
                node_kind: node.kind,
                position: Position::new(self.file_id, node.span),
            }),
        }
    }

    fn compile_boolean_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling boolean expression");

        Ok(Emission::Constant(
            Constant::Boolean(node.children.0 != 0),
            TypeId::BOOLEAN,
        ))
    }

    fn compile_byte_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling byte expression");

        Ok(Emission::Constant(
            Constant::Byte(node.children.0 as u8),
            TypeId::BYTE,
        ))
    }

    fn compile_character_expression(
        &mut self,
        node: &SyntaxNode,
    ) -> Result<Emission, CompileError> {
        info!("Compiling character expression");

        let character = char::from_u32(node.children.0).unwrap_or_default();

        Ok(Emission::Constant(
            Constant::Character(character),
            TypeId::CHARACTER,
        ))
    }

    fn compile_float_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling float expression");

        let float = SyntaxNode::decode_float(node.children);

        Ok(Emission::Constant(Constant::Float(float), TypeId::FLOAT))
    }

    fn compile_integer_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling integer expression");

        let integer = SyntaxNode::decode_integer(node.children);

        Ok(Emission::Constant(
            Constant::Integer(integer),
            TypeId::INTEGER,
        ))
    }

    fn compile_string_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling string expression");

        let string_start = node.span.0 + 1;
        let string_end = node.span.1 - 1;
        let files = self.context.source.read_files();
        let source_file =
            files
                .get(self.file_id.0 as usize)
                .ok_or(CompileError::MissingSourceFile {
                    file_id: self.file_id,
                })?;
        let string_bytes =
            &source_file.source_code.as_ref()[string_start as usize..string_end as usize];
        let (pool_start, pool_end) = self.context.constants.push_str_to_string_pool(string_bytes);

        Ok(Emission::Constant(
            Constant::String {
                pool_start,
                pool_end,
            },
            TypeId::STRING,
        ))
    }

    fn compile_list_expression(
        &mut self,
        node: &SyntaxNode,
        target: Option<TargetRegister>,
    ) -> Result<Emission, CompileError> {
        fn handle_element_emission(
            compiler: &mut PrototypeCompiler,
            instructions: &mut InstructionsEmission,
            element_emission: Emission,
            element_node: &SyntaxNode,
        ) -> Result<(Address, TypeId), CompileError> {
            match element_emission {
                Emission::Constant(constant, type_id) => {
                    let address = compiler.get_constant_address(constant);

                    Ok((address, type_id))
                }
                Emission::Function(address, type_id) => Ok((address, type_id)),
                Emission::Local(Local { address, type_id }) => Ok((address, type_id)),
                Emission::Instructions(InstructionsEmission {
                    instructions: element_instructions,
                    type_id,
                    target_register: target,
                    ..
                }) => {
                    let target = target.ok_or(CompileError::ExpectedExpression {
                        node_kind: element_node.kind,
                        position: Position::new(compiler.file_id, element_node.span),
                    })?;

                    instructions.instructions.extend(element_instructions);

                    Ok((Address::register(target.register), type_id))
                }
                Emission::None => Err(CompileError::ExpectedExpression {
                    node_kind: element_node.kind,
                    position: Position::new(compiler.file_id, element_node.span),
                }),
            }
        }

        info!("Compiling list expression");

        let (start_children, child_count) = (node.children.0 as usize, node.children.1 as usize);
        let target = target.unwrap_or_else(|| TargetRegister {
            register: self.allocate_temporary_register(),
            is_temporary: true,
        });
        let mut list_emission = {
            let mut instructions = InstructionsEmission::with_capacity(child_count + 1);

            instructions.push(Instruction::no_op());

            instructions
        };
        let mut established_element_type_id = None;
        let mut current_child_index = start_children;

        for list_index in 0..child_count {
            let child_id = *self
                .syntax_tree()?
                .children
                .get(current_child_index)
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: current_child_index as u32,
                })?;
            let child_node =
                *self
                    .syntax_tree()?
                    .get_node(child_id)
                    .ok_or(CompileError::MissingSyntaxNode {
                        syntax_id: child_id,
                    })?;
            current_child_index += 1;

            let element_emission = self.compile_expression(child_id, &child_node, None)?;
            let (element_address, element_type_id) =
                handle_element_emission(self, &mut list_emission, element_emission, &child_node)?;
            let element_operand_type = self
                .context
                .resolver
                .get_operand_type(element_type_id)
                .ok_or(CompileError::MissingType {
                    type_id: element_type_id,
                })?;

            if let Some(delcared) = &established_element_type_id
                && delcared != &element_type_id
            {
                todo!("Error");
            } else {
                established_element_type_id = Some(element_type_id);
            }

            let list_index_constant_index = self.context.constants.add_integer(list_index as i64);
            let list_index_address = Address::constant(list_index_constant_index);
            let set_list_instruction = Instruction::set_list(
                target.register,
                element_address,
                list_index_address,
                element_operand_type,
            );

            list_emission.push(set_list_instruction);
        }

        let element_type_id =
            established_element_type_id.ok_or(CompileError::CannotInferListType {
                position: Position::new(self.file_id, node.span),
            })?;
        let list_type = TypeNode::List(element_type_id);
        let list_type_id = self.context.resolver.add_type_node(list_type);
        let list_type_operand = self.context.resolver.get_operand_type(list_type_id).ok_or(
            CompileError::MissingType {
                type_id: list_type_id,
            },
        )?;
        let child_count_constant_index = self.context.constants.add_integer(child_count as i64);
        let child_count_address = Address::constant(child_count_constant_index);

        let new_list_instruction =
            Instruction::new_list(target.register, child_count_address, list_type_operand);

        list_emission.instructions[0].0 = new_list_instruction;

        list_emission.set_type(list_type_id);
        list_emission.set_target(Some(target));

        Ok(Emission::Instructions(list_emission))
    }

    fn compile_index_expression(
        &mut self,
        node: &SyntaxNode,
        target: Option<TargetRegister>,
    ) -> Result<Emission, CompileError> {
        info!("Compiling index expression");

        let list_id = SyntaxId(node.children.0);
        let list_node = *self
            .syntax_tree()?
            .get_node(list_id)
            .ok_or(CompileError::MissingSyntaxNode { syntax_id: list_id })?;

        let index_id = SyntaxId(node.children.1);
        let index_node =
            *self
                .syntax_tree()?
                .get_node(index_id)
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: index_id,
                })?;

        let mut index_emission = InstructionsEmission::new();

        let list_emission = self.compile_expression(list_id, &list_node, None)?;
        let (list_address, list_type_id) =
            self.handle_operand_emission(&mut index_emission, list_emission, &list_node)?;

        let integer_emission = self.compile_expression(index_id, &index_node, None)?;
        let (index_address, index_type) =
            self.handle_operand_emission(&mut index_emission, integer_emission, &index_node)?;

        let list_type_node =
            self.context
                .resolver
                .get_type_node(list_type_id)
                .ok_or(CompileError::MissingType {
                    type_id: list_type_id,
                })?;
        let element_type_id = list_type_node.into_list_element_type().ok_or_else(|| {
            let found_type = self
                .context
                .resolver
                .resolve_type(list_type_id)
                .unwrap_or(Type::None);

            CompileError::ExpectedList {
                found_type,
                position: Position::new(self.file_id, list_node.span),
            }
        })?;
        let element_operand_type = self
            .context
            .resolver
            .get_operand_type(element_type_id)
            .ok_or(CompileError::MissingType {
                type_id: element_type_id,
            })?;

        if index_type != TypeId::INTEGER {
            todo!("Error");
        }

        let target = target.unwrap_or_else(|| TargetRegister {
            register: self.allocate_temporary_register(),
            is_temporary: true,
        });
        let get_list_instruction = Instruction::get_list(
            target.register,
            list_address,
            index_address,
            element_operand_type,
        );

        index_emission.push(get_list_instruction);
        index_emission.set_type(element_type_id);
        index_emission.set_target(Some(target));

        Ok(Emission::Instructions(index_emission))
    }

    fn compile_math_binary(
        &mut self,
        node: &SyntaxNode,
        target: Option<TargetRegister>,
    ) -> Result<Emission, CompileError> {
        info!("Compiling math expression");

        let left_id = SyntaxId(node.children.0);
        let left_node = *self
            .syntax_tree()?
            .get_node(left_id)
            .ok_or(CompileError::MissingSyntaxNode { syntax_id: left_id })?;

        let right_id = SyntaxId(node.children.1);
        let right_node =
            *self
                .syntax_tree()?
                .get_node(right_id)
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: right_id,
                })?;

        let left_emission = self.compile_expression(left_id, &left_node, None)?;
        let right_emission = self.compile_expression(right_id, &right_node, None)?;

        if target.is_none()
            && let (
                Emission::Constant(left_value, left_type_id),
                Emission::Constant(right_value, _right_type_id),
            ) = (&left_emission, &right_emission)
        {
            let combined = self.combine_constants(
                *left_value,
                *right_value,
                node.kind,
                Position::new(self.file_id, node.span),
            )?;
            let combined_type = if left_type_id == &TypeId::CHARACTER {
                TypeId::STRING
            } else {
                *left_type_id
            };

            return Ok(Emission::Constant(combined, combined_type));
        }

        let mut math_emission = InstructionsEmission::new();

        let left_target = left_emission.target_register();
        let (left_address, left_type_id) =
            self.handle_operand_emission(&mut math_emission, left_emission, &left_node)?;
        let (right_address, right_type_id) =
            self.handle_operand_emission(&mut math_emission, right_emission, &right_node)?;

        let type_id = if left_type_id == TypeId::CHARACTER {
            TypeId::STRING
        } else {
            left_type_id
        };
        let operand_type = match (left_type_id, right_type_id) {
            (TypeId::INTEGER, TypeId::INTEGER) => OperandType::INTEGER,
            (TypeId::FLOAT, TypeId::FLOAT) => OperandType::FLOAT,
            (TypeId::BYTE, TypeId::BYTE) => OperandType::BYTE,
            (TypeId::CHARACTER, TypeId::CHARACTER) => OperandType::CHARACTER,
            (TypeId::STRING, TypeId::STRING) => OperandType::STRING,
            (TypeId::STRING, TypeId::CHARACTER) => OperandType::STRING_CHARACTER,
            (TypeId::CHARACTER, TypeId::STRING) => OperandType::CHARACTER_STRING,
            _ => todo!("Error"),
        };
        let math_instruction = match node.kind {
            SyntaxKind::AdditionExpression => {
                let target = target.unwrap_or_else(|| TargetRegister {
                    register: self.allocate_temporary_register(),
                    is_temporary: true,
                });

                math_emission.set_target(Some(target));

                if type_id == TypeId::STRING && target.is_temporary {
                    self.pending_drops.last_mut().unwrap().push(target.register);
                }

                Instruction::add(target.register, left_address, right_address, operand_type)
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
                let target = target.unwrap_or_else(|| TargetRegister {
                    register: self.allocate_temporary_register(),
                    is_temporary: true,
                });

                math_emission.set_target(Some(target));

                Instruction::subtract(target.register, left_address, right_address, operand_type)
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
                let target = target.unwrap_or_else(|| TargetRegister {
                    register: self.allocate_temporary_register(),
                    is_temporary: true,
                });

                math_emission.set_target(Some(target));

                Instruction::multiply(target.register, left_address, right_address, operand_type)
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
                let target = target.unwrap_or_else(|| TargetRegister {
                    register: self.allocate_temporary_register(),
                    is_temporary: true,
                });

                math_emission.set_target(Some(target));

                Instruction::divide(target.register, left_address, right_address, operand_type)
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
                let target = target.unwrap_or_else(|| TargetRegister {
                    register: self.allocate_temporary_register(),
                    is_temporary: true,
                });

                math_emission.set_target(Some(target));

                Instruction::modulo(target.register, left_address, right_address, operand_type)
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
                let target = target.unwrap_or_else(|| TargetRegister {
                    register: self.allocate_temporary_register(),
                    is_temporary: true,
                });

                math_emission.set_target(Some(target));

                Instruction::power(target.register, left_address, right_address, operand_type)
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
            _ => unreachable!("Expected binary expression, found {}", node.kind),
        };

        math_emission.push(math_instruction);
        math_emission.set_type(type_id);

        Ok(Emission::Instructions(math_emission))
    }

    fn compile_comparison_expression(
        &mut self,
        node: &SyntaxNode,
        target: Option<TargetRegister>,
    ) -> Result<Emission, CompileError> {
        info!("Compiling comparison expression");

        let left_id = SyntaxId(node.children.0);
        let left_node = *self
            .syntax_tree()?
            .nodes
            .get(left_id.0 as usize)
            .ok_or(CompileError::MissingSyntaxNode { syntax_id: left_id })?;

        let left_emission = self.compile_expression(left_id, &left_node, None)?;

        let right_id = SyntaxId(node.children.1);
        let right_node = *self.syntax_tree()?.nodes.get(right_id.0 as usize).ok_or(
            CompileError::MissingSyntaxNode {
                syntax_id: right_id,
            },
        )?;

        let right_emission = self.compile_expression(right_id, &right_node, None)?;

        if let (
            Emission::Constant(left_value, _left_type),
            Emission::Constant(right_value, _right_type),
        ) = (&left_emission, &right_emission)
        {
            let combined = self.combine_constants(
                *left_value,
                *right_value,
                node.kind,
                Position::new(self.file_id, node.span),
            )?;

            return Ok(Emission::Constant(combined, TypeId::BOOLEAN));
        }

        let mut comparison_emission = InstructionsEmission::new();

        let (left_address, left_type_id) =
            self.handle_operand_emission(&mut comparison_emission, left_emission, &left_node)?;
        let (right_address, right_type_id) =
            self.handle_operand_emission(&mut comparison_emission, right_emission, &right_node)?;

        if left_type_id != right_type_id {
            todo!("Error");
        }

        let target = target.unwrap_or_else(|| TargetRegister {
            register: self.allocate_temporary_register(),
            is_temporary: true,
        });
        let operand_type = self.context.resolver.get_operand_type(left_type_id).ok_or(
            CompileError::MissingType {
                type_id: left_type_id,
            },
        )?;
        let comparison_instruction = match node.kind {
            SyntaxKind::GreaterThanExpression => {
                Instruction::less_equal(false, left_address, right_address, operand_type)
            }
            SyntaxKind::GreaterThanOrEqualExpression => {
                Instruction::less(false, left_address, right_address, operand_type)
            }
            SyntaxKind::LessThanExpression => {
                Instruction::less(true, left_address, right_address, operand_type)
            }
            SyntaxKind::LessThanOrEqualExpression => {
                Instruction::less_equal(true, left_address, right_address, operand_type)
            }
            SyntaxKind::EqualExpression => {
                Instruction::equal(true, left_address, right_address, operand_type)
            }
            SyntaxKind::NotEqualExpression => {
                Instruction::equal(false, left_address, right_address, operand_type)
            }
            _ => unreachable!("Expected comparison expression, found {}", node.kind),
        };
        let load_false_instruction = Instruction::move_with_jump(
            target.register,
            Address::encoded(false as u16),
            OperandType::BOOLEAN,
            1,
            true,
        );
        let load_true_instruction = Instruction::r#move(
            target.register,
            Address::encoded(true as u16),
            OperandType::BOOLEAN,
        );

        comparison_emission.push(comparison_instruction);
        comparison_emission.push(load_false_instruction);
        comparison_emission.push(load_true_instruction);
        comparison_emission.set_type(TypeId::BOOLEAN);
        comparison_emission.set_target(Some(target));

        Ok(Emission::Instructions(comparison_emission))
    }

    fn compile_logic_expression(
        &mut self,
        node: &SyntaxNode,
        target: Option<TargetRegister>,
    ) -> Result<Emission, CompileError> {
        info!("Compiling logical expression");

        let left_id = SyntaxId(node.children.0);
        let left_node =
            *self
                .syntax_tree()?
                .get_node(left_id)
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: SyntaxId(node.children.0),
                })?;
        let left_emission = self.compile_expression(left_id, &left_node, None)?;

        let right_index = SyntaxId(node.children.1);
        let right_node =
            *self
                .syntax_tree()?
                .get_node(right_index)
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: SyntaxId(node.children.1),
                })?;
        let right_emission = self.compile_expression(right_index, &right_node, None)?;

        let comparator = match node.kind {
            SyntaxKind::AndExpression => false,
            SyntaxKind::OrExpression => true,
            _ => unreachable!("Expected logic expression, found {}", node.kind),
        };

        if let (
            Emission::Constant(left_value, left_type_id),
            Emission::Constant(right_value, _right_type_id),
        ) = (&left_emission, &right_emission)
        {
            let combined = self.combine_constants(
                *left_value,
                *right_value,
                node.kind,
                Position::new(self.file_id, node.span),
            )?;

            return Ok(Emission::Constant(combined, *left_type_id));
        }

        let mut logic_emission = InstructionsEmission::new();

        let (left_address, left_type) =
            self.handle_operand_emission(&mut logic_emission, left_emission, &left_node)?;
        let (right_address, right_type) =
            self.handle_operand_emission(&mut logic_emission, right_emission, &right_node)?;

        if left_type != TypeId::BOOLEAN {
            todo!("Error");
        }

        if right_type != TypeId::BOOLEAN {
            todo!("Error");
        }

        let (destination, is_temporary) = target
            .map(|target| (target.register, target.is_temporary))
            .unwrap_or_else(|| (self.allocate_temporary_register(), true));
        let test_instruction = Instruction::test(left_address, comparator, 1);
        let right_move_instruction =
            Instruction::move_with_jump(destination, right_address, OperandType::BOOLEAN, 1, true);
        let left_move_instruction =
            Instruction::r#move(destination, left_address, OperandType::BOOLEAN);

        logic_emission.push(test_instruction);
        logic_emission.push(right_move_instruction);
        logic_emission.push(left_move_instruction);
        logic_emission.set_type(TypeId::BOOLEAN);
        logic_emission.set_target(Some(TargetRegister {
            register: destination,
            is_temporary,
        }));

        Ok(Emission::Instructions(logic_emission))
    }

    fn compile_unary_expression(
        &mut self,
        node: &SyntaxNode,
        target: Option<TargetRegister>,
    ) -> Result<Emission, CompileError> {
        info!("Compiling unary expression");

        let operand_id = SyntaxId(node.children.0);
        let operand_node =
            *self
                .syntax_tree()?
                .get_node(operand_id)
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: operand_id,
                })?;
        let operand_emission = self.compile_expression(operand_id, &operand_node, None)?;

        if let Emission::Constant(child_value, child_type_id) = &operand_emission {
            let evaluated = match (node.kind, child_value) {
                (SyntaxKind::NotExpression, Constant::Boolean(value)) => Constant::Boolean(!value),
                (SyntaxKind::NegationExpression, Constant::Integer(value)) => {
                    Constant::Integer(-value)
                }
                (SyntaxKind::NegationExpression, Constant::Float(value)) => Constant::Float(-value),
                _ => todo!("Error"),
            };

            return Ok(Emission::Constant(evaluated, *child_type_id));
        }

        let mut unary_emission = InstructionsEmission::new();

        let (destination, is_temporary) = target
            .map(|target| (target.register, target.is_temporary))
            .unwrap_or_else(|| (self.allocate_temporary_register(), true));
        let (child_address, child_type_id) =
            self.handle_operand_emission(&mut unary_emission, operand_emission, &operand_node)?;
        let operand_type = self
            .context
            .resolver
            .get_operand_type(child_type_id)
            .ok_or(CompileError::MissingType {
                type_id: child_type_id,
            })?;
        let negate_instruction = Instruction::negate(destination, child_address, operand_type);

        unary_emission.push(negate_instruction);
        unary_emission.set_type(child_type_id);
        unary_emission.set_target(Some(TargetRegister {
            register: destination,
            is_temporary,
        }));

        Ok(Emission::Instructions(unary_emission))
    }

    fn compile_grouped_expression(
        &mut self,
        node: &SyntaxNode,
        target: Option<TargetRegister>,
    ) -> Result<Emission, CompileError> {
        info!("Compiling grouped expression");

        let child_id = SyntaxId(node.children.0);
        let child_node =
            *self
                .syntax_tree()?
                .get_node(child_id)
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: child_id,
                })?;

        self.compile_expression(child_id, &child_node, target)
    }

    fn compile_block_expression(
        &mut self,
        node_id: SyntaxId,
        node: &SyntaxNode,
        target: Option<TargetRegister>,
    ) -> Result<Emission, CompileError> {
        info!("Compiling block expression");

        let start_children = node.children.0 as usize;
        let child_count = node.children.1 as usize;

        if child_count == 0 {
            return Ok(Emission::None);
        }

        let parent_scope_id = self.current_scope_id;
        let parent_next_local_register = self.next_local_register;

        let child_scope_id = *self
            .context
            .resolver
            .get_scope_binding(&node_id)
            .ok_or(CompileError::MissingScopeBinding { syntax_id: node_id })?;

        self.enter_child_scope(child_scope_id);

        let mut block_emission = InstructionsEmission::new();

        let end_children = start_children + child_count;
        let mut current_child_index = start_children;

        while current_child_index < end_children - 1 {
            let child_id = *self
                .syntax_tree()?
                .children
                .get(current_child_index)
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: current_child_index as u32,
                })?;
            let child_node =
                *self
                    .syntax_tree()?
                    .get_node(child_id)
                    .ok_or(CompileError::MissingSyntaxNode {
                        syntax_id: child_id,
                    })?;
            current_child_index += 1;

            let statment_emission = self.compile_statement(&child_node)?;

            if let Emission::Instructions(InstructionsEmission { instructions, .. }) =
                statment_emission
            {
                block_emission.instructions.extend(instructions);
            }
        }

        let final_node_id = *self.syntax_tree()?.children.get(end_children - 1).ok_or(
            CompileError::MissingChild {
                parent_kind: node.kind,
                child_index: end_children as u32,
            },
        )?;
        let final_node = *self.syntax_tree()?.get_node(final_node_id).ok_or(
            CompileError::MissingSyntaxNode {
                syntax_id: final_node_id,
            },
        )?;

        if final_node.kind.is_item() {
            self.compile_item(&final_node)?;
            self.enter_parent_scope(parent_scope_id, parent_next_local_register);
            block_emission.add_drop(self, block_emission.target_register);

            return Ok(Emission::Instructions(block_emission));
        }

        if final_node.kind.is_statement() {
            let statement_emission = self.compile_statement(&final_node)?;

            if let Emission::Instructions(instructions) = statement_emission {
                block_emission.merge(instructions);
            }

            self.enter_parent_scope(parent_scope_id, parent_next_local_register);
            block_emission.add_drop(self, block_emission.target_register);

            return Ok(Emission::Instructions(block_emission));
        }

        let final_expression_emission =
            self.compile_expression(final_node_id, &final_node, target)?;

        match final_expression_emission {
            Emission::Constant(constant, type_id) => {
                if block_emission.is_empty() {
                    return Ok(Emission::Constant(constant, type_id));
                }

                let target = target.unwrap_or_else(|| TargetRegister {
                    register: self.allocate_temporary_register(),
                    is_temporary: true,
                });
                let address = self.get_constant_address(constant);
                let operand_type = self
                    .context
                    .resolver
                    .get_operand_type(type_id)
                    .ok_or(CompileError::MissingType { type_id })?;
                let move_instruction = Instruction::r#move(target.register, address, operand_type);

                block_emission.push(move_instruction);
                block_emission.set_type(type_id);
                block_emission.set_target(Some(target));
            }
            Emission::Function(address, type_id) => {
                if block_emission.is_empty() {
                    return Ok(Emission::Function(address, type_id));
                }

                let target = target.unwrap_or_else(|| TargetRegister {
                    register: self.allocate_temporary_register(),
                    is_temporary: true,
                });
                let operand_type = self
                    .context
                    .resolver
                    .get_operand_type(type_id)
                    .ok_or(CompileError::MissingType { type_id })?;
                let move_instruction = Instruction::r#move(target.register, address, operand_type);

                block_emission.push(move_instruction);
                block_emission.set_type(type_id);
                block_emission.set_target(Some(target));
            }
            Emission::Local(Local { address, type_id }) => {
                if block_emission.is_empty() {
                    return Ok(Emission::Local(Local { address, type_id }));
                }

                let target = target.unwrap_or(TargetRegister {
                    register: address.index,
                    is_temporary: false,
                });

                block_emission.set_type(type_id);
                block_emission.set_target(Some(target));
            }
            Emission::Instructions(instructions) => {
                block_emission.merge(instructions);
            }
            Emission::None => {}
        }

        self.enter_parent_scope(parent_scope_id, parent_next_local_register);
        block_emission.add_drop(self, block_emission.target_register);

        Ok(Emission::Instructions(block_emission))
    }

    fn compile_path_expression(
        &mut self,
        node: &SyntaxNode,
        target: Option<TargetRegister>,
    ) -> Result<Emission, CompileError> {
        info!("Compiling path expression");

        let files = self.context.source.read_files();
        let source_file =
            files
                .get(self.file_id.0 as usize)
                .ok_or(CompileError::MissingSourceFile {
                    file_id: self.file_id,
                })?;
        let variable_name_bytes =
            &source_file.source_code.as_ref()[node.span.0 as usize..node.span.1 as usize];
        let variable_name = unsafe { str::from_utf8_unchecked(variable_name_bytes) };

        let declaration_id = if let Some((declaration_id, _)) = self
            .context
            .resolver
            .find_declaration_in_scope(variable_name, self.current_scope_id)
        {
            declaration_id
        } else {
            let (declaration_id, declaration) = *self
                .context
                .resolver
                .find_declarations(variable_name)
                .ok_or(CompileError::UndeclaredVariable {
                    name: variable_name.to_string(),
                    position: Position::new(self.file_id, node.span),
                })?
                .first()
                .ok_or(CompileError::UndeclaredVariable {
                    name: variable_name.to_string(),
                    position: Position::new(self.file_id, node.span),
                })?;

            if self.declaration_id.is_some_and(|id| id == declaration_id) {
                return Ok(Emission::Function(
                    Address::register(self.prototype_index),
                    declaration.type_id,
                ));
            }

            declaration_id
        };

        let Some(local) = self.locals.get(&declaration_id).cloned() else {
            return Err(CompileError::UndeclaredVariable {
                name: variable_name.to_string(),
                position: Position::new(self.file_id, node.span),
            });
        };

        if let Some(target) = target
            && target.register != local.address.index
        {
            let operand_type = self
                .context
                .resolver
                .get_operand_type(local.type_id)
                .ok_or(CompileError::MissingType {
                    type_id: local.type_id,
                })?;
            let move_instruction =
                Instruction::r#move(target.register, local.address, operand_type);

            let mut path_emission = InstructionsEmission::with_instruction(move_instruction);

            path_emission.set_type(local.type_id);
            path_emission.set_target(Some(target));

            return Ok(Emission::Instructions(path_emission));
        }

        Ok(Emission::Local(local))
    }

    fn compile_while_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling while expression");

        let condition_id = SyntaxId(node.children.0);
        let condition_node =
            *self
                .syntax_tree()?
                .get_node(condition_id)
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: condition_id,
                })?;

        let body_id = SyntaxId(node.children.1);
        let body_node = *self
            .syntax_tree()?
            .get_node(body_id)
            .ok_or(CompileError::MissingSyntaxNode { syntax_id: body_id })?;

        let mut while_emission = InstructionsEmission::new();
        let condition_emission = self.compile_expression(condition_id, &condition_node, None)?;

        self.handle_condition_emission(&mut while_emission, condition_emission, &condition_node)?;

        let jump_forward_id = self.create_jump_id();
        let jump_backward_id = self.create_jump_id();

        while_emission
            .instructions
            .last_mut()
            .unwrap()
            .1
            .push(JumpAnchor::LoopStartHere {
                forward_id: jump_forward_id,
            });

        let body_emission = self.compile_expression_statement(&body_node)?;

        match body_emission {
            Emission::Local(Local { address, type_id }) => {
                let destination = self.allocate_temporary_register();
                let operand_type = self
                    .context
                    .resolver
                    .get_operand_type(type_id)
                    .ok_or(CompileError::MissingType { type_id })?;
                let move_instruction = Instruction::r#move(destination, address, operand_type);

                while_emission.push(move_instruction);
            }
            Emission::Constant(constant, type_id) => {
                let destination = self.allocate_temporary_register();
                let address = self.get_constant_address(constant);
                let operand_type = self
                    .context
                    .resolver
                    .get_operand_type(type_id)
                    .ok_or(CompileError::MissingType { type_id })?;
                let move_instruction = Instruction::r#move(destination, address, operand_type);

                while_emission.push(move_instruction);
            }
            Emission::Function(address, type_id) => {
                let destination = self.allocate_temporary_register();
                let operand_type = self
                    .context
                    .resolver
                    .get_operand_type(type_id)
                    .ok_or(CompileError::MissingType { type_id })?;
                let move_instruction = Instruction::r#move(destination, address, operand_type);

                while_emission.push(move_instruction);
            }
            Emission::Instructions(InstructionsEmission { instructions, .. }) => {
                while_emission.instructions.extend(instructions);
            }
            Emission::None => {}
        }

        while_emission
            .instructions
            .last_mut()
            .unwrap()
            .1
            .push(JumpAnchor::LoopEndOnNext {
                forward_id: jump_forward_id,
                backward_id: jump_backward_id,
            });

        Ok(Emission::Instructions(while_emission))
    }

    fn compile_function_expression(
        &mut self,
        node: &SyntaxNode,
        declaration_info: Option<(DeclarationId, Declaration)>,
        bound_type: Option<FunctionTypeNode>,
    ) -> Result<Emission, CompileError> {
        info!("Compiling function expression");

        let function_type_node = if let Some(function_type) = bound_type {
            function_type
        } else {
            let function_signature_id = SyntaxId(node.children.0);
            let function_signature_node = *self
                .syntax_tree()?
                .get_node(function_signature_id)
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: function_signature_id,
                })?;

            debug_assert_eq!(function_signature_node.kind, SyntaxKind::FunctionSignature);

            let value_parameter_list_id = SyntaxId(function_signature_node.children.0);
            let value_parameter_list_node = *self
                .syntax_tree()?
                .get_node(value_parameter_list_id)
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: value_parameter_list_id,
                })?;

            debug_assert_eq!(
                value_parameter_list_node.kind,
                SyntaxKind::ValueParametersDefinition
            );

            let function_scope = self.context.resolver.add_scope(Scope {
                kind: ScopeKind::Function,
                parent: self.current_scope_id,
                imports: SmallVec::new(),
                modules: SmallVec::new(),
            });
            let value_parameter_node_ids = self
                .syntax_tree()?
                .get_children(
                    value_parameter_list_node.children.0,
                    value_parameter_list_node.children.1,
                )
                .ok_or(CompileError::MissingChildren {
                    parent_kind: value_parameter_list_node.kind,
                    start_index: value_parameter_list_node.children.0,
                    count: value_parameter_list_node.children.1,
                })?;
            let value_parameter_nodes = value_parameter_node_ids
                .iter()
                .map(|&id| {
                    self.syntax_tree()?
                        .get_node(id)
                        .ok_or(CompileError::MissingSyntaxNode { syntax_id: id })
                        .copied()
                })
                .collect::<Result<SmallVec<[SyntaxNode; 4]>, CompileError>>()?;

            let files = &self.context.source.read_files();
            let file =
                files
                    .get(self.file_id.0 as usize)
                    .ok_or(CompileError::MissingSourceFile {
                        file_id: self.file_id,
                    })?;

            let mut value_parameters = SmallVec::<[TypeId; 8]>::new();
            let mut current_parameter_name = "";

            for (index, node) in value_parameter_nodes.iter().enumerate() {
                let is_name = index % 2 == 0;

                if is_name {
                    current_parameter_name = unsafe {
                        str::from_utf8_unchecked(
                            &file.source_code.as_ref()[node.span.0 as usize..node.span.1 as usize],
                        )
                    };
                } else {
                    let type_id = match node.kind {
                        SyntaxKind::BooleanType => Type::Boolean,
                        SyntaxKind::ByteType => Type::Byte,
                        SyntaxKind::CharacterType => Type::Character,
                        SyntaxKind::FloatType => Type::Float,
                        SyntaxKind::IntegerType => Type::Integer,
                        SyntaxKind::StringType => Type::String,
                        _ => {
                            todo!()
                        }
                    };
                    let type_id = self.context.resolver.add_type(&type_id);
                    let parameter_declaration = Declaration {
                        kind: DeclarationKind::Local { shadowed: None },
                        scope_id: function_scope,
                        type_id,
                        position: Position::new(self.file_id, node.span),
                        is_public: false,
                    };

                    self.context
                        .resolver
                        .add_declaration(current_parameter_name, parameter_declaration);
                    value_parameters.push(type_id);
                }
            }

            let value_parameters = self.context.resolver.add_type_members(&value_parameters);

            let return_type_node_id = SyntaxId(function_signature_node.children.1);
            let return_type_id = if return_type_node_id == SyntaxId::NONE {
                TypeId::NONE
            } else {
                let function_return_type_node = *self
                    .syntax_tree()?
                    .get_node(return_type_node_id)
                    .ok_or(CompileError::MissingSyntaxNode {
                        syntax_id: return_type_node_id,
                    })?;

                match function_return_type_node.kind {
                    SyntaxKind::BooleanType => TypeId::BOOLEAN,
                    SyntaxKind::ByteType => TypeId::BYTE,
                    SyntaxKind::CharacterType => TypeId::CHARACTER,
                    SyntaxKind::FloatType => TypeId::FLOAT,
                    SyntaxKind::IntegerType => TypeId::INTEGER,
                    SyntaxKind::StringType => TypeId::STRING,
                    _ => {
                        todo!()
                    }
                }
            };

            FunctionTypeNode {
                value_parameters,
                type_parameters: (0, 0),
                return_type: return_type_id,
            }
        };

        let body_id = SyntaxId(node.children.1);
        let body_node = *self
            .syntax_tree()?
            .get_node(body_id)
            .ok_or(CompileError::MissingSyntaxNode { syntax_id: body_id })?;
        let prototype_index_usize = self.context.prototypes.len();
        let prototype_index_u16 = prototype_index_usize as u16;

        self.context.prototypes.push(Prototype::default());

        let (new_declaration_info, function_scope_id) =
            if let Some((declaration_id, mut declaration)) = declaration_info {
                let (new_declaration_kind, scope_id) = match declaration.kind {
                    DeclarationKind::Function {
                        inner_scope_id,
                        syntax_id,
                        parameters,
                        ..
                    } => (
                        DeclarationKind::Function {
                            inner_scope_id,
                            syntax_id,
                            file_id: self.file_id,
                            parameters,
                            prototype_index: Some(prototype_index_u16),
                        },
                        inner_scope_id,
                    ),
                    _ => todo!("Error"),
                };

                declaration.kind = new_declaration_kind;

                *self
                    .context
                    .resolver
                    .get_declaration_mut(&declaration_id)
                    .unwrap() = declaration;

                (Some((declaration_id, declaration)), scope_id)
            } else {
                let function_scope = *self
                    .context
                    .resolver
                    .get_scope_binding(&body_id)
                    .ok_or(CompileError::MissingScopeBinding { syntax_id: body_id })?;

                (None, function_scope)
            };
        let mut function_compiler = PrototypeCompiler::new(
            new_declaration_info,
            prototype_index_u16,
            self.file_id,
            function_type_node,
            self.context,
            function_scope_id,
        );

        function_compiler.compile_function(&body_node)?;

        let function_prototype = function_compiler.finish(prototype_index_u16)?;
        let address = Address::constant(prototype_index_usize as u16);
        let type_id = self
            .context
            .resolver
            .add_type_node(TypeNode::Function(function_type_node));

        self.context.prototypes[prototype_index_usize] = function_prototype;

        Ok(Emission::Function(address, type_id))
    }

    fn compile_call_expression(
        &mut self,
        node: &SyntaxNode,
        target: Option<TargetRegister>,
    ) -> Result<Emission, CompileError> {
        fn handle_call_arguments(
            compiler: &mut PrototypeCompiler,
            instructions_emission: &mut InstructionsEmission,
            arguments_node: &SyntaxNode,
        ) -> Result<(), CompileError> {
            debug_assert_eq!(arguments_node.kind, SyntaxKind::CallValueArguments);

            let (start_children, child_count) = (
                arguments_node.children.0 as usize,
                arguments_node.children.1 as usize,
            );
            let end_children = start_children + child_count;
            let mut current_child_index = start_children;

            while current_child_index < end_children {
                let argument_id = *compiler
                    .syntax_tree()?
                    .children
                    .get(current_child_index)
                    .ok_or(CompileError::MissingChild {
                        parent_kind: arguments_node.kind,
                        child_index: current_child_index as u32,
                    })?;
                let argument_node = *compiler.syntax_tree()?.get_node(argument_id).ok_or(
                    CompileError::MissingSyntaxNode {
                        syntax_id: argument_id,
                    },
                )?;
                current_child_index += 1;

                let argument_emission =
                    compiler.compile_expression(argument_id, &argument_node, None)?;
                let (argument_address, argument_type_id) = compiler.handle_operand_emission(
                    instructions_emission,
                    argument_emission,
                    &argument_node,
                )?;

                let operand_type = compiler
                    .context
                    .resolver
                    .get_operand_type(argument_type_id)
                    .ok_or(CompileError::MissingType {
                        type_id: argument_type_id,
                    })?;

                compiler
                    .call_arguments
                    .push((argument_address, operand_type));
            }

            Ok(())
        }

        info!("Compiling call expression");

        let function_node_id = SyntaxId(node.children.0);
        let function_node = *self.syntax_tree()?.get_node(function_node_id).ok_or(
            CompileError::MissingSyntaxNode {
                syntax_id: function_node_id,
            },
        )?;

        let arguments_node_id = SyntaxId(node.children.1);
        let arguments_node = *self.syntax_tree()?.get_node(arguments_node_id).ok_or(
            CompileError::MissingSyntaxNode {
                syntax_id: arguments_node_id,
            },
        )?;

        debug_assert_eq!(arguments_node.kind, SyntaxKind::CallValueArguments);

        let mut call_emission = InstructionsEmission::new();

        if function_node.kind == SyntaxKind::PathExpression {
            let files = self.context.source.read_files();
            let source_file =
                files
                    .get(self.file_id.0 as usize)
                    .ok_or(CompileError::MissingSourceFile {
                        file_id: self.file_id,
                    })?;
            let name_bytes = &source_file.source_code.as_ref()
                [function_node.span.0 as usize..function_node.span.1 as usize];
            let name = unsafe { str::from_utf8_unchecked(name_bytes) };

            if let Some((_, declaration)) = self
                .context
                .resolver
                .find_declaration_in_scope(name, ScopeId::NATIVE)
                && declaration.kind == DeclarationKind::NativeFunction
            {
                let native_function =
                    NativeFunction::from_str(name).ok_or(CompileError::MissingNativeFunction {
                        native_function: name.to_string(),
                    })?;

                drop(files);

                let arguments_start_index = self.call_arguments.len() as u16;

                handle_call_arguments(self, &mut call_emission, &arguments_node)?;

                let destination = target
                    .map(|target| target.register)
                    .unwrap_or_else(|| self.allocate_temporary_register());
                let call_native_instruction =
                    Instruction::call_native(destination, native_function, arguments_start_index);
                let return_type = self
                    .context
                    .resolver
                    .get_type_node(declaration.type_id)
                    .ok_or(CompileError::MissingType {
                        type_id: declaration.type_id,
                    })?
                    .into_function_type()
                    .ok_or(CompileError::ExpectedFunction {
                        node_kind: function_node.kind,
                        position: Position::new(self.file_id, function_node.span),
                    })?
                    .return_type;

                call_emission.push(call_native_instruction);
                call_emission.set_type(return_type);

                return Ok(Emission::Instructions(call_emission));
            }
        }

        let function_emission = self.compile_expression(function_node_id, &function_node, None)?;
        let (function_address, callee_type_id) =
            self.handle_operand_emission(&mut call_emission, function_emission, &function_node)?;

        let arguments_start_index = self.call_arguments.len() as u16;

        handle_call_arguments(self, &mut call_emission, &arguments_node)?;

        let target = target.unwrap_or_else(|| TargetRegister {
            register: self.allocate_temporary_register(),
            is_temporary: true,
        });
        let callee_type_node = self.context.resolver.get_type_node(callee_type_id).ok_or(
            CompileError::MissingType {
                type_id: callee_type_id,
            },
        )?;

        if !matches!(callee_type_node, TypeNode::Function(_)) {
            return Err(CompileError::ExpectedFunction {
                node_kind: function_node.kind,
                position: Position::new(self.file_id, function_node.span),
            });
        }

        let return_type_id = callee_type_node
            .into_function_type()
            .ok_or(CompileError::ExpectedFunction {
                node_kind: function_node.kind,
                position: Position::new(self.file_id, function_node.span),
            })?
            .return_type;
        let argument_count = self.call_arguments.len() as u16 - arguments_start_index;

        let destination = if return_type_id == TypeId::NONE {
            None
        } else {
            Some(target.register)
        };
        let call_instruction = Instruction::call(
            destination,
            function_address,
            arguments_start_index,
            argument_count,
        );

        call_emission.push(call_instruction);
        call_emission.set_type(return_type_id);
        call_emission.set_target(Some(target));

        Ok(Emission::Instructions(call_emission))
    }

    fn compile_as_expression(
        &mut self,
        node: &SyntaxNode,
        target: Option<TargetRegister>,
    ) -> Result<Emission, CompileError> {
        info!("Compiling 'as' expression");

        let value_node_id = SyntaxId(node.children.0);
        let value_node = *self.syntax_tree()?.get_node(value_node_id).ok_or(
            CompileError::MissingSyntaxNode {
                syntax_id: value_node_id,
            },
        )?;

        let type_node_id = SyntaxId(node.children.1);
        let type_node =
            *self
                .syntax_tree()?
                .get_node(type_node_id)
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: type_node_id,
                })?;

        let mut instructions_emission = InstructionsEmission::new();

        let value_emission = self.compile_expression(value_node_id, &value_node, None)?;
        let (value_address, value_type_id) =
            self.handle_operand_emission(&mut instructions_emission, value_emission, &value_node)?;
        let target = target.unwrap_or_else(|| TargetRegister {
            register: self.allocate_temporary_register(),
            is_temporary: true,
        });
        let (convert_type_instruction, target_type_id) = match type_node.kind {
            SyntaxKind::StringType => {
                let operand_type = self.get_operand_type(value_type_id)?;
                let instruction =
                    Instruction::to_string(target.register, value_address, operand_type);

                (instruction, TypeId::STRING)
            }
            _ => {
                todo!()
            }
        };

        instructions_emission.push(convert_type_instruction);
        instructions_emission.set_type(target_type_id);
        instructions_emission.set_target(Some(target));

        Ok(Emission::Instructions(instructions_emission))
    }

    fn compile_if_expression(
        &mut self,
        node: &SyntaxNode,
        target: Option<TargetRegister>,
    ) -> Result<Emission, CompileError> {
        info!("Compiling if expression");

        let child_ids = self
            .syntax_tree()?
            .get_children(node.children.0, node.children.1)
            .ok_or(CompileError::MissingChildren {
                parent_kind: node.kind,
                start_index: node.children.0,
                count: node.children.1,
            })?
            .iter()
            .cloned()
            .collect::<SmallVec<[SyntaxId; 3]>>();

        let condition_id = child_ids[0];
        let condition_node =
            *self
                .syntax_tree()?
                .get_node(condition_id)
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: condition_id,
                })?;

        let then_block_id = child_ids[1];
        let then_block_node = *self.syntax_tree()?.get_node(then_block_id).ok_or(
            CompileError::MissingSyntaxNode {
                syntax_id: then_block_id,
            },
        )?;

        let target = target.unwrap_or_else(|| TargetRegister {
            register: self.allocate_temporary_register(),
            is_temporary: true,
        });

        let condition_emission = self.compile_expression(condition_id, &condition_node, None)?;
        let mut if_emission = InstructionsEmission::new();

        self.handle_condition_emission(&mut if_emission, condition_emission, &condition_node)?;

        let jump_over_then_id = self.create_jump_id();

        if_emission
            .instructions
            .last_mut()
            .unwrap()
            .1
            .push(JumpAnchor::ForwardFromHere {
                id: jump_over_then_id,
            });

        let start_else_anchor_count = self.jump_over_else_anchor_ids.len();
        let then_emission = match then_block_node.kind {
            SyntaxKind::BlockExpression => {
                self.compile_block_expression(then_block_id, &then_block_node, Some(target))?
            }
            SyntaxKind::ExpressionStatement => {
                self.compile_expression_statement(&then_block_node)?
            }
            _ => {
                todo!("Error")
            }
        };
        let then_type_id =
            self.handle_branch_emission(&mut if_emission, then_emission, target.register)?;

        if_emission
            .instructions
            .last_mut()
            .unwrap()
            .1
            .push(JumpAnchor::ForwardToNext {
                id: jump_over_then_id,
            });

        if child_ids.len() == 3 {
            let jump_over_else_id = self.create_jump_id();

            if_emission
                .instructions
                .last_mut()
                .unwrap()
                .1
                .push(JumpAnchor::ForwardFromHere {
                    id: jump_over_else_id,
                });
            self.jump_over_else_anchor_ids.push(jump_over_else_id);

            let else_block_id = child_ids[2];
            let else_block_node = *self.syntax_tree()?.get_node(else_block_id).ok_or(
                CompileError::MissingSyntaxNode {
                    syntax_id: else_block_id,
                },
            )?;
            let else_emission = self.compile_else_expression(&else_block_node, Some(target))?;
            let else_type_id =
                self.handle_branch_emission(&mut if_emission, else_emission, target.register)?;

            if then_type_id != else_type_id {
                let if_type = self.context.resolver.resolve_type(then_type_id).ok_or(
                    CompileError::MissingType {
                        type_id: then_type_id,
                    },
                )?;
                let else_type = self.context.resolver.resolve_type(else_type_id).ok_or(
                    CompileError::MissingType {
                        type_id: else_type_id,
                    },
                )?;

                return Err(CompileError::MismatchedIfElseTypes {
                    if_type,
                    else_type,
                    position: Position::new(self.file_id, node.span),
                });
            }
        }

        let end_else_anchor_count = self.jump_over_else_anchor_ids.len();

        for index in start_else_anchor_count..end_else_anchor_count {
            let jump_over_else_id = self.jump_over_else_anchor_ids[index];

            if_emission
                .instructions
                .last_mut()
                .unwrap()
                .1
                .push(JumpAnchor::ForwardToNext {
                    id: jump_over_else_id,
                });
        }

        if_emission.set_type(then_type_id);
        if_emission.set_target(Some(target));

        Ok(Emission::Instructions(if_emission))
    }

    fn compile_else_expression(
        &mut self,
        node: &SyntaxNode,
        target: Option<TargetRegister>,
    ) -> Result<Emission, CompileError> {
        info!("Compiling else expression");

        let child_id = SyntaxId(node.children.0);
        let child_node =
            *self
                .syntax_tree()?
                .get_node(child_id)
                .ok_or(CompileError::MissingSyntaxNode {
                    syntax_id: child_id,
                })?;

        match child_node.kind {
            SyntaxKind::IfExpression => self.compile_if_expression(&child_node, target),
            SyntaxKind::BlockExpression => {
                self.compile_block_expression(child_id, &child_node, target)
            }
            SyntaxKind::ExpressionStatement => self.compile_expression_statement(&child_node),
            _ => {
                todo!("Error")
            }
        }
    }

    fn compile_implicit_return(
        &mut self,
        node_id: SyntaxId,
        node: &SyntaxNode,
    ) -> Result<Emission, CompileError> {
        let mut return_emission = InstructionsEmission::new();

        if node.kind.is_item() {
            self.compile_item(node)?;

            let return_instruction = Instruction::r#return(Address::default(), OperandType::NONE);

            return_emission.push(return_instruction);
        } else if node.kind.is_statement() {
            let statement_emission = self.compile_statement(node)?;

            self.handle_return_emission(&mut return_emission, statement_emission, node)?;
        } else {
            let expression_emission = self.compile_expression(node_id, node, None)?;

            self.handle_return_emission(&mut return_emission, expression_emission, node)?;
        }

        Ok(Emission::Instructions(return_emission))
    }
}

#[derive(Clone, Debug)]
enum Emission {
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
struct InstructionsEmission {
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

    fn add_drop(
        &mut self,
        compiler: &mut PrototypeCompiler,
        target_register: Option<TargetRegister>,
    ) {
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
enum Constant {
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
struct Local {
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
