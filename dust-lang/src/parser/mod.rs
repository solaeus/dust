mod error;
mod parse_rule;

#[cfg(test)]
mod tests;

pub use error::ParseError;

use std::mem::replace;

use lexical_core::{
    ParseFloatOptions, ParseIntegerOptions, format::RUST_LITERAL, parse_with_options,
};
use smallvec::{SmallVec, smallvec};
use tracing::{debug, error};

use crate::{
    dust_error::DustError,
    lexer::Lexer,
    parser::parse_rule::{Associativity, ParseRule, Precedence},
    source::{Position, Source, SourceFile, SourceFileId, Span},
    syntax::{SyntaxId, SyntaxKind, SyntaxNode, SyntaxPayload, SyntaxPayloadKind, SyntaxTree},
    token::{Token, TokenKind},
};

pub fn parse<'src>(source_code: &'src str) -> (SyntaxTree, Option<DustError<'src>>) {
    let mut source = Source::new();
    let file = SourceFile::validated("parse", source_code);
    let file_id = source.add_file(file);
    let file_str = source.get_file(file_id).content_as_str();

    let lexer = Lexer::from_utf8(file_str);
    let parser = Parser::new(file_id, lexer);
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();
    let dust_error = if errors.is_empty() {
        None
    } else {
        Some(DustError::parse(errors, source))
    };

    (syntax_tree, dust_error)
}

pub struct Parser<'src> {
    lexer: Lexer<'src>,

    syntax_tree: SyntaxTree,

    current_token: Token,
    previous_token: Token,

    file_module_names: Vec<Span>,
    errors: Vec<ParseError>,
}

