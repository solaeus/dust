use crate::{
    compiler::CompileError,
    source::{Position, SourceFileId},
    syntax::{SyntaxKind, reader::SyntaxReader},
};

pub trait SyntaxVisitor {
    type Input;

    type Output;

    fn file_id(&self) -> SourceFileId;

    fn visit(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        match node.kind() {
            SyntaxKind::MainFunctionItem => self.visit_main_function_item(node, input),
            SyntaxKind::ModuleItem | SyntaxKind::PublicModuleItem => {
                self.visit_module_item(node, input)
            }
            SyntaxKind::FunctionItem | SyntaxKind::PublicFunctionItem => {
                self.visit_function_item(node, input)
            }
            SyntaxKind::UseItem | SyntaxKind::PublicUseItem => self.visit_use_item(node, input),
            SyntaxKind::ExpressionStatement => self.visit_expression_statement(node, input),
            SyntaxKind::LetStatement | SyntaxKind::LetMutStatement => {
                self.visit_let_statement(node, input)
            }
            SyntaxKind::ReassignmentStatement => self.visit_reassignment_statement(node, input),
            SyntaxKind::AdditionAssignmentStatement
            | SyntaxKind::SubtractionAssignmentStatement
            | SyntaxKind::MultiplicationAssignmentStatement
            | SyntaxKind::DivisionAssignmentStatement
            | SyntaxKind::ModuloAssignmentStatement
            | SyntaxKind::ExponentAssignmentStatement => {
                self.visit_binary_assignment_statement(node, input)
            }
            SyntaxKind::PathExpression => self.visit_path_expression(node, input),
            SyntaxKind::BooleanExpression => self.visit_boolean_expression(node, input),
            SyntaxKind::ByteExpression => self.visit_byte_expression(node, input),
            SyntaxKind::CharacterExpression => self.visit_character_expression(node, input),
            SyntaxKind::FloatExpression => self.visit_float_expression(node, input),
            SyntaxKind::IntegerExpression => self.visit_integer_expression(node, input),
            SyntaxKind::StringExpression => self.visit_string_expression(node, input),
            SyntaxKind::ListExpression => self.visit_list_expression(node, input),
            SyntaxKind::IndexExpression => self.visit_index_expression(node, input),
            SyntaxKind::BlockExpression => self.visit_block_expression(node, input),
            SyntaxKind::IfExpression => self.visit_if_expression(node, input),
            SyntaxKind::ElseExpression => self.visit_else_expression(node, input),
            SyntaxKind::WhileExpression => self.visit_while_expression(node, input),
            SyntaxKind::FunctionExpression => self.visit_function_expression(node, input),
            SyntaxKind::CallExpression => self.visit_call_expression(node, input),
            SyntaxKind::AdditionExpression
            | SyntaxKind::SubtractionExpression
            | SyntaxKind::MultiplicationExpression
            | SyntaxKind::DivisionExpression
            | SyntaxKind::ModuloExpression
            | SyntaxKind::ExponentExpression => self.visit_math_binary_expression(node, input),
            SyntaxKind::EqualExpression
            | SyntaxKind::NotEqualExpression
            | SyntaxKind::LessThanExpression
            | SyntaxKind::LessThanOrEqualExpression
            | SyntaxKind::GreaterThanExpression
            | SyntaxKind::GreaterThanOrEqualExpression => {
                self.visit_comparison_binary_expression(node, input)
            }
            SyntaxKind::AndExpression | SyntaxKind::OrExpression => {
                self.visit_logical_binary_expression(node, input)
            }
            SyntaxKind::NegationExpression | SyntaxKind::NotExpression => {
                self.visit_unary_negation_expression(node, input)
            }
            SyntaxKind::GroupedExpression => {
                let child = node
                    .left_child()
                    .ok_or_else(|| CompileError::ExpectedExpression {
                        node_kind: node.kind(),
                        position: Position::new(self.file_id(), node.span()),
                    })?;

                self.visit(child, input)
            }
            _ => todo!("Unhandled syntax kind: {:?}", node.kind()),
        }
    }

    fn visit_item(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        match node.kind() {
            SyntaxKind::MainFunctionItem => self.visit_main_function_item(node, input),
            SyntaxKind::ModuleItem => self.visit_module_item(node, input),
            SyntaxKind::FunctionItem => self.visit_function_item(node, input),
            SyntaxKind::UseItem => self.visit_use_item(node, input),
            _ => Err(CompileError::ExpectedItem {
                node_kind: node.kind(),
                position: Position::new(self.file_id(), node.span()),
            }),
        }
    }

