use std::collections::HashMap;

use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;
use smallvec::SmallVec;
use tracing::{Level, debug, info, span};

use crate::{
    Address, Chunk, CompileError, ConstantTable, Instruction, NativeFunction, OperandType,
    Operation, Position, Resolver, Source, Type,
    resolver::{
        DeclarationId, DeclarationKind, FunctionTypeNode, ScopeId, ScopeKind, TypeId, TypeNode,
    },
    source::SourceFile,
    syntax_tree::{SyntaxId, SyntaxKind, SyntaxNode, SyntaxTree},
};

#[derive(Debug)]
pub struct ChunkCompiler<'a> {
    /// Source code for the program being compiled.
    source: &'a Source,

    /// Target syntax tree for compilation.
    syntax_tree: &'a SyntaxTree,

    syntax_trees: &'a HashMap<DeclarationId, SyntaxTree, FxBuildHasher>,

    /// Global context for declaration, scope and type resolution.
    resolver: &'a mut Resolver,

    /// Target source code file for compilation by this chunk compiler.
    source_file: SourceFile,

    /// Constant collection that is filled during compilation after constant expression folding is
    /// applied.
    constants: &'a mut ConstantTable,

    /// Global list of function prototypes that is filled during compilation.
    prototypes: &'a mut IndexMap<DeclarationId, Chunk, FxBuildHasher>,

    /// Bytecode instruction collection that is filled during compilation.
    instructions: Vec<Instruction>,

    /// Concatenated list of arguments referenced by CALL instructions.
    call_arguments: Vec<(Address, OperandType)>,

    /// Concatenated list of register indexes that are referenced by DROP instructions.
    drop_lists: Vec<u16>,

    /// Type of the function being compiled. This field is modified when emitting RETURN
    /// instructions. Subsequent RETURN instructions must match the type of the function.
    r#type: FunctionTypeNode,

    /// Local variables declared in the function being compiled.
    locals: HashMap<DeclarationId, Local, FxBuildHasher>,

    /// Lowest register index after registers have been allocated for function arguments.
    minimum_register: u16,
}