impl<'src> Parser<'src> {
    pub fn new(file_id: SourceFileId, lexer: Lexer<'src>) -> Self {
        Self {
            lexer,
            syntax_tree: SyntaxTree::new(file_id),
            current_token: Token::default(),
            previous_token: Token::default(),
            file_module_names: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn parse(mut self) -> ParseResult {
        self.advance();

        let _root_id = self
            .syntax_tree
            .push(SyntaxNode::empty(SyntaxKind::Root, Span::default()));

        debug_assert_eq!(_root_id, SyntaxId::ROOT);

        match self.parse_root() {
            Ok(root_node) => {
                self.syntax_tree.replace_node(SyntaxId::ROOT, root_node);
            }
            Err(error) => self.errors.push(error),
        }

        ParseResult {
            syntax_tree: self.syntax_tree,
            errors: self.errors,
            file_module_names: self.file_module_names,
        }
    }

    pub fn source(&self) -> &[u8] {
        self.lexer.source()
    }

    fn create_node_with_children(
        &mut self,
        kind: SyntaxKind,
        span: Span,
        children: SmallVec<[SyntaxId; 4]>,
    ) -> SyntaxNode {
        match children.len() {
            0 => SyntaxNode::empty(kind, span),
            1 => SyntaxNode::with_child(kind, span, children[0]),
            2 => SyntaxNode::with_binary_children(kind, span, children[0], children[1]),
            _ => {
                let payload = self.syntax_tree.add_children(children);

                SyntaxNode {
                    kind,
                    span,
                    payload,
                    payload_kind: SyntaxPayloadKind::MultipleChildren,
                }
            }
        }
    }

    fn current_position(&self) -> Position {
        Position::new(self.syntax_tree.file_id, self.current_token.span)
    }

    fn current_source(&self) -> &[u8] {
        &self.source()[self.current_token.span.as_usize_range()]
    }

    fn new_child_buffer() -> SmallVec<[SyntaxId; 4]> {
        SmallVec::<[SyntaxId; 4]>::new()
    }

    fn pratt(&mut self, minimum_precedence: Precedence) -> Result<SyntaxNode, ParseError> {
        let prefix_rule = ParseRule::from(self.current_token.kind);
        let prefix_parser = prefix_rule.prefix.ok_or(ParseError::UnexpectedToken {
            found: self.current_token.kind,
            position: self.current_position(),
        })?;

        let mut node = prefix_parser(self)?;
        let mut infix_rule = ParseRule::from(self.current_token.kind);

        while let Some(infix_parser) = infix_rule.infix
            && minimum_precedence <= infix_rule.precedence
            && self.previous_token.kind != TokenKind::Semicolon
        {
            node = infix_parser(self, node)?;
            infix_rule = ParseRule::from(self.current_token.kind);
        }

        Ok(node)
    }

    fn advance(&mut self) {
        if let Some(next_token) = self.lexer.next() {
            debug!("Parsing {}", next_token.kind);

            self.previous_token = replace(&mut self.current_token, next_token);
        }

        if let Some(index) = self.lexer.error_index() {
            let position = Position::new(self.syntax_tree.file_id, Span::new(index, index));

            self.recover(ParseError::InvalidUtf8 { position });
        }
    }

    fn recover(&mut self, error: ParseError) {
        debug!(
            "Encountered an error, on {} at {}",
            self.current_token.kind, self.current_token.span
        );

        self.errors.push(error);

        while !matches!(
            self.current_token.kind,
            TokenKind::RightCurlyBrace
                | TokenKind::RightParenthesis
                | TokenKind::Semicolon
                | TokenKind::Eof
                | TokenKind::Unknown
        ) {
            self.advance();
        }

        self.advance();

        debug!(
            "Recovered from an error, now on {} at {}",
            self.current_token.kind, self.current_token.span
        );
    }

    fn is_eof(&self) -> bool {
        self.current_token.kind == TokenKind::Eof
    }

    fn allow(&mut self, allowed: TokenKind) -> Result<bool, ParseError> {
        let allowed = self.current_token.kind == allowed;

        if allowed {
            self.advance();
        }

        Ok(allowed)
    }

    fn expect(&mut self, expected: TokenKind) -> Result<(), ParseError> {
        if self.current_token.kind != expected {
            return Err(ParseError::ExpectedToken {
                expected,
                found: self.current_token.kind,
                position: self.current_position(),
            });
        }

        self.advance();

        Ok(())
    }

    fn parse_item(&mut self) -> Result<SyntaxNode, ParseError> {
        match self.pratt(Precedence::None) {
            Ok(node) if node.kind.is_item() => Ok(node),
            Ok(node) => Err(ParseError::ExpectedItem {
                found: node.kind,
                position: Position::new(self.syntax_tree.file_id, node.span),
            }),
            Err(error) => Err(error),
        }
    }

    fn parse_expression(&mut self) -> Result<SyntaxNode, ParseError> {
        match self.pratt(Precedence::None) {
            Ok(node) if node.kind.is_expression() => Ok(node),
            Ok(node) => Err(ParseError::ExpectedExpression {
                found: Some(node.kind),
                position: Position::new(self.syntax_tree.file_id, node.span),
            }),
            Err(error) => Err(error),
        }
    }

    fn parse_sub_expression(&mut self, precedence: Precedence) -> Result<SyntaxNode, ParseError> {
        match self.pratt(precedence) {
            Ok(node) if node.kind.is_expression() => Ok(node),
            Ok(node) => Err(ParseError::ExpectedExpression {
                found: Some(node.kind),
                position: Position::new(self.syntax_tree.file_id, node.span),
            }),
            Err(error) => Err(error),
        }
    }

    fn parse_unexpected(&mut self) -> Result<SyntaxNode, ParseError> {
        Err(ParseError::UnexpectedToken {
            found: self.current_token.kind,
            position: self.current_position(),
        })
    }

    fn parse_root(&mut self) -> Result<SyntaxNode, ParseError> {
        let mut children = Self::new_child_buffer();

        while !self.is_eof() {
            match self.parse_item() {
                Ok(child) => {
                    let child_id = self.syntax_tree.push(child);

                    children.push(child_id);
                }
                Err(error) => self.recover(error),
            }
        }

        let root_node = self.create_node_with_children(
            SyntaxKind::Root,
            Span::new(0, self.previous_token.span.end()),
            children,
        );

        Ok(root_node)
    }

    fn parse_prefix_pub_keyword(&mut self) -> Result<SyntaxNode, ParseError> {
        self.advance();

        match self.current_token.kind {
            TokenKind::Mod => {
                let mut mod_node = self.parse_prefix_mod_keyword()?;

                mod_node.kind = SyntaxKind::PublicModuleItem;

                Ok(mod_node)
            }
            TokenKind::Use => {
                let mut use_node = self.parse_prefix_use_keyword()?;

                use_node.kind = SyntaxKind::PublicUseItem;

                Ok(use_node)
            }
            TokenKind::Fn => {
                let mut function_node = self.parse_prefix_fn_keyword()?;

                if function_node.kind == SyntaxKind::FunctionExpression {
                    return Err(ParseError::ExpectedItem {
                        found: function_node.kind,
                        position: Position::new(self.syntax_tree.file_id, function_node.span),
                    });
                }

                function_node.kind = SyntaxKind::PublicFunctionItem;

                Ok(function_node)
            }
            TokenKind::Struct => {
                let mut struct_node = self.parse_prefix_struct_keyword()?;

                struct_node.kind = SyntaxKind::PublicStructItem;

                Ok(struct_node)
            }
            TokenKind::Enum => {
                let mut enum_node = self.parse_prefix_enum_keyword()?;

                enum_node.kind = SyntaxKind::PublicEnumItem;

                Ok(enum_node)
            }
            _ => Err(ParseError::ExpectedMultipleTokens {
                expected: &[TokenKind::Use, TokenKind::Mod, TokenKind::Fn],
                found: self.current_token.kind,
                position: self.current_position(),
            }),
        }
    }

    fn parse_prefix_mod_keyword(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.advance();

        let module_name_node = self.parse_simple_path()?;
        let module_name_id = self.syntax_tree.push(module_name_node);

        match self.current_token.kind {
            TokenKind::Semicolon => {
                self.advance();

                self.file_module_names.push(module_name_node.span);

                Ok(SyntaxNode::with_child(
                    SyntaxKind::ModuleItem,
                    Span::new(start, self.previous_token.span.end()),
                    module_name_id,
                ))
            }
            TokenKind::LeftCurlyBrace => {
                self.advance();

                let mut children = Self::new_child_buffer();

                while !self.allow(TokenKind::RightCurlyBrace)? && !self.is_eof() {
                    match self.parse_item() {
                        Ok(child) => {
                            let child_id = self.syntax_tree.push(child);

                            children.push(child_id);
                        }
                        Err(error) => self.recover(error),
                    }
                }

                Ok(self.create_node_with_children(
                    SyntaxKind::ModuleItem,
                    Span::new(start, self.previous_token.span.end()),
                    children,
                ))
            }
            _ => Err(ParseError::ExpectedMultipleTokens {
                expected: &[TokenKind::Semicolon, TokenKind::LeftCurlyBrace],
                found: self.current_token.kind,
                position: self.current_position(),
            }),
        }
    }

    fn parse_prefix_use_keyword(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.advance();

        let path_node = self.parse_path()?;
        let path_id = self.syntax_tree.push(path_node);

        self.expect(TokenKind::Semicolon)?;

        Ok(SyntaxNode::with_child(
            SyntaxKind::UseItem,
            Span::new(start, self.previous_token.span.end()),
            path_id,
        ))
    }

    fn parse_prefix_struct_keyword(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.advance();

        let mut children = Self::new_child_buffer();

        let path_node = self.parse_simple_path()?;
        let path_id = self.syntax_tree.push(path_node);

        children.push(path_id);

        if let Some(type_parameters_node) = self.parse_optional_type_parameters()? {
            let type_parameters_id = self.syntax_tree.push(type_parameters_node);

            children.push(type_parameters_id);
        }

        match self.current_token.kind {
            TokenKind::Semicolon => {
                self.advance();
            }
            TokenKind::LeftParenthesis => {
                let fields_node = self.parse_tuple_fields()?;
                let fields_id = self.syntax_tree.push(fields_node);

                children.push(fields_id);

                self.expect(TokenKind::Semicolon)?;
            }
            TokenKind::LeftCurlyBrace => {
                let fields_node = self.parse_struct_fields()?;
                let fields_id = self.syntax_tree.push(fields_node);

                children.push(fields_id);
            }
            _ => {
                return Err(ParseError::ExpectedMultipleTokens {
                    expected: &[
                        TokenKind::Semicolon,
                        TokenKind::LeftParenthesis,
                        TokenKind::LeftCurlyBrace,
                    ],
                    found: self.current_token.kind,
                    position: self.current_position(),
                });
            }
        };

        Ok(self.create_node_with_children(
            SyntaxKind::StructItem,
            Span::new(start, self.previous_token.span.end()),
            children,
        ))
    }

    fn parse_prefix_enum_keyword(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.advance();

        let mut children = Self::new_child_buffer();

        let path_node = self.parse_simple_path()?;
        let path_id = self.syntax_tree.push(path_node);

        children.push(path_id);

        let type_parameters_id = self
            .parse_optional_type_parameters()?
            .map(|node| self.syntax_tree.push(node));

        self.expect(TokenKind::LeftCurlyBrace)?;

        let mut variant_nodes = Self::new_child_buffer();

        while !self.allow(TokenKind::RightCurlyBrace)? && !self.is_eof() {
            let variant_node = {
                let start = self.current_token.span.start();

                let path_node = self.parse_simple_path()?;
                let path_id = self.syntax_tree.push(path_node);

                let fields_node = match self.current_token.kind {
                    TokenKind::Comma => {
                        self.advance();

                        let variant_node = SyntaxNode::with_child(
                            SyntaxKind::EnumVariant,
                            Span::new(start, self.previous_token.span.end()),
                            path_id,
                        );
                        let variant_id = self.syntax_tree.push(variant_node);

                        variant_nodes.push(variant_id);

                        continue;
                    }
                    TokenKind::LeftParenthesis => self.parse_tuple_fields()?,
                    TokenKind::LeftCurlyBrace => self.parse_struct_fields()?,
                    _ => {
                        return Err(ParseError::ExpectedMultipleTokens {
                            expected: &[
                                TokenKind::Comma,
                                TokenKind::LeftParenthesis,
                                TokenKind::LeftCurlyBrace,
                                TokenKind::Less,
                            ],
                            found: self.current_token.kind,
                            position: self.current_position(),
                        });
                    }
                };
                let fields_id = self.syntax_tree.push(fields_node);

                Ok(SyntaxNode::with_binary_children(
                    SyntaxKind::EnumVariant,
                    Span::new(start, self.previous_token.span.end()),
                    path_id,
                    fields_id,
                ))
            }?;
            let variant_id = self.syntax_tree.push(variant_node);

            variant_nodes.push(variant_id);

            self.allow(TokenKind::Comma)?;
        }

        let variant_list_node = self.create_node_with_children(
            SyntaxKind::EnumVariants,
            Span::new(start, self.previous_token.span.end()),
            variant_nodes,
        );
        let variant_list_id = self.syntax_tree.push(variant_list_node);

        children.push(variant_list_id);

        if let Some(type_parameters_id) = type_parameters_id {
            children.push(type_parameters_id);
        }

        let enum_node = self.create_node_with_children(
            SyntaxKind::EnumItem,
            Span::new(start, self.previous_token.span.end()),
            children,
        );

        Ok(enum_node)
    }

    fn parse_prefix_fn_keyword(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.advance();

        match self.current_token.kind {
            TokenKind::Identifier => {
                let path_node = self.parse_simple_path()?;
                let path_id = self.syntax_tree.push(path_node);

                let function_expression = self.parse_function_expression()?;
                let function_expression = self.syntax_tree.push(function_expression);

                let function_item_node = SyntaxNode::with_binary_children(
                    SyntaxKind::FunctionItem,
                    Span::new(start, self.current_token.span.start()),
                    path_id,
                    function_expression,
                );

                Ok(function_item_node)
            }
            TokenKind::LeftParenthesis => self.parse_function_expression(),
            _ => Err(ParseError::ExpectedMultipleTokens {
                expected: &[TokenKind::Identifier, TokenKind::LeftParenthesis],
                found: self.current_token.kind,
                position: self.current_position(),
            }),
        }
    }

    fn parse_function_expression(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        let function_signature_node = self.parse_function_signature()?;
        let function_signature_id = self.syntax_tree.push(function_signature_node);

        let function_body_node = self.parse_prefix_left_brace()?;
        let function_body_id = self.syntax_tree.push(function_body_node);

        Ok(SyntaxNode::with_binary_children(
            SyntaxKind::FunctionExpression,
            Span::new(start, self.previous_token.span.end()),
            function_signature_id,
            function_body_id,
        ))
    }

    fn parse_function_signature(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        let mut children = Self::new_child_buffer();

        let type_parameters_id = self
            .parse_optional_type_parameters()?
            .map(|node| self.syntax_tree.push(node));
        let value_parameters_node = self.parse_function_value_parameters()?;
        let value_parameters_id = self.syntax_tree.push(value_parameters_node);

        children.push(value_parameters_id);

        if self.allow(TokenKind::ArrowThin)? {
            let return_type_node = self.parse_type()?;
            let return_type_id = self.syntax_tree.push(return_type_node);

            children.push(return_type_id);
        }

        if let Some(type_parameters_id) = type_parameters_id {
            children.push(type_parameters_id);
        }

        Ok(self.create_node_with_children(
            SyntaxKind::FunctionSignature,
            Span::new(start, self.previous_token.span.end()),
            children,
        ))
    }

    fn parse_function_value_parameters(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.expect(TokenKind::LeftParenthesis)?;

        let mut children = Self::new_child_buffer();

        while !self.allow(TokenKind::RightParenthesis)? {
            if self.current_token.kind == TokenKind::Eof {
                break;
            }

            let parameter_path_node = self.parse_path()?;
            let parameter_path_id = self.syntax_tree.push(parameter_path_node);

            self.expect(TokenKind::Colon)?;

            let parameter_type_node_id = self.parse_type()?;
            let parameter_type_id = self.syntax_tree.push(parameter_type_node_id);

            children.push(parameter_path_id);
            children.push(parameter_type_id);

            self.allow(TokenKind::Comma)?;
        }

        let node = self.create_node_with_children(
            SyntaxKind::ValueParameters,
            Span::new(start, self.previous_token.span.end()),
            children,
        );

        Ok(node)
    }

    fn parse_type(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        match self.current_token.kind {
            TokenKind::Any => {
                self.advance();

                Ok(SyntaxNode::empty(
                    SyntaxKind::AnyType,
                    Span::new(start, self.previous_token.span.end()),
                ))
            }
            TokenKind::Bool => {
                self.advance();

                Ok(SyntaxNode::empty(
                    SyntaxKind::BooleanType,
                    Span::new(start, self.previous_token.span.end()),
                ))
            }
            TokenKind::Byte => {
                self.advance();

                Ok(SyntaxNode::empty(
                    SyntaxKind::ByteType,
                    Span::new(start, self.previous_token.span.end()),
                ))
            }
            TokenKind::Char => {
                self.advance();

                Ok(SyntaxNode::empty(
                    SyntaxKind::CharacterType,
                    Span::new(start, self.previous_token.span.end()),
                ))
            }
            TokenKind::Float => {
                self.advance();

                Ok(SyntaxNode::empty(
                    SyntaxKind::FloatType,
                    Span::new(start, self.previous_token.span.end()),
                ))
            }
            TokenKind::Int => {
                self.advance();

                Ok(SyntaxNode::empty(
                    SyntaxKind::IntegerType,
                    Span::new(start, self.previous_token.span.end()),
                ))
            }
            TokenKind::Str => {
                self.advance();

                Ok(SyntaxNode::empty(
                    SyntaxKind::StringType,
                    Span::new(start, self.previous_token.span.end()),
                ))
            }
            TokenKind::Identifier => {
                let mut path_node = self.parse_path()?;

                path_node.kind = SyntaxKind::TypePath;

                Ok(path_node)
            }
            TokenKind::LeftSquareBracket => {
                self.advance();

                let element_type_node = self.parse_type()?;
                let element_type_id = self.syntax_tree.push(element_type_node);

                self.expect(TokenKind::RightSquareBracket)?;

                Ok(SyntaxNode::with_child(
                    SyntaxKind::ListType,
                    Span::new(start, self.previous_token.span.end()),
                    element_type_id,
                ))
            }
            TokenKind::Fn => {
                self.advance();
                self.expect(TokenKind::LeftParenthesis)?;

                let mut children = Self::new_child_buffer();

                while !self.allow(TokenKind::RightParenthesis)? {
                    if self.current_token.kind == TokenKind::Eof {
                        break;
                    }

                    let parameter_type_node = self.parse_type()?;
                    let parameter_type_id = self.syntax_tree.push(parameter_type_node);

                    children.push(parameter_type_id);

                    self.allow(TokenKind::Comma)?;
                }

                let value_parameter_types_node = self.create_node_with_children(
                    SyntaxKind::ValueParameterTypes,
                    Span::new(start, self.previous_token.span.end()),
                    children,
                );
                let value_parameter_types_node_id =
                    self.syntax_tree.push(value_parameter_types_node);

                if self.allow(TokenKind::ArrowThin)? {
                    let return_type_node = self.parse_type()?;
                    let return_type_id = self.syntax_tree.push(return_type_node);

                    Ok(SyntaxNode::with_binary_children(
                        SyntaxKind::FunctionType,
                        Span::new(start, self.previous_token.span.end()),
                        value_parameter_types_node_id,
                        return_type_id,
                    ))
                } else {
                    Ok(SyntaxNode::with_child(
                        SyntaxKind::FunctionType,
                        Span::new(start, self.previous_token.span.end()),
                        value_parameter_types_node_id,
                    ))
                }
            }
            _ => Err(ParseError::ExpectedMultipleTokens {
                expected: &[
                    TokenKind::Bool,
                    TokenKind::Byte,
                    TokenKind::Char,
                    TokenKind::Float,
                    TokenKind::Int,
                    TokenKind::Str,
                    TokenKind::Identifier,
                    TokenKind::Fn,
                    TokenKind::LeftSquareBracket,
                ],
                found: self.current_token.kind,
                position: self.current_position(),
            }),
        }
    }

    fn parse_prefix_let_keyword(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.advance();

        let kind = if self.allow(TokenKind::Mut)? {
            SyntaxKind::LetMutStatement
        } else {
            SyntaxKind::LetStatement
        };

        let path_node = self.parse_path()?;
        let path_id = self.syntax_tree.push(path_node);
        let type_notation_id = if self.allow(TokenKind::Colon)? {
            let type_node = self.parse_type()?;
            let type_id = self.syntax_tree.push(type_node);

            Some(type_id)
        } else {
            None
        };

        self.expect(TokenKind::Equal)?;

        let expression_node = self.parse_expression()?;
        let expression_id = self.syntax_tree.push(expression_node);

        self.expect(TokenKind::Semicolon)?;

        let end = self.previous_token.span.end();

        let let_statement_node = if let Some(type_notation_id) = type_notation_id {
            let payload =
                self.syntax_tree
                    .add_children(smallvec![path_id, expression_id, type_notation_id]);

            SyntaxNode {
                kind,
                span: Span::new(start, end),
                payload,
                payload_kind: SyntaxPayloadKind::MultipleChildren,
            }
        } else {
            SyntaxNode::with_binary_children(kind, Span::new(start, end), path_id, expression_id)
        };

        Ok(let_statement_node)
    }

    fn parse_infix_assignment_operator(
        &mut self,
        left: SyntaxNode,
    ) -> Result<SyntaxNode, ParseError> {
        let (start, path_id) = if left.kind == SyntaxKind::PathExpression {
            let start = left.span.start();
            let path_id = left.payload.left_id();

            (start, path_id)
        } else {
            return Err(ParseError::ExpectedSyntax {
                found: left.kind,
                expected: SyntaxKind::Path,
                position: Position::new(self.syntax_tree.file_id, left.span),
            });
        };

        self.expect(TokenKind::Equal)?;

        let expression_node = self.parse_expression()?;
        let expression_id = self.syntax_tree.push(expression_node);

        self.expect(TokenKind::Semicolon)?;

        Ok(SyntaxNode::with_binary_children(
            SyntaxKind::ReassignmentStatement,
            Span::new(start, self.previous_token.span.end()),
            path_id,
            expression_id,
        ))
    }

    fn parse_prefix_boolean(&mut self) -> Result<SyntaxNode, ParseError> {
        let boolean = match self.current_token.kind {
            TokenKind::TrueValue => true,
            TokenKind::FalseValue => false,
            _ => {
                return Err(ParseError::ExpectedMultipleTokens {
                    expected: &[TokenKind::TrueValue, TokenKind::FalseValue],
                    found: self.current_token.kind,
                    position: self.current_position(),
                });
            }
        };

        self.advance();

        let payload = SyntaxPayload::encode_boolean(boolean);

        Ok(SyntaxNode {
            kind: SyntaxKind::BooleanExpression,
            span: self.previous_token.span,
            payload,
            payload_kind: SyntaxPayloadKind::Value,
        })
    }

    fn parse_prefix_byte(&mut self) -> Result<SyntaxNode, ParseError> {
        let byte_str = &self.current_source()[2..]; // Skip the "0x" prefix
        let byte = u8::from_ascii_radix(byte_str, 16).unwrap_or_default();
        let payload = SyntaxPayload::encode_byte(byte);

        self.advance();

        Ok(SyntaxNode {
            kind: SyntaxKind::ByteExpression,
            span: self.previous_token.span,
            payload,
            payload_kind: SyntaxPayloadKind::Value,
        })
    }

    fn parse_prefix_character(&mut self) -> Result<SyntaxNode, ParseError> {
        let character_bytes = &self.current_source()[1..self.current_source().len() - 1];

        debug_assert!(character_bytes.len() <= 4);

        let character = unsafe { str::from_utf8_unchecked(character_bytes) }
            .chars()
            .next()
            .unwrap_or_else(|| {
                error!(
                    "Invalid character literal at {}: {:#?} is not a valid Unicode scalar value",
                    self.current_token.span, character_bytes
                );

                char::default()
            });
        let payload = SyntaxPayload::encode_character(character);

        self.advance();

        Ok(SyntaxNode {
            kind: SyntaxKind::CharacterExpression,
            span: self.previous_token.span,
            payload,
            payload_kind: SyntaxPayloadKind::Value,
        })
    }

    fn parse_prefix_float(&mut self) -> Result<SyntaxNode, ParseError> {
        let float_text = self.current_source();
        let float =
            parse_with_options::<f64, RUST_LITERAL>(float_text, &ParseFloatOptions::default())
                .unwrap_or_default();
        let payload = SyntaxPayload::encode_float(float);

        self.advance();

        Ok(SyntaxNode {
            kind: SyntaxKind::FloatExpression,
            span: self.previous_token.span,
            payload,
            payload_kind: SyntaxPayloadKind::Value,
        })
    }

    fn parse_prefix_integer(&mut self) -> Result<SyntaxNode, ParseError> {
        let integer_text = self.current_source();
        let integer =
            parse_with_options::<i64, RUST_LITERAL>(integer_text, &ParseIntegerOptions::default())
                .unwrap_or_default();
        let payload = SyntaxPayload::encode_integer(integer);

        self.advance();

        Ok(SyntaxNode {
            kind: SyntaxKind::IntegerExpression,
            span: self.previous_token.span,
            payload,
            payload_kind: SyntaxPayloadKind::Value,
        })
    }

    fn parse_prefix_string(&mut self) -> Result<SyntaxNode, ParseError> {
        let span_without_quotes = self.current_token.span.shrink(1);
        let string_source = &self.source()[span_without_quotes.as_usize_range()];
        let payload = SyntaxPayload::encode_string(string_source);

        self.advance();

        Ok(SyntaxNode {
            kind: SyntaxKind::StringExpression,
            span: self.previous_token.span,
            payload,
            payload_kind: SyntaxPayloadKind::Value,
        })
    }

    fn parse_prefix_unary_operator(&mut self) -> Result<SyntaxNode, ParseError> {
        let operator = self.current_token.kind;
        let node_kind = match operator {
            TokenKind::Minus => SyntaxKind::NegationExpression,
            TokenKind::Bang => SyntaxKind::NotExpression,
            _ => {
                return Err(ParseError::ExpectedMultipleTokens {
                    expected: &[TokenKind::Minus, TokenKind::Bang],
                    found: operator,
                    position: self.current_position(),
                });
            }
        };
        let operator_precedence = ParseRule::from(operator).precedence;
        let start = self.current_token.span.start();

        self.advance();

        let expression_node = self.parse_sub_expression(operator_precedence)?;
        let expression_id = self.syntax_tree.push(expression_node);

        let end = self.previous_token.span.end();

        Ok(SyntaxNode::with_child(
            node_kind,
            Span::new(start, end),
            expression_id,
        ))
    }

    fn parse_infix_binary_operator(&mut self, left: SyntaxNode) -> Result<SyntaxNode, ParseError> {
        let start = left.span.start();

        let operator = self.current_token.kind;
        let (node_kind, is_statement) = match operator {
            TokenKind::Plus => (SyntaxKind::AdditionExpression, false),
            TokenKind::PlusEqual => (SyntaxKind::AdditionAssignmentStatement, true),
            TokenKind::Minus => (SyntaxKind::SubtractionExpression, false),
            TokenKind::MinusEqual => (SyntaxKind::SubtractionAssignmentStatement, true),
            TokenKind::Asterisk => (SyntaxKind::MultiplicationExpression, false),
            TokenKind::AsteriskEqual => (SyntaxKind::MultiplicationAssignmentStatement, true),
            TokenKind::Slash => (SyntaxKind::DivisionExpression, false),
            TokenKind::SlashEqual => (SyntaxKind::DivisionAssignmentStatement, true),
            TokenKind::Percent => (SyntaxKind::ModuloExpression, false),
            TokenKind::PercentEqual => (SyntaxKind::ModuloAssignmentStatement, true),
            TokenKind::Caret => (SyntaxKind::ExponentExpression, false),
            TokenKind::CaretEqual => (SyntaxKind::ExponentAssignmentStatement, true),
            TokenKind::DoubleEqual => (SyntaxKind::EqualExpression, false),
            TokenKind::BangEqual => (SyntaxKind::NotEqualExpression, false),
            TokenKind::Greater => (SyntaxKind::GreaterThanExpression, false),
            TokenKind::GreaterEqual => (SyntaxKind::GreaterThanOrEqualExpression, false),
            TokenKind::Less => (SyntaxKind::LessThanExpression, false),
            TokenKind::LessEqual => (SyntaxKind::LessThanOrEqualExpression, false),
            TokenKind::DoubleAmpersand => (SyntaxKind::AndExpression, false),
            TokenKind::DoublePipe => (SyntaxKind::OrExpression, false),
            _ => {
                return Err(ParseError::ExpectedMultipleTokens {
                    expected: &[
                        TokenKind::Plus,
                        TokenKind::PlusEqual,
                        TokenKind::Minus,
                        TokenKind::MinusEqual,
                        TokenKind::Asterisk,
                        TokenKind::AsteriskEqual,
                        TokenKind::Slash,
                        TokenKind::SlashEqual,
                        TokenKind::Percent,
                        TokenKind::PercentEqual,
                        TokenKind::Caret,
                        TokenKind::Greater,
                        TokenKind::GreaterEqual,
                        TokenKind::Less,
                        TokenKind::LessEqual,
                        TokenKind::DoubleEqual,
                        TokenKind::BangEqual,
                        TokenKind::DoubleAmpersand,
                        TokenKind::DoublePipe,
                    ],
                    found: operator,
                    position: self.current_position(),
                });
            }
        };

        let left_id = if is_statement {
            if left.kind == SyntaxKind::PathExpression {
                left.payload.left_id()
            } else {
                return Err(ParseError::ExpectedSyntax {
                    found: left.kind,
                    expected: SyntaxKind::Path,
                    position: Position::new(self.syntax_tree.file_id, left.span),
                });
            }
        } else {
            self.syntax_tree.push(left)
        };

        let parse_rule = ParseRule::from(operator);
        let operator_precedence = parse_rule.precedence;
        let right_precedence = match parse_rule.associativity {
            Associativity::Left => operator_precedence.increment(),
            Associativity::Right => operator_precedence,
        };

        self.advance();

        let right = self.parse_sub_expression(right_precedence)?;
        let right_id = self.syntax_tree.push(right);

        if is_statement {
            self.expect(TokenKind::Semicolon)?;
        }

        Ok(SyntaxNode::with_binary_children(
            node_kind,
            Span::new(start, self.previous_token.span.end()),
            left_id,
            right_id,
        ))
    }

    fn parse_infix_as_keyword(&mut self, left: SyntaxNode) -> Result<SyntaxNode, ParseError> {
        let start = left.span.start();
        let left_id = self.syntax_tree.push(left);

        self.advance();

        let type_node = self.parse_type()?;
        let type_id = self.syntax_tree.push(type_node);
        let end = self.previous_token.span.end();

        Ok(SyntaxNode::with_binary_children(
            SyntaxKind::AsExpression,
            Span::new(start, end),
            left_id,
            type_id,
        ))
    }

    fn parse_prefix_left_parenthesis(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.advance();

        let expression_node = self.parse_expression()?;
        let expression_id = self.syntax_tree.push(expression_node);

        self.expect(TokenKind::RightParenthesis)?;

        let end = self.previous_token.span.end();

        Ok(SyntaxNode::with_child(
            SyntaxKind::GroupedExpression,
            Span::new(start, end),
            expression_id,
        ))
    }

    fn parse_prefix_left_brace(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.advance();

        let mut children = Self::new_child_buffer();
        let mut is_expression_statement = false;

        while !self.allow(TokenKind::RightCurlyBrace)? && !self.is_eof() {
            let is_last_child = self.current_token.kind == TokenKind::RightCurlyBrace;

            match self.pratt(Precedence::None) {
                Ok(node) => {
                    if is_last_child && !node.kind.is_expression() {
                        is_expression_statement = true;
                    }

                    let child_id = self.syntax_tree.push(node);

                    children.push(child_id);
                }
                Err(error) => self.recover(error),
            }
        }

        is_expression_statement = self.allow(TokenKind::Semicolon)? || is_expression_statement;
        let block_expressio_node = self.create_node_with_children(
            SyntaxKind::BlockExpression,
            Span::new(start, self.previous_token.span.end()),
            children,
        );

        if is_expression_statement {
            let block_expression_id = self.syntax_tree.push(block_expressio_node);

            Ok(SyntaxNode::with_child(
                SyntaxKind::ExpressionStatement,
                Span::new(start, self.previous_token.span.end()),
                block_expression_id,
            ))
        } else {
            Ok(block_expressio_node)
        }
    }

    fn parse_prefix_if_keyword(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.advance();

        let condition_node = self.parse_expression()?;
        let condition_id = self.syntax_tree.push(condition_node);

        let then_node = self.parse_prefix_left_brace()?;
        let then_id = self.syntax_tree.push(then_node);

        let mut children = Self::new_child_buffer();

        children.push(condition_id);
        children.push(then_id);

        if self.current_token.kind == TokenKind::Else {
            let else_node = self.parse_else_expression()?;
            let else_id = self.syntax_tree.push(else_node);

            children.push(else_id);
        }

        let end = self.previous_token.span.end();

        if then_node.kind.is_expression() {
            Ok(self.create_node_with_children(
                SyntaxKind::IfExpression,
                Span::new(start, end),
                children,
            ))
        } else {
            let if_node = self.create_node_with_children(
                SyntaxKind::IfExpression,
                Span::new(start, end),
                children,
            );
            let if_node_id = self.syntax_tree.push(if_node);

            Ok(SyntaxNode::with_child(
                SyntaxKind::ExpressionStatement,
                Span::new(start, end),
                if_node_id,
            ))
        }
    }

    fn parse_else_expression(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.advance();

        let body_node = if self.current_token.kind == TokenKind::If {
            self.parse_prefix_if_keyword()?
        } else {
            self.parse_prefix_left_brace()?
        };
        let body_id = self.syntax_tree.push(body_node);

        Ok(SyntaxNode::with_child(
            SyntaxKind::ElseExpression,
            Span::new(start, self.previous_token.span.end()),
            body_id,
        ))
    }

    fn parse_prefix_while_keyword(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.advance();

        let condition_node = self.parse_expression()?;
        let condition_id = self.syntax_tree.push(condition_node);

        let body_node = self.parse_prefix_left_brace()?;
        let body_id = self.syntax_tree.push(body_node);

        let end = self.previous_token.span.end();
        let while_node = SyntaxNode::with_binary_children(
            SyntaxKind::WhileExpression,
            Span::new(start, end),
            condition_id,
            body_id,
        );
        let while_node_id = self.syntax_tree.push(while_node);

        Ok(SyntaxNode::with_child(
            SyntaxKind::ExpressionStatement,
            while_node.span,
            while_node_id,
        ))
    }

    fn parse_prefix_break_keyword(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.advance();
        self.allow(TokenKind::Semicolon)?;

        let end = self.previous_token.span.end();

        Ok(SyntaxNode::empty(
            SyntaxKind::BreakExpression,
            Span::new(start, end),
        ))
    }

    fn parse_prefix_return_keyord(&mut self) -> Result<SyntaxNode, ParseError> {
        todo!()
    }

    fn parse_prefix_identifier(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();
        let may_be_struct = !matches!(self.previous_token.kind, TokenKind::If | TokenKind::While);

        let path_node = self.parse_path()?;
        let path_id = self.syntax_tree.push(path_node);

        if may_be_struct && self.allow(TokenKind::LeftCurlyBrace)? {
            let mut children = Self::new_child_buffer();

            children.push(path_id);

            while !self.allow(TokenKind::RightCurlyBrace)? && !self.is_eof() {
                let field_path_node = self.parse_path()?;
                let field_path_id = self.syntax_tree.push(field_path_node);

                self.expect(TokenKind::Colon)?;

                let field_expression_node = self.parse_expression()?;
                let field_expression_id = self.syntax_tree.push(field_expression_node);

                self.allow(TokenKind::Comma)?;

                children.push(field_path_id);
                children.push(field_expression_id);
            }

            Ok(self.create_node_with_children(
                SyntaxKind::StructExpression,
                Span::new(start, self.previous_token.span.end()),
                children,
            ))
        } else {
            Ok(SyntaxNode {
                kind: SyntaxKind::PathExpression,
                span: path_node.span,
                payload: path_node.payload,
                payload_kind: path_node.payload_kind,
            })
        }
    }

    fn parse_prefix_left_bracket(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.advance();

        let mut children = Self::new_child_buffer();

        while !self.allow(TokenKind::RightSquareBracket)? {
            if self.current_token.kind == TokenKind::Eof {
                break;
            }

            let child_node = self.parse_expression()?;
            let child_id = self.syntax_tree.push(child_node);

            children.push(child_id);
            self.allow(TokenKind::Comma)?;
        }

        let end = self.previous_token.span.end();

        Ok(self.create_node_with_children(
            SyntaxKind::ListExpression,
            Span::new(start, end),
            children,
        ))
    }

    fn parse_infix_left_bracket(&mut self, left: SyntaxNode) -> Result<SyntaxNode, ParseError> {
        let start = left.span.start();
        let left_id = self.syntax_tree.push(left);

        self.advance();

        let index_node = self.parse_expression()?;
        let index_id = self.syntax_tree.push(index_node);

        self.expect(TokenKind::RightSquareBracket)?;

        let end = self.previous_token.span.end();

        Ok(SyntaxNode::with_binary_children(
            SyntaxKind::ListIndexExpression,
            Span::new(start, end),
            left_id,
            index_id,
        ))
    }

    fn parse_infix_left_parenthesis(&mut self, left: SyntaxNode) -> Result<SyntaxNode, ParseError> {
        let start = left.span.start();
        let left_id = self.syntax_tree.push(left);

        self.advance();

        let mut value_arguments = Self::new_child_buffer();

        while !self.allow(TokenKind::RightParenthesis)? {
            if self.current_token.kind == TokenKind::Eof {
                break;
            }

            let argument_node = self.parse_expression()?;
            let argument_id = self.syntax_tree.push(argument_node);

            value_arguments.push(argument_id);

            self.allow(TokenKind::Comma)?;
        }

        let end = self.previous_token.span.end();
        let call_value_arguments_node = self.create_node_with_children(
            SyntaxKind::ValueArguments,
            Span::new(left.span.start(), self.previous_token.span.end()),
            value_arguments,
        );
        let call_value_arguments_id = self.syntax_tree.push(call_value_arguments_node);

        Ok(SyntaxNode::with_binary_children(
            SyntaxKind::CallExpression,
            Span::new(start, end),
            left_id,
            call_value_arguments_id,
        ))
    }

    fn parse_path(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.expect(TokenKind::Identifier)?;

        let first_segment_node =
            SyntaxNode::empty(SyntaxKind::PathSegment, self.previous_token.span);
        let first_segment_id = self.syntax_tree.push(first_segment_node);

        let mut children = Self::new_child_buffer();

        children.push(first_segment_id);

        while self.allow(TokenKind::DoubleColon)? {
            let identifier_span = self.current_token.span;

            self.expect(TokenKind::Identifier)?;

            let segment_node = SyntaxNode::empty(SyntaxKind::PathSegment, identifier_span);
            let segment_id = self.syntax_tree.push(segment_node);

            children.push(segment_id);
        }

        let end = self.previous_token.span.end();

        Ok(self.create_node_with_children(SyntaxKind::Path, Span::new(start, end), children))
    }

    fn parse_simple_path(&mut self) -> Result<SyntaxNode, ParseError> {
        let identifier_span = self.current_token.span;

        self.expect(TokenKind::Identifier)?;

        Ok(SyntaxNode::empty(SyntaxKind::SimplePath, identifier_span))
    }

    fn parse_optional_type_parameters(&mut self) -> Result<Option<SyntaxNode>, ParseError> {
        if !self.allow(TokenKind::Less)? {
            return Ok(None);
        }

        let start = self.previous_token.span.start();

        let mut type_parameter_nodes = Self::new_child_buffer();

        while !self.allow(TokenKind::Greater)? && !self.is_eof() {
            if !type_parameter_nodes.is_empty() {
                self.expect(TokenKind::Comma)?;
            }

            let type_parameter_path_node = self.parse_simple_path()?;
            let type_parameter_path_id = self.syntax_tree.push(type_parameter_path_node);

            type_parameter_nodes.push(type_parameter_path_id);
        }

        Ok(Some(self.create_node_with_children(
            SyntaxKind::TypeParameters,
            Span::new(start, self.previous_token.span.end()),
            type_parameter_nodes,
        )))
    }

    fn parse_tuple_fields(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.advance();

        let mut field_type_nodes = Self::new_child_buffer();

        while !self.allow(TokenKind::RightParenthesis)? && !self.is_eof() {
            let field_type_node = self.parse_type()?;
            let field_type_id = self.syntax_tree.push(field_type_node);

            field_type_nodes.push(field_type_id);

            self.allow(TokenKind::Comma)?;
        }

        Ok(self.create_node_with_children(
            SyntaxKind::TupleFields,
            Span::new(start, self.previous_token.span.end()),
            field_type_nodes,
        ))
    }

    fn parse_struct_fields(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.advance();

        let mut fields = Self::new_child_buffer();

        while !self.allow(TokenKind::RightCurlyBrace)? && !self.is_eof() {
            let field_path_node = self.parse_simple_path()?;
            let field_path_id = self.syntax_tree.push(field_path_node);

            self.expect(TokenKind::Colon)?;

            let field_type_node = self.parse_type()?;
            let field_type_id = self.syntax_tree.push(field_type_node);

            self.allow(TokenKind::Comma)?;

            let field_node = SyntaxNode::with_binary_children(
                SyntaxKind::StructField,
                Span::new(start, self.previous_token.span.end()),
                field_path_id,
                field_type_id,
            );
            let field_id = self.syntax_tree.push(field_node);

            fields.push(field_id);
            self.allow(TokenKind::Comma)?;
        }

        Ok(self.create_node_with_children(
            SyntaxKind::StructFields,
            Span::new(start, self.previous_token.span.end()),
            fields,
        ))
    }
}

pub struct ParseResult {
    pub syntax_tree: SyntaxTree,
    pub errors: Vec<ParseError>,
    pub file_module_names: Vec<Span>,
}
