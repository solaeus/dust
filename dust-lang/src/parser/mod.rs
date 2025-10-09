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
use tracing::{Level, error, info, span, warn};

use crate::{
    dust_error::DustError,
    lexer::Lexer,
    parser::parse_rule::{ParseRule, Precedence},
    source::{Position, Source, SourceCode, SourceFile, SourceFileId, Span},
    syntax_tree::{SyntaxId, SyntaxKind, SyntaxNode, SyntaxTree},
    token::{Token, TokenKind},
};

pub fn parse_main(source_code: String) -> (SyntaxTree, Option<DustError>) {
    let source = Source::new();
    let mut files = source.write_files();
    let file = SourceFile {
        name: "eval".to_string(),
        source_code: SourceCode::String(source_code),
    };

    files.push(file);

    let file = files.first().unwrap();
    let lexer = Lexer::new(file.source_code.as_ref());
    let parser = Parser::new(SourceFileId(0), lexer);
    let ParseResult {
        syntax_tree,
        errors,
    } = parser.parse_main();
    let dust_error = if errors.is_empty() {
        None
    } else {
        drop(files);

        Some(DustError::parse(errors, source))
    };

    (syntax_tree, dust_error)
}

pub struct ParseResult {
    pub syntax_tree: SyntaxTree,
    pub errors: Vec<ParseError>,
}

pub struct Parser<'src> {
    file_id: SourceFileId,

    lexer: Lexer<'src>,

    syntax_tree: SyntaxTree,

    current_token: Token,
    previous_token: Token,

    errors: Vec<ParseError>,
}

impl<'src> Parser<'src> {
    pub fn new(file_id: SourceFileId, lexer: Lexer<'src>) -> Self {
        Self {
            file_id,
            lexer,
            syntax_tree: SyntaxTree::new(),
            current_token: Token {
                kind: TokenKind::Unknown,
                span: Span::default(),
            },
            previous_token: Token {
                kind: TokenKind::Unknown,
                span: Span::default(),
            },
            errors: Vec::new(),
        }
    }

    /// Parses a source string as a complete file, returning the syntax tree and any parse errors.
    /// The parser is consumed and cannot be reused.
    pub fn parse_main(mut self) -> ParseResult {
        let span = span!(Level::INFO, "parse_main");
        let _enter = span.enter();

        self.current_token = match self.lexer.next() {
            Some(Ok(token)) => token,
            Some(Err(index)) => {
                let error = ParseError::InvalidUtf8 {
                    position: Position::new(self.file_id, Span(index as u32, index as u32 + 1)),
                };

                self.recover(error);

                return ParseResult {
                    syntax_tree: self.syntax_tree,
                    errors: self.errors,
                };
            }
            None => Token {
                kind: TokenKind::Eof,
                span: Span(0, 0),
            },
        };

        self.parse_main_function_item()
            .unwrap_or_else(|error| self.recover(error));

        ParseResult {
            syntax_tree: self.syntax_tree,
            errors: self.errors,
        }
    }

    pub fn parse_file_module(mut self) -> ParseResult {
        let span = span!(Level::INFO, "parse_module");
        let _enter = span.enter();

        self.current_token = match self.lexer.next() {
            Some(Ok(token)) => token,
            Some(Err(index)) => {
                let error = ParseError::InvalidUtf8 {
                    position: Position::new(self.file_id, Span(index as u32, index as u32 + 1)),
                };

                self.recover(error);

                return ParseResult {
                    syntax_tree: self.syntax_tree,
                    errors: self.errors,
                };
            }
            None => Token {
                kind: TokenKind::Eof,
                span: Span(0, 0),
            },
        };

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
        Position::new(self.file_id, self.current_token.span)
    }

    fn current_source(&self) -> &[u8] {
        &self.source()[self.current_token.span.0 as usize..self.current_token.span.1 as usize]
    }

    fn new_child_buffer() -> SmallVec<[SyntaxId; 4]> {
        SmallVec::<[SyntaxId; 4]>::new()
    }

