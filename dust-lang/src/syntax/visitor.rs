use crate::syntax::{SyntaxKind, reader::SyntaxReader};

pub trait SyntaxVisitor {
    type Output;
    type Error;

    fn visit(&mut self, node: SyntaxReader<'_>) -> Result<Self::Output, Self::Error> {
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

    fn visit_item(&mut self, node: SyntaxReader<'_>) -> Result<Self::Output, Self::Error>;

    fn visit_statement(&mut self, node: SyntaxReader<'_>) -> Result<Self::Output, Self::Error>;

    fn visit_expression(&mut self, node: SyntaxReader<'_>) -> Result<Self::Output, Self::Error>;

    fn visit_main_function_item(
        &mut self,
        node: SyntaxReader<'_>,
    ) -> Result<Self::Output, Self::Error>;

    fn visit_module_item(&mut self, node: SyntaxReader<'_>) -> Result<Self::Output, Self::Error>;

    fn visit_function_item(&mut self, node: SyntaxReader<'_>) -> Result<Self::Output, Self::Error>;

    fn visit_use_item(&mut self, node: SyntaxReader<'_>) -> Result<Self::Output, Self::Error>;

    fn visit_expression_statement(
        &mut self,
        node: SyntaxReader<'_>,
    ) -> Result<Self::Output, Self::Error>;

    fn visit_reassignment_statement(
        &mut self,
        node: SyntaxReader<'_>,
    ) -> Result<Self::Output, Self::Error>;

    fn visit_integer_expression(
        &mut self,
        node: SyntaxReader<'_>,
    ) -> Result<Self::Output, Self::Error>;

    fn visit_path_expression(
        &mut self,
        node: SyntaxReader<'_>,
    ) -> Result<Self::Output, Self::Error>;

    fn visit_block_expression(
        &mut self,
        node: SyntaxReader<'_>,
    ) -> Result<Self::Output, Self::Error>;

    fn visit_if_expression(&mut self, node: SyntaxReader<'_>) -> Result<Self::Output, Self::Error>;

    fn visit_let_statement(&mut self, node: SyntaxReader<'_>) -> Result<Self::Output, Self::Error>;

    fn visit_math_expression(
        &mut self,
        node: SyntaxReader<'_>,
    ) -> Result<Self::Output, Self::Error>;

    fn visit_while_expression(
        &mut self,
        node: SyntaxReader<'_>,
    ) -> Result<Self::Output, Self::Error>;

    fn visit_function_expression(
        &mut self,
        node: SyntaxReader<'_>,
    ) -> Result<Self::Output, Self::Error>;

    fn visit_call_expression(
        &mut self,
        node: SyntaxReader<'_>,
    ) -> Result<Self::Output, Self::Error>;
}
