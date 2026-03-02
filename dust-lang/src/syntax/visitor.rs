use crate::{
    compiler::error::CompileError,
    dust_error::{ErrorKind, InternalError},
    syntax::{SyntaxKind, SyntaxReader},
};

pub trait SyntaxVisitor {
    type RootOutput;

    type StatementOutput;

    type ExpressionInput;

    type ExpressionOutput;

    type TypeOutput;

    type PathOutput;

    fn visit_item(&mut self, node: SyntaxReader) -> Result<(), ErrorKind> {
        match node.kind() {
            SyntaxKind::ModuleItem | SyntaxKind::PublicModuleItem => self.visit_module_item(node),
            SyntaxKind::FunctionItem | SyntaxKind::PublicFunctionItem => {
                self.visit_function_item(node)
            }
            SyntaxKind::UseItem | SyntaxKind::PublicUseItem => self.visit_use_item(node),
            SyntaxKind::StructItem | SyntaxKind::PublicStructItem => self.visit_struct_item(node),
            SyntaxKind::EnumItem | SyntaxKind::PublicEnumItem => self.visit_enum_item(node),
            _ => Err(ErrorKind::Compile(CompileError::Unimplemented {
                syntax_kind: node.kind(),
                position: node.position(),
            })),
        }
    }

    fn visit_statement(&mut self, node: SyntaxReader) -> Result<Self::StatementOutput, ErrorKind> {
        match node.kind() {
            SyntaxKind::ExpressionStatement => self.visit_expression_statement(node),
            SyntaxKind::ReassignmentStatement => self.visit_reassignment_statement(node),
            SyntaxKind::LetStatement | SyntaxKind::LetMutStatement => {
                self.visit_let_statement(node)
            }
            SyntaxKind::AdditionAssignmentStatement
            | SyntaxKind::SubtractionAssignmentStatement
            | SyntaxKind::MultiplicationAssignmentStatement
            | SyntaxKind::DivisionAssignmentStatement
            | SyntaxKind::ModuloAssignmentStatement
            | SyntaxKind::ExponentAssignmentStatement => {
                self.visit_binary_assignment_statement(node)
            }
            _ => Err(ErrorKind::Compile(CompileError::Unimplemented {
                syntax_kind: node.kind(),
                position: node.position(),
            })),
        }
    }

    fn visit_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind> {
        match node.kind() {
            SyntaxKind::PathExpression => self.visit_path_expression(node, input),
            SyntaxKind::BooleanExpression => self.visit_boolean_expression(node, input),
            SyntaxKind::ByteExpression => self.visit_byte_expression(node, input),
            SyntaxKind::CharacterExpression => self.visit_character_expression(node, input),
            SyntaxKind::FloatExpression => self.visit_float_expression(node, input),
            SyntaxKind::IntegerExpression => self.visit_integer_expression(node, input),
            SyntaxKind::StringExpression => self.visit_string_expression(node, input),
            SyntaxKind::ListExpression => self.visit_list_expression(node, input),
            SyntaxKind::ListIndexExpression => self.visit_index_expression(node, input),
            SyntaxKind::StructExpression => self.visit_struct_expression(node, input),
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
            SyntaxKind::GroupedExpression => self.visit_expression(node.child()?, input),
            _ => Err(ErrorKind::Compile(CompileError::Unimplemented {
                syntax_kind: node.kind(),
                position: node.position(),
            })),
        }
    }

    fn visit_root(&mut self, node: SyntaxReader) -> Result<Self::RootOutput, ErrorKind>;

    fn visit_module_item(&mut self, node: SyntaxReader) -> Result<(), ErrorKind>;

    fn visit_function_item(&mut self, node: SyntaxReader) -> Result<(), ErrorKind>;

    fn visit_use_item(&mut self, node: SyntaxReader) -> Result<(), ErrorKind>;

    fn visit_struct_item(&mut self, node: SyntaxReader) -> Result<(), ErrorKind>;

    fn visit_enum_item(&mut self, node: SyntaxReader) -> Result<(), ErrorKind>;

    fn visit_expression_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, ErrorKind>;

    fn visit_reassignment_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, ErrorKind>;

    fn visit_let_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, ErrorKind>;

    fn visit_binary_assignment_statement(
        &mut self,
        node: SyntaxReader,
    ) -> Result<Self::StatementOutput, ErrorKind>;

    fn visit_boolean_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_byte_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_character_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_float_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_integer_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_string_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_list_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_index_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_path_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_struct_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_block_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_if_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_else_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_math_binary_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_comparison_binary_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_logical_binary_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_unary_negation_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_while_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_function_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_call_expression(
        &mut self,
        node: SyntaxReader,
        input: Self::ExpressionInput,
    ) -> Result<Self::ExpressionOutput, ErrorKind>;

    fn visit_type(&mut self, node: SyntaxReader) -> Result<Self::TypeOutput, ErrorKind>;

    fn visit_path(
        &mut self,
        node: SyntaxReader,
        local: bool,
    ) -> Result<Self::PathOutput, ErrorKind>;
}
