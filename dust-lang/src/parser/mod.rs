mod parse_rule;

use std::{
    fmt::{self, Display, Formatter},
    mem::replace,
};

use lexical_core::{ParseFloatOptions, format::RUST_LITERAL, parse_with_options};
use smallvec::SmallVec;
use tracing::{Level, debug, error, info, span, warn};

use crate::{
    LexError, Lexer, Span, Token,
    dust_error::{AnnotatedError, DustError, ErrorMessage},
    parser::parse_rule::{ParseRule, Precedence},
    syntax_tree::{SyntaxId, SyntaxKind, SyntaxNode, SyntaxTree},
};

pub fn parse(source: &'_ str, is_main: bool) -> (SyntaxTree, Option<DustError<'_>>) {
    let lexer = Lexer::new(source);
    let parser = Parser::new(lexer);
    let (syntax_tree, errors) = parser.parse(is_main);
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
    current_position: Span,
    previous_token: Token,
    previous_position: Span,

    errors: Vec<ParseError>,
}

impl<'src> Parser<'src> {
    pub fn new(mut lexer: Lexer<'src>) -> Self {
        let mut errors = Vec::new();

        let (current_token, current_position) = lexer.next_token().unwrap_or_else(|error| {
            errors.push(ParseError::LexError { error });

            (Token::Eof, Span::default())
        });

        Self {
            lexer,
            syntax_tree: SyntaxTree::default(),
            current_token,
            current_position,
            previous_token: Token::Eof,
            previous_position: Span::default(),
            errors,
        }
    }

    pub fn parse(mut self, is_main: bool) -> (SyntaxTree, Vec<ParseError>) {
        if is_main {
            self.parse_main_function_item();
        } else {
            self.parse_module_item();
        }

        (self.syntax_tree, self.errors)
    }

    fn new_child_buffer() -> SmallVec<[SyntaxId; 4]> {
        SmallVec::<[SyntaxId; 4]>::new()
    }

    fn pratt(&mut self, precedence: Precedence) -> Result<(), ParseError> {
        let prefix_rule = ParseRule::from(self.current_token);

        if let Some(prefix_parser) = prefix_rule.prefix {
            debug!(
                "{} at {} is prefix",
                self.current_token, self.current_position,
            );

            prefix_parser(self)?;
        }

        let mut infix_rule = ParseRule::from(self.current_token);

        while precedence <= infix_rule.precedence
            && let Some(infix_parser) = infix_rule.infix
        {
            debug!(
                "{} at {} as infix {}",
                self.current_token, self.current_position, infix_rule.precedence,
            );

            infix_parser(self)?;

            infix_rule = ParseRule::from(self.current_token);
        }

        Ok(())
    }

    fn advance(&mut self) -> Result<(), ParseError> {
        let (next_token, next_position) = self
            .lexer
            .next_token()
            .map_err(|error| ParseError::LexError { error })?;

        self.previous_token = replace(&mut self.current_token, next_token);
        self.previous_position = replace(&mut self.current_position, next_position);

        info!("{} at {}", self.current_token, self.current_position);

        Ok(())
    }

    fn recover(&mut self, error: ParseError) {
        error!("{error}");

        self.errors.push(error);

        if self.previous_token == Token::Semicolon {
            warn!("Error recovery is continuing without skipping tokens");

            return;
        }

        while !matches!(
            self.current_token,
            Token::Semicolon | Token::RightCurlyBrace | Token::Eof
        ) {
            self.advance().map_err(|error| self.recover(error));
        }

        warn!(
            "Error recovery has skipped to {} at {}",
            self.current_token, self.current_position
        );

        if self.current_token == Token::Semicolon {
            self.advance().map_err(|error| self.recover(error));
        }
    }

    fn allow(&mut self, allowed: Token) -> Result<bool, ParseError> {
        let allowed = self.current_token == allowed;

        if allowed {
            self.advance()?;
        }

        Ok(allowed)
    }

    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        if self.current_token != expected {
            return Err(ParseError::ExpectedToken {
                expected,
                actual: self.current_token,
                position: self.current_position,
            });
        }

        self.advance()?;

