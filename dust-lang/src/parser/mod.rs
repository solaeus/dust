mod error;
mod parse_rule;

#[cfg(test)]
mod tests;

pub use error::ParseError;

use std::mem::replace;

use lexical_core::{
    ParseFloatOptions, ParseIntegerOptions, format::RUST_LITERAL, parse_with_options,
};
use smallvec::SmallVec;
use tracing::{error, info};

use crate::{
    dust_error::DustError,
    lexer::Lexer,
    parser::{
        parse_rule::{Associativity, ParseRule, Precedence},
        syntax::{SyntaxId, SyntaxKind, SyntaxNode, SyntaxPayload, SyntaxTree},
    },
    source::{Position, Source, SourceFile, SourceFileId, Span},
    token::{Token, TokenKind},
};

pub fn parse<'src>(source_code: &'src str) -> (SyntaxTree, Option<DustError<'src>>) {
    let mut source = Source::new();
    let file = SourceFile::embedded_validated("eval", source_code);
    let file_id = source.add_file(file);
    let file_str = source.get_file(file_id).content_as_str();

    let lexer = Lexer::from_utf8(file_str);
    let parser = Parser::new(file_id, lexer);
    let ParseResult {
        syntax_tree,
        errors,
    } = parser.parse_main();
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

    errors: Vec<ParseError>,
}

impl<'src> Parser<'src> {
    pub fn new(file_id: SourceFileId, lexer: Lexer<'src>) -> Self {
        Self {
            lexer,
            syntax_tree: SyntaxTree::new(file_id),
            current_token: Token::default(),
            previous_token: Token::default(),
            errors: Vec::new(),
        }
    }

    pub fn parse_main(mut self) -> ParseResult {
        self.advance();
        self.parse_main_function_item()
            .unwrap_or_else(|error| self.recover(error));

        ParseResult {
            syntax_tree: self.syntax_tree,
            errors: self.errors,
        }
    }

    pub fn parse_module(mut self) -> ParseResult {
        self.advance();
        self.parse_module_item()
            .unwrap_or_else(|error| self.recover(error));

        ParseResult {
            syntax_tree: self.syntax_tree,
            errors: self.errors,
        }
    }

