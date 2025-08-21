use tracing::{Level, info, span, trace};

use crate::{
    Address, Chunk, FunctionType, Instruction, Lexer, OperandType, Operation, Program, Span, Type,
    Value,
    dust_error::{AnnotatedError, DustError, ErrorMessage},
    parser::Parser,
    syntax_tree::{SyntaxKind, SyntaxNode, SyntaxTree},
};

pub fn compile(source: &'_ str) -> Result<Chunk, DustError<'_>> {
    let lexer = Lexer::new(source);
    let parser = Parser::new(lexer);
    let (syntax_tree, _errors) = parser.parse();
    let compiler = ChunkCompiler::new(syntax_tree);

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
pub struct ChunkCompiler {
    /// Target syntax tree for compilation.
    ///
    /// The tree is owned because it is mutated in a way that makes it invalid.
    syntax_tree: SyntaxTree,

    /// Bytecode instruction collection that is filled during compilation.
    instructions: Vec<Instruction>,

    /// Deduplicated and sorted list of constants (by their index in the syntax tree) that are used
    /// in the final program. Instructions that use constants will refer to them by their index in
    /// this list.
    stable_constants: Vec<u16>,

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
}

impl ChunkCompiler {
    pub fn new(syntax_tree: SyntaxTree) -> Self {
        let local_registers = vec![0; syntax_tree.locals.len()];

        Self {
            syntax_tree,
            instructions: Vec::new(),
            stable_constants: Vec::new(),
            call_arguments: Vec::new(),
            drop_lists: Vec::new(),
            return_type: Type::None,
            prototype_index: 0,
            local_registers,
            next_unused_register: 0,
            reusable_registers: Vec::new(),
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
            .ok_or(CompileError::MissingMainFunction)?;

        self.compile_statement(main_function_node)?;

        let mut constants = Vec::with_capacity(self.stable_constants.len());

        for index in self.stable_constants {
            self.syntax_tree.constants.push(Value::Boolean(false));

            let constant = self.syntax_tree.constants.swap_remove(index as usize);

            constants.push(constant);
        }

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

    fn get_next_register(&mut self) -> u16 {
        if let Some(register) = self.reusable_registers.pop() {
            register
        } else {
            let next_register = self.next_unused_register;
            self.next_unused_register += 1;

            next_register
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

    fn push_stable_constant(&mut self, constant_index: u16) -> u16 {
        let stable_index = self.stable_constants.len() as u16;

        if !self.stable_constants.contains(&constant_index) {
            self.stable_constants.push(constant_index);
        }

        stable_index
    }

    fn fold_expression(&mut self, node: SyntaxNode) -> Result<Option<usize>, CompileError> {
        match node.kind {
            SyntaxKind::IntegerExpression => {
                let constant_index = node.payload as usize;

                Ok(Some(constant_index))
            }
            SyntaxKind::AdditionExpression
            | SyntaxKind::SubtractionExpression
            | SyntaxKind::MultiplicationExpression
            | SyntaxKind::DivisionExpression
            | SyntaxKind::ModuloExpression => {
                let left_index = node.child as usize;
                let right_index = node.payload as usize;

                let left_node = self.syntax_tree.nodes.get(left_index).copied().ok_or(
                    CompileError::MissingChild {
                        parent_kind: node.kind,
                        child_index: node.child,
                    },
                )?;
                let right_node = self.syntax_tree.nodes.get(right_index).copied().ok_or(
                    CompileError::MissingChild {
                        parent_kind: node.kind,
                        child_index: node.payload,
                    },
                )?;

                let Some(left_constant_index) = self.fold_expression(left_node)? else {
                    return Ok(None);
                };
                let Some(right_constant_index) = self.fold_expression(right_node)? else {
                    return Ok(None);
                };

                let left_constant = self.syntax_tree.constants.get(left_constant_index).ok_or(
                    CompileError::MissingConstant {
                        constant_index: left_constant_index as u32,
                    },
                )?;
                let right_constant = self.syntax_tree.constants.get(right_constant_index).ok_or(
                    CompileError::MissingConstant {
                        constant_index: right_constant_index as u32,
                    },
                )?;

                let folded_constant = match (left_constant, right_constant, node.kind) {
                    (
                        Value::Integer(left),
                        Value::Integer(right),
                        SyntaxKind::SubtractionExpression,
                    ) => Value::Integer(left.saturating_sub(*right)),
                    (
                        Value::Integer(left),
                        Value::Integer(right),
                        SyntaxKind::AdditionExpression,
                    ) => Value::Integer(left.saturating_add(*right)),
                    (
                        Value::Integer(left),
                        Value::Integer(right),
                        SyntaxKind::MultiplicationExpression,
                    ) => Value::Integer(left.saturating_mul(*right)),
                    (
                        Value::Integer(left),
                        Value::Integer(right),
                        SyntaxKind::DivisionExpression,
                    ) => {
                        if *right == 0 {
                            return Err(CompileError::DivisionByZero {
                                node_kind: node.kind,
                                position: right_node.span,
                            });
                        }
                        Value::Integer(left.saturating_div(*right))
                    }
                    (Value::Integer(left), Value::Integer(right), SyntaxKind::ModuloExpression) => {
                        if *right == 0 {
                            return Err(CompileError::DivisionByZero {
                                node_kind: node.kind,
                                position: right_node.span,
                            });
                        }
                        Value::Integer(left % *right)
                    }
                    _ => return Ok(None),
                };

                trace!(
                    "Folding expression: {left} {operator} {right} = {folded_constant}",
                    left = left_constant,
                    right = right_constant,
                    operator = match node.kind {
                        SyntaxKind::AdditionExpression => "+",
                        SyntaxKind::SubtractionExpression => "-",
                        SyntaxKind::MultiplicationExpression => "*",
                        SyntaxKind::DivisionExpression => "/",
                        SyntaxKind::ModuloExpression => "%",
                        _ => unreachable!("Expected binary expression, found: {}", node.kind),
                    }
                );

                let folded_constant_index = self.syntax_tree.push_constant(folded_constant) as u16;

                Ok(Some(folded_constant_index as usize))
            }
            SyntaxKind::GroupedExpression => {
                let child_index = node.child as usize;
                let child_node = self.syntax_tree.nodes.get(child_index).copied().ok_or(
                    CompileError::MissingChild {
                        parent_kind: node.kind,
                        child_index: node.child,
                    },
                )?;

                self.fold_expression(child_node)
            }
            _ => Ok(None),
        }
    }

    fn compile_statement(&mut self, node: SyntaxNode) -> Result<(), CompileError> {
        info!("{}", node.kind);

        match node.kind {
            SyntaxKind::MainFunctionStatement => self.compile_main_function_statement(node),
            SyntaxKind::LetStatement => self.compile_let_statement(node),
            SyntaxKind::FunctionStatement => todo!("Compile function statement"),
            SyntaxKind::ExpressionStatement => todo!("Compile expression statement"),
            SyntaxKind::SemicolonStatement => todo!("Compile semicolon statement"),
            _ => Err(CompileError::ExpectedStatement {
                node_kind: node.kind,
                position: node.span,
            }),
        }
    }

    fn compile_expression(&mut self, node: SyntaxNode) -> Result<Instruction, CompileError> {
        info!("{}", node.kind);

        match node.kind {
            SyntaxKind::IntegerExpression => self.compile_integer_expression(node),
            SyntaxKind::AdditionExpression
            | SyntaxKind::SubtractionExpression
            | SyntaxKind::MultiplicationExpression
            | SyntaxKind::DivisionExpression
            | SyntaxKind::ModuloExpression => self.compile_binary_expression(node),
            SyntaxKind::GroupedExpression => self.compile_grouped_expression(node),
            _ => Err(CompileError::ExpectedExpression {
                node_kind: node.kind,
                position: node.span,
            }),
        }
    }

    fn compile_main_function_statement(&mut self, node: SyntaxNode) -> Result<(), CompileError> {
        let start_children = node.child as usize;
        let child_count = node.payload as usize;
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
                .ok_or(CompileError::MissingSyntaxNode { node_index })?;

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
        let local_index = node.payload as usize;
        let expression_node_index = node.child as usize;
        let expression_node = self
            .syntax_tree
            .nodes
            .get(expression_node_index)
            .copied()
            .ok_or(CompileError::MissingChild {
                parent_kind: node.kind,
                child_index: node.child,
            })?;
        let value_instruction = self.compile_expression(expression_node)?;
        let local_address = value_instruction.a_field();

        self.local_registers[local_index] = local_address;

        self.instructions.push(value_instruction);

        Ok(())
    }

    fn compile_integer_expression(
        &mut self,
        node: SyntaxNode,
    ) -> Result<Instruction, CompileError> {
        let constant_index = node.payload;
        let stable_constant_index = self.push_stable_constant(constant_index as u16);
        let address = Address::constant(stable_constant_index);
        let destination = Address::register(self.get_next_register());
        let load = Instruction::load(destination, address, OperandType::INTEGER, false);

        Ok(load)
    }

    fn compile_binary_expression(&mut self, node: SyntaxNode) -> Result<Instruction, CompileError> {
        if let Some(constant_index) = self.fold_expression(node)? {
            let stable_constant_index = self.push_stable_constant(constant_index as u16);
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

        if let Some(constant_index) = self.fold_expression(node)? {
            let stable_constant_index = self.push_stable_constant(constant_index as u16);
            let address = Address::constant(stable_constant_index);
            let destination = Address::register(self.get_next_register());
            let load = Instruction::load(destination, address, OperandType::INTEGER, false);

            Ok(load)
        } else {
            self.compile_expression(child_node)
        }
    }

    fn compile_implicit_return(&mut self, expression_node_index: u32) -> Result<(), CompileError> {
        let expression_type = self.syntax_tree.resolve_type(expression_node_index);
        let expression_node = self
            .syntax_tree
            .nodes
            .get(expression_node_index as usize)
            .copied()
            .ok_or(CompileError::MissingChild {
                parent_kind: SyntaxKind::MainFunctionStatement,
                child_index: expression_node_index,
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

#[derive(Debug, Clone)]
pub enum CompileError {
    DivisionByZero {
        node_kind: SyntaxKind,
        position: Span,
    },
    ExpectedExpression {
        node_kind: SyntaxKind,
        position: Span,
    },
    ExpectedStatement {
        node_kind: SyntaxKind,
        position: Span,
    },
    MissingChild {
        parent_kind: SyntaxKind,
        child_index: u32,
    },
    MissingConstant {
        constant_index: u32,
    },
    MissingMainFunction,
    MissingLocal {
        node_kind: SyntaxKind,
        local_index: u32,
    },
    MissingSyntaxNode {
        node_index: u32,
    },
}

impl AnnotatedError for CompileError {
    fn annotated_error(&self) -> ErrorMessage {
        let title = "Compilation Error";

        match self {
            CompileError::DivisionByZero { position, .. } => ErrorMessage {
                title,
                description: "Dividing by zero is mathematically undefined for integers. Dust does not allow it.",
                detail_snippets: vec![("This value is zero.".to_string(), *position)],
                help_snippet: Some("This is a compile-time error caused by hard-coded values. Check your math for errors. If you absolutely must divide by zero, floats allow it but the result is always Infinity or NaN.".to_string()),
            },
            CompileError::ExpectedExpression { node_kind, position } => ErrorMessage {
                title,
                description: "The syntax tree contains a statement where an expression was expected.",
                detail_snippets: vec![(node_kind.to_string(), *position)],
                help_snippet: Some("This a bug in the parser.".to_string()),
            },
            CompileError::ExpectedStatement { node_kind, position } => ErrorMessage {
                title,
                description: "The syntax tree contains an expression where a statement was expected.",
                detail_snippets: vec![(node_kind.to_string(), *position)],
                help_snippet: Some("This a bug in the parser.".to_string()),
            },
            CompileError::MissingChild {
                parent_kind,
                child_index,
            } => todo!(),
            CompileError::MissingConstant { constant_index } => todo!(),
            CompileError::MissingMainFunction => todo!(),
            CompileError::MissingLocal {
                node_kind,
                local_index,
            } => todo!(),
            CompileError::MissingSyntaxNode { node_index } => ErrorMessage {
                title,
                description: "The syntax tree is missing a node that is required for compilation.",
                detail_snippets: vec![(format!("Node index {node_index}"), Span::default())],
                help_snippet: Some("This is a bug in the parser.".to_string()),
            },
        }
    }
}