    fn pratt(&mut self, minimum_precedence: Precedence) -> Result<(), ParseError> {
        let prefix_rule = ParseRule::from(self.current_token.kind);
        let prefix_parser = prefix_rule.prefix.ok_or(ParseError::UnexpectedToken {
            actual: self.current_token.kind,
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

    fn advance(&mut self) -> Result<(), ParseError> {
        let next_token = match self.lexer.next() {
            Some(Ok(token)) => token,
            Some(Err(index)) => {
                return Err(ParseError::InvalidUtf8 {
                    position: Position::new(self.file_id, Span(index as u32, index as u32 + 1)),
                });
            }
            None => Token {
                kind: TokenKind::Eof,
                span: Span(0, 0),
            },
        };

        self.previous_token = replace(&mut self.current_token, next_token);

        Ok(())
    }

    fn recover(&mut self, error: ParseError) {
        error!("{error:?}");

        self.errors.push(error);

        while !matches!(
            self.current_token.kind,
            TokenKind::Semicolon | TokenKind::RightCurlyBrace | TokenKind::Eof
        ) {
            let _ = self.advance().map_err(|error| self.errors.push(error));
        }

        warn!(
            "Error recovery has skipped to {} at {}",
            self.current_token, self.current_token.span
        );

        let _ = self.advance().map_err(|error| self.errors.push(error));
    }

    fn allow(&mut self, allowed: TokenKind) -> Result<bool, ParseError> {
        let allowed = self.current_token.kind == allowed;

        if allowed {
            self.advance()?;
        }

        Ok(allowed)
    }

    fn expect(&mut self, expected: TokenKind) -> Result<(), ParseError> {
        if self.current_token.kind != expected {
            return Err(ParseError::ExpectedToken {
                expected,
                actual: self.current_token.kind,
                position: self.current_position(),
            });
        }

        self.advance()?;

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
                actual: node.kind,
                position: Position::new(self.file_id, node.span),
            })
        } else {
            Err(ParseError::UnexpectedToken {
                actual: self.previous_token.kind,
                position: Position::new(self.file_id, self.previous_token.span),
            })
        }
    }

    pub fn parse_pub_item(&mut self) -> Result<(), ParseError> {
        info!("Parsing pub item");

        self.advance()?;

        match self.current_token.kind {
            TokenKind::Use => self.parse_use_item()?,
            TokenKind::Mod => self.parse_module_item()?,
            TokenKind::Fn => self.parse_function_item_or_expression()?,
            _ => {
                return Err(ParseError::ExpectedMultipleTokens {
                    expected: &[TokenKind::Use, TokenKind::Mod, TokenKind::Fn],
                    actual: self.current_token.kind,
                    position: self.current_position(),
                });
            }
        }

        Ok(())
    }

    fn _parse_statement(&mut self) -> Result<(), ParseError> {
        self.pratt(Precedence::None)?;

        if let Some(node) = self.syntax_tree.last_node()
            && !node.kind.is_statement()
        {
            Err(ParseError::ExpectedStatement {
                actual: node.kind,
                position: Position::new(self.file_id, node.span),
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
                actual: node.kind,
                position: Position::new(self.file_id, node.span),
            })
        } else {
            Ok(())
        }
    }

    fn parse_sub_expression(&mut self, precedence: Precedence) -> Result<(), ParseError> {
        self.pratt(precedence.increment())?;

        if let Some(node) = self.syntax_tree.last_node()
            && !node.kind.is_expression()
        {
            Err(ParseError::ExpectedExpression {
                actual: node.kind,
                position: Position::new(self.file_id, node.span),
            })
        } else {
            Ok(())
        }
    }

    fn parse_unexpected(&mut self) -> Result<(), ParseError> {
        Err(ParseError::UnexpectedToken {
            actual: self.current_token.kind,
            position: self.current_position(),
        })
    }