    pub fn source(&self) -> &[u8] {
        self.lexer.source()
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

    fn pratt(&mut self, minimum_precedence: Precedence) -> Result<(), ParseError> {
        let prefix_rule = ParseRule::from(self.current_token.kind);
        let prefix_parser = prefix_rule.prefix.ok_or(ParseError::UnexpectedToken {
            found: self.current_token.kind,
            position: self.current_position(),
        })?;

        prefix_parser(self)?;

        let mut infix_rule = ParseRule::from(self.current_token.kind);

        while minimum_precedence <= infix_rule.precedence
            && let Some(infix_parser) = infix_rule.infix
            && self.previous_token.kind != TokenKind::Semicolon
        {
            infix_parser(self)?;

            infix_rule = ParseRule::from(self.current_token.kind);
        }

        Ok(())
    }

    fn advance(&mut self) {
        if let Some(next_token) = self.lexer.next() {
            self.previous_token = replace(&mut self.current_token, next_token);
        }

        if let Some(index) = self.lexer.error_index() {
            let position = Position::new(self.syntax_tree.file_id, Span::new(index, index));

            self.recover(ParseError::InvalidUtf8 { position });
        }
    }

    fn recover(&mut self, error: ParseError) {
        self.errors.push(error);

        while !matches!(
            self.current_token.kind,
            TokenKind::Semicolon | TokenKind::RightCurlyBrace | TokenKind::Eof
        ) {
            self.advance();
        }

        self.advance();
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

    fn parse_item(&mut self) -> Result<(), ParseError> {
        self.pratt(Precedence::None)?;

        let last_node_id = self.syntax_tree.last_node_id();

        if let Some(node) = self.syntax_tree.get_node(last_node_id) {
            if node.kind.is_item() {
                return Ok(());
            }

            Err(ParseError::ExpectedItem {
                found: node.kind,
                position: Position::new(self.syntax_tree.file_id, node.span),
            })
        } else {
            Err(ParseError::UnexpectedToken {
                found: self.previous_token.kind,
                position: Position::new(self.syntax_tree.file_id, self.previous_token.span),
            })
        }
    }

    pub fn parse_pub_item(&mut self) -> Result<(), ParseError> {
        info!("Parsing pub item");

        self.advance();

        match self.current_token.kind {
            TokenKind::Use => self.parse_use_item()?,
            TokenKind::Mod => self.parse_module_item()?,
            TokenKind::Fn => self.parse_function_item_or_expression()?,
            _ => {
                return Err(ParseError::ExpectedMultipleTokens {
                    expected: &[TokenKind::Use, TokenKind::Mod, TokenKind::Fn],
                    found: self.current_token.kind,
                    position: self.current_position(),
                });
            }
        }

        Ok(())
    }

    fn parse_statement(&mut self) -> Result<(), ParseError> {
        self.pratt(Precedence::None)?;

        if let Some(node) = self.syntax_tree.last_node()
            && !node.kind.is_statement()
        {
            Err(ParseError::ExpectedStatement {
                found: node.kind,
                position: Position::new(self.syntax_tree.file_id, node.span),
            })
        } else {
            Ok(())
        }
    }

    fn parse_expression(&mut self) -> Result<(), ParseError> {
        self.pratt(Precedence::None)?;

        if let Some(node) = self.syntax_tree.last_node()
            && !node.kind.is_expression()
        {
            Err(ParseError::ExpectedExpression {
                found: Some(node.kind),
                position: Position::new(self.syntax_tree.file_id, node.span),
            })
        } else {
            Ok(())
        }
    }

    fn parse_sub_expression(&mut self, precedence: Precedence) -> Result<(), ParseError> {
        self.pratt(precedence)?;

        if let Some(node) = self.syntax_tree.last_node()
            && !node.kind.is_expression()
        {
            Err(ParseError::ExpectedExpression {
                found: Some(node.kind),
                position: Position::new(self.syntax_tree.file_id, node.span),
            })
        } else {
            Ok(())
        }
    }

    fn parse_unexpected(&mut self) -> Result<(), ParseError> {
        Err(ParseError::UnexpectedToken {
            found: self.current_token.kind,
            position: self.current_position(),
        })
    }

    fn parse_main_function_item(&mut self) -> Result<(), ParseError> {
        info!("Parsing main function item");

        let placeholder_node = SyntaxNode {
            kind: SyntaxKind::MainFunctionItem,
            span: Span::default(),
            payload: SyntaxPayload::empty(),
        };

        let _main_function_item_id = self.syntax_tree.add_node(placeholder_node);

        debug_assert_eq!(_main_function_item_id, SyntaxId::ROOT);

        let mut children = Self::new_child_buffer();

        while self.current_token.kind != TokenKind::Eof {
            if let Err(error) = self.pratt(Precedence::None) {
                self.recover(error);
            } else {
                let child_id = self.syntax_tree.last_node_id();

                if child_id == SyntaxId::ROOT {
                    break;
                }

                children.push(child_id);
            }
        }

        self.syntax_tree.nodes[0] = SyntaxNode {
            kind: SyntaxKind::MainFunctionItem,
            span: Span(0, self.current_token.span.1),
            payload: self.syntax_tree.add_children(&children),
        };

        Ok(())
    }

    fn parse_module_item(&mut self) -> Result<(), ParseError> {
        info!("Parsing module item");

        let start = self.current_token.span.0;
        let kind = if self.previous_token.kind == TokenKind::Pub {
            SyntaxKind::PublicModuleItem
        } else {
            SyntaxKind::ModuleItem
        };
        let placeholder_node = SyntaxNode {
            kind,
            span: Span::default(),
            payload: SyntaxPayload::empty(),
        };
        let node_index = self.syntax_tree.nodes.len();

        self.syntax_tree.add_node(placeholder_node);

        let end_token = if self.current_token.kind == TokenKind::Mod {
            self.advance();

            self.expect(TokenKind::Identifier)?;
            self.expect(TokenKind::LeftCurlyBrace)?;

            TokenKind::RightCurlyBrace
        } else {
            TokenKind::Eof
        };

        let mut children = Self::new_child_buffer();

        while !self.allow(end_token)? {
            if self.current_token.kind == TokenKind::Eof {
                break;
            }

            self.parse_item()?;

            children.push(self.syntax_tree.last_node_id());
        }

        let end = self.previous_token.span.1;

        let first_child = self.syntax_tree.children.len();
        let child_count = children.len();
        let node = SyntaxNode {
            kind,
            span: Span(start, end),
            payload: SyntaxPayload::child_indices(first_child, child_count),
        };

        self.syntax_tree.nodes[node_index] = node;
        self.syntax_tree.children.extend(children);

        Ok(())
    }

    fn parse_use_item(&mut self) -> Result<(), ParseError> {
        info!("Parsing use statement");

        let start = self.current_token.span.0;

        self.advance();
        self.parse_path()?;
        self.allow(TokenKind::Semicolon)?;

        let end = self.previous_token.span.1;
        let path_id = self.syntax_tree.last_node_id();

        self.syntax_tree.add_node(SyntaxNode {
            kind: SyntaxKind::UseItem,
            span: Span(start, end),
            payload: SyntaxPayload::child(path_id),
        });

        Ok(())
    }

    fn parse_struct_item(&mut self) -> Result<(), ParseError> {
        info!("Parsing struct item");

        let start = self.current_token.span.0;
        let is_public = self.previous_token.kind == TokenKind::Pub;

        self.advance();
        self.parse_path()?;

        let path_id = self.syntax_tree.last_node_id();

        if self.allow(TokenKind::LeftCurlyBrace)? {
            let mut children = Self::new_child_buffer();

            while !self.allow(TokenKind::RightCurlyBrace)? {
                if self.current_token.kind == TokenKind::Eof {
                    break;
                }

                info!("Parsing struct field definition");

                let field_start = self.current_token.span.0;

                self.parse_path()?;

                let field_name_id = self.syntax_tree.last_node_id();

                self.expect(TokenKind::Colon)?;

                let field_type_id = self.parse_type()?;

                self.allow(TokenKind::Comma)?;

                let field_end = self.previous_token.span.1;
                let field_node = SyntaxNode {
                    kind: SyntaxKind::StructFieldDefinition,
                    span: Span(field_start, field_end),
                    payload: SyntaxPayload::children(field_name_id, field_type_id),
                };
                let field_node_id = self.syntax_tree.add_node(field_node);

                children.push(field_node_id);
            }

            let end = self.previous_token.span.1;
            let children = self.syntax_tree.add_children(&children);
            let struct_fields = SyntaxNode {
                kind: SyntaxKind::StructFieldsDefinition,
                span: Span(start, end),
                payload: children,
            };
            let struct_fields_id = self.syntax_tree.add_node(struct_fields);
            let struct_kind = if is_public {
                SyntaxKind::PublicStructItem
            } else {
                SyntaxKind::StructItem
            };
            let node = SyntaxNode {
                kind: struct_kind,
                span: Span(start, end),
                payload: SyntaxPayload::children(path_id, struct_fields_id),
            };

            self.syntax_tree.add_node(node);
        } else if self.allow(TokenKind::LeftParenthesis)? {
            todo!()
        } else {
            return Err(ParseError::ExpectedMultipleTokens {
                found: self.current_token.kind,
                expected: &[TokenKind::LeftCurlyBrace, TokenKind::LeftParenthesis],
                position: self.current_position(),
            });
        }

        Ok(())
    }

    fn parse_function_item_or_expression(&mut self) -> Result<(), ParseError> {
        let start = self.current_token.span.0;
        let kind = if self.previous_token.kind == TokenKind::Pub {
            SyntaxKind::PublicFunctionItem
        } else {
            SyntaxKind::FunctionItem
        };

        self.advance();

        match self.current_token.kind {
            TokenKind::Identifier => {
                info!("Parsing function statement");

                self.parse_path()?;

                let path_id = self.syntax_tree.last_node_id();

                self.parse_function_expression()?;

                let end = self.previous_token.span.1;
                let function_expression_id = self.syntax_tree.last_node_id();
                let node = SyntaxNode {
                    kind,
                    span: Span(start, end),
                    payload: SyntaxPayload::children(path_id, function_expression_id),
                };

                self.syntax_tree.add_node(node);

                Ok(())
            }
            TokenKind::LeftParenthesis => {
                self.parse_function_expression()?;

                Ok(())
            }
            _ => Err(ParseError::ExpectedMultipleTokens {
                expected: &[TokenKind::Identifier, TokenKind::LeftParenthesis],
                found: self.current_token.kind,
                position: self.current_position(),
            }),
        }
    }

    fn parse_function_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing function expression");

        let start = self.current_token.span.0;
        let function_signature_id = self.parse_function_signature()?;

        self.parse_block_expression()?;

        let block_id = self.syntax_tree.last_node_id();
        let end = self.previous_token.span.1;
        let node = SyntaxNode {
            kind: SyntaxKind::FunctionExpression,
            span: Span(start, end),
            payload: SyntaxPayload::children(function_signature_id, block_id),
        };

        self.syntax_tree.add_node(node);

        Ok(())
    }

    fn parse_function_signature(&mut self) -> Result<SyntaxId, ParseError> {
        info!("Parsing function signature");

        let start = self.current_token.span.0;
        let value_parameter_list_node_id = self.parse_function_value_parameters()?;
        let return_type_node_id = if self.allow(TokenKind::ArrowThin)? {
            self.parse_type()?
        } else {
            SyntaxId::NONE
        };

        let end = self.previous_token.span.1;
        let node = SyntaxNode {
            kind: SyntaxKind::FunctionSignature,
            span: Span(start, end),
            payload: SyntaxPayload::children(value_parameter_list_node_id, return_type_node_id),
        };
        let node_id = self.syntax_tree.add_node(node);

        Ok(node_id)
    }

    fn parse_function_value_parameters(&mut self) -> Result<SyntaxId, ParseError> {
        info!("Parsing function value parameters");

        let start = self.current_token.span.0;

        self.expect(TokenKind::LeftParenthesis)?;

        let mut children = Self::new_child_buffer();

        while !self.allow(TokenKind::RightParenthesis)? {
            if self.current_token.kind == TokenKind::Eof {
                break;
            }

            info!("Parsing function value parameter");

            let parameter_start = self.current_token.span.0;
            let identifier_position =
                Position::new(self.syntax_tree.file_id, self.current_token.span);
            let parameter_name_node = SyntaxNode {
                kind: SyntaxKind::ValueParameterName,
                span: identifier_position.span,
                payload: SyntaxPayload::empty(),
            };
            let parameter_name_node_id = self.syntax_tree.add_node(parameter_name_node);

            self.advance();
            self.expect(TokenKind::Colon)?;

            let type_node_id = self.parse_type()?;
            let parameter_end = self.previous_token.span.1;
            let node = SyntaxNode {
                kind: SyntaxKind::ValueParameterDefinition,
                span: Span(parameter_start, parameter_end),
                payload: SyntaxPayload::children(parameter_name_node_id, type_node_id),
            };
            let node_id = self.syntax_tree.add_node(node);

            children.push(node_id);

            self.allow(TokenKind::Comma)?;
        }

        let children = self.syntax_tree.add_children(&children);
        let end = self.previous_token.span.1;
        let node = SyntaxNode {
            kind: SyntaxKind::ValueParametersDefinition,
            span: Span(start, end),
            payload: children,
        };
        let node_id = self.syntax_tree.add_node(node);

        Ok(node_id)
    }

    fn parse_type(&mut self) -> Result<SyntaxId, ParseError> {
        info!("Parsing type");

        let start = self.current_token.span.0;

        let (node_kind, payload) = match self.current_token.kind {
            TokenKind::Bool => {
                self.advance();

                (SyntaxKind::BooleanType, SyntaxPayload::empty())
            }
            TokenKind::Byte => {
                self.advance();

                (SyntaxKind::ByteType, SyntaxPayload::empty())
            }
            TokenKind::Char => {
                self.advance();

                (SyntaxKind::CharacterType, SyntaxPayload::empty())
            }
            TokenKind::Float => {
                self.advance();

                (SyntaxKind::FloatType, SyntaxPayload::empty())
            }
            TokenKind::Int => {
                self.advance();

                (SyntaxKind::IntegerType, SyntaxPayload::empty())
            }
            TokenKind::Str => {
                self.advance();

                (SyntaxKind::StringType, SyntaxPayload::empty())
            }
            TokenKind::Identifier => {
                self.parse_path()?;

                let path_id = self.syntax_tree.last_node_id();

                (SyntaxKind::TypePath, SyntaxPayload::child(path_id))
            }
            TokenKind::LeftSquareBracket => {
                self.advance();

                let child_node_id = self.parse_type()?;

                self.expect(TokenKind::RightSquareBracket)?;

                (SyntaxKind::ListType, SyntaxPayload::child(child_node_id))
            }
            TokenKind::Fn => {
                self.advance();
                self.expect(TokenKind::LeftParenthesis)?;

                let mut children = Self::new_child_buffer();

                while !self.allow(TokenKind::RightParenthesis)? {
                    if self.current_token.kind == TokenKind::Eof {
                        break;
                    }

                    let child_node_id = self.parse_type()?;

                    children.push(child_node_id);

                    self.allow(TokenKind::Comma)?;
                }

                let children = self.syntax_tree.add_children(&children);
                let value_parameter_types_node = SyntaxNode {
                    kind: SyntaxKind::ValueParameterTypes,
                    span: Span(start, self.previous_token.span.1),
                    payload: children,
                };
                let value_parameter_types_node_id =
                    self.syntax_tree.add_node(value_parameter_types_node);

                let return_type_node_id = if self.allow(TokenKind::ArrowThin)? {
                    self.parse_type()?
                } else {
                    SyntaxId::NONE
                };

                (
                    SyntaxKind::FunctionType,
                    SyntaxPayload::children(value_parameter_types_node_id, return_type_node_id),
                )
            }
            _ => {
                return Err(ParseError::ExpectedMultipleTokens {
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
                });
            }
        };

        let end = self.previous_token.span.1;
        let node = SyntaxNode {
            kind: node_kind,
            span: Span(start, end),
            payload,
        };
        let node_id = self.syntax_tree.add_node(node);

        Ok(node_id)
    }

    fn parse_let_statement(&mut self) -> Result<(), ParseError> {
        info!("Parsing let statement");

        let start = self.current_token.span.0;

        self.advance();

        let kind = if self.allow(TokenKind::Mut)? {
            SyntaxKind::LetMutStatement
        } else {
            SyntaxKind::LetStatement
        };

        self.parse_path()?;

        let path_id = self.syntax_tree.last_node_id();
        let type_notation_id = if self.allow(TokenKind::Colon)? {
            self.parse_type()?
        } else {
            SyntaxId::NONE
        };

        self.expect(TokenKind::Equal)?;
        self.pratt(Precedence::None)?;

        let end = self.previous_token.span.1;
        let (expression_statement_id, expression_statement_node) =
            self.syntax_tree
                .last()
                .ok_or(ParseError::ExpectedExpression {
                    found: None,
                    position: self.current_position(),
                })?;

        if expression_statement_node.kind != SyntaxKind::ExpressionStatement {
            return Err(ParseError::ExpectedToken {
                found: self.current_token.kind,
                expected: TokenKind::Semicolon,
                position: self.current_position(),
            });
        }

        let node = SyntaxNode {
            kind,
            span: Span(start, end),
            payload: self.syntax_tree.add_children(&[
                path_id,
                expression_statement_id,
                type_notation_id,
            ]),
        };

        self.syntax_tree.add_node(node);

        Ok(())
    }

    fn parse_reassignment_statement(&mut self) -> Result<(), ParseError> {
        info!("Parsing reassignment statement");

        let start = self.previous_token.span.0;
        let path_id = self.syntax_tree.last_node_id();

        self.expect(TokenKind::Equal)?;
        self.parse_statement()?;

        let end = self.previous_token.span.1;
        let (expression_statement_id, expression_statement_node) =
            self.syntax_tree
                .last()
                .ok_or(ParseError::ExpectedExpression {
                    found: None,
                    position: self.current_position(),
                })?;

        if expression_statement_node.kind != SyntaxKind::ExpressionStatement {
            return Err(ParseError::ExpectedToken {
                found: self.current_token.kind,
                expected: TokenKind::Semicolon,
                position: self.current_position(),
            });
        }

        let node = SyntaxNode {
            kind: SyntaxKind::ReassignmentStatement,
            span: Span(start, end),
            payload: SyntaxPayload::children(path_id, expression_statement_id),
        };

        self.syntax_tree.add_node(node);

        Ok(())
    }

    fn parse_boolean_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing boolean expression");

        let span = self.current_token.span;
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
        let payload = SyntaxPayload::encode_boolean(boolean);
        let node = SyntaxNode {
            kind: SyntaxKind::BooleanExpression,
            span,
            payload,
        };

        self.advance();
        self.syntax_tree.add_node(node);

        Ok(())
    }

