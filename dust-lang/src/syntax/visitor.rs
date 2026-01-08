use crate::{
    compiler::CompileError,
    source::{Position, SourceFileId},
    syntax::{SyntaxKind, reader::SyntaxReader},
};

pub trait SyntaxVisitor {
    type Output;

    fn file_id(&self) -> SourceFileId;

    fn visit(&mut self, node: SyntaxReader<'_>) -> Result<Self::Output, CompileError> {
        match node.kind() {
            SyntaxKind::MainFunctionItem => self.visit_main_function_item(node),
            SyntaxKind::ModuleItem | SyntaxKind::PublicModuleItem => self.visit_module_item(node),
            SyntaxKind::FunctionItem | SyntaxKind::PublicFunctionItem => {
                self.visit_function_item(node)
            }
            SyntaxKind::UseItem | SyntaxKind::PublicUseItem => self.visit_use_item(node),
            SyntaxKind::ExpressionStatement => self.visit_expression_statement(node),
            SyntaxKind::LetStatement | SyntaxKind::LetMutStatement => {
                self.visit_let_statement(node)
            }
            SyntaxKind::ReassignmentStatement => self.visit_reassignment_statement(node),

            SyntaxKind::IntegerExpression => self.visit_integer_expression(node),

            SyntaxKind::PathExpression => self.visit_path_expression(node),
            SyntaxKind::BlockExpression => self.visit_block_expression(node),
            SyntaxKind::IfExpression => self.visit_if_expression(node),
            SyntaxKind::WhileExpression => self.visit_while_expression(node),
            SyntaxKind::FunctionExpression => self.visit_function_expression(node),
            SyntaxKind::CallExpression => self.visit_call_expression(node),
            SyntaxKind::AdditionExpression
            | SyntaxKind::SubtractionExpression
            | SyntaxKind::MultiplicationExpression
            | SyntaxKind::DivisionExpression
            | SyntaxKind::ModuloExpression
            | SyntaxKind::ExponentExpression => self.visit_math_expression(node),
            _ => todo!(),
        }
    }

    fn visit_item(&mut self, node: SyntaxReader<'_>) -> Result<Self::Output, CompileError> {
        match node.kind() {
            SyntaxKind::MainFunctionItem => self.visit_main_function_item(node),
            SyntaxKind::ModuleItem => self.visit_module_item(node),
            SyntaxKind::FunctionItem => self.visit_function_item(node),
            SyntaxKind::UseItem => self.visit_use_item(node),
            _ => Err(CompileError::ExpectedItem {
                node_kind: node.kind(),
                position: Position::new(self.file_id(), node.span()),
            }),
        }
    }

    fn visit_statement(&mut self, node: SyntaxReader<'_>) -> Result<Self::Output, CompileError> {
        match node.kind() {
            SyntaxKind::ExpressionStatement => self.visit_expression_statement(node),
            SyntaxKind::ReassignmentStatement => self.visit_reassignment_statement(node),
            SyntaxKind::LetStatement | SyntaxKind::LetMutStatement => {
                self.visit_let_statement(node)
            }
            _ => Err(CompileError::ExpectedStatement {
                node_kind: node.kind(),
                position: Position::new(self.file_id(), node.span()),
            }),
        }
    }

    fn visit_expression(&mut self, node: SyntaxReader<'_>) -> Result<Self::Output, CompileError> {
        match node.kind() {
            SyntaxKind::IntegerExpression => self.visit_integer_expression(node),
            SyntaxKind::StringExpression => self.visit_string_expression(node),
            SyntaxKind::BlockExpression => self.visit_block_expression(node),
            SyntaxKind::IfExpression => self.visit_if_expression(node),
            SyntaxKind::WhileExpression => self.visit_while_expression(node),
            SyntaxKind::FunctionExpression => self.visit_function_expression(node),
            SyntaxKind::CallExpression => self.visit_call_expression(node),
            _ => Err(CompileError::ExpectedExpression {
                node_kind: node.kind(),
                position: Position::new(self.file_id(), node.span()),
            }),
        }
    }

    fn visit_main_function_item(
        &mut self,
        node: SyntaxReader<'_>,
    ) -> Result<Self::Output, CompileError>;

    fn visit_module_item(&mut self, node: SyntaxReader<'_>) -> Result<Self::Output, CompileError>;

    fn visit_function_item(&mut self, node: SyntaxReader<'_>)
    -> Result<Self::Output, CompileError>;

    fn visit_use_item(&mut self, node: SyntaxReader<'_>) -> Result<Self::Output, CompileError>;

    fn visit_expression_statement(
        &mut self,
        node: SyntaxReader<'_>,
    ) -> Result<Self::Output, CompileError>;

    fn visit_reassignment_statement(
        &mut self,
        node: SyntaxReader<'_>,
    ) -> Result<Self::Output, CompileError>;

    fn visit_integer_expression(
        &mut self,
        node: SyntaxReader<'_>,
    ) -> Result<Self::Output, CompileError>;

    fn visit_string_expression(
        &mut self,
        node: SyntaxReader<'_>,
    ) -> Result<Self::Output, CompileError>;

    fn visit_path_expression(
        &mut self,
        node: SyntaxReader<'_>,
    ) -> Result<Self::Output, CompileError>;

    fn visit_block_expression(
        &mut self,
        node: SyntaxReader<'_>,
    ) -> Result<Self::Output, CompileError>;

    fn visit_if_expression(&mut self, node: SyntaxReader<'_>)
    -> Result<Self::Output, CompileError>;

    fn visit_let_statement(&mut self, node: SyntaxReader<'_>)
    -> Result<Self::Output, CompileError>;

    fn visit_math_expression(
        &mut self,
        node: SyntaxReader<'_>,
    ) -> Result<Self::Output, CompileError>;

    fn visit_while_expression(
        &mut self,
        node: SyntaxReader<'_>,
    ) -> Result<Self::Output, CompileError>;

    fn visit_function_expression(
        &mut self,
        node: SyntaxReader<'_>,
    ) -> Result<Self::Output, CompileError>;

    fn visit_call_expression(
        &mut self,
        node: SyntaxReader<'_>,
    ) -> Result<Self::Output, CompileError>;
}