    fn parse_main_function_item(&mut self) -> Result<(), ParseError> {
        info!("Parsing main function item");

        let placeholder_node = SyntaxNode {
            kind: SyntaxKind::MainFunctionItem,
            span: Span::default(),
            children: (0, 0),
        };

        let _main_function_item_id = self.syntax_tree.push_node(placeholder_node);

        debug_assert_eq!(_main_function_item_id, SyntaxId(0));

        let mut children = Self::new_child_buffer();

        while self.current_token.kind != TokenKind::Eof {
            if let Err(error) = self.pratt(Precedence::None) {
                self.recover(error);
            } else {
                let child_id = self.syntax_tree.last_node_id();

                if child_id == SyntaxId(0) {
                    break;
                }

                children.push(child_id);
            }
        }

        self.syntax_tree.nodes[0] = SyntaxNode {
            kind: SyntaxKind::MainFunctionItem,
            span: Span(0, self.current_token.span.1),
            children: self.syntax_tree.add_children(&children),
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
            children: (0, 0),
        };
        let node_index = self.syntax_tree.nodes.len();

        self.syntax_tree.push_node(placeholder_node);

        // Allows for nested modules and whole file modules
        let end_token = if self.current_token.kind == TokenKind::Mod {
            self.advance()?;

            self.expect(TokenKind::Identifier)?;
            self.expect(TokenKind::LeftCurlyBrace)?;

            TokenKind::RightCurlyBrace
        } else {
            TokenKind::Eof
        };

        let mut children = Self::new_child_buffer();

        while !self.allow(end_token)? {
            self.parse_item()?;

            children.push(self.syntax_tree.last_node_id());
        }

        let end = self.previous_token.span.1;

        let first_child = self.syntax_tree.children.len() as u32;
        let child_count = children.len() as u32;
        let node = SyntaxNode {
            kind,
            span: Span(start, end),
            children: (first_child, child_count),
        };

        self.syntax_tree.nodes[node_index] = node;
        self.syntax_tree.children.extend(children);

        Ok(())
    }

    fn parse_use_item(&mut self) -> Result<(), ParseError> {
        info!("Parsing use statement");

        let start = self.current_token.span.0;

        self.advance()?;
        self.parse_path()?;
        self.allow(TokenKind::Semicolon)?;

        let end = self.previous_token.span.1;
        let path_id = self.syntax_tree.last_node_id();

        self.syntax_tree.push_node(SyntaxNode {
            kind: SyntaxKind::UseItem,
            span: Span(start, end),
            children: (path_id.0, 0),
        });

        Ok(())
    }

    fn parse_function_item_or_expression(&mut self) -> Result<(), ParseError> {
        let start = self.current_token.span.0;
        let kind = if self.previous_token.kind == TokenKind::Pub {
            SyntaxKind::PublicFunctionItem
        } else {
            SyntaxKind::FunctionItem
        };

        self.advance()?;

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
                    children: (path_id.0, function_expression_id.0),
                };

                self.syntax_tree.push_node(node);

