use tracing::trace;

use crate::{
    Address, Chunk, FunctionType, Instruction, Lexer, OperandType, Operation, Type, Value,
    instruction::Load,
    parser::Parser,
    syntax_tree::{SyntaxKind, SyntaxNode, SyntaxTree},
};

pub fn compile(source: &str) -> Result<Chunk, CompileError> {
    let lexer = Lexer::new(source);
    let parser = Parser::new(lexer);
    let syntax_tree = parser.parse_main();
    let compiler = Compiler::new(syntax_tree);

    compiler.compile()
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

        for index in self.stable_constants.into_iter().rev() {
            println!("{index}");

            let constant = self.syntax_tree.constants.remove(index as usize);

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

    fn compile_statement(&mut self, node: SyntaxNode) -> Result<(), CompileError> {
        match node.kind {
            SyntaxKind::MainFunctionStatement => self.compile_main_function_statement(node),
            SyntaxKind::LetStatement => self.compile_let_statement(node),
            SyntaxKind::FunctionStatement => todo!("Compile function statement"),
            SyntaxKind::ExpressionStatement => todo!("Compile expression statement"),
            SyntaxKind::SemicolonStatement => todo!("Compile semicolon statement"),
            _ => Err(CompileError::ExpectedStatement { found: node.kind }),
        }
    }

    fn compile_expression(&mut self, node: SyntaxNode) -> Result<Instruction, CompileError> {
        match node.kind {
            SyntaxKind::IntegerExpression => self.compile_integer_expression(node),
            SyntaxKind::SubtractionExpression => self.compile_subtraction_expression(node),
            _ => Err(CompileError::ExpectedExpression { found: node.kind }),
        }
    }

    fn compile_main_function_statement(&mut self, node: SyntaxNode) -> Result<(), CompileError> {
        trace!("Compiling main function statement");

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
        let expression_node = self
            .syntax_tree
            .get_node(node.child)
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
        let constant_index = node.payload as u16;
        let stable_constant_index = self.stable_constants.len() as u16;
        let address = Address::constant(stable_constant_index);
        let destination = Address::register(self.get_next_register());
        let load = Instruction::load(destination, address, OperandType::INTEGER, false);

        self.stable_constants.push(constant_index);

        Ok(load)
    }

    fn compile_subtraction_expression(
        &mut self,
        node: SyntaxNode,
    ) -> Result<Instruction, CompileError> {
        let left =
            self.syntax_tree
                .get_node(node.child)
                .copied()
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: node.child,
                })?;
        let right =
            self.syntax_tree
                .get_node(node.payload)
                .copied()
                .ok_or(CompileError::MissingChild {
                    parent_kind: node.kind,
                    child_index: node.payload,
                })?;

        match (left.kind, right.kind) {
            (SyntaxKind::IntegerExpression, SyntaxKind::IntegerExpression) => {
                let left_constant_index = left.payload as usize;
                let left_value = self
                    .syntax_tree
                    .constants
                    .get(left_constant_index)
                    .ok_or(CompileError::MissingConstant {
                        node_kind: SyntaxKind::IntegerExpression,
                        constant_index: left.payload,
                    })?
                    .as_integer()
                    .unwrap();
                let right_value = self
                    .syntax_tree
                    .constants
                    .get(right.payload as usize)
                    .ok_or(CompileError::MissingConstant {
                        node_kind: SyntaxKind::IntegerExpression,
                        constant_index: right.payload,
                    })?
                    .as_integer()
                    .unwrap();
                let folded_constant = Value::Integer(left_value - right_value);
                self.syntax_tree.constants[left_constant_index] = folded_constant;
                let stable_constant_index = self.stable_constants.len() as u16;
                let constant_address = Address::constant(stable_constant_index);

                self.stable_constants.push(left.payload as u16);

                let destination = Address::register(self.get_next_register());
                let load =
                    Instruction::load(destination, constant_address, OperandType::INTEGER, false);

                return Ok(load);
            }
            (SyntaxKind::FloatExpression, SyntaxKind::FloatExpression) => {
                todo!()
            }
            _ => {}
        }

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
            _ => return Err(CompileError::ExpectedExpression { found: node.kind }),
        };

        let destination = Address::register(self.get_next_register());
        let subtract = Instruction::subtract(destination, left_address, right_address, result_type);

        Ok(subtract)
    }
}

#[derive(Debug, Clone)]
pub enum CompileError {
    ExpectedExpression {
        found: SyntaxKind,
    },
    ExpectedStatement {
        found: SyntaxKind,
    },
    MissingChild {
        parent_kind: SyntaxKind,
        child_index: u32,
    },
    MissingConstant {
        node_kind: SyntaxKind,
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
