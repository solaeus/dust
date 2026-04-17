pub mod error;
mod parse_rule;

#[cfg(test)]
mod tests;

use std::mem::replace;

use smallvec::SmallVec;
use tracing::debug;

use crate::{
    lexer::Lexer,
    parser::{
        error::ParseError,
        parse_rule::{Associativity, ParseRule, Precedence},
    },
    source::{FileId, Position, Span},
    syntax::{
        SyntaxId,
        node::{SyntaxChildren, SyntaxChildrenKind, SyntaxFlags, SyntaxKind, SyntaxNode},
        tree::SyntaxTree,
    },
    token::{Token, TokenKind},
};

pub fn parse(source_code: &str) -> (SyntaxTree, Vec<ParseError>) {
    let lexer = Lexer::with_validated_source(source_code);
    let parser = Parser::new(FileId::MAIN, lexer);
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    (syntax_tree, errors)
}

pub struct Parser<'src> {
    lexer: Lexer<'src>,

    tree: SyntaxTree,

    current_token: Token,
    previous_token: Token,

    file_module_names: Vec<Span>,
    errors: Vec<ParseError>,
}

impl<'src> Parser<'src> {
    pub fn new(file_id: FileId, lexer: Lexer<'src>) -> Self {
        Self {
            lexer,
            tree: SyntaxTree::new(file_id),
            current_token: Token {
                kind: TokenKind::Unknown,
                span: Span::empty(),
            },
            previous_token: Token {
                kind: TokenKind::Unknown,
                span: Span::empty(),
            },
            file_module_names: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn source(&self) -> &[u8] {
        self.lexer.source()
    }

    pub fn parse(mut self) -> ParseResult {
        self.advance();

        let _root_id = self.tree.add_node(SyntaxKind::Root.empty(Span::empty()));

        debug_assert_eq!(_root_id, SyntaxId::ROOT);

        match self.parse_root() {
            Ok(root_node) => {
                self.tree.replace_node(SyntaxId::ROOT, root_node);
            }
            Err(error) => self.errors.push(error),
        }

        ParseResult {
            syntax_tree: self.tree,
            errors: self.errors,
            file_module_names: self.file_module_names,
        }
    }

    fn create_node(
        &mut self,
        kind: SyntaxKind,
        children: SmallVec<[SyntaxId; 4]>,
        flags: SyntaxFlags,
        span: Span,
    ) -> SyntaxNode {
        match children.len() {
            0 => SyntaxNode {
                kind,
                children: SyntaxChildren::empty(),
                children_kind: SyntaxChildrenKind::None,
                flags,
                span,
            },
            1 => SyntaxNode {
                kind,
                children: SyntaxChildren::new(children[0].inner(), 0),
                children_kind: SyntaxChildrenKind::Single,
                flags,
                span,
            },
            2 => kind.with_binary_children(span, children[0], children[1]),
            _ => {
                let children = self.tree.add_children(children);

                kind.with_children(span, children)
            }
        }
    }

    fn current_position(&self) -> Position {
        Position::new(self.tree.file_id(), self.current_token.span)
    }

    fn new_child_buffer() -> SmallVec<[SyntaxId; 4]> {
        SmallVec::<[SyntaxId; 4]>::new()
    }

    fn pratt(&mut self, minimum_precedence: Precedence) -> Result<SyntaxNode, ParseError> {
        let prefix_rule = ParseRule::from(self.current_token.kind);

        let mut node = (prefix_rule.prefix)(self)?;
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
            let position = Position::new(self.tree.file_id(), Span::new(index, index));

            self.recover(ParseError::InvalidUtf8 { position });
        }
    }

    fn recover(&mut self, error: ParseError) {
        debug!(
            "Encountered an error, on {} at {}",
            self.current_token.kind, self.current_token.span
        );

        self.errors.push(error);
        self.advance();

        loop {
            match self.current_token.kind {
                TokenKind::Semicolon | TokenKind::RightCurlyBrace => {
                    self.advance();

                    break;
                }
                TokenKind::Pub
                | TokenKind::Mod
                | TokenKind::Use
                | TokenKind::Fn
                | TokenKind::Struct
                | TokenKind::Enum
                | TokenKind::Let
                | TokenKind::Eof
                | TokenKind::Unknown => break,
                _ => self.advance(),
            }
        }

        debug!(
            "Recovered from an error, now on {} at {}",
            self.current_token.kind, self.current_token.span
        );
    }

    fn is_eof(&self) -> bool {
        self.current_token.kind == TokenKind::Eof
    }

    fn allow(&mut self, allowed: TokenKind) -> Result<bool, ParseError> {
        if self.is_eof() {
            return Ok(true);
        }

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

    fn check(&self, expected: TokenKind) -> Result<(), ParseError> {
        if self.current_token.kind != expected {
            return Err(ParseError::ExpectedToken {
                expected,
                found: self.current_token.kind,
                position: self.current_position(),
            });
        }

        Ok(())
    }

    fn parse_unexpected(&mut self) -> Result<SyntaxNode, ParseError> {
        Err(ParseError::UnexpectedToken {
            found: self.current_token.kind,
            position: self.current_position(),
        })
    }

    fn parse_item(&mut self) -> Result<SyntaxNode, ParseError> {
        match self.pratt(Precedence::None) {
            Ok(node) if node.kind.is_item() => Ok(node),
            Ok(node) => Err(ParseError::ExpectedItem {
                found: node.kind,
                position: Position::new(self.tree.file_id(), node.span),
            }),
            Err(error) => Err(error),
        }
    }

    fn parse_expression(&mut self) -> Result<SyntaxNode, ParseError> {
        match self.pratt(Precedence::None) {
            Ok(node) if node.kind.is_expression() => Ok(node),
            Ok(node) => Err(ParseError::ExpectedExpression {
                found: Some(node.kind),
                position: Position::new(self.tree.file_id(), node.span),
            }),
            Err(error) => Err(error),
        }
    }

    fn parse_sub_expression(&mut self, precedence: Precedence) -> Result<SyntaxNode, ParseError> {
        match self.pratt(precedence) {
            Ok(node) if node.kind.is_expression() => Ok(node),
            Ok(node) => Err(ParseError::ExpectedExpression {
                found: Some(node.kind),
                position: Position::new(self.tree.file_id(), node.span),
            }),
            Err(error) => Err(error),
        }
    }

    fn parse_root(&mut self) -> Result<SyntaxNode, ParseError> {
        let mut children = Self::new_child_buffer();

        while !self.is_eof() {
            match self.parse_item() {
                Ok(child) => {
                    let child_id = self.tree.add_node(child);

                    children.push(child_id);
                }
                Err(error) => self.recover(error),
            }
        }

        let root_node = self.create_node(
            SyntaxKind::Root,
            children,
            SyntaxFlags::default(),
            Span::new(0, self.previous_token.span.end()),
        );

        Ok(root_node)
    }

    fn parse_prefix_pub_keyword(&mut self) -> Result<SyntaxNode, ParseError> {
        self.advance();

        match self.current_token.kind {
            TokenKind::Mod => {
                let mut mod_node = self.parse_prefix_mod_keyword()?;

                mod_node.flags.set_flag(SyntaxFlags::PUBLIC);

                Ok(mod_node)
            }
            TokenKind::Use => {
                let mut use_node = self.parse_prefix_use_keyword()?;

                use_node.flags.set_flag(SyntaxFlags::PUBLIC);

                Ok(use_node)
            }
            TokenKind::Fn => {
                let mut function_node = self.parse_prefix_fn_keyword()?;

                function_node.flags.set_flag(SyntaxFlags::PUBLIC);

                Ok(function_node)
            }
            TokenKind::Struct => {
                let mut struct_node = self.parse_prefix_struct_keyword()?;

                struct_node.flags.set_flag(SyntaxFlags::PUBLIC);

                Ok(struct_node)
            }
            TokenKind::Enum => {
                let mut enum_node = self.parse_prefix_enum_keyword()?;

                enum_node.flags.set_flag(SyntaxFlags::PUBLIC);

                Ok(enum_node)
            }
            TokenKind::Const => {
                let mut const_node = self.parse_prefix_const_keyword()?;

                const_node.flags.set_flag(SyntaxFlags::PUBLIC);

                Ok(const_node)
            }
            TokenKind::Trait => {
                let mut trait_node = self.parse_prefix_trait_keyword()?;

                trait_node.flags.set_flag(SyntaxFlags::PUBLIC);

                Ok(trait_node)
            }
            TokenKind::Type => {
                let mut type_node = self.parse_prefix_type_keyword()?;

                type_node.flags.set_flag(SyntaxFlags::PUBLIC);

                Ok(type_node)
            }
            TokenKind::Impl => {
                let mut impl_node = self.parse_prefix_impl_keyword()?;

                impl_node.flags.set_flag(SyntaxFlags::PUBLIC);

                Ok(impl_node)
            }
            _ => Err(ParseError::ExpectedMultipleTokens {
                expected: &[
                    TokenKind::Mod,
                    TokenKind::Use,
                    TokenKind::Fn,
                    TokenKind::Struct,
                    TokenKind::Enum,
                    TokenKind::Const,
                    TokenKind::Trait,
                    TokenKind::Type,
                    TokenKind::Impl,
                ],
                found: self.current_token.kind,
                position: self.current_position(),
            }),
        }
    }

    fn parse_prefix_mod_keyword(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.advance();

        let module_name_node = self.expect_simple_path()?;
        let module_name_id = self.tree.add_node(module_name_node);

        match self.current_token.kind {
            TokenKind::Semicolon => {
                self.advance();

                self.file_module_names.push(module_name_node.span);

                Ok(SyntaxKind::ModItem.with_single_child(
                    Span::new(start, self.previous_token.span.end()),
                    module_name_id,
                ))
            }
            TokenKind::LeftCurlyBrace => {
                self.advance();

                let body_start = self.previous_token.span.start();

                let mut children = Self::new_child_buffer();

                while !self.allow(TokenKind::RightCurlyBrace)? {
                    match self.parse_item() {
                        Ok(child) => {
                            let child_id = self.tree.add_node(child);

                            children.push(child_id);
                        }
                        Err(error) => self.recover(error),
                    }
                }

                let span = Span::new(body_start, self.previous_token.span.end());

                let module_body = self.create_node(
                    SyntaxKind::ModuleBody,
                    children,
                    SyntaxFlags::default(),
                    span,
                );
                let module_body_id = self.tree.add_node(module_body);

                Ok(SyntaxKind::ModItem.with_binary_children(
                    Span::new(start, self.previous_token.span.end()),
                    module_name_id,
                    module_body_id,
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

        let path_node = self.expect_path()?;
        let path_id = self.tree.add_node(path_node);

        self.expect(TokenKind::Semicolon)?;

        Ok(SyntaxKind::UseItem
            .with_single_child(Span::new(start, self.previous_token.span.end()), path_id))
    }

    fn parse_prefix_struct_keyword(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.advance();

        let mut struct_children = Self::new_child_buffer();
        let mut struct_flags = SyntaxFlags::default();

        let path_node = self.expect_simple_path()?;
        let path_id = self.tree.add_node(path_node);

        struct_children.push(path_id);

        if let Some(type_parameters_node) = self.allow_type_parameters()? {
            let type_parameters_id = self.tree.add_node(type_parameters_node);

            struct_children.push(type_parameters_id);
            struct_flags.set_flag(SyntaxFlags::TYPE_PARAMETERS);
        }

        if let Some(where_clause_node) = self.allow_where_clause()? {
            let where_clause_id = self.tree.add_node(where_clause_node);

            struct_children.push(where_clause_id);
            struct_flags.set_flag(SyntaxFlags::WHERE_CLAUSE);
        }

        match self.current_token.kind {
            TokenKind::LeftCurlyBrace => {
                let fields_node = self.parse_struct_fields()?;
                let fields_id = self.tree.add_node(fields_node);

                struct_children.push(fields_id);
            }
            TokenKind::LeftParenthesis => {
                let fields_node = self.parse_tuple_fields()?;
                let fields_id = self.tree.add_node(fields_node);

                struct_children.push(fields_id);

                self.expect(TokenKind::Semicolon)?;
            }
            TokenKind::Semicolon => {
                self.advance();
            }
            _ => {
                return Err(ParseError::ExpectedMultipleTokens {
                    expected: &[
                        TokenKind::LeftCurlyBrace,
                        TokenKind::LeftParenthesis,
                        TokenKind::Semicolon,
                    ],
                    found: self.current_token.kind,
                    position: self.current_position(),
                });
            }
        }

        Ok(self.create_node(
            SyntaxKind::StructItem,
            struct_children,
            struct_flags,
            Span::new(start, self.previous_token.span.end()),
        ))
    }

    fn parse_struct_fields(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.advance();

        let mut field_ids = Self::new_child_buffer();

        while !self.allow(TokenKind::RightCurlyBrace)? {
            let is_public = self.allow(TokenKind::Pub)?;

            let mut field_name_node = self.expect_simple_path()?;
            if is_public {
                field_name_node.flags.set_flag(SyntaxFlags::PUBLIC);
            }
            let field_name_id = self.tree.add_node(field_name_node);

            self.expect(TokenKind::Colon)?;

            let field_type_node = self.expect_type()?;
            let field_type_id = self.tree.add_node(field_type_node);

            field_ids.push(field_name_id);
            field_ids.push(field_type_id);

            match self.current_token.kind {
                TokenKind::Comma => self.advance(),
                TokenKind::RightCurlyBrace => {}
                _ => {
                    return Err(ParseError::ExpectedMultipleTokens {
                        expected: &[TokenKind::Comma, TokenKind::RightCurlyBrace],
                        found: self.current_token.kind,
                        position: self.current_position(),
                    });
                }
            }
        }

        Ok(self.create_node(
            SyntaxKind::NamedFields,
            field_ids,
            SyntaxFlags::default(),
            Span::new(start, self.previous_token.span.end()),
        ))
    }

    fn parse_tuple_fields(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.advance();

        let mut field_ids = Self::new_child_buffer();

        while !self.allow(TokenKind::RightParenthesis)? {
            let is_public = self.allow(TokenKind::Pub)?;
            let mut field_type_node = self.expect_type()?;
            if is_public {
                field_type_node.flags.set_flag(SyntaxFlags::PUBLIC);
            }
            let field_type_id = self.tree.add_node(field_type_node);

            field_ids.push(field_type_id);

            match self.current_token.kind {
                TokenKind::Comma => self.advance(),
                TokenKind::RightParenthesis => {}
                _ => {
                    return Err(ParseError::ExpectedMultipleTokens {
                        expected: &[TokenKind::Comma, TokenKind::RightParenthesis],
                        found: self.current_token.kind,
                        position: self.current_position(),
                    });
                }
            }
        }

        Ok(self.create_node(
            SyntaxKind::TupleFields,
            field_ids,
            SyntaxFlags::default(),
            Span::new(start, self.previous_token.span.end()),
        ))
    }

    fn parse_prefix_enum_keyword(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.advance();

        let mut enum_children = Self::new_child_buffer();
        let mut enum_flags = SyntaxFlags::default();

        let name_node = self.expect_simple_path()?;
        let name_id = self.tree.add_node(name_node);

        enum_children.push(name_id);

        if let Some(type_parameters_node) = self.allow_type_parameters()? {
            let type_parameters_id = self.tree.add_node(type_parameters_node);

            enum_children.push(type_parameters_id);
            enum_flags.set_flag(SyntaxFlags::TYPE_PARAMETERS);
        }

        self.expect(TokenKind::LeftCurlyBrace)?;

        let start_child_count = enum_children.len();

        while !self.allow(TokenKind::RightCurlyBrace)? {
            if enum_children.len() > start_child_count {
                self.expect(TokenKind::Comma)?;
            }

            let start = self.current_token.span.start();

            let path_node = self.expect_simple_path()?;

            match self.current_token.kind {
                TokenKind::Comma | TokenKind::RightCurlyBrace => {
                    self.advance();

                    let variant_node = SyntaxKind::EnumUnitVariant.empty(path_node.span);
                    let variant_id = self.tree.add_node(variant_node);

                    enum_children.push(variant_id);
                }
                TokenKind::LeftParenthesis => {
                    let path_id = self.tree.add_node(path_node);

                    let fields_node = self.parse_tuple_fields()?;
                    let fields_id = self.tree.add_node(fields_node);

                    let variant_node = SyntaxKind::EnumTupleFieldsVariant.with_binary_children(
                        Span::new(start, self.previous_token.span.end()),
                        path_id,
                        fields_id,
                    );
                    let variant_id = self.tree.add_node(variant_node);

                    enum_children.push(variant_id);
                }
                TokenKind::LeftCurlyBrace => {
                    let path_id = self.tree.add_node(path_node);

                    let fields_node = self.parse_struct_fields()?;
                    let fields_id = self.tree.add_node(fields_node);

                    let variant_node = SyntaxKind::EnumNamedFieldsVariant.with_binary_children(
                        Span::new(start, self.previous_token.span.end()),
                        path_id,
                        fields_id,
                    );
                    let variant_id = self.tree.add_node(variant_node);

                    enum_children.push(variant_id);
                }
                _ => {
                    return Err(ParseError::ExpectedMultipleTokens {
                        expected: &[
                            TokenKind::Comma,
                            TokenKind::LeftParenthesis,
                            TokenKind::LeftCurlyBrace,
                            TokenKind::RightCurlyBrace,
                        ],
                        found: self.current_token.kind,
                        position: self.current_position(),
                    });
                }
            };
        }

        Ok(self.create_node(
            SyntaxKind::EnumItem,
            enum_children,
            enum_flags,
            Span::new(start, self.previous_token.span.end()),
        ))
    }

    fn parse_prefix_fn_keyword(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.advance();

        let mut function_children = Self::new_child_buffer();
        let mut function_flags = SyntaxFlags::default();

        let name_node = self.expect_simple_path()?;
        let name_id = self.tree.add_node(name_node);

        function_children.push(name_id);

        self.parse_function_signature(&mut function_children, &mut function_flags)?;

        let body_node = self.parse_block_expression()?;
        let body_id = self.tree.add_node(body_node);

        function_children.push(body_id);

        Ok(self.create_node(
            SyntaxKind::FnItem,
            function_children,
            function_flags,
            Span::new(start, self.previous_token.span.end()),
        ))
    }

    fn parse_function_signature(
        &mut self,
        children: &mut SmallVec<[SyntaxId; 4]>,
        flags: &mut SyntaxFlags,
    ) -> Result<(), ParseError> {
        if let Some(type_parameters_node) = self.allow_type_parameters()? {
            let type_parameters_id = self.tree.add_node(type_parameters_node);

            children.push(type_parameters_id);
            flags.set_flag(SyntaxFlags::TYPE_PARAMETERS);
        }

        if let Some(value_parameters_node) = self.parse_value_parameters()? {
            let value_parameters_id = self.tree.add_node(value_parameters_node);

            children.push(value_parameters_id);
            flags.set_flag(SyntaxFlags::VALUE_PARAMETERS);
        }

        if self.allow(TokenKind::ArrowThin)? {
            let return_type_node = self.expect_type()?;
            let return_type_id = self.tree.add_node(return_type_node);

            children.push(return_type_id);
            flags.set_flag(SyntaxFlags::RETURN_TYPE);
        }

        if let Some(where_clause_node) = self.allow_where_clause()? {
            let where_clause_id = self.tree.add_node(where_clause_node);

            children.push(where_clause_id);
            flags.set_flag(SyntaxFlags::WHERE_CLAUSE);
        }

        Ok(())
    }

    fn parse_block_expression(&mut self) -> Result<SyntaxNode, ParseError> {
        self.check(TokenKind::LeftCurlyBrace)?;
        self.parse_prefix_left_brace()
    }

    fn parse_value_parameters(&mut self) -> Result<Option<SyntaxNode>, ParseError> {
        self.expect(TokenKind::LeftParenthesis)?;

        let start = self.previous_token.span.start();

        let mut value_parameters_children = Self::new_child_buffer();
        let mut value_parameters_flags = SyntaxFlags::default();

        if self.allow(TokenKind::SelfValue)? {
            if self.current_token.kind != TokenKind::RightParenthesis {
                self.expect(TokenKind::Comma)?;
            }

            value_parameters_flags.set_flag(SyntaxFlags::SELF_VALUE);
        }

        while !self.allow(TokenKind::RightParenthesis)? {
            let parameter_path_node = self.expect_simple_path()?;
            let parameter_path_id = self.tree.add_node(parameter_path_node);

            self.expect(TokenKind::Colon)?;

            let parameter_type_node_id = self.expect_type()?;
            let parameter_type_id = self.tree.add_node(parameter_type_node_id);

            value_parameters_children.push(parameter_path_id);
            value_parameters_children.push(parameter_type_id);

            match self.current_token.kind {
                TokenKind::Comma => self.advance(),
                TokenKind::RightParenthesis => {}
                _ => {
                    return Err(ParseError::ExpectedMultipleTokens {
                        expected: &[TokenKind::Comma, TokenKind::RightParenthesis],
                        found: self.current_token.kind,
                        position: self.current_position(),
                    });
                }
            }
        }

        if value_parameters_flags.get_flag(SyntaxFlags::SELF_VALUE)
            || !value_parameters_children.is_empty()
        {
            Ok(Some(self.create_node(
                SyntaxKind::ValueParameters,
                value_parameters_children,
                value_parameters_flags,
                Span::new(start, self.previous_token.span.end()),
            )))
        } else {
            Ok(None)
        }
    }

    fn parse_prefix_impl_keyword(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        let mut impl_children = Self::new_child_buffer();
        let mut impl_flags = SyntaxFlags::default();

        self.advance();

        if let Some(type_parameters_node) = self.allow_type_parameters()? {
            let type_parameters_id = self.tree.add_node(type_parameters_node);

            impl_children.push(type_parameters_id);
            impl_flags.set_flag(SyntaxFlags::TYPE_PARAMETERS);
        }

        let first_path_node = self.expect_path()?;
        let first_path_id = self.tree.add_node(first_path_node);

        impl_children.push(first_path_id);

        if let Some(type_arguments_node) = self.allow_type_arguments()? {
            let type_arguments_id = self.tree.add_node(type_arguments_node);

            impl_children.push(type_arguments_id);
            impl_flags.set_flag(SyntaxFlags::TYPE_ARGUMENTS);
        }

        if self.allow(TokenKind::For)? {
            let type_name_node = {
                let mut node = self.expect_path()?;
                node.kind = SyntaxKind::TypePath;

                node
            };
            let type_name_id = self.tree.add_node(type_name_node);

            impl_children.push(type_name_id);
            impl_flags.set_flag(SyntaxFlags::TYPE_NAME);
        }

        if let Some(where_clause_node) = self.allow_where_clause()? {
            let where_clause_id = self.tree.add_node(where_clause_node);

            impl_children.push(where_clause_id);
            impl_flags.set_flag(SyntaxFlags::WHERE_CLAUSE);
        }

        self.expect(TokenKind::LeftCurlyBrace)?;

        let body_start = self.previous_token.span.start();
        let mut body_children = Self::new_child_buffer();

        while !self.allow(TokenKind::RightCurlyBrace)? {
            match self.expect_impl_item() {
                Ok(item) => {
                    let item_id = self.tree.add_node(item);

                    body_children.push(item_id);
                }
                Err(error) => self.recover(error),
            }
        }

        let body_node = self.create_node(
            SyntaxKind::ImplBody,
            body_children,
            SyntaxFlags::default(),
            Span::new(body_start, self.previous_token.span.end()),
        );
        let body_id = self.tree.add_node(body_node);

        impl_children.push(body_id);

        Ok(self.create_node(
            SyntaxKind::ImplItem,
            impl_children,
            impl_flags,
            Span::new(start, self.previous_token.span.end()),
        ))
    }

    fn parse_prefix_trait_keyword(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.advance();

        let mut trait_children = Self::new_child_buffer();
        let mut trait_flags = SyntaxFlags::default();

        let name_node = self.expect_simple_path()?;
        let name_id = self.tree.add_node(name_node);

        trait_children.push(name_id);

        if let Some(type_parameters_node) = self.allow_type_parameters()? {
            let type_parameters_id = self.tree.add_node(type_parameters_node);

            trait_children.push(type_parameters_id);
            trait_flags.set_flag(SyntaxFlags::TYPE_PARAMETERS);
        }

        if self.allow(TokenKind::Colon)? {
            let supertraits_node = self.parse_trait_bounds()?;
            let supertraits_id = self.tree.add_node(supertraits_node);

            trait_children.push(supertraits_id);
            trait_flags.set_flag(SyntaxFlags::SUPERTRAITS);
        }

        if let Some(where_clause_node) = self.allow_where_clause()? {
            let where_clause_id = self.tree.add_node(where_clause_node);

            trait_children.push(where_clause_id);
            trait_flags.set_flag(SyntaxFlags::WHERE_CLAUSE);
        }

        self.expect(TokenKind::LeftCurlyBrace)?;

        let body_start = self.previous_token.span.start();
        let mut body_children = Self::new_child_buffer();

        while !self.allow(TokenKind::RightCurlyBrace)? {
            match self.current_token.kind {
                TokenKind::Fn => {
                    let method_node = self.parse_trait_function_item()?;
                    let method_id = self.tree.add_node(method_node);

                    body_children.push(method_id);
                }
                TokenKind::Const => {
                    let const_node = self.parse_trait_const_item()?;
                    let const_id = self.tree.add_node(const_node);

                    body_children.push(const_id);
                }
                TokenKind::Type => {
                    let type_node = self.parse_trait_type()?;
                    let type_id = self.tree.add_node(type_node);

                    body_children.push(type_id);
                }
                _ => {
                    return Err(ParseError::ExpectedMultipleTokens {
                        expected: &[TokenKind::Fn, TokenKind::Const, TokenKind::Type],
                        found: self.current_token.kind,
                        position: self.current_position(),
                    });
                }
            }
        }

        let body_node = self.create_node(
            SyntaxKind::TraitBody,
            body_children,
            SyntaxFlags::default(),
            Span::new(body_start, self.previous_token.span.end()),
        );
        let body_id = self.tree.add_node(body_node);

        trait_children.push(body_id);

        Ok(self.create_node(
            SyntaxKind::TraitItem,
            trait_children,
            trait_flags,
            Span::new(start, self.previous_token.span.end()),
        ))
    }

    fn parse_prefix_type_keyword(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.advance();

        let mut children = Self::new_child_buffer();

        let name_node = self.expect_simple_path()?;
        let name_id = self.tree.add_node(name_node);

        children.push(name_id);

        let type_parameters_id = self
            .allow_type_parameters()?
            .map(|node| self.tree.add_node(node));

        self.expect(TokenKind::Equal)?;

        let type_node = self.expect_type()?;
        let type_id = self.tree.add_node(type_node);

        children.push(type_id);

        if let Some(type_parameters_id) = type_parameters_id {
            children.push(type_parameters_id);
        }

        if let Some(type_arguments_node) = self.allow_type_arguments()? {
            let type_arguments_id = self.tree.add_node(type_arguments_node);

            children.push(type_arguments_id);
        }

        self.expect(TokenKind::Semicolon)?;

        Ok(self.create_node(
            SyntaxKind::TypeItem,
            children,
            SyntaxFlags::default(),
            Span::new(start, self.previous_token.span.end()),
        ))
    }

    fn parse_prefix_const_keyword(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.advance();

        let name_node = self.expect_simple_path()?;
        let name_id = self.tree.add_node(name_node);

        self.expect(TokenKind::Colon)?;

        let type_node = self.expect_type()?;
        let type_id = self.tree.add_node(type_node);

        self.expect(TokenKind::Equal)?;

        let expression_statement_node = self.pratt(Precedence::None)?;

        if expression_statement_node.kind != SyntaxKind::ExpressionStatement {
            return Err(ParseError::ExpectedToken {
                expected: TokenKind::Semicolon,
                found: self.current_token.kind,
                position: self.current_position(),
            });
        }

        let expression_id = expression_statement_node.children.left_id();

        let children = self.tree.add_children([name_id, type_id, expression_id]);

        Ok(SyntaxKind::ConstItem
            .with_children(Span::new(start, self.previous_token.span.end()), children))
    }

    fn expect_type(&mut self) -> Result<SyntaxNode, ParseError> {
        match self.current_token.kind {
            TokenKind::Bang => {
                self.advance();

                Ok(SyntaxKind::NeverType.empty(self.previous_token.span))
            }
            TokenKind::Bool => {
                self.advance();

                Ok(SyntaxKind::BooleanType.empty(self.previous_token.span))
            }
            TokenKind::Char => {
                self.advance();

                Ok(SyntaxKind::CharacterType.empty(self.previous_token.span))
            }
            TokenKind::SelfType => {
                self.advance();

                Ok(SyntaxKind::SelfType.empty(self.previous_token.span))
            }
            TokenKind::Str => {
                self.advance();

                Ok(SyntaxKind::StringType.empty(self.previous_token.span))
            }
            TokenKind::U8 => {
                self.advance();

                Ok(SyntaxKind::U8Type.empty(self.previous_token.span))
            }
            TokenKind::I8 => {
                self.advance();

                Ok(SyntaxKind::I8Type.empty(self.previous_token.span))
            }
            TokenKind::U16 => {
                self.advance();

                Ok(SyntaxKind::U16Type.empty(self.previous_token.span))
            }
            TokenKind::I16 => {
                self.advance();

                Ok(SyntaxKind::I16Type.empty(self.previous_token.span))
            }
            TokenKind::U32 => {
                self.advance();

                Ok(SyntaxKind::U32Type.empty(self.previous_token.span))
            }
            TokenKind::I32 => {
                self.advance();

                Ok(SyntaxKind::I32Type.empty(self.previous_token.span))
            }
            TokenKind::U64 => {
                self.advance();

                Ok(SyntaxKind::U64Type.empty(self.previous_token.span))
            }
            TokenKind::I64 => {
                self.advance();

                Ok(SyntaxKind::I64Type.empty(self.previous_token.span))
            }
            TokenKind::U128 => {
                self.advance();

                Ok(SyntaxKind::U128Type.empty(self.previous_token.span))
            }
            TokenKind::I128 => {
                self.advance();

                Ok(SyntaxKind::I128Type.empty(self.previous_token.span))
            }
            TokenKind::USize => {
                self.advance();

                Ok(SyntaxKind::USizeType.empty(self.previous_token.span))
            }
            TokenKind::ISize => {
                self.advance();

                Ok(SyntaxKind::ISizeType.empty(self.previous_token.span))
            }
            TokenKind::F32 => {
                self.advance();

                Ok(SyntaxKind::F32Type.empty(self.previous_token.span))
            }
            TokenKind::F64 => {
                self.advance();

                Ok(SyntaxKind::F64Type.empty(self.previous_token.span))
            }
            TokenKind::Identifier => {
                let start = self.current_token.span.start();

                self.expect(TokenKind::Identifier)?;

                let mut children = Self::new_child_buffer();
                let mut last_segment_span = self.previous_token.span;

                loop {
                    if !self.allow(TokenKind::DoubleColon)? {
                        if let Some(type_arguments) = self.allow_type_arguments()? {
                            let type_arguments_id = self.tree.add_node(type_arguments);
                            let segment = SyntaxKind::PathSegment
                                .with_single_child(last_segment_span, type_arguments_id);

                            children.push(self.tree.add_node(segment));

                            break;
                        }

                        let segment = SyntaxKind::PathSegment.empty(last_segment_span);

                        children.push(self.tree.add_node(segment));

                        break;
                    }

                    if !self.allow(TokenKind::Less)? {
                        let segment = SyntaxKind::PathSegment.empty(last_segment_span);

                        children.push(self.tree.add_node(segment));
                        self.expect(TokenKind::Identifier)?;

                        last_segment_span = self.previous_token.span;

                        continue;
                    }

                    let Some(type_arguments) = self.allow_type_arguments()? else {
                        let segment = SyntaxKind::PathSegment.empty(last_segment_span);

                        children.push(self.tree.add_node(segment));

                        break;
                    };

                    let type_arguments_id = self.tree.add_node(type_arguments);
                    let segment = SyntaxKind::PathSegment
                        .with_single_child(last_segment_span, type_arguments_id);

                    children.push(self.tree.add_node(segment));

                    if !self.allow(TokenKind::DoubleColon)? {
                        break;
                    }

                    self.expect(TokenKind::Identifier)?;

                    last_segment_span = self.previous_token.span;

                    continue;
                }

                let end = self.previous_token.span.end();

                Ok(self.create_node(
                    SyntaxKind::TypePath,
                    children,
                    SyntaxFlags::default(),
                    Span::new(start, end),
                ))
            }
            TokenKind::LeftParenthesis => {
                let start = self.current_token.span.start();

                self.advance();

                let mut children = Self::new_child_buffer();

                while !self.allow(TokenKind::RightParenthesis)? {
                    let type_node = self.expect_type()?;
                    let type_id = self.tree.add_node(type_node);

                    children.push(type_id);

                    self.allow(TokenKind::Comma)?;
                }

                Ok(self.create_node(
                    SyntaxKind::TupleType,
                    children,
                    SyntaxFlags::default(),
                    Span::new(start, self.previous_token.span.end()),
                ))
            }
            TokenKind::LeftSquareBracket => {
                let start = self.current_token.span.start();

                self.advance();

                let element_type_node = self.expect_type()?;
                let element_type_id = self.tree.add_node(element_type_node);

                match self.current_token.kind {
                    TokenKind::Semicolon => {
                        self.advance();
                        self.expect(TokenKind::IntegerLiteral)?;

                        let length_node =
                            SyntaxKind::IntegerExpression.empty(self.previous_token.span);
                        let length_id = self.tree.add_node(length_node);

                        self.expect(TokenKind::RightSquareBracket)?;

                        Ok(SyntaxKind::ArrayType.with_binary_children(
                            Span::new(start, self.previous_token.span.end()),
                            element_type_id,
                            length_id,
                        ))
                    }
                    TokenKind::RightSquareBracket => {
                        self.advance();

                        Ok(SyntaxKind::SliceType.with_single_child(
                            Span::new(start, self.previous_token.span.end()),
                            element_type_id,
                        ))
                    }
                    _ => Err(ParseError::ExpectedMultipleTokens {
                        expected: &[TokenKind::Semicolon, TokenKind::RightSquareBracket],
                        found: self.current_token.kind,
                        position: self.current_position(),
                    }),
                }
            }
            TokenKind::Fn => {
                let start = self.current_token.span.start();

                self.advance();
                self.expect(TokenKind::LeftParenthesis)?;

                let mut children = Self::new_child_buffer();

                while !self.allow(TokenKind::RightParenthesis)? {
                    let parameter_type_node = self.expect_type()?;
                    let parameter_type_id = self.tree.add_node(parameter_type_node);

                    children.push(parameter_type_id);

                    self.allow(TokenKind::Comma)?;
                }

                let value_parameter_types_node = self.create_node(
                    SyntaxKind::FunctionTypeValueParameterTypes,
                    children,
                    SyntaxFlags::default(),
                    Span::new(start, self.previous_token.span.end()),
                );
                let value_parameter_types_node_id = self.tree.add_node(value_parameter_types_node);

                if self.allow(TokenKind::ArrowThin)? {
                    let return_type_node = self.expect_type()?;
                    let return_type_id = self.tree.add_node(return_type_node);

                    Ok(SyntaxKind::FunctionType.with_binary_children(
                        Span::new(start, self.previous_token.span.end()),
                        value_parameter_types_node_id,
                        return_type_id,
                    ))
                } else {
                    Ok(SyntaxKind::FunctionType.with_single_child(
                        Span::new(start, self.previous_token.span.end()),
                        value_parameter_types_node_id,
                    ))
                }
            }
            _ => Err(ParseError::ExpectedMultipleTokens {
                expected: &[
                    TokenKind::Identifier,
                    TokenKind::Bool,
                    TokenKind::I8,
                    TokenKind::I16,
                    TokenKind::I32,
                    TokenKind::I64,
                    TokenKind::I128,
                    TokenKind::ISize,
                    TokenKind::U8,
                    TokenKind::U16,
                    TokenKind::U32,
                    TokenKind::U64,
                    TokenKind::U128,
                    TokenKind::USize,
                    TokenKind::F32,
                    TokenKind::F64,
                    TokenKind::Char,
                    TokenKind::Str,
                    TokenKind::Fn,
                    TokenKind::SelfType,
                    TokenKind::LeftParenthesis,
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

        let is_mutable = self.allow(TokenKind::Mut)?;
        let path_node = self.expect_simple_path()?;
        let path_id = self.tree.add_node(path_node);
        let type_notation_id = if self.allow(TokenKind::Colon)? {
            let type_node = self.expect_type()?;
            let type_id = self.tree.add_node(type_node);

            Some(type_id)
        } else {
            None
        };

        self.expect(TokenKind::Equal)?;

        let expression_id = {
            let expression_statement_node = self.pratt(Precedence::None)?;

            if expression_statement_node.kind != SyntaxKind::ExpressionStatement {
                return Err(ParseError::ExpectedToken {
                    found: self.current_token.kind,
                    expected: TokenKind::Semicolon,
                    position: self.current_position(),
                });
            }

            expression_statement_node.children.left_id()
        };

        let span = Span::new(start, self.previous_token.span.end());

        let mut let_statement_node = if let Some(type_notation_id) = type_notation_id {
            let children = self
                .tree
                .add_children([path_id, expression_id, type_notation_id]);

            SyntaxKind::LetStatement.with_children(span, children)
        } else {
            SyntaxKind::LetStatement.with_binary_children(span, path_id, expression_id)
        };
        if is_mutable {
            let_statement_node.flags.set_flag(SyntaxFlags::PUBLIC);
        }

        Ok(let_statement_node)
    }

    fn parse_infix_assignment_operator(
        &mut self,
        left: SyntaxNode,
    ) -> Result<SyntaxNode, ParseError> {
        let left_id = self.tree.add_node(left);

        self.advance();

        let expression_node = self.parse_sub_expression(Precedence::Assignment)?;
        let expression_id = self.tree.add_node(expression_node);

        Ok(SyntaxKind::AssignmentExpression.with_binary_children(
            Span::new(left.span.start(), self.previous_token.span.end()),
            left_id,
            expression_id,
        ))
    }

    fn parse_prefix_boolean(&mut self) -> Result<SyntaxNode, ParseError> {
        self.advance();

        let mut node = SyntaxKind::BooleanExpression.empty(self.previous_token.span);

        if self.previous_token.kind == TokenKind::True {
            node.flags.set_flag(SyntaxFlags::BOOLEAN_TRUE);
        }

        Ok(node)
    }

    fn parse_prefix_character(&mut self) -> Result<SyntaxNode, ParseError> {
        self.advance();

        Ok(SyntaxKind::CharacterExpression.empty(self.previous_token.span))
    }

    fn parse_prefix_float(&mut self) -> Result<SyntaxNode, ParseError> {
        self.advance();

        Ok(SyntaxKind::FloatExpression.empty(self.previous_token.span))
    }

    fn parse_prefix_hexadecimal_integer(&mut self) -> Result<SyntaxNode, ParseError> {
        self.advance();

        Ok(SyntaxKind::HexadecimalExpression.empty(self.previous_token.span))
    }

    fn parse_prefix_integer(&mut self) -> Result<SyntaxNode, ParseError> {
        self.advance();

        Ok(SyntaxKind::IntegerExpression.empty(self.previous_token.span))
    }

    fn parse_prefix_string(&mut self) -> Result<SyntaxNode, ParseError> {
        self.advance();

        Ok(SyntaxKind::StringExpression.empty(self.previous_token.span))
    }

    fn parse_prefix_unary_operator(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();
        let kind = match self.current_token.kind {
            TokenKind::Minus => SyntaxKind::NegationExpression,
            TokenKind::Bang => SyntaxKind::NotExpression,
            _ => {
                return Err(ParseError::ExpectedMultipleTokens {
                    expected: &[TokenKind::Minus, TokenKind::Bang],
                    found: self.current_token.kind,
                    position: self.current_position(),
                });
            }
        };
        // NOTE: Consider writing a `Precedence::from_prefix` method to avoid creating a whole
        // `ParseRule`.
        let operator_precedence = ParseRule::from(self.current_token.kind).precedence;

        self.advance();

        let expression_node = self.parse_sub_expression(operator_precedence)?;
        let expression_id = self.tree.add_node(expression_node);

        let end = self.previous_token.span.end();

        Ok(kind.with_single_child(Span::new(start, end), expression_id))
    }

    fn parse_infix_binary_operator(&mut self, left: SyntaxNode) -> Result<SyntaxNode, ParseError> {
        let start = left.span.start();

        let operator = self.current_token.kind;
        let node_kind = match operator {
            TokenKind::Plus => SyntaxKind::AdditionExpression,
            TokenKind::PlusEqual => SyntaxKind::AdditionAssignmentExpression,
            TokenKind::Minus => SyntaxKind::SubtractionExpression,
            TokenKind::MinusEqual => SyntaxKind::SubtractionAssignmentExpression,
            TokenKind::Asterisk => SyntaxKind::MultiplicationExpression,
            TokenKind::AsteriskEqual => SyntaxKind::MultiplicationAssignmentExpression,
            TokenKind::Slash => SyntaxKind::DivisionExpression,
            TokenKind::SlashEqual => SyntaxKind::DivisionAssignmentExpression,
            TokenKind::Percent => SyntaxKind::ModuloExpression,
            TokenKind::PercentEqual => SyntaxKind::ModuloAssignmentExpression,
            TokenKind::Caret => SyntaxKind::ExponentExpression,
            TokenKind::CaretEqual => SyntaxKind::ExponentAssignmentExpression,
            TokenKind::DoubleEqual => SyntaxKind::EqualExpression,
            TokenKind::BangEqual => SyntaxKind::NotEqualExpression,
            TokenKind::Greater => SyntaxKind::GreaterThanExpression,
            TokenKind::GreaterEqual => SyntaxKind::GreaterThanOrEqualExpression,
            TokenKind::Less => SyntaxKind::LessThanExpression,
            TokenKind::LessEqual => SyntaxKind::LessThanOrEqualExpression,
            TokenKind::DoubleAmpersand => SyntaxKind::AndExpression,
            TokenKind::DoublePipe => SyntaxKind::OrExpression,
            TokenKind::DoubleDot => SyntaxKind::RangeExpression,
            TokenKind::DoubleDotEqual => SyntaxKind::RangeInclusiveExpression,
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
                        TokenKind::DoubleDot,
                        TokenKind::DoubleDotEqual,
                    ],
                    found: operator,
                    position: self.current_position(),
                });
            }
        };

        let left_id = self.tree.add_node(left);

        let parse_rule = ParseRule::from(operator);
        let operator_precedence = parse_rule.precedence;
        let right_precedence = match parse_rule.associativity {
            Associativity::Left => operator_precedence.increment(),
            Associativity::Right => operator_precedence,
        };

        self.advance();

        let right = self.parse_sub_expression(right_precedence)?;
        let right_id = self.tree.add_node(right);

        Ok(node_kind.with_binary_children(
            Span::new(start, self.previous_token.span.end()),
            left_id,
            right_id,
        ))
    }

    fn parse_infix_as_keyword(&mut self, left: SyntaxNode) -> Result<SyntaxNode, ParseError> {
        let start = left.span.start();
        let left_id = self.tree.add_node(left);

        self.advance();

        let type_node = self.expect_type()?;
        let type_id = self.tree.add_node(type_node);
        let end = self.previous_token.span.end();

        Ok(SyntaxKind::AsExpression.with_binary_children(Span::new(start, end), left_id, type_id))
    }

    fn parse_prefix_left_parenthesis(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.advance();

        if self.allow(TokenKind::RightParenthesis)? {
            return Ok(SyntaxKind::GroupedExpression
                .empty(Span::new(start, self.previous_token.span.end())));
        }

        let expression_node = self.parse_expression()?;
        let expression_id = self.tree.add_node(expression_node);

        self.expect(TokenKind::RightParenthesis)?;

        let end = self.previous_token.span.end();

        Ok(SyntaxKind::GroupedExpression.with_single_child(Span::new(start, end), expression_id))
    }

    fn parse_prefix_left_brace(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.advance();

        let mut children = Self::new_child_buffer();

        while !self.allow(TokenKind::RightCurlyBrace)? {
            match self.pratt(Precedence::None) {
                Ok(node) => {
                    let child_id = self.tree.add_node(node);

                    children.push(child_id);
                }
                Err(error) => self.recover(error),
            }
        }

        Ok(self.create_node(
            SyntaxKind::BlockExpression,
            children,
            SyntaxFlags::default(),
            Span::new(start, self.previous_token.span.end()),
        ))
    }

    fn parse_prefix_if_keyword(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.advance();

        let condition_node = self.parse_expression()?;
        let condition_id = self.tree.add_node(condition_node);

        let then_node = self.parse_prefix_left_brace()?;
        let then_id = self.tree.add_node(then_node);

        if self.current_token.kind == TokenKind::Else {
            let else_node = self.parse_else_expression()?;
            let else_id = self.tree.add_node(else_node);

            let children = self.tree.add_children([condition_id, then_id, else_id]);

            Ok(SyntaxKind::IfExpression
                .with_children(Span::new(start, self.previous_token.span.end()), children))
        } else {
            Ok(SyntaxKind::IfExpression.with_binary_children(
                Span::new(start, self.previous_token.span.end()),
                condition_id,
                then_id,
            ))
        }
    }

    fn parse_else_expression(&mut self) -> Result<SyntaxNode, ParseError> {
        self.advance();

        match self.current_token.kind {
            TokenKind::If => self.parse_prefix_if_keyword(),
            TokenKind::LeftCurlyBrace => self.parse_prefix_left_brace(),
            _ => Err(ParseError::ExpectedMultipleTokens {
                expected: &[TokenKind::If, TokenKind::LeftCurlyBrace],
                found: self.current_token.kind,
                position: self.current_position(),
            }),
        }
    }

    fn parse_prefix_while_keyword(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.advance();

        let condition_node = self.parse_expression()?;
        let condition_id = self.tree.add_node(condition_node);

        let body_node = self.parse_prefix_left_brace()?;
        let body_id = self.tree.add_node(body_node);

        Ok(SyntaxKind::WhileExpression.with_binary_children(
            Span::new(start, self.previous_token.span.end()),
            condition_id,
            body_id,
        ))
    }

    fn parse_prefix_break_keyword(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.advance();

        if self.allow(TokenKind::Semicolon)? {
            let end = self.previous_token.span.end();

            Ok(SyntaxKind::BreakExpression.empty(Span::new(start, end)))
        } else {
            let expression_node = self.parse_expression()?;
            let expression_id = self.tree.add_node(expression_node);
            let end = self.previous_token.span.end();

            Ok(SyntaxKind::BreakExpression.with_single_child(Span::new(start, end), expression_id))
        }
    }

    fn parse_prefix_return_keyord(&mut self) -> Result<SyntaxNode, ParseError> {
        todo!()
    }

    fn parse_prefix_self_value(&mut self) -> Result<SyntaxNode, ParseError> {
        let span = self.current_token.span;
        let self_node = SyntaxKind::SimplePath.empty(span);
        let self_id = self.tree.add_node(self_node);

        self.advance();

        Ok(SyntaxKind::PathExpression.with_single_child(span, self_id))
    }

    fn parse_prefix_identifier(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();
        let may_be_struct = !matches!(self.previous_token.kind, TokenKind::If | TokenKind::While);

        let mut path_node = self.expect_path()?;

        if may_be_struct && let Some(struct_fields_node) = self.allow_struct_expression_fields()? {
            let path_id = self.tree.add_node(path_node);
            let struct_fields_id = self.tree.add_node(struct_fields_node);

            return Ok(SyntaxKind::StructExpression.with_binary_children(
                Span::new(start, self.previous_token.span.end()),
                path_id,
                struct_fields_id,
            ));
        }

        path_node.kind = SyntaxKind::PathExpression;

        Ok(path_node)
    }

    fn parse_prefix_left_bracket(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.advance();

        let first_expression_node = self.parse_sub_expression(Precedence::Assignment)?;
        let first_expression_id = self.tree.add_node(first_expression_node);

        if self.allow(TokenKind::Semicolon)? {
            self.expect(TokenKind::IntegerLiteral)?;

            let integer_expression_node =
                SyntaxKind::IntegerExpression.empty(self.previous_token.span);
            let length_expression_id = self.tree.add_node(integer_expression_node);

            self.expect(TokenKind::RightSquareBracket)?;

            return Ok(SyntaxKind::ArrayRepeatExpression.with_binary_children(
                Span::new(start, self.previous_token.span.end()),
                first_expression_id,
                length_expression_id,
            ));
        }

        let mut children = Self::new_child_buffer();

        children.push(first_expression_id);

        if self.current_token.kind != TokenKind::RightSquareBracket {
            self.expect(TokenKind::Comma)?;
        }

        while !self.allow(TokenKind::RightSquareBracket)? {
            let child_node = self.parse_expression()?;
            let child_id = self.tree.add_node(child_node);

            children.push(child_id);

            match self.current_token.kind {
                TokenKind::Comma => self.advance(),
                TokenKind::RightSquareBracket => {}
                _ => {
                    return Err(ParseError::ExpectedMultipleTokens {
                        expected: &[TokenKind::Comma, TokenKind::RightSquareBracket],
                        found: self.current_token.kind,
                        position: self.current_position(),
                    });
                }
            };
        }

        Ok(self.create_node(
            SyntaxKind::ArrayExpression,
            children,
            SyntaxFlags::default(),
            Span::new(start, self.previous_token.span.end()),
        ))
    }

    fn parse_infix_dot(&mut self, left: SyntaxNode) -> Result<SyntaxNode, ParseError> {
        let start = left.span.start();
        let left_id = self.tree.add_node(left);

        self.advance();

        let field_name_node = self.expect_simple_path()?;
        let field_name_id = self.tree.add_node(field_name_node);

        let end = self.previous_token.span.end();

        Ok(SyntaxKind::FieldAccessExpression.with_binary_children(
            Span::new(start, end),
            left_id,
            field_name_id,
        ))
    }

    fn parse_infix_left_bracket(&mut self, left: SyntaxNode) -> Result<SyntaxNode, ParseError> {
        let start = left.span.start();
        let left_id = self.tree.add_node(left);

        self.advance();

        let index_node = self.parse_expression()?;
        let index_id = self.tree.add_node(index_node);

        self.expect(TokenKind::RightSquareBracket)?;

        let end = self.previous_token.span.end();

        Ok(SyntaxKind::IndexExpression.with_binary_children(
            Span::new(start, end),
            left_id,
            index_id,
        ))
    }

    fn parse_infix_left_parenthesis(&mut self, left: SyntaxNode) -> Result<SyntaxNode, ParseError> {
        let start = left.span.start();
        let left_id = self.tree.add_node(left);

        self.advance();

        let mut value_arguments = Self::new_child_buffer();

        while !self.allow(TokenKind::RightParenthesis)? {
            let argument_node = self.parse_expression()?;
            let argument_id = self.tree.add_node(argument_node);

            value_arguments.push(argument_id);

            match self.current_token.kind {
                TokenKind::Comma => self.advance(),
                TokenKind::RightParenthesis => {}
                _ => {
                    return Err(ParseError::ExpectedMultipleTokens {
                        expected: &[TokenKind::Comma, TokenKind::RightParenthesis],
                        found: self.current_token.kind,
                        position: self.current_position(),
                    });
                }
            };
        }

        let end = self.previous_token.span.end();
        let call_value_arguments_node = self.create_node(
            SyntaxKind::ValueArguments,
            value_arguments,
            SyntaxFlags::default(),
            Span::new(left.span.start(), self.previous_token.span.end()),
        );
        let call_value_arguments_id = self.tree.add_node(call_value_arguments_node);

        Ok(SyntaxKind::CallExpression.with_binary_children(
            Span::new(start, end),
            left_id,
            call_value_arguments_id,
        ))
    }

    fn expect_path(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.expect(TokenKind::Identifier)?;

        let mut children = Self::new_child_buffer();
        let mut last_segment_span = self.previous_token.span;

        loop {
            if !self.allow(TokenKind::DoubleColon)? {
                let segment = SyntaxKind::PathSegment.empty(last_segment_span);

                children.push(self.tree.add_node(segment));

                break;
            }

            if self.current_token.kind == TokenKind::Less {
                let Some(type_arguments) = self.allow_type_arguments()? else {
                    let segment = SyntaxKind::PathSegment.empty(last_segment_span);

                    children.push(self.tree.add_node(segment));

                    break;
                };
                let type_arguments_id = self.tree.add_node(type_arguments);
                let segment =
                    SyntaxKind::PathSegment.with_single_child(last_segment_span, type_arguments_id);

                children.push(self.tree.add_node(segment));

                if !self.allow(TokenKind::DoubleColon)? {
                    break;
                }

                self.expect(TokenKind::Identifier)?;

                last_segment_span = self.previous_token.span;

                continue;
            }

            let segment = SyntaxKind::PathSegment.empty(last_segment_span);

            children.push(self.tree.add_node(segment));
            self.expect(TokenKind::Identifier)?;

            last_segment_span = self.previous_token.span;
        }

        let end = self.previous_token.span.end();

        Ok(self.create_node(
            SyntaxKind::Path,
            children,
            SyntaxFlags::default(),
            Span::new(start, end),
        ))
    }

    fn parse_infix_semicolon(&mut self, left: SyntaxNode) -> Result<SyntaxNode, ParseError> {
        let start = left.span.start();
        let left_id = self.tree.add_node(left);

        self.advance();

        Ok(SyntaxKind::ExpressionStatement
            .with_single_child(Span::new(start, self.previous_token.span.end()), left_id))
    }

    fn expect_simple_path(&mut self) -> Result<SyntaxNode, ParseError> {
        self.expect(TokenKind::Identifier)?;

        Ok(SyntaxKind::SimplePath.empty(self.previous_token.span))
    }

    fn allow_type_parameters(&mut self) -> Result<Option<SyntaxNode>, ParseError> {
        if !self.allow(TokenKind::Less)? {
            return Ok(None);
        }

        let start = self.previous_token.span.start();

        let mut type_parameter_nodes = Self::new_child_buffer();

        while !self.allow(TokenKind::Greater)? {
            let param_start = self.current_token.span.start();
            let type_parameter_path_node = self.expect_simple_path()?;
            let type_parameter_path_id = self.tree.add_node(type_parameter_path_node);

            let type_parameter_node = if self.allow(TokenKind::Colon)? {
                let bounds_node = self.parse_trait_bounds()?;
                let bounds_id = self.tree.add_node(bounds_node);

                SyntaxKind::TypeParameter.with_binary_children(
                    Span::new(param_start, self.previous_token.span.end()),
                    type_parameter_path_id,
                    bounds_id,
                )
            } else {
                SyntaxKind::TypeParameter.with_single_child(
                    Span::new(param_start, self.previous_token.span.end()),
                    type_parameter_path_id,
                )
            };
            let type_parameter_id = self.tree.add_node(type_parameter_node);

            type_parameter_nodes.push(type_parameter_id);

            match self.current_token.kind {
                TokenKind::Comma => self.advance(),
                TokenKind::Greater => {}
                _ => {
                    return Err(ParseError::ExpectedMultipleTokens {
                        expected: &[TokenKind::Comma, TokenKind::Greater],
                        found: self.current_token.kind,
                        position: self.current_position(),
                    });
                }
            };
        }

        Ok(Some(self.create_node(
            SyntaxKind::TypeParameters,
            type_parameter_nodes,
            SyntaxFlags::default(),
            Span::new(start, self.previous_token.span.end()),
        )))
    }

    fn parse_trait_bounds(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();
        let mut bound_ids = Self::new_child_buffer();

        let first_bound = self.expect_path()?;
        let first_bound_id = self.tree.add_node(first_bound);

        bound_ids.push(first_bound_id);

        while self.allow(TokenKind::Plus)? {
            let bound = self.expect_path()?;
            let bound_id = self.tree.add_node(bound);

            bound_ids.push(bound_id);
        }

        Ok(self.create_node(
            SyntaxKind::TraitBounds,
            bound_ids,
            SyntaxFlags::default(),
            Span::new(start, self.previous_token.span.end()),
        ))
    }

    fn allow_where_clause(&mut self) -> Result<Option<SyntaxNode>, ParseError> {
        if !self.allow(TokenKind::Where)? {
            return Ok(None);
        }

        let start = self.previous_token.span.start();
        let mut predicate_ids = Self::new_child_buffer();

        loop {
            if matches!(
                self.current_token.kind,
                TokenKind::LeftCurlyBrace | TokenKind::Semicolon | TokenKind::Eof
            ) {
                break;
            }

            let predicate_start = self.current_token.span.start();

            let mut type_node = self.expect_path()?;
            type_node.kind = SyntaxKind::TypePath;
            let type_id = self.tree.add_node(type_node);

            self.expect(TokenKind::Colon)?;

            let bounds_node = self.parse_trait_bounds()?;
            let bounds_id = self.tree.add_node(bounds_node);

            let predicate_node = SyntaxKind::WherePredicate.with_binary_children(
                Span::new(predicate_start, self.previous_token.span.end()),
                type_id,
                bounds_id,
            );
            let predicate_id = self.tree.add_node(predicate_node);
            predicate_ids.push(predicate_id);

            if !self.allow(TokenKind::Comma)? {
                break;
            }
        }

        Ok(Some(self.create_node(
            SyntaxKind::WhereClause,
            predicate_ids,
            SyntaxFlags::default(),
            Span::new(start, self.previous_token.span.end()),
        )))
    }

    fn expect_impl_item(&mut self) -> Result<SyntaxNode, ParseError> {
        match self.current_token.kind {
            TokenKind::Fn => Ok(self.parse_prefix_fn_keyword()?),
            TokenKind::Const => Ok(self.parse_prefix_const_keyword()?),
            TokenKind::Type => Ok(self.parse_prefix_type_keyword()?),
            TokenKind::Pub => {
                self.advance();

                match self.current_token.kind {
                    TokenKind::Fn => {
                        let mut node = self.parse_prefix_fn_keyword()?;
                        node.flags.set_flag(SyntaxFlags::PUBLIC);
                        Ok(node)
                    }
                    TokenKind::Const => {
                        let mut node = self.parse_prefix_const_keyword()?;
                        node.flags.set_flag(SyntaxFlags::PUBLIC);
                        Ok(node)
                    }
                    TokenKind::Type => {
                        let mut node = self.parse_prefix_type_keyword()?;
                        node.flags.set_flag(SyntaxFlags::PUBLIC);
                        Ok(node)
                    }
                    _ => Err(ParseError::ExpectedMultipleTokens {
                        expected: &[TokenKind::Fn, TokenKind::Const, TokenKind::Type],
                        found: self.current_token.kind,
                        position: self.current_position(),
                    }),
                }
            }
            _ => Err(ParseError::ExpectedMultipleTokens {
                expected: &[
                    TokenKind::Fn,
                    TokenKind::Const,
                    TokenKind::Type,
                    TokenKind::Pub,
                ],
                found: self.current_token.kind,
                position: self.current_position(),
            }),
        }
    }

    fn allow_type_arguments(&mut self) -> Result<Option<SyntaxNode>, ParseError> {
        if !self.allow(TokenKind::Less)? {
            return Ok(None);
        }

        let start = self.previous_token.span.start();

        let mut type_argument_nodes = Self::new_child_buffer();

        while !self.allow(TokenKind::Greater)? {
            let type_argument_node = self.expect_type()?;
            let type_argument_id = self.tree.add_node(type_argument_node);

            type_argument_nodes.push(type_argument_id);

            match self.current_token.kind {
                TokenKind::Comma => self.advance(),
                TokenKind::Greater => {}
                _ => {
                    return Err(ParseError::ExpectedMultipleTokens {
                        expected: &[TokenKind::Comma, TokenKind::Greater],
                        found: self.current_token.kind,
                        position: self.current_position(),
                    });
                }
            };
        }

        Ok(Some(self.create_node(
            SyntaxKind::TypeArguments,
            type_argument_nodes,
            SyntaxFlags::default(),
            Span::new(start, self.previous_token.span.end()),
        )))
    }

    fn allow_struct_expression_fields(&mut self) -> Result<Option<SyntaxNode>, ParseError> {
        if self.current_token.kind == TokenKind::LeftCurlyBrace {
            let start = self.current_token.span.start();

            self.advance();

            let mut fields = Self::new_child_buffer();

            while !self.allow(TokenKind::RightCurlyBrace)? {
                let field_path_node = self.expect_simple_path()?;
                let field_path_id = self.tree.add_node(field_path_node);

                self.expect(TokenKind::Colon)?;

                let field_expression_node = self.parse_expression()?;
                let field_expression_id = self.tree.add_node(field_expression_node);

                fields.push(field_path_id);
                fields.push(field_expression_id);

                match self.current_token.kind {
                    TokenKind::Comma => self.advance(),
                    TokenKind::RightCurlyBrace => {}
                    _ => {
                        return Err(ParseError::ExpectedMultipleTokens {
                            expected: &[TokenKind::Comma, TokenKind::RightCurlyBrace],
                            found: self.current_token.kind,
                            position: self.current_position(),
                        });
                    }
                }
            }

            return Ok(Some(self.create_node(
                SyntaxKind::StructExpressionStructFields,
                fields,
                SyntaxFlags::default(),
                Span::new(start, self.previous_token.span.end()),
            )));
        }

        Ok(None)
    }

    fn parse_trait_const_item(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.advance();

        let name_node = self.expect_simple_path()?;
        let name_id = self.tree.add_node(name_node);

        self.expect(TokenKind::Colon)?;

        let type_node = self.expect_type()?;
        let type_id = self.tree.add_node(type_node);

        if self.allow(TokenKind::Equal)? {
            let expression_node = self.parse_expression()?;
            let expression_id = self.tree.add_node(expression_node);

            self.expect(TokenKind::Semicolon)?;

            let children = self.tree.add_children([name_id, type_id, expression_id]);

            Ok(SyntaxKind::ConstItem
                .with_children(Span::new(start, self.previous_token.span.end()), children))
        } else {
            self.expect(TokenKind::Semicolon)?;

            Ok(SyntaxKind::ConstItem.with_binary_children(
                Span::new(start, self.previous_token.span.end()),
                name_id,
                type_id,
            ))
        }
    }

    fn parse_trait_type(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.advance();

        let name_node = self.expect_simple_path()?;
        let name_id = self.tree.add_node(name_node);

        if self.allow(TokenKind::Equal)? {
            let aliased_type_node = self.expect_type()?;
            let aliased_type_id = self.tree.add_node(aliased_type_node);

            self.expect(TokenKind::Semicolon)?;

            Ok(SyntaxKind::TypeItem.with_binary_children(
                Span::new(start, self.previous_token.span.end()),
                name_id,
                aliased_type_id,
            ))
        } else {
            self.expect(TokenKind::Semicolon)?;

            Ok(SyntaxKind::TypeItem
                .with_single_child(Span::new(start, self.previous_token.span.end()), name_id))
        }
    }

    fn parse_trait_function_item(&mut self) -> Result<SyntaxNode, ParseError> {
        let start = self.current_token.span.start();

        self.advance();

        let name_node = self.expect_simple_path()?;
        let name_id = self.tree.add_node(name_node);

        let mut signature_children = Self::new_child_buffer();
        let mut signature_flags = SyntaxFlags::default();

        if let Some(type_parameters_node) = self.allow_type_parameters()? {
            let type_parameters_id = self.tree.add_node(type_parameters_node);

            signature_children.push(type_parameters_id);
            signature_flags.set_flag(SyntaxFlags::TYPE_PARAMETERS);
        }

        self.expect(TokenKind::LeftParenthesis)?;

        let mut value_parameters_children = Self::new_child_buffer();
        let mut value_parameters_flags = SyntaxFlags::default();

        if self.current_token.kind == TokenKind::SelfValue {
            self.advance();
            self.allow(TokenKind::Comma)?;

            value_parameters_flags.set_flag(SyntaxFlags::SELF_VALUE);
        }

        while !self.allow(TokenKind::RightParenthesis)? {
            let parameter_path_node = self.expect_simple_path()?;
            let parameter_path_id = self.tree.add_node(parameter_path_node);

            self.expect(TokenKind::Colon)?;

            let parameter_type_node_id = self.expect_type()?;
            let parameter_type_id = self.tree.add_node(parameter_type_node_id);

            value_parameters_children.push(parameter_path_id);
            value_parameters_children.push(parameter_type_id);

            match self.current_token.kind {
                TokenKind::Comma => self.advance(),
                TokenKind::RightParenthesis => {}
                _ => {
                    return Err(ParseError::ExpectedMultipleTokens {
                        expected: &[TokenKind::Comma, TokenKind::RightParenthesis],
                        found: self.current_token.kind,
                        position: self.current_position(),
                    });
                }
            }
        }

        if value_parameters_flags.get_flag(SyntaxFlags::SELF_VALUE)
            || !value_parameters_children.is_empty()
        {
            let value_parameters_node = self.create_node(
                SyntaxKind::ValueParameters,
                value_parameters_children,
                value_parameters_flags,
                Span::new(start, self.previous_token.span.end()),
            );
            let value_parameters_id = self.tree.add_node(value_parameters_node);

            signature_children.push(value_parameters_id);
            signature_flags.set_flag(SyntaxFlags::VALUE_PARAMETERS);
        }

        if self.allow(TokenKind::ArrowThin)? {
            let return_type_node = self.expect_type()?;
            let return_type_id = self.tree.add_node(return_type_node);

            signature_children.push(return_type_id);
            signature_flags.set_flag(SyntaxFlags::RETURN_TYPE);
        }

        if let Some(where_clause_node) = self.allow_where_clause()? {
            let where_clause_id = self.tree.add_node(where_clause_node);

            signature_children.push(where_clause_id);
            signature_flags.set_flag(SyntaxFlags::WHERE_CLAUSE);
        }

        let signature_node = self.create_node(
            SyntaxKind::FunctionSignature,
            signature_children,
            signature_flags,
            Span::new(start, self.previous_token.span.end()),
        );
        let signature_id = self.tree.add_node(signature_node);

        if self.current_token.kind == TokenKind::LeftCurlyBrace {
            let body_node = self.parse_prefix_left_brace()?;
            let body_id = self.tree.add_node(body_node);

            let children = self.tree.add_children([name_id, signature_id, body_id]);

            Ok(SyntaxKind::FnItem
                .with_children(Span::new(start, self.previous_token.span.end()), children))
        } else {
            self.expect(TokenKind::Semicolon)?;

            Ok(SyntaxKind::FnItem.with_binary_children(
                Span::new(start, self.previous_token.span.end()),
                name_id,
                signature_id,
            ))
        }
    }
}

pub struct ParseResult {
    pub syntax_tree: SyntaxTree,
    pub errors: Vec<ParseError>,
    pub file_module_names: Vec<Span>,
}