                Ok(())
            }
            TokenKind::LeftParenthesis => {
                self.parse_function_expression()?;

                Ok(())
            }
            _ => Err(ParseError::ExpectedMultipleTokens {
                expected: &[TokenKind::Identifier, TokenKind::LeftParenthesis],
                actual: self.current_token.kind,
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
            children: (function_signature_id.0, block_id.0),
        };

        self.syntax_tree.push_node(node);

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
            children: (value_parameter_list_node_id.0, return_type_node_id.0),
        };
        let node_id = self.syntax_tree.push_node(node);

        Ok(node_id)
    }

    fn parse_function_value_parameters(&mut self) -> Result<SyntaxId, ParseError> {
        info!("Parsing function value parameters");

        let start = self.current_token.span.0;

        self.expect(TokenKind::LeftParenthesis)?;

        let mut children = Self::new_child_buffer();

        while !self.allow(TokenKind::RightParenthesis)? {
            info!("Parsing function value parameter");

            let parameter_start = self.current_token.span.0;
            let identifier_position = Position::new(self.file_id, self.current_token.span);
            let parameter_name_node = SyntaxNode {
                kind: SyntaxKind::FunctionValueParameterName,
                span: identifier_position.span,
                children: (0, 0),
            };
            let parameter_name_node_id = self.syntax_tree.push_node(parameter_name_node);

            self.expect(TokenKind::Colon)?;

            let type_node_id = self.parse_type()?;
            let parameter_end = self.previous_token.span.1;
            let node = SyntaxNode {
                kind: SyntaxKind::FunctionValueParameter,
                span: Span(parameter_start, parameter_end),
                children: (parameter_name_node_id.0, type_node_id.0),
            };
            let node_id = self.syntax_tree.push_node(node);

            children.push(node_id);

            self.allow(TokenKind::Comma)?;
        }

        let children = self.syntax_tree.add_children(&children);
        let end = self.previous_token.span.1;
        let node = SyntaxNode {
            kind: SyntaxKind::FunctionValueParameters,
            span: Span(start, end),
            children,
        };
        let node_id = self.syntax_tree.push_node(node);

        Ok(node_id)
    }

    fn parse_type(&mut self) -> Result<SyntaxId, ParseError> {
        info!("Parsing type");

        let start = self.current_token.span.0;

        let node_kind = match self.current_token.kind {
            TokenKind::Bool => SyntaxKind::BooleanType,
            TokenKind::Byte => SyntaxKind::ByteType,
            TokenKind::Char => SyntaxKind::CharacterType,
            TokenKind::Float => SyntaxKind::FloatType,
            TokenKind::Int => SyntaxKind::IntegerType,
            TokenKind::Str => SyntaxKind::StringType,
            TokenKind::Identifier => SyntaxKind::TypePath,
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
                    ],
                    actual: self.current_token.kind,
                    position: self.current_position(),
                });
            }
        };

        self.advance()?;

        let end = self.previous_token.span.1;
        let node = SyntaxNode {
            kind: node_kind,
            span: Span(start, end),
            children: (0, 0),
        };
        let node_id = self.syntax_tree.push_node(node);

        Ok(node_id)
    }

    fn parse_let_statement(&mut self) -> Result<(), ParseError> {
        info!("Parsing let statement");

        let start = self.current_token.span.0;

        self.advance()?;

        let is_mutable = self.allow(TokenKind::Mut)?;

        self.parse_path()?;

        let path_id = self.syntax_tree.last_node_id();
        let kind = if is_mutable {
            SyntaxKind::LetMutStatement
        } else {
            SyntaxKind::LetStatement
        };

        if self.allow(TokenKind::Colon)? {
            self.parse_type()?
        } else {
            SyntaxId::NONE
        };

        self.expect(TokenKind::Equal)?;
        self.pratt(Precedence::None)?;

        let end = self.previous_token.span.1;
        let expression_statement_id = self.syntax_tree.last_node_id();
        let expression_statement_node =
            self.syntax_tree
                .get_node(expression_statement_id)
                .ok_or(ParseError::MissingNode {
                    id: expression_statement_id,
                })?;

        if expression_statement_node.kind != SyntaxKind::ExpressionStatement {
            return Err(ParseError::ExpectedToken {
                actual: self.current_token.kind,
                expected: TokenKind::Semicolon,
                position: self.current_position(),
            });
        }

        let node = SyntaxNode {
            kind,
            span: Span(start, end),
            children: (path_id.0, expression_statement_id.0),
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_reassign_statement(&mut self) -> Result<(), ParseError> {
        info!("Parsing reassign statement");

        let operator = self.current_token.kind;
        let operator_precedence = ParseRule::from(operator).precedence;

        let path_node_id = self.syntax_tree.last_node_id();
        let path_node = *self
            .syntax_tree
            .get_node(path_node_id)
            .ok_or(ParseError::MissingNode { id: path_node_id })?;
        let start = path_node.span.0;

        if path_node.kind != SyntaxKind::PathExpression {
            return Err(ParseError::InvalidAssignmentTarget {
                found: path_node.kind,
                position: Position::new(self.file_id, path_node.span),
            });
        }

        self.expect(TokenKind::Equal)?;
        self.parse_sub_expression(operator_precedence)?;

        let expression_id = self.syntax_tree.last_node_id();
        let expression_node = self
            .syntax_tree
            .get_node(expression_id)
            .ok_or(ParseError::MissingNode { id: expression_id })?;

        if expression_node.kind != SyntaxKind::ExpressionStatement {
            self.expect(TokenKind::Semicolon)?;
        }

        let end = self.previous_token.span.1;
        let node = SyntaxNode {
            kind: SyntaxKind::ReassignStatement,
            span: Span(start, end),
            children: (path_node_id.0, expression_id.0),
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_boolean_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing boolean expression");

        let span = self.current_token.span;
        let boolean_payload = match self.current_token.kind {
            TokenKind::TrueValue => true as u32,
            TokenKind::FalseValue => false as u32,
            _ => {
                return Err(ParseError::ExpectedMultipleTokens {
                    expected: &[TokenKind::TrueValue, TokenKind::FalseValue],
                    actual: self.current_token.kind,
                    position: self.current_position(),
                });
            }
        };
        let node = SyntaxNode {
            kind: SyntaxKind::BooleanExpression,
            span,
            children: (boolean_payload, 0),
        };

        self.advance()?;
        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_byte_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing byte expression");

        let span = self.current_token.span;
        let byte_text_utf8 = &self.current_source()[2..]; // Skip the "0x" prefix
        let byte_payload = u8::from_ascii_radix(byte_text_utf8, 16).unwrap_or_default() as u32;
        let node = SyntaxNode {
            kind: SyntaxKind::ByteExpression,
            span,
            children: (byte_payload, 0),
        };

        self.syntax_tree.push_node(node);
        self.advance()?;

        Ok(())
    }

    fn parse_character_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing character expression");

        let span = self.current_token.span;
        let character_bytes = &self.current_source()[1..self.current_source().len() - 1];

        debug_assert!(character_bytes.len() <= 4);

        let character_payload = (
            character_bytes.first().copied().unwrap_or_default() as u32
                | (character_bytes.get(1).copied().unwrap_or_default() as u32) << 8,
            (character_bytes.get(2).copied().unwrap_or_default() as u32)
                | (character_bytes.get(3).copied().unwrap_or_default() as u32) << 8,
        );

        self.advance()?;

        let node = SyntaxNode {
            kind: SyntaxKind::CharacterExpression,
            span,
            children: character_payload,
        };
        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_float_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing float expression");

        let span = self.current_token.span;
        let float_text = self.current_source();
        let float =
            parse_with_options::<f64, RUST_LITERAL>(float_text, &ParseFloatOptions::default())
                .unwrap_or_default();
        let payload = SyntaxNode::encode_float(float);

        self.advance()?;

        let node = SyntaxNode {
            kind: SyntaxKind::FloatExpression,
            span,
            children: payload,
        };
        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_integer_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing integer expression");

        let span = self.current_token.span;
        let integer_text = self.current_source();
        let integer =
            parse_with_options::<i64, RUST_LITERAL>(integer_text, &ParseIntegerOptions::default())
                .unwrap_or_default();
        let (left_payload, right_payload) = SyntaxNode::encode_integer(integer);

        self.advance()?;

        let node = SyntaxNode {
            kind: SyntaxKind::IntegerExpression,
            span,
            children: (left_payload, right_payload),
        };
        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_string_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing string expression");

        let node = SyntaxNode {
            kind: SyntaxKind::StringExpression,
            span: self.current_token.span,
            children: (0, 0),
        };

        self.advance()?;
        self.syntax_tree.push_node(node);

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
                    actual: operator,
                    position: self.current_position(),
                });
            }
        };
        let operator_precedence = ParseRule::from(operator).precedence;
        let start = self.current_token.span.0;

        self.advance()?;
        self.parse_sub_expression(operator_precedence)?;

        let operand_id = self.syntax_tree.last_node_id();
        let end = self.previous_token.span.1;
        let node = SyntaxNode {
            kind: node_kind,
            span: Span(start, end),
            children: (operand_id.0, 0),
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_binary_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing math binary expression");

        let left_id = self.syntax_tree.last_node_id();
        let left_node = *self
            .syntax_tree
            .get_node(left_id)
            .ok_or(ParseError::MissingNode { id: left_id })?;
        let start = left_node.span.0;
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
            TokenKind::Greater => SyntaxKind::GreaterThanExpression,
            TokenKind::GreaterEqual => SyntaxKind::GreaterThanOrEqualExpression,
            TokenKind::Less => SyntaxKind::LessThanExpression,
            TokenKind::LessEqual => SyntaxKind::LessThanOrEqualExpression,
            TokenKind::DoubleEqual => SyntaxKind::EqualExpression,
            TokenKind::BangEqual => SyntaxKind::NotEqualExpression,
            TokenKind::DoubleAmpersand => SyntaxKind::AndExpression,
            TokenKind::DoublePipe => SyntaxKind::OrExpression,
            _ => {
                return Err(ParseError::ExpectedMultipleTokens {
                    expected: &[
                        TokenKind::Plus,
                        TokenKind::Minus,
                        TokenKind::Asterisk,
                        TokenKind::Slash,
                        TokenKind::Percent,
                    ],
                    actual: operator,
                    position: self.current_position(),
                });
            }
        };

        let operator_precedence = ParseRule::from(operator).precedence;

        self.advance()?;
        self.parse_sub_expression(operator_precedence)?;

        let right_id = self.syntax_tree.last_node_id();
        let end = self.previous_token.span.1;
        let node = SyntaxNode {
            kind: node_kind,
            span: Span(start, end),
            children: (left_id.0, right_id.0),
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_call_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing call expression");

        self.advance()?;

        let function_node_id = self.syntax_tree.last_node_id();
        let function_node =
            *self
                .syntax_tree
                .get_node(function_node_id)
                .ok_or(ParseError::MissingNode {
                    id: function_node_id,
                })?;
        let start = function_node.span.0;
        let mut value_arguments = Self::new_child_buffer();

        info!("Parsing call arguments");

        while !self.allow(TokenKind::RightParenthesis)? {
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
            children,
        };
        let call_value_arguments_id = self.syntax_tree.push_node(call_value_arguments_node);
        let node = SyntaxNode {
            kind: SyntaxKind::CallExpression,
            span: Span(start, end),
            children: (function_node_id.0, call_value_arguments_id.0),
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_grouped_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing grouped expression");

        let start = self.current_token.span.0;

        self.advance()?;
        self.parse_expression()?;
        self.expect(TokenKind::RightParenthesis)?;

        let end = self.previous_token.span.1;
        let expression_id = self.syntax_tree.last_node_id();
        let node = SyntaxNode {
            kind: SyntaxKind::GroupedExpression,
            span: Span(start, end),
            children: (expression_id.0, 0),
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_block_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing block expression");

        let start = self.current_token.span.0;

        self.advance()?;

        let mut children = Self::new_child_buffer();

        while !self.allow(TokenKind::RightCurlyBrace)? {
            if let Err(error) = self.pratt(Precedence::None) {
                self.recover(error);
            } else {
                let child_id = self.syntax_tree.last_node_id();

                if child_id == SyntaxId(0) {
                    break;
                }

                children.push(child_id);
            }
        }

        let first_child = self.syntax_tree.children.len() as u32;
        let child_count = children.len() as u32;

        let last_node = self
            .syntax_tree
            .last_node()
            .ok_or(ParseError::MissingNode {
                id: self.syntax_tree.last_node_id(),
            })?;

        if last_node.kind.is_expression() {
            let block_node = SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                span: Span(start, self.previous_token.span.1),
                children: (first_child, child_count),
            };

            self.syntax_tree.push_node(block_node);
            self.syntax_tree.children.extend(children);
        } else {
            let block_node = SyntaxNode {
                kind: SyntaxKind::BlockExpression,
                span: Span(start, self.previous_token.span.1),
                children: (first_child, child_count),
            };
            let block_node_id = self.syntax_tree.push_node(block_node);

            self.syntax_tree.children.extend(children);

            let expression_statement_node = SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                span: block_node.span,
                children: (block_node_id.0, 0),
            };

            self.syntax_tree.push_node(expression_statement_node);
        }

        Ok(())
    }

    fn parse_if(&mut self) -> Result<(), ParseError> {
        todo!()
    }

    fn parse_while_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing while expression");

        let start = self.current_token.span.0;

        self.advance()?;
        self.parse_expression()?;

        let condition_id = self.syntax_tree.last_node_id();

        self.parse_block_expression()?;

        let body_id = self.syntax_tree.last_node_id();
        let end = self.previous_token.span.1;
        let while_node = SyntaxNode {
            kind: SyntaxKind::WhileExpression,
            span: Span(start, end),
            children: (condition_id.0, body_id.0),
        };
        let while_node_id = self.syntax_tree.push_node(while_node);
        let expression_statement_node = SyntaxNode {
            kind: SyntaxKind::ExpressionStatement,
            span: while_node.span,
            children: (while_node_id.0, 0),
        };

        self.syntax_tree.push_node(expression_statement_node);

        Ok(())
    }

    fn parse_break_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing break statement");

        let start = self.current_token.span.0;

        self.advance()?;
        self.allow(TokenKind::Semicolon)?;

        let end = self.previous_token.span.1;
        let node = SyntaxNode {
            kind: SyntaxKind::BreakExpression,
            span: Span(start, end),
            children: (0, 0),
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_array(&mut self) -> Result<(), ParseError> {
        todo!()
    }

    fn parse_return(&mut self) -> Result<(), ParseError> {
        todo!()
    }

    fn parse_path_expression(&mut self) -> Result<(), ParseError> {
        info!("Parsing path expression");

        let span = self.current_token.span;

        self.parse_path()?;

        let path_id = self.syntax_tree.last_node_id();
        let node = SyntaxNode {
            kind: SyntaxKind::PathExpression,
            span,
            children: (path_id.0, 0),
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_list(&mut self) -> Result<(), ParseError> {
        todo!()
    }

    fn parse_semicolon(&mut self) -> Result<(), ParseError> {
        let start = self.current_token.span.0;

        self.advance()?;

        let end = self.previous_token.span.1;
        let Some(last_node) = self.syntax_tree.last_node() else {
            return Err(ParseError::UnexpectedToken {
                actual: self.current_token.kind,
                position: self.current_position(),
            });
        };
        let is_optional = last_node.kind.has_block();

        let node = if is_optional {
            SyntaxNode {
                kind: SyntaxKind::SemicolonStatement,
                span: Span(start, end),
                children: (is_optional as u32, 0),
            }
        } else {
            let span = Span(last_node.span.0, end);
            let expression_id = self.syntax_tree.last_node_id();

            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                span,
                children: (expression_id.0, 0),
            }
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_path(&mut self) -> Result<(), ParseError> {
        info!("Parsing path");

        let first_identifier_id = if self.current_token.kind == TokenKind::Identifier {
            let identifier_span = self.current_token.span;

            self.advance()?;

            let segment_node = SyntaxNode {
                kind: SyntaxKind::PathSegment,
                span: identifier_span,
                children: (0, 0),
            };

            self.syntax_tree.push_node(segment_node)
        } else {
            self.syntax_tree.last_node_id()
        };
        let first_identifier_node =
            *self
                .syntax_tree
                .get_node(first_identifier_id)
                .ok_or(ParseError::MissingNode {
                    id: first_identifier_id,
                })?;
        let start = first_identifier_node.span.0;

        let mut children = Self::new_child_buffer();

        children.push(first_identifier_id);

        while self.allow(TokenKind::DoubleColon)? {
            let identifier_span = self.current_token.span;

            self.expect(TokenKind::Identifier)?;

            let segment_node = SyntaxNode {
                kind: SyntaxKind::PathSegment,
                span: identifier_span,
                children: (0, 0),
            };
            let segment_id = self.syntax_tree.push_node(segment_node);

            children.push(segment_id);
        }

        let end = self.previous_token.span.1;
        let node = SyntaxNode {
            kind: SyntaxKind::Path,
            span: Span(start, end),
            children: self.syntax_tree.add_children(&children),
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_str(&mut self) -> Result<(), ParseError> {
        todo!()
    }
}