impl<'a> ChunkCompiler<'a> {
    pub fn new(
        syntax_trees: &'a HashMap<DeclarationId, SyntaxTree, FxBuildHasher>,
        syntax_tree: &'a SyntaxTree,
        resolver: &'a mut Resolver,
        constants: &'a mut ConstantTable,
        source: &'a Source,
        source_file: SourceFile,
        prototypes: &'a mut IndexMap<DeclarationId, Chunk, FxBuildHasher>,
    ) -> Self {
        Self {
            source_file,
            syntax_tree,
            syntax_trees,
            resolver,
            source,
            constants,
            prototypes,
            instructions: Vec::new(),
            call_arguments: Vec::new(),
            drop_lists: Vec::new(),
            r#type: FunctionTypeNode {
                type_parameters: (0, 0),
                value_parameters: (0, 0),
                return_type: TypeId::NONE,
            },
            locals: HashMap::default(),
            minimum_register: 0,
        }
    }

    pub fn compile(mut self) -> Result<Chunk, CompileError> {
        let span = span!(Level::INFO, "Compiling");
        let _enter = span.enter();

        let root_node = *self
            .syntax_tree
            .get_node(SyntaxId(0))
            .ok_or(CompileError::MissingSyntaxNode { id: SyntaxId(0) })?;

        self.compile_item(&root_node)?;
        self.finish((0, 0))
    }

    pub fn finish(mut self, value_parameters: (u32, u32)) -> Result<Chunk, CompileError> {
        self.constants.finalize_string_pool();

        self.r#type.value_parameters = value_parameters;

        let r#type = self.resolver.add_type(TypeNode::Function(self.r#type));
        let register_count = self.get_next_register();

        Ok(Chunk {
            name: self.source_file.name.clone(),
            r#type,
            instructions: self.instructions,
            call_arguments: self.call_arguments,
            drop_lists: self.drop_lists,
            register_count,
            is_recursive: false,
        })
    }

    fn get_next_register(&mut self) -> u16 {
        self.instructions
            .iter()
            .fold(self.minimum_register, |acc, instruction| {
                if instruction.yields_value() {
                    acc.max(instruction.destination().index + 1)
                } else {
                    acc
                }
            })
    }

    fn get_constant_address(&mut self, constant: Constant) -> Address {
        let index = match constant {
            Constant::Boolean(boolean) => return Address::encoded(boolean as u16),
            Constant::Byte(byte) => return Address::encoded(byte as u16),
            Constant::Character(character) => self.constants.add_character(character),
            Constant::Float(float) => self.constants.add_float(float),
            Constant::Integer(integer) => self.constants.add_integer(integer),
            Constant::String {
                pool_start,
                pool_end,
            } => self.constants.add_pooled_string(pool_start, pool_end),
            Constant::Function {
                prototype_index, ..
            } => prototype_index,
            Constant::NativeFunction {
                native_function, ..
            } => native_function.index,
        };

        Address::constant(index)
    }

    fn combine_constants(
        &mut self,
        left: Constant,
        right: Constant,
        operation: SyntaxKind,
    ) -> Result<Constant, CompileError> {
        debug!(
            "Combining constants: {:?} {:?} {:?}",
            left, right, operation
        );

        let combined = match (left, right) {
            (Constant::Boolean(left), Constant::Boolean(right)) => match operation {
                SyntaxKind::AndExpression => Constant::Boolean(left && right),
                SyntaxKind::OrExpression => Constant::Boolean(left || right),
                SyntaxKind::GreaterThanExpression => Constant::Boolean(left && right),
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
                SyntaxKind::DivisionExpression => Constant::Byte(left.saturating_div(right)),
                SyntaxKind::ModuloExpression => Constant::Byte(left % right),
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
                SyntaxKind::DivisionExpression => Constant::Float(left / right),
                SyntaxKind::ModuloExpression => Constant::Float(left % right),
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
                SyntaxKind::DivisionExpression => Constant::Integer(left.saturating_div(right)),
                SyntaxKind::ModuloExpression => Constant::Integer(left % right),
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

                    let combined = self.constants.push_str_to_string_pool(&string);

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
                    .constants
                    .get_string_pool_range(left_pool_start as usize..left_pool_end as usize);
                let right = self
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

                        let combined = self.constants.push_str_to_string_pool(&string);

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
                    .constants
                    .get_string_pool_range(pool_start as usize..pool_end as usize);
                let mut string = String::with_capacity(1 + right.len());

                string.push(left);
                string.push_str(right);

                let combined = match operation {
                    SyntaxKind::AdditionExpression => {
                        self.constants.push_str_to_string_pool(&string)
                    }
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
                    .constants
                    .get_string_pool_range(pool_start as usize..pool_end as usize);
                let mut string = String::with_capacity(left.len() + 1);

                string.push_str(left);
                string.push(right);

                let combined = match operation {
                    SyntaxKind::AdditionExpression => {
                        self.constants.push_str_to_string_pool(&string)
                    }
                    _ => todo!("Error"),
                };

                Constant::String {
                    pool_start: combined.0,
                    pool_end: combined.1,
                }
            }
            _ => todo!(),
        };

        Ok(combined)
    }

    fn handle_operand(&mut self, instruction: Instruction) -> Address {
        match instruction.operation() {
            Operation::LOAD => instruction.b_address(),
            _ => {
                self.instructions.push(instruction);

                instruction.destination()
            }
        }
    }

    fn compile_item(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        match node.kind {
            SyntaxKind::MainFunctionItem => self.compile_main_function_statement(node),
            SyntaxKind::ModuleItem => self.compile_module_item(node),
            SyntaxKind::FunctionStatement => self.compile_function_statement(node),
            SyntaxKind::UseItem => self.compile_use_item(node),
            _ => Err(CompileError::ExpectedItem {
                node_kind: node.kind,
                position: Position::new(self.syntax_tree.file_index, node.span),
            }),
        }
    }

    fn compile_statement(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        match node.kind {
            SyntaxKind::FunctionStatement => self.compile_function_statement(node),
            SyntaxKind::ExpressionStatement => self.compile_expression_statement(node),
            SyntaxKind::LetStatement | SyntaxKind::LetMutStatement => {
                self.compile_let_statement(node)
            }
            SyntaxKind::ReassignStatement => self.compile_reassign_statement(node),
            SyntaxKind::SemicolonStatement => todo!("Compile semicolon statement"),
            _ => Err(CompileError::ExpectedStatement {
                node_kind: node.kind,
                position: Position::new(self.syntax_tree.file_index, node.span),
            }),
        }
    }

    fn compile_main_function_statement(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Compiling main function");

        let (start_children, child_count) = (node.children.0 as usize, node.children.1 as usize);
        let end_children = start_children + child_count;

        if child_count == 0 {
            let return_instruction =
                Instruction::r#return(false, Address::default(), OperandType::NONE);

            self.instructions.push(return_instruction);

            return Ok(());
        }

        let mut current_child_index = start_children;

        while current_child_index < end_children {
            let child_id = *self.syntax_tree.children.get(current_child_index).ok_or(
                CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: current_child_index as u32,
                },
            )?;
            let child_node = *self
                .syntax_tree
                .get_node(child_id)
                .ok_or(CompileError::MissingSyntaxNode { id: child_id })?;
            current_child_index += 1;

            if current_child_index == end_children {
                self.compile_implicit_return(&child_node)?;
            } else if child_node.kind.is_statement() {
                self.compile_statement(&child_node)?;
            } else if child_node.kind.is_item() {
                self.compile_item(&child_node)?;
            } else {
                return Err(CompileError::ExpectedStatement {
                    node_kind: child_node.kind,
                    position: Position::new(self.syntax_tree.file_index, child_node.span),
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
            let child_id = *self.syntax_tree.children.get(current_child_index).ok_or(
                CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: current_child_index as u32,
                },
            )?;
            let child_node = *self
                .syntax_tree
                .get_node(child_id)
                .ok_or(CompileError::MissingSyntaxNode { id: child_id })?;
            current_child_index += 1;

            self.compile_item(&child_node)?;
        }

        Ok(())
    }

    fn compile_use_item(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Compiling use item");

        let file_declaration_id = DeclarationId(node.children.0);
        let import_declaration_id = DeclarationId(node.payload);
        let import_declaration = self.resolver.get_declaration(import_declaration_id).ok_or(
            CompileError::MissingDeclaration {
                declaration_id: import_declaration_id,
            },
        )?;

        match import_declaration.kind {
            DeclarationKind::Function => {
                let file_tree = if file_declaration_id == DeclarationId::ANONYMOUS {
                    self.syntax_tree
                } else {
                    self.syntax_trees.get(&file_declaration_id).ok_or(
                        CompileError::MissingSyntaxTree {
                            declaration_id: import_declaration_id,
                        },
                    )?
                };
                let source_file = self
                    .source
                    .get_file(import_declaration.identifier_position.file_index)
                    .ok_or(CompileError::MissingSourceFile {
                        file_index: file_tree.file_index,
                    })?
                    .clone();
                let function_compiler = ChunkCompiler::new(
                    self.syntax_trees,
                    file_tree,
                    self.resolver,
                    self.constants,
                    self.source,
                    source_file,
                    self.prototypes,
                );

                let _ = function_compiler.compile()?;
                let prototype_index = self.prototypes.get_index_of(&import_declaration_id).ok_or(
                    CompileError::MissingPrototype {
                        declaration_id: import_declaration_id,
                    },
                )? as u16;

                self.locals.insert(
                    import_declaration_id,
                    Local {
                        location: prototype_index,
                    },
                );
            }
            _ => todo!(),
        }

        Ok(())
    }

    fn compile_expression_statement(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Compiling expression statement");

        let expression_id = SyntaxId(node.children.0);
        let expression_node = *self
            .syntax_tree
            .get_node(expression_id)
            .ok_or(CompileError::MissingSyntaxNode { id: expression_id })?;
        let expression_emission = self.compile_expression(&expression_node)?;

        match expression_emission {
            Emission::Instruction(instruction, _) => {
                self.instructions.push(instruction);
            }
            Emission::Instructions(instructions, _) => {
                self.instructions.extend(instructions.iter());
            }
            Emission::Constant(constant) => {
                let r#type = constant.operand_type();
                let address = self.get_constant_address(constant);
                let destination = Address::register(self.get_next_register());
                let load_instruction = Instruction::load(destination, address, r#type, false);

                self.instructions.push(load_instruction);
            }
            Emission::None => {}
        }

        Ok(())
    }

    fn compile_let_statement(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Compiling let statement");

        let declaration_id = DeclarationId(node.payload);
        let expression_statement_id = SyntaxId(node.children.1);

        let expression_statement_node = self.syntax_tree.get_node(expression_statement_id).ok_or(
            CompileError::MissingSyntaxNode {
                id: expression_statement_id,
            },
        )?;
        let expression_id = SyntaxId(expression_statement_node.children.0);
        let expression_node = *self
            .syntax_tree
            .get_node(expression_id)
            .ok_or(CompileError::MissingSyntaxNode { id: expression_id })?;
        let expression_emission = self.compile_expression(&expression_node)?;
        let destination_register = match expression_emission {
            Emission::Instruction(instruction, _) => {
                let destination_register = instruction.destination().index;

                self.instructions.push(instruction);

                destination_register
            }
            Emission::Instructions(instructions, _) => {
                let first_instruction = instructions[0];
                let destination_register = first_instruction.destination().index;

                self.instructions.extend(instructions.iter());

                destination_register
            }
            Emission::Constant(constant) => {
                let r#type = constant.operand_type();
                let address = self.get_constant_address(constant);
                let destination = Address::register(self.get_next_register());
                let instruction = Instruction::load(destination, address, r#type, false);

                self.instructions.push(instruction);

                destination.index
            }
            Emission::None => {
                return Err(CompileError::ExpectedExpression {
                    node_kind: expression_node.kind,
                    position: Position::new(self.syntax_tree.file_index, node.span),
                });
            }
        };

        self.locals.insert(
            declaration_id,
            Local {
                location: destination_register,
            },
        );

        Ok(())
    }

    fn compile_reassign_statement(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Compiling reassign statement");

        let declaration_id = DeclarationId(node.payload);
        let expression_id = SyntaxId(node.children.1);

        let local = *self
            .locals
            .get(&declaration_id)
            .ok_or(CompileError::MissingLocal { declaration_id })?;
        let expression_node = *self
            .syntax_tree
            .get_node(expression_id)
            .ok_or(CompileError::MissingSyntaxNode { id: expression_id })?;
        let expression_emission = self.compile_expression(&expression_node)?;
        let destination_register = local.location;
        match expression_emission {
            Emission::Instruction(instruction, _) => {
                let mut instruction = instruction;

                instruction.set_destination(Address::register(destination_register));

                self.instructions.push(instruction);
            }
            Emission::Instructions(instructions, _) => {
                for mut instruction in instructions {
                    instruction.set_destination(Address::register(destination_register));

                    self.instructions.push(instruction);
                }
            }
            Emission::Constant(constant) => {
                let r#type = constant.operand_type();
                let address = self.get_constant_address(constant);
                let destination = Address::register(destination_register);
                let load_instruction = Instruction::load(destination, address, r#type, false);

                self.instructions.push(load_instruction);
            }
            Emission::None => {
                return Err(CompileError::ExpectedExpression {
                    node_kind: expression_node.kind,
                    position: Position::new(self.syntax_tree.file_index, expression_node.span),
                });
            }
        };

        Ok(())
    }

    fn compile_function_statement(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        info!("Compiling function statement");

        let declaration_id = DeclarationId(node.children.0);
        let function_expression_id = SyntaxId(node.children.1);
        let function_node = *self.syntax_tree.get_node(function_expression_id).ok_or(
            CompileError::MissingSyntaxNode {
                id: function_expression_id,
            },
        )?;

        if self.locals.contains_key(&declaration_id) {
            let declaration = self
                .resolver
                .get_declaration(declaration_id)
                .ok_or(CompileError::MissingDeclaration { declaration_id })?;
            let identifier = &self
                .source
                .get_file(declaration.identifier_position.file_index)
                .ok_or(CompileError::MissingSourceFile {
                    file_index: declaration.identifier_position.file_index,
                })?
                .source_code[declaration.identifier_position.span.as_usize_range()];

            return Err(CompileError::DuplicateFunctionDeclaration {
                identifier: identifier.to_string(),
                first_position: declaration.identifier_position,
                second_position: Position::new(self.syntax_tree.file_index, node.span),
            });
        }

        let Emission::Constant(Constant::Function {
            prototype_index, ..
        }) = self.compile_function_expression(&function_node, declaration_id)?
        else {
            return Err(CompileError::ExpectedFunction {
                node_kind: function_node.kind,
                position: Position::new(self.syntax_tree.file_index, function_node.span),
            });
        };

        self.locals.insert(
            declaration_id,
            Local {
                location: prototype_index,
            },
        );

        Ok(())
    }

    fn compile_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        match node.kind {
            SyntaxKind::BooleanExpression => self.compile_boolean_expression(node),
            SyntaxKind::ByteExpression => self.compile_byte_expression(node),
            SyntaxKind::CharacterExpression => self.compile_character_expression(node),
            SyntaxKind::FloatExpression => self.compile_float_expression(node),
            SyntaxKind::IntegerExpression => self.compile_integer_expression(node),
            SyntaxKind::StringExpression => self.compile_string_expression(node),
            SyntaxKind::PathExpression => self.compile_path_expression(node),
            SyntaxKind::AdditionExpression
            | SyntaxKind::SubtractionExpression
            | SyntaxKind::MultiplicationExpression
            | SyntaxKind::DivisionExpression
            | SyntaxKind::ModuloExpression
            | SyntaxKind::AdditionAssignmentExpression
            | SyntaxKind::SubtractionAssignmentExpression
            | SyntaxKind::MultiplicationAssignmentExpression
            | SyntaxKind::DivisionAssignmentExpression
            | SyntaxKind::ModuloAssignmentExpression => self.compile_math_expression(node),
            SyntaxKind::GreaterThanExpression
            | SyntaxKind::GreaterThanOrEqualExpression
            | SyntaxKind::LessThanExpression
            | SyntaxKind::LessThanOrEqualExpression
            | SyntaxKind::EqualExpression
            | SyntaxKind::NotEqualExpression => self.compile_comparison_expression(node),
            SyntaxKind::AndExpression | SyntaxKind::OrExpression => {
                self.compile_logical_expression(node)
            }
            SyntaxKind::NotExpression | SyntaxKind::NegationExpression => {
                self.compile_unary_expression(node)
            }
            SyntaxKind::GroupedExpression => self.compile_grouped_expression(node),
            SyntaxKind::BlockExpression => self.compile_block_expression(node),
            SyntaxKind::WhileExpression => self.compile_while_expression(node),
            SyntaxKind::FunctionExpression => {
                self.compile_function_expression(node, DeclarationId::ANONYMOUS)
            }
            SyntaxKind::CallExpression => self.compile_call_expression(node),
            _ => Err(CompileError::ExpectedExpression {
                node_kind: node.kind,
                position: Position::new(self.syntax_tree.file_index, node.span),
            }),
        }
    }

    fn compile_boolean_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling boolean expression");

        Ok(Emission::Constant(Constant::Boolean(node.children.0 != 0)))
    }

    fn compile_byte_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling byte expression");

        Ok(Emission::Constant(Constant::Byte(node.children.0 as u8)))
    }

    fn compile_character_expression(
        &mut self,
        node: &SyntaxNode,
    ) -> Result<Emission, CompileError> {
        info!("Compiling character expression");

        let character = char::from_u32(node.children.0).unwrap_or_default();

        Ok(Emission::Constant(Constant::Character(character)))
    }

    fn compile_float_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling float expression");

        let float = SyntaxNode::decode_float(node.children);

        Ok(Emission::Constant(Constant::Float(float)))
    }

    fn compile_integer_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling integer expression");

        let integer = SyntaxNode::decode_integer(node.children);

        Ok(Emission::Constant(Constant::Integer(integer)))
    }

    fn compile_string_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling string expression");

        let string_start = node.span.0 + 1;
        let string_end = node.span.1 - 1;
        let string = &self.source_file.source_code[string_start as usize..string_end as usize];
        let (pool_start, pool_end) = self.constants.push_str_to_string_pool(string);

        Ok(Emission::Constant(Constant::String {
            pool_start,
            pool_end,
        }))
    }

    fn compile_math_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling math expression");

        let left_index = SyntaxId(node.children.0);
        let left = *self.syntax_tree.nodes.get(left_index.0 as usize).ok_or(
            CompileError::MissingChild {
                parent_kind: node.kind,
                child_index: node.children.0,
            },
        )?;
        let right_index = SyntaxId(node.children.1);
        let right = *self.syntax_tree.nodes.get(right_index.0 as usize).ok_or(
            CompileError::MissingChild {
                parent_kind: node.kind,
                child_index: node.children.1,
            },
        )?;

        let left_emission = self.compile_expression(&left)?;
        let right_emission = self.compile_expression(&right)?;

        if let (Emission::Constant(left_value), Emission::Constant(right_value)) =
            (&left_emission, &right_emission)
        {
            let combined = self.combine_constants(*left_value, *right_value, node.kind)?;

            return Ok(Emission::Constant(combined));
        }

        let instructions_count_before = self.instructions.len();
        let (left_address, _) = left_emission.handle_as_operand(self);
        let (right_address, _) = right_emission.handle_as_operand(self);

        let destination = Address::register(self.get_next_register());
        let instruction = match node.kind {
            SyntaxKind::AdditionExpression => {
                let operand_type = self
                    .resolver
                    .resolve_type(TypeId(node.payload))
                    .ok_or(CompileError::MissingType {
                        type_id: TypeId(node.payload),
                    })?
                    .as_operand_type();

                Instruction::add(destination, left_address, right_address, operand_type)
            }
            SyntaxKind::AdditionAssignmentExpression => {
                let operand_type = self
                    .resolver
                    .resolve_type(TypeId(left.payload))
                    .ok_or(CompileError::MissingType {
                        type_id: TypeId(node.payload),
                    })?
                    .as_operand_type();

                self.instructions.truncate(instructions_count_before);

                Instruction::add(left_address, left_address, right_address, operand_type)
            }
            SyntaxKind::SubtractionExpression => {
                let operand_type = self
                    .resolver
                    .resolve_type(TypeId(node.payload))
                    .ok_or(CompileError::MissingType {
                        type_id: TypeId(node.payload),
                    })?
                    .as_operand_type();

                Instruction::subtract(destination, left_address, right_address, operand_type)
            }
            SyntaxKind::SubtractionAssignmentExpression => {
                let operand_type = self
                    .resolver
                    .resolve_type(TypeId(left.payload))
                    .ok_or(CompileError::MissingType {
                        type_id: TypeId(node.payload),
                    })?
                    .as_operand_type();

                self.instructions.truncate(instructions_count_before);

                Instruction::subtract(left_address, left_address, right_address, operand_type)
            }
            SyntaxKind::MultiplicationExpression => {
                let operand_type = self
                    .resolver
                    .resolve_type(TypeId(node.payload))
                    .ok_or(CompileError::MissingType {
                        type_id: TypeId(node.payload),
                    })?
                    .as_operand_type();

                Instruction::multiply(destination, left_address, right_address, operand_type)
            }
            SyntaxKind::MultiplicationAssignmentExpression => {
                let operand_type = self
                    .resolver
                    .resolve_type(TypeId(left.payload))
                    .ok_or(CompileError::MissingType {
                        type_id: TypeId(node.payload),
                    })?
                    .as_operand_type();

                self.instructions.truncate(instructions_count_before);

                Instruction::multiply(left_address, left_address, right_address, operand_type)
            }
            SyntaxKind::DivisionExpression => {
                let operand_type = self
                    .resolver
                    .resolve_type(TypeId(node.payload))
                    .ok_or(CompileError::MissingType {
                        type_id: TypeId(node.payload),
                    })?
                    .as_operand_type();

                Instruction::divide(destination, left_address, right_address, operand_type)
            }
            SyntaxKind::DivisionAssignmentExpression => {
                let operand_type = self
                    .resolver
                    .resolve_type(TypeId(left.payload))
                    .ok_or(CompileError::MissingType {
                        type_id: TypeId(node.payload),
                    })?
                    .as_operand_type();

                self.instructions.truncate(instructions_count_before);

                Instruction::divide(left_address, left_address, right_address, operand_type)
            }
            SyntaxKind::ModuloExpression => {
                let operand_type = self
                    .resolver
                    .resolve_type(TypeId(node.payload))
                    .ok_or(CompileError::MissingType {
                        type_id: TypeId(node.payload),
                    })?
                    .as_operand_type();

                Instruction::modulo(destination, left_address, right_address, operand_type)
            }
            SyntaxKind::ModuloAssignmentExpression => {
                let operand_type = self
                    .resolver
                    .resolve_type(TypeId(left.payload))
                    .ok_or(CompileError::MissingType {
                        type_id: TypeId(node.payload),
                    })?
                    .as_operand_type();

                self.instructions.truncate(instructions_count_before);

                Instruction::modulo(left_address, left_address, right_address, operand_type)
            }
            _ => unreachable!("Expected binary expression, found {}", node.kind),
        };

        Ok(Emission::Instruction(instruction, TypeId(node.payload)))
    }

    fn compile_comparison_expression(
        &mut self,
        node: &SyntaxNode,
    ) -> Result<Emission, CompileError> {
        let left_index = SyntaxId(node.children.0);
        let left = *self.syntax_tree.nodes.get(left_index.0 as usize).ok_or(
            CompileError::MissingChild {
                parent_kind: node.kind,
                child_index: node.children.0,
            },
        )?;
        let right_index = SyntaxId(node.children.1);
        let right = *self.syntax_tree.nodes.get(right_index.0 as usize).ok_or(
            CompileError::MissingChild {
                parent_kind: node.kind,
                child_index: node.children.1,
            },
        )?;

        let left_emission = self.compile_expression(&left)?;
        let right_emission = self.compile_expression(&right)?;

        if let (Emission::Constant(left_value), Emission::Constant(right_value)) =
            (&left_emission, &right_emission)
        {
            let combined = self.combine_constants(*left_value, *right_value, node.kind)?;

            return Ok(Emission::Constant(combined));
        }

        let destination = Address::register(self.get_next_register());
        let (left_address, left_type) = match &left_emission {
            Emission::Instruction(instruction, _) => (
                self.handle_operand(*instruction),
                instruction.operand_type(),
            ),
            Emission::Instructions(instructions, _) => {
                self.instructions.extend(instructions);

                (
                    instructions[0].destination(),
                    instructions[0].operand_type(),
                )
            }
            Emission::Constant(constant) => {
                let r#type = constant.operand_type();
                let address = self.get_constant_address(*constant);
                let load_instruction = Instruction::load(destination, address, r#type, false);

                (self.handle_operand(load_instruction), r#type)
            }
            Emission::None => {
                return Err(CompileError::ExpectedExpression {
                    node_kind: left.kind,
                    position: Position::new(self.syntax_tree.file_index, left.span),
                });
            }
        };
        let (right_address, right_type) = match right_emission {
            Emission::Instruction(instruction, _) => {
                (self.handle_operand(instruction), instruction.operand_type())
            }
            Emission::Instructions(mut instructions, _) => {
                let destination = instructions[0].destination();
                let operand_type = instructions[0].operand_type();

                self.instructions.append(&mut instructions);

                (destination, operand_type)
            }
            Emission::Constant(constant) => {
                let r#type = constant.operand_type();
                let address = self.get_constant_address(constant);
                let load_instruction = Instruction::load(destination, address, r#type, false);

                (self.handle_operand(load_instruction), r#type)
            }
            Emission::None => {
                return Err(CompileError::ExpectedExpression {
                    node_kind: right.kind,
                    position: Position::new(self.syntax_tree.file_index, right.span),
                });
            }
        };

        if left_type != right_type {
            todo!("Error");
        }

        let destination = Address::register(self.get_next_register());
        let comparison_instruction = match node.kind {
            SyntaxKind::GreaterThanExpression => {
                Instruction::less_equal(false, left_address, right_address, left_type)
            }
            SyntaxKind::GreaterThanOrEqualExpression => {
                Instruction::less(false, left_address, right_address, left_type)
            }
            SyntaxKind::LessThanExpression => {
                Instruction::less(true, left_address, right_address, left_type)
            }
            SyntaxKind::LessThanOrEqualExpression => {
                Instruction::less_equal(true, left_address, right_address, left_type)
            }
            SyntaxKind::EqualExpression => {
                Instruction::equal(true, left_address, right_address, left_type)
            }
            SyntaxKind::NotEqualExpression => {
                Instruction::equal(false, left_address, right_address, left_type)
            }
            _ => unreachable!("Expected comparison expression, found {}", node.kind),
        };
        let jump_instruction = Instruction::jump(1, true);
        let load_true_instruction = Instruction::load(
            destination,
            Address::encoded(true as u16),
            OperandType::BOOLEAN,
            true,
        );
        let load_false_instruction = Instruction::load(
            destination,
            Address::encoded(false as u16),
            OperandType::BOOLEAN,
            false,
        );

        Ok(Emission::Instructions(
            vec![
                comparison_instruction,
                jump_instruction,
                load_true_instruction,
                load_false_instruction,
            ],
            TypeId::BOOLEAN,
        ))
    }

    fn compile_logical_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling logical expression");

        let left_index = SyntaxId(node.children.0);
        let left = *self.syntax_tree.nodes.get(left_index.0 as usize).ok_or(
            CompileError::MissingChild {
                parent_kind: node.kind,
                child_index: node.children.0,
            },
        )?;
        let right_index = SyntaxId(node.children.1);
        let right = *self.syntax_tree.nodes.get(right_index.0 as usize).ok_or(
            CompileError::MissingChild {
                parent_kind: node.kind,
                child_index: node.children.1,
            },
        )?;

        let left_emission = self.compile_expression(&left)?;
        let right_emission = self.compile_expression(&right)?;

        if let (Emission::Constant(left_value), Emission::Constant(right_value)) =
            (&left_emission, &right_emission)
        {
            let combined = self.combine_constants(*left_value, *right_value, node.kind)?;

            return Ok(Emission::Constant(combined));
        }

        let left_address = match &left_emission {
            Emission::Instruction(instruction, _) => {
                self.instructions.push(*instruction);

                instruction.destination()
            }
            Emission::Instructions(instructions, _) => {
                self.instructions.extend(instructions.iter());

                instructions.last().unwrap().destination()
            }
            Emission::Constant(constant) => {
                let r#type = constant.operand_type();
                let address = self.get_constant_address(*constant);
                let destination = Address::register(self.get_next_register());
                let load_instruction = Instruction::load(destination, address, r#type, false);

                self.handle_operand(load_instruction)
            }
            Emission::None => {
                return Err(CompileError::ExpectedExpression {
                    node_kind: left.kind,
                    position: Position::new(self.syntax_tree.file_index, left.span),
                });
            }
        };

        let comparator = match node.kind {
            SyntaxKind::AndExpression => true,
            SyntaxKind::OrExpression => false,
            _ => unreachable!("Expected logical expression, found {}", node.kind),
        };
        let test_instruction = Instruction::test(left_address, comparator);
        let jump_instruction = Instruction::jump(1, true);

        self.instructions.push(test_instruction);
        self.instructions.push(jump_instruction);

        let emission = match right_emission {
            Emission::Instruction(instruction, _) => {
                let mut instruction = instruction;

                instruction.set_destination(left_address);

                Emission::Instruction(instruction, TypeId(right.payload))
            }
            Emission::Instructions(instructions, r#type) => {
                Emission::Instructions(instructions, r#type)
            }
            Emission::Constant(constant) => Emission::Constant(constant),
            Emission::None => {
                return Err(CompileError::ExpectedExpression {
                    node_kind: right.kind,
                    position: Position::new(self.syntax_tree.file_index, right.span),
                });
            }
        };

        Ok(emission)
    }

    fn compile_unary_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling unary expression");

        let child_id = SyntaxId(node.children.0);
        let child_node =
            *self
                .syntax_tree
                .get_node(child_id)
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: node.children.0,
                })?;
        let child_emission = self.compile_expression(&child_node)?;

        if let Emission::Constant(child_value) = &child_emission {
            let evaluated = match (node.kind, child_value) {
                (SyntaxKind::NotExpression, Constant::Boolean(value)) => Constant::Boolean(!value),
                (SyntaxKind::NegationExpression, Constant::Integer(value)) => {
                    Constant::Integer(-value)
                }
                (SyntaxKind::NegationExpression, Constant::Float(value)) => Constant::Float(-value),
                _ => todo!("Error"),
            };

            return Ok(Emission::Constant(evaluated));
        }

        let (child_address, operand_type) = child_emission.handle_as_operand(self);
        let destination = Address::register(self.get_next_register());
        let negate_instruction = Instruction::negate(destination, child_address, operand_type);

        Ok(Emission::Instruction(
            negate_instruction,
            TypeId(node.payload),
        ))
    }

    fn compile_grouped_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling grouped expression");

        let child_id = SyntaxId(node.children.0);
        let child_node =
            *self
                .syntax_tree
                .get_node(child_id)
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: node.children.0,
                })?;

        self.compile_expression(&child_node)
    }

    fn compile_block_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling block expression");

        let (start_children, child_count) = (node.children.0 as usize, node.children.1 as usize);

        if child_count == 0 {
            return Ok(Emission::None);
        }

        let end_children = start_children + child_count - 1;
        let mut current_child_index = start_children;

        while current_child_index < end_children {
            let child_id = *self.syntax_tree.children.get(current_child_index).ok_or(
                CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: current_child_index as u32,
                },
            )?;
            let child_node = *self
                .syntax_tree
                .get_node(child_id)
                .ok_or(CompileError::MissingSyntaxNode { id: child_id })?;
            current_child_index += 1;

            self.compile_statement(&child_node)?;
        }

        let last_child_id =
            *self
                .syntax_tree
                .children
                .get(end_children)
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: end_children as u32,
                })?;
        let last_child_node = *self
            .syntax_tree
            .get_node(last_child_id)
            .ok_or(CompileError::MissingSyntaxNode { id: last_child_id })?;
        let outer_scope_id = ScopeId(node.payload);
        let outer_scope = self
            .resolver
            .get_scope(outer_scope_id)
            .ok_or(CompileError::MissingScope { id: outer_scope_id })?;

        if last_child_node.kind.is_statement() {
            self.compile_statement(&last_child_node)?;

            Ok(Emission::None)
        } else if outer_scope.kind == ScopeKind::Function && outer_scope_id != ScopeId::MAIN {
            self.compile_implicit_return(&last_child_node)?;

            Ok(Emission::None)
        } else {
            self.compile_expression(&last_child_node)
        }
    }

    fn compile_path_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling path expression");

        let declaration_id = DeclarationId(node.children.0);
        let declaration = self
            .resolver
            .get_declaration(declaration_id)
            .ok_or(CompileError::MissingDeclaration { declaration_id })?;

        if declaration.kind == DeclarationKind::NativeFunction {
            let identifier =
                &self.source_file.source_code[node.span.0 as usize..node.span.1 as usize];
            let native_function = NativeFunction::from_str(identifier).ok_or(
                CompileError::InvalidNativeFunction {
                    name: identifier.to_string(),
                    position: Position::new(self.syntax_tree.file_index, node.span),
                },
            )?;

            return Ok(Emission::Constant(Constant::NativeFunction {
                type_id: TypeId(node.payload),
                native_function,
            }));
        }

        let operand_register = self
            .locals
            .get(&declaration_id)
            .ok_or(CompileError::MissingLocal { declaration_id })?
            .location;
        let r#type = self
            .resolver
            .resolve_type(TypeId(node.payload))
            .ok_or(CompileError::MissingType {
                type_id: TypeId(node.payload),
            })?
            .as_operand_type();

        if r#type == OperandType::FUNCTION {
            let prototype_index = self
                .locals
                .get(&declaration_id)
                .ok_or(CompileError::MissingLocal { declaration_id })?
                .location;

            return Ok(Emission::Constant(Constant::Function {
                type_id: TypeId(node.payload),
                prototype_index,
            }));
        }

        let load_instruction = Instruction::load(
            Address::register(self.get_next_register()),
            Address::register(operand_register),
            r#type,
            false,
        );

        Ok(Emission::Instruction(
            load_instruction,
            TypeId(node.payload),
        ))
    }

    fn compile_while_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        info!("Compiling while expression");

        let condition_id = SyntaxId(node.children.0);
        let body_id = SyntaxId(node.children.1);

        let condition_node =
            *self
                .syntax_tree
                .get_node(condition_id)
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: node.children.0,
                })?;
        let body_node = *self
            .syntax_tree
            .get_node(body_id)
            .ok_or(CompileError::MissingChild {
                parent_kind: node.kind,
                child_index: node.children.1,
            })?;
        let condition_emission = self.compile_expression(&condition_node)?;

        match condition_emission {
            Emission::Instruction(instruction, _) => {
                self.handle_operand(instruction);
            }
            Emission::Instructions(instructions, _) => {
                let comparison_or_test_instruction = instructions[0];

                self.instructions.push(comparison_or_test_instruction);
            }
            Emission::Constant(constant) => {
                let r#type = constant.operand_type();
                let address = self.get_constant_address(constant);
                let destination = Address::register(self.get_next_register());
                let load_instruction = Instruction::load(destination, address, r#type, false);

                self.handle_operand(load_instruction);
            }
            Emission::None => {
                return Err(CompileError::ExpectedExpression {
                    node_kind: condition_node.kind,
                    position: Position::new(self.syntax_tree.file_index, condition_node.span),
                });
            }
        }

        let jump_forward_index = self.instructions.len();

        self.instructions.push(Instruction::no_op());
        self.compile_expression_statement(&body_node)?;

        let jump_distance = (self.instructions.len() - jump_forward_index) as u16;

        let jump_forward_instruction = Instruction::jump(jump_distance, true);
        let jump_back_instruction = Instruction::jump(jump_distance, false);

        self.instructions[jump_forward_index] = jump_forward_instruction;
        self.instructions.push(jump_back_instruction);

        Ok(Emission::None)
    }

    fn compile_function_expression(
        &mut self,
        node: &SyntaxNode,
        declaration_id: DeclarationId,
    ) -> Result<Emission, CompileError> {
        info!("Compiling function expression");

        let function_signature_id = SyntaxId(node.children.0);
        let block_id = SyntaxId(node.children.1);
        let function_type_id = TypeId(node.payload);

        let function_signature_node = *self.syntax_tree.get_node(function_signature_id).ok_or(
            CompileError::MissingSyntaxNode {
                id: function_signature_id,
            },
        )?;
        let value_parameters_node_id = function_signature_node.children.0;
        let value_parameters_node = *self
            .syntax_tree
            .get_node(SyntaxId(value_parameters_node_id))
            .ok_or(CompileError::MissingSyntaxNode {
                id: SyntaxId(value_parameters_node_id),
            })?;

        debug_assert_eq!(
            value_parameters_node.kind,
            SyntaxKind::FunctionValueParameters
        );

        let body_node = *self
            .syntax_tree
            .get_node(block_id)
            .ok_or(CompileError::MissingSyntaxNode { id: block_id })?;
        let Some(value_parameter_nodes) = self
            .syntax_tree
            .get_children(
                value_parameters_node.children.0,
                value_parameters_node.children.1,
            )
            .map(|children| children.to_vec())
        else {
            return Err(CompileError::MissingChild {
                parent_kind: function_signature_node.kind,
                child_index: self.r#type.value_parameters.0,
            });
        };
        let mut function_compiler = ChunkCompiler::new(
            self.syntax_trees,
            self.syntax_tree,
            self.resolver,
            self.constants,
            self.source,
            self.source_file.clone(),
            self.prototypes,
        );

        let mut value_parameter_types =
            SmallVec::<[TypeId; 4]>::with_capacity(value_parameter_nodes.len());

        for syntax_id in value_parameter_nodes {
            let parameter_node = *self
                .syntax_tree
                .get_node(syntax_id)
                .ok_or(CompileError::MissingSyntaxNode { id: syntax_id })?;
            let parameter_declaration_id = DeclarationId(parameter_node.payload);
            let r#type = function_compiler
                .resolver
                .get_declaration(parameter_declaration_id)
                .ok_or(CompileError::MissingDeclaration {
                    declaration_id: parameter_declaration_id,
                })?
                .type_id;
            let register = function_compiler.get_next_register();
            function_compiler.minimum_register += 1;

            function_compiler
                .locals
                .insert(parameter_declaration_id, Local { location: register });
            value_parameter_types.push(r#type);
        }

        function_compiler.compile_implicit_return(&body_node)?;

        let value_parameter_types = function_compiler
            .resolver
            .push_type_members(&value_parameter_types);
        let function_chunk = function_compiler.finish(value_parameter_types)?;
        let prototype_index = self.prototypes.len() as u16;

        self.prototypes.insert(declaration_id, function_chunk);

        Ok(Emission::Constant(Constant::Function {
            type_id: function_type_id,
            prototype_index,
        }))
    }

    fn compile_call_expression(&mut self, node: &SyntaxNode) -> Result<Emission, CompileError> {
        fn handle_call_arguments(
            compiler: &mut ChunkCompiler,
            arguments_node: &SyntaxNode,
        ) -> Result<(), CompileError> {
            debug_assert_eq!(arguments_node.kind, SyntaxKind::CallValueArguments);

            let children = compiler
                .syntax_tree
                .get_children(arguments_node.children.0, arguments_node.children.1)
                .ok_or(CompileError::MissingChild {
                    parent_kind: arguments_node.kind,
                    child_index: arguments_node.children.0,
                })?
                .to_vec();

            for child_id in children {
                let child_node = *compiler
                    .syntax_tree
                    .get_node(child_id)
                    .ok_or(CompileError::MissingSyntaxNode { id: child_id })?;
                let argument_emission = compiler.compile_expression(&child_node)?;
                let argument_address = argument_emission.handle_as_operand(compiler);

                compiler.call_arguments.push(argument_address);
            }

            Ok(())
        }

        info!("Compiling call expression");

        let function_node_id = SyntaxId(node.children.0);
        let arguments_node_id = SyntaxId(node.children.1);

        let function_node =
            *self
                .syntax_tree
                .get_node(function_node_id)
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: node.children.0,
                })?;
        let arguments_node =
            *self
                .syntax_tree
                .get_node(arguments_node_id)
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: node.children.1,
                })?;

        let expression_emission = self.compile_expression(&function_node)?;

        let Emission::Constant(Constant::Function {
            type_id,
            prototype_index,
        }) = expression_emission
        else {
            if let Emission::Constant(Constant::NativeFunction {
                native_function, ..
            }) = expression_emission
            {
                let destination = Address::register(self.get_next_register());
                let call_arguments_start_index = self.call_arguments.len() as u16;

                handle_call_arguments(self, &arguments_node)?;

                let call_native_instruction = Instruction::call_native(
                    destination,
                    native_function,
                    call_arguments_start_index,
                );

                return Ok(Emission::Instruction(
                    call_native_instruction,
                    TypeId(node.payload),
                ));
            }

            return Err(CompileError::ExpectedFunction {
                node_kind: function_node.kind,
                position: Position::new(self.syntax_tree.file_index, function_node.span),
            });
        };

        let arguments_start_index = self.call_arguments.len() as u16;

        handle_call_arguments(self, &arguments_node)?;

        let destination = Address::register(self.get_next_register());
        let Type::Function(function_type) = self
            .resolver
            .resolve_type(type_id)
            .ok_or(CompileError::MissingType { type_id })?
        else {
            return Err(CompileError::ExpectedFunction {
                node_kind: function_node.kind,
                position: Position::new(self.syntax_tree.file_index, function_node.span),
            });
        };
        let operand_type = function_type.return_type.as_operand_type();
        let call_instruction = Instruction::call(
            destination,
            prototype_index,
            arguments_start_index,
            operand_type,
        );

        Ok(Emission::Instruction(
            call_instruction,
            TypeId(node.payload),
        ))
    }

    fn compile_implicit_return(&mut self, node: &SyntaxNode) -> Result<(), CompileError> {
        let (return_instruction, return_type) = if node.kind.is_item() {
            self.compile_item(node)?;

            (
                Instruction::r#return(false, Address::default(), OperandType::NONE),
                TypeId::NONE,
            )
        } else if node.kind.is_statement() {
            self.compile_statement(node)?;

            (
                Instruction::r#return(false, Address::default(), OperandType::NONE),
                TypeId::NONE,
            )
        } else {
            let return_emission = self.compile_expression(node)?;
            let (return_operand, return_operand_type) = match return_emission {
                Emission::Instruction(instruction, r#type) => {
                    let operand_type = self
                        .resolver
                        .resolve_type(r#type)
                        .ok_or(CompileError::MissingType { type_id: r#type })?
                        .as_operand_type();
                    let mut return_operand = self.handle_operand(instruction);

                    if let Some(last_instruction) = self.instructions.last()
                        && last_instruction.operation() == Operation::LOAD
                        && last_instruction.destination() == return_operand
                    {
                        return_operand = last_instruction.b_address();

                        self.instructions.pop();
                    }

                    (return_operand, operand_type)
                }
                Emission::Instructions(instructions, r#type) => {
                    let last_instruction = instructions.last().unwrap();
                    let operand_type = self
                        .resolver
                        .resolve_type(r#type)
                        .ok_or(CompileError::MissingType { type_id: r#type })?
                        .as_operand_type();

                    self.instructions.extend(instructions.iter());

                    (last_instruction.destination(), operand_type)
                }
                Emission::Constant(constant) => {
                    let operand_type = constant.operand_type();
                    let address = self.get_constant_address(constant);
                    let destination = Address::register(self.get_next_register());
                    let instruction = Instruction::load(destination, address, operand_type, false);
                    let return_operand = self.handle_operand(instruction);

                    (return_operand, operand_type)
                }
                Emission::None => (Address::default(), OperandType::NONE),
            };

            (
                Instruction::r#return(
                    return_operand_type != OperandType::NONE,
                    return_operand,
                    return_operand_type,
                ),
                TypeId(node.payload),
            )
        };

        if self
            .instructions
            .last()
            .is_some_and(|instruction| instruction.operation() == Operation::RETURN)
        {
            return Ok(());
        }

        self.r#type.return_type = return_type;

        self.instructions.push(return_instruction);

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum Emission {
    Instruction(Instruction, TypeId),
    Instructions(Vec<Instruction>, TypeId),
    Constant(Constant),
    None,
}

impl Emission {
    fn handle_as_operand(self, compiler: &mut ChunkCompiler) -> (Address, OperandType) {
        match self {
            Emission::Instruction(instruction, _) => (
                compiler.handle_operand(instruction),
                instruction.operand_type(),
            ),
            Emission::Instructions(instructions, _) => {
                let first_instruction = instructions[0];
                let destination = first_instruction.destination();

                compiler.instructions.extend(instructions.iter());

                (destination, first_instruction.operand_type())
            }
            Emission::Constant(constant) => {
                let r#type = constant.operand_type();
                let address = compiler.get_constant_address(constant);
                let destination = Address::register(compiler.get_next_register());
                let load_instruction = Instruction::load(destination, address, r#type, false);

                (compiler.handle_operand(load_instruction), r#type)
            }
            Emission::None => (Address::default(), OperandType::NONE),
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
    String {
        pool_start: u32,
        pool_end: u32,
    },
    Function {
        type_id: TypeId,
        prototype_index: u16,
    },
    NativeFunction {
        type_id: TypeId,
        native_function: NativeFunction,
    },
}

impl Constant {
    fn operand_type(&self) -> OperandType {
        match self {
            Constant::Boolean(_) => OperandType::BOOLEAN,
            Constant::Byte(_) => OperandType::BYTE,
            Constant::Character(_) => OperandType::CHARACTER,
            Constant::Float(_) => OperandType::FLOAT,
            Constant::Integer(_) => OperandType::INTEGER,
            Constant::String { .. } => OperandType::STRING,
            Constant::Function { .. } => OperandType::FUNCTION,
            Constant::NativeFunction { .. } => OperandType::FUNCTION,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
struct Local {
    location: u16,
}
