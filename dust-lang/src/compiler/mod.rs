use tracing::{Level, info, span, trace};

use crate::{
    Address, Chunk, FunctionType, Instruction, Lexer, OperandType, Operation, Span, Type, Value,
    dust_error::{AnnotatedError, DustError, ErrorMessage},
    parser::Parser,
    syntax_tree::{SyntaxKind, SyntaxNode, SyntaxTree},
};

pub fn compile(source: &'_ str) -> Result<Chunk, DustError<'_>> {
    let lexer = Lexer::new(source);
    let parser = Parser::new(lexer);
    let (syntax_tree, _errors) = parser.parse();
    let compiler = Compiler::new(syntax_tree);

    compiler
        .compile()
        .map_err(|error| DustError::compile(error, source))
}

#[derive(Debug)]
pub struct Compiler {
    syntax_tree: SyntaxTree,

    instructions: Vec<Instruction>,

    stable_constants: Vec<u16>,

    call_arguments: Vec<(Address, OperandType)>,

    drop_lists: Vec<u16>,

    return_type: Type,

    prototype_index: u16,

    local_registers: Vec<u16>,

    next_unused_register: u16,

    reusable_registers: Vec<u16>,
}

impl Compiler {
    pub fn new(syntax_tree: SyntaxTree) -> Self {
        Self {
            syntax_tree,
            instructions: Vec::new(),
            stable_constants: Vec::new(),
            call_arguments: Vec::new(),
            drop_lists: Vec::new(),
            return_type: Type::None,
            prototype_index: 0,
            local_registers: Vec::new(),
            next_unused_register: 0,
            reusable_registers: Vec::new(),
        }
    }

    pub fn compile(mut self) -> Result<Chunk, CompileError> {
        let span = span!(Level::INFO, "Compiling");
        let _enter = span.enter();

        self.local_registers
            .resize(self.syntax_tree.locals.len(), 0);

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

        constants.reverse();

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
            self.instructions
                .iter()
                .map(|instruction| instruction.a_field() + 1)
                .max()
                .unwrap_or_default()
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
        match self.stable_constants.binary_search(&constant_index) {
            Ok(index) => index as u16,
            Err(index) => {
                self.stable_constants.insert(index, constant_index);

                index as u16
            }
        }
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
                    CompileError::MissingSyntaxNode {
                        index: left_index as u32,
                    },
                )?;
                let right_node = self.syntax_tree.nodes.get(right_index).copied().ok_or(
                    CompileError::MissingSyntaxNode {
                        index: right_index as u32,
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
            _ => Err(CompileError::InvalidSyntaxNode {
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
            _ => Err(CompileError::InvalidSyntaxNode {
                node_kind: node.kind,
                position: node.span,
            }),
        }
    }

    fn compile_main_function_statement(&mut self, node: SyntaxNode) -> Result<(), CompileError> {
        let start_children = node.child;
        let child_count = node.payload;
        let end_children = start_children + child_count;

        let mut current_child_index = start_children;

        while current_child_index < end_children {
            let node_index = self
                .syntax_tree
                .children
                .get(current_child_index as usize)
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: current_child_index,
                })?;
            let child_node = self.syntax_tree.get_node(*node_index).copied().ok_or(
                CompileError::MissingSyntaxNode {
                    index: current_child_index,
                },
            )?;

            self.compile_statement(child_node)?;

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
            .ok_or(CompileError::MissingSyntaxNode { index: node.child })?;
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
        let constant_index = self
            .fold_expression(node)?
            .ok_or(CompileError::MissingConstant {
                constant_index: node.payload,
            })?;
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
            _ => {
                return Err(CompileError::InvalidSyntaxNode {
                    node_kind: node.kind,
                    position: node.span,
                });
            }
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
            _ => unreachable!("Expected binary expression, found: {:?}", node.kind),
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
}

#[derive(Debug, Clone)]
pub enum CompileError {
    DivisionByZero {
        node_kind: SyntaxKind,
        position: Span,
    },
    InvalidSyntaxNode {
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
    MissingSyntaxNode {
        index: u32,
    },
    MissingLocal {
        node_kind: SyntaxKind,
        local_index: u32,
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
            CompileError::InvalidSyntaxNode { node_kind, position } => ErrorMessage {
                title,
                description: "The syntax tree contains an invalid node.",
                detail_snippets: vec![(node_kind.to_string(), *position)],
                help_snippet: Some("This a bug in the parser.".to_string()),
            },
            CompileError::MissingChild {
                parent_kind,
                child_index,
            } => todo!(),
            CompileError::MissingConstant { constant_index } => todo!(),
            CompileError::MissingMainFunction => todo!(),
            CompileError::MissingSyntaxNode { index } => todo!(),
            CompileError::MissingLocal {
                node_kind,
                local_index,
            } => todo!(),
        }
    }
}
