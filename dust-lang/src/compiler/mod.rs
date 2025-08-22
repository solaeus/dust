mod abstract_constant;
mod error;
mod fold_constants;
mod local;
mod path_resolver;
mod scope;
mod type_resolver;

pub use abstract_constant::AbstractConstant;
pub use error::CompileError;
pub use local::Local;
use path_resolver::PathResolver;
pub use scope::Scope;
use type_resolver::TypeResolver;

use tracing::{Level, info, span, trace};

use crate::{
    Address, Chunk, FunctionType, Instruction, Lexer, OperandType, Operation, Type,
    chunk::ConstantTable,
    dust_error::DustError,
    parser::Parser,
    syntax_tree::{SyntaxId, SyntaxKind, SyntaxNode, SyntaxTree},
};

pub fn compile(source: &'_ str) -> Result<Chunk, DustError<'_>> {
    let lexer = Lexer::new(source);
    let parser = Parser::new(lexer);
    let (syntax_tree, _errors) = parser.parse(true);
    let compiler = ChunkCompiler::new(&syntax_tree, source);

    compiler
        .compile()
        .map_err(|error| DustError::compile(error, source))
}

pub struct Compiler {
    allow_native_functions: bool,
}

impl Compiler {
    pub fn new(allow_native_functions: bool) -> Self {
        Self {
            allow_native_functions,
        }
    }

    // pub fn compile(&self, sources: &[(&str, &str)]) -> Result<Program, DustError<'_>> {}
}

#[derive(Debug)]
pub struct ChunkCompiler<'a> {
    /// Target syntax tree for compilation.
    syntax_tree: &'a SyntaxTree,

    /// Source code being compiled.
    source: &'a str,

    /// Dependency graph of modules that is progressively filled as one module imports another.
    module_graph: PathResolver,

    /// Type resolver that assigns types to nodes in the syntax tree.
    type_table: TypeResolver,

    /// Bytecode instruction collection that is filled during compilation.
    instructions: Vec<Instruction>,

    /// Directed acyclic graph of expressions that will
    constants: Vec<AbstractConstant>,

    /// Concatenated list of arguments referenced by CALL instructions.
    call_arguments: Vec<(Address, OperandType)>,

    /// Concatenated list of register indexes that are referenced by DROP instructions.
    drop_lists: Vec<u16>,

    /// Apparent return type of the function being compiled. This field is modified during compilation
    /// to reflect the actual return type of the function
    return_type: Type,

    /// Index of the the chunk being compiled in the program's prototype list. For the main function,
    /// this is 0 as a default but the main chunk is actually the last one in the list.
    prototype_index: u16,

    /// Registers used by local variables in the function being compiled. Each entry in this list
    /// corresponds to a local in the syntax tree at the same index.
    local_registers: Vec<u16>,

    /// Incremented counter that tracks the next available register. This is used to allocate new
    /// registers and also represents the number that have been allocated so far.
    next_unused_register: u16,

    /// Deduplicated and reverse-sorted list of registers that are were previously used but whose
    /// value has gone out of scope. Register indexes are always popped from this list if available
    /// before allocating a new register.
    reusable_registers: Vec<u16>,

    current_scope: Scope,
}

impl<'a> ChunkCompiler<'a> {
    pub fn new(syntax_tree: &'a SyntaxTree, source: &'a str) -> Self {
        Self {
            syntax_tree,
            source,
            module_graph: PathResolver::new(),
            type_table: TypeResolver::new(syntax_tree),
            instructions: Vec::new(),
            constants: Vec::new(),
            call_arguments: Vec::new(),
            drop_lists: Vec::new(),
            return_type: Type::None,
            prototype_index: 0,
            local_registers: Vec::new(),
            next_unused_register: 0,
            reusable_registers: Vec::new(),
            current_scope: Scope::default(),
        }
    }