    fn parse_byte_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing byte expression");

        let byte_str = &self.current_source()[2..]; // Skip the "0x" prefix
        let byte = u8::from_ascii_radix(byte_str, 16).unwrap_or_default();
        let payload = SyntaxPayload::encode_byte(byte);
        let node = SyntaxNode {
            kind: SyntaxKind::ByteExpression,
            span: self.current_token.span,
            payload,
        };

        self.syntax_tree.add_node(node);
        self.advance();

        Ok(())
    }

    fn parse_character_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing character expression");

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
        let node = SyntaxNode {
            kind: SyntaxKind::CharacterExpression,
            span: self.current_token.span,
            payload,
        };

        self.advance();
        self.syntax_tree.add_node(node);

        Ok(())
    }

    fn parse_float_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing float expression");

        let float_text = self.current_source();
        let float =
            parse_with_options::<f64, RUST_LITERAL>(float_text, &ParseFloatOptions::default())
                .unwrap_or_default();
        let payload = SyntaxPayload::encode_float(float);
        let node = SyntaxNode {
            kind: SyntaxKind::FloatExpression,
            span: self.current_token.span,
            payload,
        };

        self.advance();
        self.syntax_tree.add_node(node);

        Ok(())
    }

    fn parse_integer_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing integer expression");

        let integer_text = self.current_source();
        let integer =
            parse_with_options::<i64, RUST_LITERAL>(integer_text, &ParseIntegerOptions::default())
                .unwrap_or_default();
        let payload = SyntaxPayload::encode_integer(integer);
        let node = SyntaxNode {
            kind: SyntaxKind::IntegerExpression,
            span: self.current_token.span,
            payload,
        };

        self.advance();
        self.syntax_tree.add_node(node);

        Ok(())
    }

    fn parse_string_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing string expression");

        let span_without_quotes = self.current_token.span.shrink(1);
        let string_source = &self.source()[span_without_quotes.as_usize_range()];
        let payload = SyntaxPayload::encode_string(string_source);
        let node = SyntaxNode {
            kind: SyntaxKind::StringExpression,
            span: self.current_token.span,
            payload,
        };

        self.advance();
        self.syntax_tree.add_node(node);

        Ok(())
    }

    fn parse_unary_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing unary expression");

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
        let start = self.current_token.span.0;

        self.advance();
        self.parse_sub_expression(operator_precedence)?;

        let operand_id = self.syntax_tree.last_node_id();
        let end = self.previous_token.span.1;
        let node = SyntaxNode {
            kind: node_kind,
            span: Span(start, end),
            payload: SyntaxPayload::child(operand_id),
        };

        self.syntax_tree.add_node(node);

        Ok(())
    }

    fn parse_binary_operator(&mut self) -> Result<(), ParseError> {
        info!("Parsing binary operator");

        let (left_id, left_node) =
            self.syntax_tree
                .last()
                .ok_or(ParseError::ExpectedExpression {
                    found: None,
                    position: self.current_position(),
                })?;
        let start = left_node.span.0;
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
            TokenKind::Greater => (SyntaxKind::GreaterThanExpression, false),
            TokenKind::GreaterEqual => (SyntaxKind::GreaterThanOrEqualExpression, false),
            TokenKind::Less => (SyntaxKind::LessThanExpression, false),
            TokenKind::LessEqual => (SyntaxKind::LessThanOrEqualExpression, false),
            TokenKind::DoubleEqual => (SyntaxKind::EqualExpression, false),
            TokenKind::BangEqual => (SyntaxKind::NotEqualExpression, false),
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

        let parse_rule = ParseRule::from(operator);
        let operator_precedence = parse_rule.precedence;
        let right_precedence = match parse_rule.associativity {
            Associativity::Left => operator_precedence.increment(),
            Associativity::Right => operator_precedence,
        };

        self.advance();
        self.parse_sub_expression(right_precedence)?;

        if is_statement {
            self.expect(TokenKind::Semicolon)?;
        }

        let right_id = self.syntax_tree.last_node_id();
        let end = self.previous_token.span.1;
        let node = SyntaxNode {
            kind: node_kind,
            span: Span(start, end),
            payload: SyntaxPayload::children(left_id, right_id),
        };

        self.syntax_tree.add_node(node);

        Ok(())
    }

    fn parse_as_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing as expression");

        let (expression_id, expression_node) =
            self.syntax_tree
                .last()
                .ok_or(ParseError::ExpectedExpression {
                    found: None,
                    position: self.current_position(),
                })?;
        let start = expression_node.span.0;

        self.advance();

        let type_id = self.parse_type()?;
        let end = self.previous_token.span.1;
        let node = SyntaxNode {
            kind: SyntaxKind::AsExpression,
            span: Span(start, end),
            payload: SyntaxPayload::children(expression_id, type_id),
        };

        self.syntax_tree.add_node(node);

        Ok(())
    }

    fn parse_call_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing call expression");

        self.advance();

        let (function_node_id, function_node) = self
            .syntax_tree
            .last()
            .map(|(id, node)| (id, *node))
            .ok_or(ParseError::ExpectedExpression {
                found: None,
                position: self.current_position(),
            })?;
        let start = function_node.span.0;
        let mut value_arguments = Self::new_child_buffer();

        info!("Parsing call arguments");

        while !self.allow(TokenKind::RightParenthesis)? {
            if self.current_token.kind == TokenKind::Eof {
                break;
            }

            info!("Parsing call argument");

            self.parse_expression()?;

            let argument_id = self.syntax_tree.last_node_id();

            value_arguments.push(argument_id);

            self.allow(TokenKind::Comma)?;
        }

        let end = self.previous_token.span.1;
        let children = self.syntax_tree.add_children(&value_arguments);
        let call_value_arguments_node = SyntaxNode {
            kind: SyntaxKind::CallValueArguments,
            span: Span(function_node.span.1, self.previous_token.span.1),
            payload: children,
        };
        let call_value_arguments_id = self.syntax_tree.add_node(call_value_arguments_node);
        let node = SyntaxNode {
            kind: SyntaxKind::CallExpression,
            span: Span(start, end),
            payload: SyntaxPayload::children(function_node_id, call_value_arguments_id),
        };

        self.syntax_tree.add_node(node);

        Ok(())
    }

    fn parse_grouped_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing grouped expression");

        let start = self.current_token.span.0;

        self.advance();
        self.parse_expression()?;
        self.expect(TokenKind::RightParenthesis)?;

        let end = self.previous_token.span.1;
        let expression_id = self.syntax_tree.last_node_id();
        let node = SyntaxNode {
            kind: SyntaxKind::GroupedExpression,
            span: Span(start, end),
            payload: SyntaxPayload::child(expression_id),
        };

        self.syntax_tree.add_node(node);

        Ok(())
    }

    fn parse_block_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing block expression");

        let start = self.current_token.span.0;

        self.advance();

        let mut children = Self::new_child_buffer();

        while !self.allow(TokenKind::RightCurlyBrace)? && !self.is_eof() {
            let position_before = self.current_token.span;

            match self.pratt(Precedence::None) {
                Ok(()) => {
                    let child_id = self.syntax_tree.last_node_id();

                    children.push(child_id);
                }
                Err(error) => {
                    self.recover(error);

                    if self.current_token.kind == TokenKind::Eof
                        || self.current_token.span == position_before
                    {
                        break;
                    }
                }
            }
        }

        let last_node = *self
            .syntax_tree
            .last_node()
            .ok_or(ParseError::ExpectedExpression {
                found: None,
                position: self.current_position(),
            })?;
        let payload = self.syntax_tree.add_children(&children);

        if last_node.kind.is_expression() {
            let block_node = SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                span: Span(start, self.previous_token.span.1),
                payload,
            };

            self.syntax_tree.add_node(block_node);
        } else {
            let block_node = SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                span: Span(start, self.previous_token.span.1),
                payload,
            };
            let block_node_id = self.syntax_tree.add_node(block_node);
            let expression_statement_node = SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                span: block_node.span,
                payload: SyntaxPayload::child(block_node_id),
            };

            self.syntax_tree.add_node(expression_statement_node);
        }

        Ok(())
    }

    fn parse_if_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing if expression");

        let start = self.current_token.span.0;

        self.advance();
        self.parse_expression()?;

        let condition_id = self.syntax_tree.last_node_id();

        self.parse_block_expression()?;

        let (then_id, then_node) = self
            .syntax_tree
            .last()
            .map(|(id, node)| (id, *node))
            .ok_or(ParseError::ExpectedExpression {
                found: None,
                position: self.current_position(),
            })?;
        let mut children = Self::new_child_buffer();

        children.push(condition_id);
        children.push(then_id);

        if self.current_token.kind == TokenKind::Else {
            self.parse_else_expression()?;

            let else_id = self.syntax_tree.last_node_id();

            children.push(else_id);
        }

        let end = self.previous_token.span.1;

        if then_node.kind.is_expression() {
            let node = SyntaxNode {
                kind: SyntaxKind::IfExpression,
                span: Span(start, end),
                payload: self.syntax_tree.add_children(&children),
            };

            self.syntax_tree.add_node(node);
        } else {
            let if_node = SyntaxNode {
                kind: SyntaxKind::IfExpression,
                span: Span(start, end),
                payload: self.syntax_tree.add_children(&children),
            };
            let if_node_id = self.syntax_tree.add_node(if_node);
            let expression_statement_node = SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                span: Span(start, end),
                payload: SyntaxPayload::child(if_node_id),
            };

            self.syntax_tree.add_node(expression_statement_node);
        }

        Ok(())
    }

    fn parse_else_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing else expression");

        self.advance();

        if self.current_token.kind == TokenKind::If {
            self.parse_if_expression()?;
        } else {
            self.parse_block_expression()?;
        };

        let (last_node_id, last_node) =
            self.syntax_tree
                .last()
                .ok_or(ParseError::ExpectedExpression {
                    found: None,
                    position: self.current_position(),
                })?;
        let end = last_node.span.1;
        let node = SyntaxNode {
            kind: SyntaxKind::ElseExpression,
            span: Span(last_node.span.0, end),
            payload: SyntaxPayload::child(last_node_id),
        };

        self.syntax_tree.add_node(node);

        Ok(())
    }

    fn parse_while_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing while expression");

        let start = self.current_token.span.0;

        self.advance();
        self.parse_expression()?;

        let condition_id = self.syntax_tree.last_node_id();

        self.parse_block_expression()?;

        let body_id = self.syntax_tree.last_node_id();
        let end = self.previous_token.span.1;
        let while_node = SyntaxNode {
            kind: SyntaxKind::WhileExpression,
            span: Span(start, end),
            payload: SyntaxPayload::children(condition_id, body_id),
        };
        let while_node_id = self.syntax_tree.add_node(while_node);
        let expression_statement_node = SyntaxNode {
            kind: SyntaxKind::ExpressionStatement,
            span: while_node.span,
            payload: SyntaxPayload::child(while_node_id),
        };

        self.syntax_tree.add_node(expression_statement_node);

        Ok(())
    }

    fn parse_break_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing break statement");

        let start = self.current_token.span.0;

        self.advance();
        self.allow(TokenKind::Semicolon)?;

        let end = self.previous_token.span.1;
        let node = SyntaxNode {
            kind: SyntaxKind::BreakExpression,
            span: Span(start, end),
            payload: SyntaxPayload::empty(),
        };

        self.syntax_tree.add_node(node);

        Ok(())
    }

    fn parse_return(&mut self) -> Result<(), ParseError> {
        todo!()
    }

    fn parse_path_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing path expression");

        let may_be_struct_expression = !matches!(
            self.previous_token.kind,
            TokenKind::If
                | TokenKind::Else
                | TokenKind::While
                | TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Asterisk
                | TokenKind::Slash
                | TokenKind::Percent
                | TokenKind::Caret
                | TokenKind::DoubleEqual
                | TokenKind::BangEqual
                | TokenKind::Greater
                | TokenKind::GreaterEqual
                | TokenKind::Less
                | TokenKind::LessEqual
                | TokenKind::DoubleAmpersand
                | TokenKind::DoublePipe
        );

        let span = self.current_token.span;

        self.parse_path()?;

        let path_id = self.syntax_tree.last_node_id();
        let node = if may_be_struct_expression && self.allow(TokenKind::LeftCurlyBrace)? {
            let fields_start = self.current_token.span.0;
            let mut fields = Self::new_child_buffer();

            while !self.allow(TokenKind::RightCurlyBrace)? {
                if self.current_token.kind == TokenKind::Eof {
                    break;
                }

                let field_start = self.current_token.span.0;

                self.parse_path()?;

                let field_path_id = self.syntax_tree.last_node_id();

                self.expect(TokenKind::Colon)?;
                self.parse_expression()?;

                let field_expression_id = self.syntax_tree.last_node_id();
                let field_end = self.previous_token.span.1;

                self.allow(TokenKind::Comma)?;

                let field_node = SyntaxNode {
                    kind: SyntaxKind::StructField,
                    span: Span(field_start, field_end),
                    payload: SyntaxPayload::children(field_path_id, field_expression_id),
                };
                let field_node_id = self.syntax_tree.add_node(field_node);

                fields.push(field_node_id);
            }

            let fields_children = self.syntax_tree.add_children(&fields);
            let fields_end = fields
                .last()
                .and_then(|&id| self.syntax_tree.get_node(id))
                .map_or(fields_start, |node| node.span.1);
            let struct_fields_node = SyntaxNode {
                kind: SyntaxKind::StructFields,
                span: Span(fields_start, fields_end),
                payload: fields_children,
            };
            let struct_fields_node_id = self.syntax_tree.add_node(struct_fields_node);

            SyntaxNode {
                kind: SyntaxKind::StructExpression,
                span,
                payload: SyntaxPayload::children(path_id, struct_fields_node_id),
            }
        } else {
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                span,
                payload: SyntaxPayload::child(path_id),
            }
        };

        self.syntax_tree.add_node(node);

        Ok(())
    }

    fn parse_list_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing list expression");

        let start = self.current_token.span.0;

        self.advance();

        let mut children = Self::new_child_buffer();

        while !self.allow(TokenKind::RightSquareBracket)? {
            if self.current_token.kind == TokenKind::Eof {
                break;
            }

            self.parse_expression()?;

            let child_id = self.syntax_tree.last_node_id();

            children.push(child_id);
            self.allow(TokenKind::Comma)?;
        }

        let end = self.previous_token.span.1;
        let node = SyntaxNode {
            kind: SyntaxKind::ListExpression,
            span: Span(start, end),
            payload: self.syntax_tree.add_children(&children),
        };

        self.syntax_tree.add_node(node);

        Ok(())
    }

    fn parse_index_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing index expression");

        let (target_id, target_node) =
            self.syntax_tree
                .last()
                .ok_or(ParseError::ExpectedExpression {
                    found: None,
                    position: self.current_position(),
                })?;
        let start = target_node.span.0;

        self.advance();
        self.parse_expression()?;
        self.expect(TokenKind::RightSquareBracket)?;

        let index_id = self.syntax_tree.last_node_id();
        let end = self.previous_token.span.1;
        let node = SyntaxNode {
            kind: SyntaxKind::ListIndexExpression,
            span: Span(start, end),
            payload: SyntaxPayload::children(target_id, index_id),
        };

        self.syntax_tree.add_node(node);

        Ok(())
    }

    fn parse_semicolon(&mut self) -> Result<(), ParseError> {
        self.advance();

        let end = self.previous_token.span.1;
        let Some(last_node) = self.syntax_tree.last_node() else {
            return Err(ParseError::UnexpectedToken {
                found: self.previous_token.kind,
                position: Position::new(self.syntax_tree.file_id, self.previous_token.span),
            });
        };
        let span = Span(last_node.span.0, end);
        let expression_id = self.syntax_tree.last_node_id();
        let node = SyntaxNode {
            kind: SyntaxKind::ExpressionStatement,
            span,
            payload: SyntaxPayload::child(expression_id),
        };

        self.syntax_tree.add_node(node);

        Ok(())
    }

    fn parse_path(&mut self) -> Result<(), ParseError> {
        info!("Parsing path");

        let (first_segment_id, first_segment_node) =
            if self.current_token.kind == TokenKind::Identifier {
                let identifier_span = self.current_token.span;

                self.advance();

                let node = SyntaxNode {
                    kind: SyntaxKind::PathSegment,
                    span: identifier_span,
                    payload: SyntaxPayload::empty(),
                };
                let id = self.syntax_tree.add_node(node);

                (id, node)
            } else {
                self.syntax_tree
                    .last()
                    .map(|(id, node)| (id, *node))
                    .ok_or(ParseError::ExpectedToken {
                        found: self.current_token.kind,
                        expected: TokenKind::Identifier,
                        position: self.current_position(),
                    })?
            };
        let start = first_segment_node.span.0;

        let mut children = Self::new_child_buffer();

        children.push(first_segment_id);

        while self.allow(TokenKind::DoubleColon)? {
            let identifier_span = self.current_token.span;

            self.expect(TokenKind::Identifier)?;

            let segment_node = SyntaxNode {
                kind: SyntaxKind::PathSegment,
                span: identifier_span,
                payload: SyntaxPayload::empty(),
            };
            let segment_id = self.syntax_tree.add_node(segment_node);

            children.push(segment_id);
        }

        let end = self.previous_token.span.1;
        let node = SyntaxNode {
            kind: SyntaxKind::Path,
            span: Span(start, end),
            payload: self.syntax_tree.add_children(&children),
        };

        self.syntax_tree.add_node(node);

        Ok(())
    }
}

pub struct ParseResult {
    pub syntax_tree: SyntaxTree,
    pub errors: Vec<ParseError>,
}