        Ok(())
    }

    fn current_source(&self) -> &'src str {
        &self.lexer.source()[self.current_position.as_usize_range()]
    }

    fn parse_item(&mut self) -> Result<(), ParseError> {
        self.pratt(Precedence::None)?;

        if let Some(node) = self.syntax_tree.last_node()
            && !node.kind.is_item()
        {
            Err(ParseError::ExpectedItem {
                actual: node.kind,
                position: node.position,
            })
        } else {
            Ok(())
        }
    }

    fn parse_statement(&mut self) -> Result<(), ParseError> {
        self.pratt(Precedence::None)?;

        if let Some(node) = self.syntax_tree.last_node()
            && !node.kind.is_statement()
        {
            Err(ParseError::ExpectedStatement {
                actual: node.kind,
                position: node.position,
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
                position: node.position,
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
                position: node.position,
            })
        } else {
            Ok(())
        }
    }

    fn parse_unexpected(&mut self) -> Result<(), ParseError> {
        Err(ParseError::UnexpectedToken {
            actual: self.current_token,
            position: self.current_position,
        })
    }

    pub fn parse_main_function_item(&mut self) {
        let span = span!(Level::INFO, "Parsing Main");
        let _enter = span.enter();

        let placeholder_node = SyntaxNode {
            kind: SyntaxKind::MainFunctionItem,
            position: Span::default(),
            payload: (0, 0),
        };

        self.syntax_tree.push_node(placeholder_node);

        let mut children = Self::new_child_buffer();

        while self.current_token != Token::Eof {
            if let Err(error) = self.pratt(Precedence::None) {
                self.recover(error);
            } else {
                let child_id = self.syntax_tree.last_node_id();

                children.push(child_id);
            }
        }

        if let Some(last_child) = self.syntax_tree.last_node()
            && last_child.kind == SyntaxKind::ExpressionStatement
        {
            children.pop();
        }

        let first_child = self.syntax_tree.children.len() as u32;
        let child_count = children.len() as u32;

        self.syntax_tree.nodes[0] = SyntaxNode {
            kind: SyntaxKind::MainFunctionItem,
            position: Span(0, self.current_position.1),
            payload: (first_child, child_count),
        };

        self.syntax_tree.children.extend(children);
    }

    pub fn parse_module_item(&mut self) {
        let span = span!(Level::INFO, "Parsing Module");
        let _enter = span.enter();

        let end_token = if self.current_token == Token::Mod {
            self.advance().map_err(|error| self.recover(error));
            self.expect(Token::LeftCurlyBrace)
                .map_err(|error| self.recover(error));

            Token::RightCurlyBrace
        } else {
            Token::Eof
        };

        let node_index = self.syntax_tree.nodes.len();
        let placeholder_node = SyntaxNode {
            kind: SyntaxKind::MainFunctionItem,
            position: Span::default(),
            payload: (0, 0),
        };

        self.syntax_tree.push_node(placeholder_node);

        let mut children = Self::new_child_buffer();

        while self.current_token != end_token {
            self.parse_item().map_err(|error| self.recover(error));
            children.push(self.syntax_tree.last_node_id());
        }

        let first_child = self.syntax_tree.children.len() as u32;
        let child_count = children.len() as u32;

        self.syntax_tree.nodes[node_index] = SyntaxNode {
            kind: SyntaxKind::MainFunctionItem,
            position: Span(0, self.current_position.1),
            payload: (first_child, child_count),
        };

        self.syntax_tree.children.extend(children);
    }

    fn parse_function_statement_or_expression(&mut self) -> Result<(), ParseError> {
        let start = self.current_position.0;

        self.advance()?;

        let function_kind = match self.current_token {
            Token::Identifier => SyntaxKind::FunctionStatement,
            Token::LeftParenthesis => SyntaxKind::FunctionExpression,
            _ => {
                return Err(ParseError::ExpectedMultipleTokens {
                    expected: &[Token::Identifier, Token::LeftParenthesis],
                    actual: self.current_token,
                    position: self.current_position,
                });
            }
        };

        self.parse_function_signature()?;

        let function_signature_id = self.syntax_tree.last_node_id();

        if self.current_token != Token::LeftCurlyBrace {
            return Err(ParseError::ExpectedToken {
                expected: Token::LeftCurlyBrace,
                actual: self.current_token,
                position: self.current_position,
            });
        }

        self.parse_block()?;

        let block_id = self.syntax_tree.last_node_id();
        let end = self.previous_position.1;
        let node = SyntaxNode {
            kind: function_kind,
            position: Span(start, end),
            payload: (function_signature_id.0, block_id.0),
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_function_signature(&mut self) -> Result<(), ParseError> {
        let start = self.current_position.0;
        let mut children = Self::new_child_buffer();

        if self.current_token == Token::Identifier {
            self.parse_identifier()?;
            children.push(self.syntax_tree.last_node_id());
        }

        self.expect(Token::LeftParenthesis)?;

        while !self.allow(Token::RightParenthesis)? {
            self.parse_function_parameter()?;
            children.push(self.syntax_tree.last_node_id());
        }

        if self.current_token.is_type() {
            self.parse_type()?;
            children.push(self.syntax_tree.last_node_id());
        }

        let end = self.previous_position.1;
        let first_child = self.syntax_tree.children.len() as u32;
        let child_count = children.len() as u32;
        let node = SyntaxNode {
            kind: SyntaxKind::FunctionSignature,
            position: Span(start, end),
            payload: (first_child, child_count),
        };

        self.syntax_tree.children.extend(children);
        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_function_parameter(&mut self) -> Result<(), ParseError> {
        let start = self.current_position.0;

        if let Token::Identifier = self.current_token {
            self.parse_identifier()?;
        } else {
            return Err(ParseError::ExpectedToken {
                expected: Token::Identifier,
                actual: self.current_token,
                position: self.current_position,
            });
        }

        let identifier_id = self.syntax_tree.last_node_id();

        self.expect(Token::Colon)?;

        if !self.current_token.is_type() {
            return Err(ParseError::ExpectedMultipleTokens {
                expected: &[
                    Token::Bool,
                    Token::ByteValue,
                    Token::CharacterValue,
                    Token::FloatValue,
                    Token::IntegerValue,
                    Token::StringValue,
                ],
                actual: self.current_token,
                position: self.current_position,
            });
        }

        self.parse_type()?;

        let type_id = self.syntax_tree.last_node_id();
        let end = self.previous_position.1;
        let node = SyntaxNode {
            kind: SyntaxKind::FunctionParameter,
            position: Span(start, end),
            payload: (identifier_id.0, type_id.0),
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_type(&mut self) -> Result<(), ParseError> {
        let start = self.current_position.0;

        let node_kind = match self.current_token {
            Token::Bool => SyntaxKind::BooleanType,
            Token::Byte => SyntaxKind::ByteType,
            Token::Char => SyntaxKind::CharacterType,
            Token::Float => SyntaxKind::FloatType,
            Token::Int => SyntaxKind::IntegerType,
            Token::Str => SyntaxKind::StringType,
            Token::Identifier => SyntaxKind::TypePath,
            _ => {
                return Err(ParseError::ExpectedMultipleTokens {
                    expected: &[
                        Token::Bool,
                        Token::Byte,
                        Token::Char,
                        Token::Float,
                        Token::Int,
                        Token::Str,
                    ],
                    actual: self.current_token,
                    position: self.current_position,
                });
            }
        };

        self.advance()?;

        let end = self.previous_position.1;
        let node = SyntaxNode {
            kind: node_kind,
            position: Span(start, end),
            payload: (0, 0),
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_let_statement(&mut self) -> Result<(), ParseError> {
        let start = self.current_position.0;
        let mut children = Self::new_child_buffer();

        self.advance()?;

        let kind = if self.allow(Token::Mut)? {
            SyntaxKind::LetMutStatement
        } else {
            SyntaxKind::LetStatement
        };

        if self.current_token != Token::Identifier {
            return Err(ParseError::ExpectedToken {
                expected: Token::Identifier,
                actual: self.current_token,
                position: self.current_position,
            });
        }

        self.parse_identifier()?;
        children.push(self.syntax_tree.last_node_id());

        if self.allow(Token::Colon)? {
            self.parse_type()?;
            children.push(self.syntax_tree.last_node_id());
        }

        self.expect(Token::Equal)?;
        self.parse_expression()?;
        self.allow(Token::Semicolon)?;
        children.push(self.syntax_tree.last_node_id());

        let end = self.previous_position.1;
        let children_start = self.syntax_tree.children.len() as u32;
        let child_count = children.len() as u32;
        let node = SyntaxNode {
            kind,
            position: Span(start, end),
            payload: (children_start, child_count),
        };

        self.syntax_tree.push_node(node);
        self.syntax_tree.children.extend(children);

        Ok(())
    }

    fn parse_boolean_expression(&mut self) -> Result<(), ParseError> {
        let start = self.current_position.0;

        self.advance()?;

        let end = self.previous_position.1;
        let boolean_payload = match self.current_token {
            Token::TrueValue => true as u32,
            Token::FalseValue => false as u32,
            _ => {
                return Err(ParseError::ExpectedMultipleTokens {
                    expected: &[Token::TrueValue, Token::FalseValue],
                    actual: self.current_token,
                    position: self.current_position,
                });
            }
        };
        let node = SyntaxNode {
            kind: SyntaxKind::BooleanExpression,
            position: Span(start, end),
            payload: (boolean_payload, 0),
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_byte_expression(&mut self) -> Result<(), ParseError> {
        let start = self.current_position.0;

        self.advance()?;

        let end = self.previous_position.1;
        let byte_text_utf8 = &self.current_source().as_bytes()[2..]; // Skip the "0x" prefix
        let byte_payload = u8::from_ascii_radix(byte_text_utf8, 16).unwrap_or_default() as u32;
        let node = SyntaxNode {
            kind: SyntaxKind::ByteExpression,
            position: Span(start, end),
            payload: (byte_payload, 0),
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_character_expression(&mut self) -> Result<(), ParseError> {
        let start = self.current_position.0;
        let character_text = self.current_source();

        self.advance()?;

        let end = self.previous_position.1;
        let character_payload = character_text.chars().next().unwrap_or_default() as u32;
        let node = SyntaxNode {
            kind: SyntaxKind::CharacterExpression,
            position: Span(start, end),
            payload: (character_payload, 0),
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_float_expression(&mut self) -> Result<(), ParseError> {
        let start = self.current_position.0;
        let float_text = self.current_source();

        self.advance()?;

        let end = self.previous_position.1;
        let float = parse_with_options::<f64, RUST_LITERAL>(
            float_text.as_bytes(),
            &ParseFloatOptions::default(),
        )
        .unwrap_or_default();
        let float_bytes = float.to_le_bytes();
        let left_payload = u32::from_le_bytes([
            float_bytes[0],
            float_bytes[1],
            float_bytes[2],
            float_bytes[3],
        ]);
        let right_payload = u32::from_le_bytes([
            float_bytes[4],
            float_bytes[5],
            float_bytes[6],
            float_bytes[7],
        ]);
        let node = SyntaxNode {
            kind: SyntaxKind::FloatExpression,
            position: Span(start, end),
            payload: (left_payload, right_payload),
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_integer_expression(&mut self) -> Result<(), ParseError> {
        let start = self.current_position.0;
        let integer_text = self.current_source();

        self.advance()?;

        let end = self.previous_position.1;
        let integer = Self::parse_integer(integer_text);
        let integer_bytes = integer.to_le_bytes();
        let left_payload = u32::from_le_bytes([
            integer_bytes[0],
            integer_bytes[1],
            integer_bytes[2],
            integer_bytes[3],
        ]);
        let right_payload = u32::from_le_bytes([
            integer_bytes[4],
            integer_bytes[5],
            integer_bytes[6],
            integer_bytes[7],
        ]);
        let node = SyntaxNode {
            kind: SyntaxKind::IntegerExpression,
            position: Span(start, end),
            payload: (left_payload, right_payload),
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_integer(text: &str) -> i64 {
        let mut integer = 0_i64;
        let mut chars = text.chars().peekable();

        let is_positive = if chars.peek() == Some(&'-') {
            chars.next();

            false
        } else {
            true
        };

        let mut digit_place = 0;

        for character in chars.rev() {
            let Some(digit) = character.to_digit(10) else {
                continue;
            };

            let place_value = 10_i64.pow(digit_place);
            let digit_value = digit as i64 * place_value;
            digit_place += 1;

            integer = integer.saturating_add(digit_value);
        }

        if is_positive { integer } else { -integer }
    }

    fn parse_string_expression(&mut self) -> Result<(), ParseError> {
        let start = self.current_position.0;

        self.advance()?;

        let end = self.previous_position.1;
        let node = SyntaxNode {
            kind: SyntaxKind::StringExpression,
            position: Span(start, end),
            payload: (0, 0),
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_unary(&mut self) -> Result<(), ParseError> {
        todo!()
    }

    fn parse_comparison_binary(&mut self) -> Result<(), ParseError> {
        todo!()
    }

    fn parse_math_binary(&mut self) -> Result<(), ParseError> {
        let left = self.syntax_tree.last_node_id();
        let left_node = self.syntax_tree.get_node(left);
        let start = left_node.map(|node| node.position).unwrap_or_default().0;
        let node_kind = match self.current_token {
            Token::Plus => SyntaxKind::AdditionExpression,
            Token::Minus => SyntaxKind::SubtractionExpression,
            Token::Asterisk => SyntaxKind::MultiplicationExpression,
            Token::Slash => SyntaxKind::DivisionExpression,
            Token::Percent => SyntaxKind::ModuloExpression,
            _ => {
                return Err(ParseError::ExpectedMultipleTokens {
                    expected: &[
                        Token::Plus,
                        Token::Minus,
                        Token::Asterisk,
                        Token::Slash,
                        Token::Percent,
                    ],
                    actual: self.current_token,
                    position: self.current_position,
                });
            }
        };
        let operator_precedence = ParseRule::from(self.current_token).precedence;

        self.advance()?;
        self.parse_sub_expression(operator_precedence)?;

        let right = self.syntax_tree.last_node_id();
        let end = self.current_position.0;
        let node = SyntaxNode {
            kind: node_kind,
            position: Span(start, end),
            payload: (left.0, right.0),
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_logical_binary(&mut self) -> Result<(), ParseError> {
        todo!()
    }

    fn parse_call(&mut self) -> Result<(), ParseError> {
        todo!()
    }

    fn parse_grouped(&mut self) -> Result<(), ParseError> {
        let start = self.current_position.0;

        self.advance()?;
        self.parse_expression()?;
        self.expect(Token::RightParenthesis)?;

        let end = self.previous_position.1;
        let expression_id = self.syntax_tree.last_node_id();
        let node = SyntaxNode {
            kind: SyntaxKind::GroupedExpression,
            position: Span(start, end),
            payload: (expression_id.0, 0),
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_if(&mut self) -> Result<(), ParseError> {
        todo!()
    }

    fn parse_while(&mut self) -> Result<(), ParseError> {
        todo!()
    }

    fn parse_block(&mut self) -> Result<(), ParseError> {
        todo!()
    }

    fn parse_array(&mut self) -> Result<(), ParseError> {
        todo!()
    }

    fn parse_return(&mut self) -> Result<(), ParseError> {
        todo!()
    }

    fn parse_identifier(&mut self) -> Result<(), ParseError> {
        let start = self.current_position.0;

        self.advance()?;

        let end = self.previous_position.1;
        let node = SyntaxNode {
            kind: SyntaxKind::PathExpression,
            position: Span(start, end),
            payload: (0, 0),
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_use(&mut self) -> Result<(), ParseError> {
        todo!()
    }

    fn parse_list(&mut self) -> Result<(), ParseError> {
        todo!()
    }

    fn parse_mod(&mut self) -> Result<(), ParseError> {
        todo!()
    }

    fn parse_semicolon(&mut self) -> Result<(), ParseError> {
        let start = self.current_position.0;

        self.advance()?;

        let end = self.previous_position.1;
        let Some(last_node) = self.syntax_tree.last_node() else {
            return Err(ParseError::UnexpectedToken {
                actual: self.current_token,
                position: self.current_position,
            });
        };
        let is_optional = last_node.kind.has_block();

        let node = if is_optional {
            SyntaxNode {
                kind: SyntaxKind::SemicolonStatement,
                position: Span(start, end),
                payload: (is_optional as u32, 0),
            }
        } else {
            let span = Span(last_node.position.0, end);
            let expression_id = self.syntax_tree.last_node_id();

            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                position: span,
                payload: (expression_id.0, 0),
            }
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_str(&mut self) -> Result<(), ParseError> {
        todo!()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError {
    ExpectedItem {
        actual: SyntaxKind,
        position: Span,
    },
    ExpectedStatement {
        actual: SyntaxKind,
        position: Span,
    },
    ExpectedExpression {
        actual: SyntaxKind,
        position: Span,
    },

    ExpectedToken {
        actual: Token,
        expected: Token,
        position: Span,
    },
    ExpectedMultipleTokens {
        actual: Token,
        expected: &'static [Token],
        position: Span,
    },
    UnexpectedToken {
        actual: Token,
        position: Span,
    },

    LexError {
        error: LexError,
    },
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ParseError::ExpectedItem { actual, position } => {
                write!(f, "Expected item, found {actual} at {position}")
            }
            ParseError::ExpectedStatement { actual, position } => {
                write!(f, "Expected statement, found {actual} at {position}")
            }
            ParseError::ExpectedExpression {
                actual: found,
                position,
            } => {
                write!(f, "Expected expression, found {found} at {position}")
            }
            ParseError::ExpectedToken {
                actual,
                expected,
                position,
            } => {
                write!(
                    f,
                    "Found '{expected}' at {position} but expected '{actual}'",
                )
            }
            ParseError::ExpectedMultipleTokens {
                expected,
                actual,
                position,
            } => {
                write!(
                    f,
                    "Found \"{actual}\" at {position} but expected one of the following: ",
                )?;

                for (i, expected_token) in expected.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    } else if i == expected.len() - 1 {
                        write!(f, "or ")?;
                    }

                    write!(f, "\"{expected_token}\"")?;
                }

                write!(f, ".")
            }
            ParseError::UnexpectedToken {
                actual: found,
                position,
            } => {
                write!(f, "Unexpected token {found} at {position}")
            }
            ParseError::LexError { error } => write!(f, "{error}"),
        }
    }
}

impl AnnotatedError for ParseError {
    fn annotated_error(&self) -> ErrorMessage {
        let title = "Parsing Error";

        match self {
            ParseError::ExpectedItem { actual, position } => ErrorMessage {
                title,
                description: "Expected an item",
                detail_snippets: vec![(
                    format!("This is a {actual}, which cannot be used here."),
                    *position,
                )],
                help_snippet: None,
            },
            ParseError::ExpectedStatement { actual, position } => ErrorMessage {
                title,
                description: "Expected a statement",
                detail_snippets: vec![(
                    format!("This is a {actual}, which cannot be used here."),
                    *position,
                )],
                help_snippet: None,
            },
            ParseError::ExpectedExpression {
                actual: found,
                position,
            } => ErrorMessage {
                title,
                description: "Expected an expression",
                detail_snippets: vec![(
                    format!("This is a {found}, which cannot be used here."),
                    *position,
                )],
                help_snippet: None,
            },
            ParseError::ExpectedToken { position, .. } => ErrorMessage {
                title,
                description: "Expected a specific token",
                detail_snippets: vec![(self.to_string(), *position)],
                help_snippet: None,
            },
            ParseError::ExpectedMultipleTokens { position, .. } => ErrorMessage {
                title: "Expected Multiple Tokens",
                description: "Expected one of several tokens",
                detail_snippets: vec![(self.to_string(), *position)],
                help_snippet: None,
            },
            ParseError::UnexpectedToken { position, .. } => ErrorMessage {
                title: "Unexpected Token",
                description: "Unexpected token",
                detail_snippets: vec![("Found here".to_string(), *position)],
                help_snippet: None,
            },
            ParseError::LexError { error } => error.annotated_error(),
        }
    }
}