    fn visit_statement(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        match node.kind() {
            SyntaxKind::ExpressionStatement => self.visit_expression_statement(node, input),
            SyntaxKind::ReassignmentStatement => self.visit_reassignment_statement(node, input),
            SyntaxKind::LetStatement | SyntaxKind::LetMutStatement => {
                self.visit_let_statement(node, input)
            }
            _ => Err(CompileError::ExpectedStatement {
                node_kind: node.kind(),
                position: Position::new(self.file_id(), node.span()),
            }),
        }
    }

    fn visit_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError> {
        match node.kind() {
            SyntaxKind::PathExpression => self.visit_path_expression(node, input),
            SyntaxKind::BooleanExpression => self.visit_boolean_expression(node, input),
            SyntaxKind::ByteExpression => self.visit_byte_expression(node, input),
            SyntaxKind::CharacterExpression => self.visit_character_expression(node, input),
            SyntaxKind::FloatExpression => self.visit_float_expression(node, input),
            SyntaxKind::IntegerExpression => self.visit_integer_expression(node, input),
            SyntaxKind::StringExpression => self.visit_string_expression(node, input),
            SyntaxKind::ListExpression => self.visit_list_expression(node, input),
            SyntaxKind::IndexExpression => self.visit_index_expression(node, input),
            SyntaxKind::AdditionExpression
            | SyntaxKind::SubtractionExpression
            | SyntaxKind::MultiplicationExpression
            | SyntaxKind::DivisionExpression
            | SyntaxKind::ModuloExpression
            | SyntaxKind::ExponentExpression => self.visit_math_binary_expression(node, input),
            SyntaxKind::EqualExpression
            | SyntaxKind::NotEqualExpression
            | SyntaxKind::LessThanExpression
            | SyntaxKind::LessThanOrEqualExpression
            | SyntaxKind::GreaterThanExpression
            | SyntaxKind::GreaterThanOrEqualExpression => {
                self.visit_comparison_binary_expression(node, input)
            }
            SyntaxKind::AndExpression | SyntaxKind::OrExpression => {
                self.visit_logical_binary_expression(node, input)
            }
            SyntaxKind::BlockExpression => self.visit_block_expression(node, input),
            SyntaxKind::IfExpression => self.visit_if_expression(node, input),
            SyntaxKind::WhileExpression => self.visit_while_expression(node, input),
            SyntaxKind::FunctionExpression => self.visit_function_expression(node, input),
            SyntaxKind::CallExpression => self.visit_call_expression(node, input),
            SyntaxKind::GroupedExpression => {
                let child = node
                    .left_child()
                    .ok_or_else(|| CompileError::ExpectedExpression {
                        node_kind: node.kind(),
                        position: Position::new(self.file_id(), node.span()),
                    })?;

                self.visit(child, input)
            }
            _ => Err(CompileError::ExpectedExpression {
                node_kind: node.kind(),
                position: Position::new(self.file_id(), node.span()),
            }),
        }
    }

    fn visit_main_function_item(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError>;

    fn visit_module_item(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError>;

    fn visit_function_item(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError>;

    fn visit_use_item(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError>;

    fn visit_expression_statement(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError>;

    fn visit_reassignment_statement(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError>;

    fn visit_boolean_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError>;

    fn visit_byte_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError>;

    fn visit_character_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError>;

    fn visit_float_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError>;

    fn visit_integer_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError>;

    fn visit_string_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError>;

    fn visit_list_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError>;

    fn visit_index_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError>;

    fn visit_path_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError>;

    fn visit_block_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError>;

    fn visit_if_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError>;

    fn visit_else_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError>;

    fn visit_let_statement(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError>;

    fn visit_binary_assignment_statement(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError>;

    fn visit_math_binary_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError>;

    fn visit_comparison_binary_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError>;

    fn visit_logical_binary_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError>;

    fn visit_unary_negation_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError>;

    fn visit_while_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError>;

    fn visit_function_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError>;

    fn visit_call_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::Input,
    ) -> Result<Self::Output, CompileError>;
}