    pub fn compile(mut self) -> Result<Chunk, CompileError> {
        let span = span!(Level::INFO, "Compiling");
        let _enter = span.enter();

        let main_function_node = self
            .syntax_tree
            .nodes
            .first()
            .copied()
            .ok_or(CompileError::MissingSyntaxNode { id: SyntaxId(0) })?;

        self.compile_statement(main_function_node)?;

        let constants = self.materialize_constant_table()?;

        Ok(Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], self.return_type),
            instructions: self.instructions,
            constants,
            call_arguments: self.call_arguments,
            drop_lists: self.drop_lists,
            register_count: self.next_unused_register,
            prototype_index: self.prototype_index,
        })
    }

    fn materialize_constant_table(&self) -> Result<ConstantTable, CompileError> {
        let mut constants = ConstantTable::new(0, 0);

        for constant in &self.constants {
            match constant {
                AbstractConstant::Raw { expression } => {
                    let expession_node = self
                        .syntax_tree
                        .get_node(*expression)
                        .ok_or(CompileError::MissingSyntaxNode { id: *expression })?;

                    self.materialize_constant(expession_node, &mut constants)?;
                }
                _ => todo!(),
            }
        }

        Ok(constants)
    }

    fn materialize_constant(
        &self,
        node: &SyntaxNode,
        constants: &mut ConstantTable,
    ) -> Result<(), CompileError> {
        match node.kind {
            SyntaxKind::IntegerExpression => {
                let mut bytes = [0; 8];

                bytes[0..4].copy_from_slice(&node.payload.0.to_le_bytes());
                bytes[4..8].copy_from_slice(&node.payload.1.to_le_bytes());

                let integer = i64::from_le_bytes(bytes);

                constants.add_integer(integer);
            }
            _ => todo!(),
        }

        Ok(())
    }

    fn get_next_register(&mut self) -> u16 {
        if let Some(register) = self.reusable_registers.pop() {
            register
        } else {
            self.instructions
                .iter()
                .map(|instruction| instruction.a_field() + 1)
                .max()
                .unwrap_or_default()
        }
    }

    fn add_reusable_register(&mut self, register: u16) {
        if let Err(insertion_index) = self
            .reusable_registers
            .binary_search_by(|reusable| reusable.cmp(&register).reverse())
        {
            self.reusable_registers.insert(insertion_index, register);
        }
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

    fn push_constant(&mut self, stable_constant: AbstractConstant) -> u16 {
        let stable_index = self.constants.len() as u16;

        if !self.constants.contains(&stable_constant) {
            self.constants.push(stable_constant);
        }

        stable_index
    }

    fn fold_expression(&mut self, expression: SyntaxId) -> Result<Option<u16>, CompileError> {
        let node = self
            .syntax_tree
            .get_node(expression)
            .copied()
            .ok_or(CompileError::MissingSyntaxNode { id: expression })?;

        match node.kind {
            SyntaxKind::CharacterExpression
            | SyntaxKind::FloatExpression
            | SyntaxKind::IntegerExpression
            | SyntaxKind::StringExpression => {
                let constant = AbstractConstant::Raw { expression };
                let constant_index = self.push_constant(constant);

                Ok(Some(constant_index))
            }
            SyntaxKind::AdditionExpression
            | SyntaxKind::SubtractionExpression
            | SyntaxKind::MultiplicationExpression
            | SyntaxKind::DivisionExpression
            | SyntaxKind::ModuloExpression => {
                let (left_index, right_index) =
                    (SyntaxId(node.payload.0), SyntaxId(node.payload.1));

                let Some(left_constant_index) = self.fold_expression(left_index)? else {
                    return Ok(None);
                };
                let Some(right_constant_index) = self.fold_expression(right_index)? else {
                    return Ok(None);
                };

                let left_constant = *self.constants.get(left_constant_index as usize).ok_or(
                    CompileError::MissingConstant {
                        constant_index: left_constant_index,
                    },
                )?;
                let right_constant = *self.constants.get(right_constant_index as usize).ok_or(
                    CompileError::MissingConstant {
                        constant_index: right_constant_index,
                    },
                )?;

                let folded_constant = left_constant.fold(right_constant);
                let folded_stable_index = self.push_constant(folded_constant);

                trace!(
                    "Folding {kind}: {left_constant:?} {operator} {right_constant:?} -> {folded_constant:?}",
                    kind = node.kind,
                    operator = match node.kind {
                        SyntaxKind::AdditionExpression => "+",
                        SyntaxKind::SubtractionExpression => "-",
                        SyntaxKind::MultiplicationExpression => "*",
                        SyntaxKind::DivisionExpression => "/",
                        SyntaxKind::ModuloExpression => "%",
                        _ => unreachable!("Expected binary expression, found: {}", node.kind),
                    },
                );

                Ok(Some(folded_stable_index))
            }
            SyntaxKind::GroupedExpression => self.fold_expression(SyntaxId(node.payload.0)),
            _ => Ok(None),
        }
    }

    fn compile_statement(&mut self, node: SyntaxNode) -> Result<(), CompileError> {
        info!("{}", node.kind);

        match node.kind {
            SyntaxKind::MainFunctionItem => self.compile_main_function_statement(node),
            SyntaxKind::LetStatement => self.compile_let_statement(node),
            SyntaxKind::FunctionStatement => todo!("Compile function statement"),
            SyntaxKind::ExpressionStatement => todo!("Compile expression statement"),
            SyntaxKind::SemicolonStatement => todo!("Compile semicolon statement"),
            _ => Err(CompileError::ExpectedStatement {
                node_kind: node.kind,
                position: node.position,
            }),
        }
    }

    fn compile_expression(&mut self, node: SyntaxNode) -> Result<Instruction, CompileError> {
        info!("{}", node.kind);

        match node.kind {
            SyntaxKind::CharacterExpression => self.compile_character_value_expression(node),
            SyntaxKind::AdditionExpression
            | SyntaxKind::SubtractionExpression
            | SyntaxKind::MultiplicationExpression
            | SyntaxKind::DivisionExpression
            | SyntaxKind::ModuloExpression => self.compile_binary_expression(node),
            SyntaxKind::GroupedExpression => self.compile_grouped_expression(node),
            SyntaxKind::PathExpression => self.parse_path_expression(node),
            _ => Err(CompileError::ExpectedExpression {
                node_kind: node.kind,
                position: node.position,
            }),
        }
    }

    fn compile_main_function_statement(&mut self, node: SyntaxNode) -> Result<(), CompileError> {
        let (start_children, child_count) = (node.payload.0 as usize, node.payload.1 as usize);
        let end_children = start_children + child_count;

        let mut current_child_index = start_children;

        while current_child_index < end_children {
            let node_index = self
                .syntax_tree
                .children
                .get(current_child_index)
                .copied()
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: current_child_index as u32,
                })?;
            let child_node = self
                .syntax_tree
                .get_node(node_index)
                .copied()
                .ok_or(CompileError::MissingSyntaxNode { id: node_index })?;

            match self.compile_statement(child_node) {
                Ok(()) => {}
                Err(CompileError::ExpectedStatement { .. })
                    if current_child_index == end_children - 1 =>
                {
                    self.compile_implicit_return(node_index)?;
                }
                Err(error) => {
                    return Err(error);
                }
            }

            current_child_index += 1;
        }

        Ok(())
    }

    fn compile_let_statement(&mut self, node: SyntaxNode) -> Result<(), CompileError> {
        let (children_start, child_count) = (node.payload.0, node.payload.1);
        let identifier_index = children_start as usize;

        self.type_table.Ok(())
    }

    fn compile_character_value_expression(
        &mut self,
        node: SyntaxNode,
    ) -> Result<Instruction, CompileError> {
        let character_value = char::from_u32(node.payload.0).unwrap_or_default();

        let constant = AbstractConstant::Raw {
            expression: constant_index,
        };
        let stable_constant_index = self.push_constant(constant);
        let address = Address::constant(stable_constant_index);
        let destination = Address::register(self.get_next_register());
        let load = Instruction::load(destination, address, OperandType::INTEGER, false);

        Ok(load)
    }

    fn compile_binary_expression(&mut self, node: SyntaxNode) -> Result<Instruction, CompileError> {
        if let Some(stable_constant_index) = self.fold_expression(node)? {
            let address = Address::constant(stable_constant_index);
            let destination = Address::register(self.get_next_register());
            let load = Instruction::load(destination, address, OperandType::INTEGER, false);

            return Ok(load);
        }

        let left_index = node.child as usize;
        let left =
            self.syntax_tree
                .nodes
                .get(left_index)
                .copied()
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: node.child,
                })?;
        let right_index = node.payload as usize;
        let right =
            self.syntax_tree
                .nodes
                .get(right_index)
                .copied()
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: node.payload,
                })?;

        let left_instruction = self.compile_expression(left)?;
        let left_address = self.handle_operand(left_instruction);

        let right_instruction = self.compile_expression(right)?;
        let right_address = self.handle_operand(right_instruction);

        let result_type = match (
            left_instruction.operand_type(),
            right_instruction.operand_type(),
        ) {
            (OperandType::INTEGER, OperandType::INTEGER) => OperandType::INTEGER,
            (OperandType::FLOAT, OperandType::FLOAT) => OperandType::FLOAT,
            _ => todo!("Type error"),
        };

        let destination = Address::register(self.get_next_register());
        let instruction = match node.kind {
            SyntaxKind::AdditionExpression => {
                Instruction::add(destination, left_address, right_address, result_type)
            }
            SyntaxKind::SubtractionExpression => {
                Instruction::subtract(destination, left_address, right_address, result_type)
            }
            SyntaxKind::MultiplicationExpression => {
                Instruction::multiply(destination, left_address, right_address, result_type)
            }
            SyntaxKind::DivisionExpression => {
                Instruction::divide(destination, left_address, right_address, result_type)
            }
            SyntaxKind::ModuloExpression => {
                Instruction::modulo(destination, left_address, right_address, result_type)
            }
            _ => unreachable!("Expected binary expression, found {}", node.kind),
        };

        Ok(instruction)
    }

    fn compile_grouped_expression(
        &mut self,
        node: SyntaxNode,
    ) -> Result<Instruction, CompileError> {
        let child_index = node.child as usize;
        let child_node =
            self.syntax_tree
                .nodes
                .get(child_index)
                .copied()
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: node.child,
                })?;

        if let Some(stable_constant_index) = self.fold_expression(node)? {
            let address = Address::constant(stable_constant_index);
            let destination = Address::register(self.get_next_register());
            let load = Instruction::load(destination, address, OperandType::INTEGER, false);

            Ok(load)
        } else {
            self.compile_expression(child_node)
        }
    }

    fn parse_path_expression(&mut self, node: SyntaxNode) -> Result<Instruction, CompileError> {
        let local_index = node.payload as usize;
        let destination = Address::register(self.get_next_register());
        let local = self
            .syntax_tree
            .locals
            .get(local_index)
            .ok_or(CompileError::MissingLocal {
                node_kind: node.kind,
                local_index: local_index as u32,
            })?;
        let local_register = self.local_registers[local_index];
        let local_address = Address::register(local_register);
        let load = Instruction::load(
            destination,
            local_address,
            local.r#type.as_operand_type(),
            false,
        );

        Ok(load)
    }

    fn compile_implicit_return(&mut self, expression: SyntaxId) -> Result<(), CompileError> {
        let definition = self.module_graph.get_definition();
        let expression_type = self.syntax_tree.resolve_type(expression);
        let expression_node =
            self.syntax_tree
                .get_node(expression)
                .copied()
                .ok_or(CompileError::MissingChild {
                    parent_kind: SyntaxKind::MainFunctionItem,
                    child_index: expression,
                })?;
        let expression_instruction = self.compile_expression(expression_node)?;
        let (returns_value, value_address) = if expression_type == Type::None {
            self.instructions.push(expression_instruction);

            (false, Address::default())
        } else {
            let address = self.handle_operand(expression_instruction);

            (true, address)
        };
        let r#return = Instruction::r#return(
            returns_value,
            value_address,
            expression_type.as_operand_type(),
        );

        self.instructions.push(r#return);

        Ok(())
    }
}
