mod error;
mod parse_rule;

#[cfg(test)]
mod tests;

pub use error::ParseError;

use std::mem::{replace, take};

use lexical_core::{ParseFloatOptions, format::RUST_LITERAL, parse_with_options};
use smallvec::SmallVec;
use tracing::{Level, debug, error, info, span, warn};

use crate::{
    Lexer, Resolver, Span, Token, Type,
    dust_error::DustError,
    parser::parse_rule::{ParseRule, Precedence},
    resolver::{Declaration, DeclarationKind, ScopeId, TypeId},
    syntax_tree::{SyntaxId, SyntaxKind, SyntaxNode, SyntaxTree},
};

pub fn parse_main(source: &'_ str) -> (SyntaxTree, Option<DustError<'_>>) {
    let parser = Parser::new();
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse_once(source, ScopeId::MAIN);
    let dust_error = if errors.is_empty() {
        None
    } else {
        Some(DustError::parse(errors, source))
    };

    (syntax_tree, dust_error)
}

pub struct ParseResult {
    pub syntax_tree: SyntaxTree,
    pub resolver: Resolver,
    pub errors: Vec<ParseError>,
}

pub struct Parser<'src> {
    lexer: Lexer<'src>,
    resolver: Resolver,

    syntax_tree: SyntaxTree,

    current_token: Token,
    current_position: Span,
    previous_token: Token,
    previous_position: Span,

    current_scope: ScopeId,

    errors: Vec<ParseError>,
}

impl<'src> Parser<'src> {
    pub fn new() -> Self {
        Self {
            lexer: Lexer::new(),
            resolver: Resolver::new(),
            syntax_tree: SyntaxTree::default(),
            current_token: Token::Eof,
            current_position: Span::default(),
            previous_token: Token::Eof,
            previous_position: Span::default(),
            current_scope: ScopeId::MAIN,
            errors: Vec::new(),
        }
    }

    /// Parses a source string as a complete file, returning the syntax tree and any parse errors.
    /// The parser is consumed and cannot be reused.
    pub fn parse_once(mut self, source: &'src str, scope: ScopeId) -> ParseResult {
        let (token, span) = self.lexer.initialize(source, 0);

        self.current_token = token;
        self.current_position = span;
        self.current_scope = scope;

        if scope == ScopeId::MAIN {
            self.parse_main_function_item();
        } else {
            self.parse_module_item();
        }

        ParseResult {
            syntax_tree: self.syntax_tree,
            resolver: self.resolver,
            errors: self.errors,
        }
    }

    /// Parses a source string as a complete file, returning the syntax tree and any parse errors.
    /// Afterwards, the parser is reset and can be reused.
    pub fn parse(&mut self, source: &'src str, scope: ScopeId) -> ParseResult {
        let (token, span) = self.lexer.initialize(source, 0);

        self.current_token = token;
        self.current_position = span;
        self.current_scope = scope;

        if scope == ScopeId::MAIN {
            self.parse_main_function_item();
        } else {
            self.parse_module_item();
        }

        let syntax_tree = take(&mut self.syntax_tree);
        let resolver = take(&mut self.resolver);
        let errors = take(&mut self.errors);

        ParseResult {
            syntax_tree,
            resolver,
            errors,
        }
    }

    /// Parses a source string, allowing it to be a subtree of a larger syntax tree. Afterwards, the
    /// parser is reset and can be reused.
    pub fn parse_subtree(
        &mut self,
        source: &'src str,
        offset: usize,
        scope: ScopeId,
    ) -> ParseResult {
        let (token, span) = self.lexer.initialize(source, offset);

        self.current_token = token;
        self.current_position = span;
        self.current_scope = scope;

        let placeholder_node = SyntaxNode {
            kind: SyntaxKind::MainFunctionItem,
            position: Span::default(),
            payload: (0, 0),
            r#type: TypeId::NONE,
        };

        self.syntax_tree.push_node(placeholder_node);

        while self.current_token != Token::Eof {
            let _ = self
                .pratt(Precedence::None)
                .map_err(|error| self.recover(error));
        }

        self.syntax_tree.nodes.swap_remove(0);

        let syntax_tree = take(&mut self.syntax_tree);
        let resolver = take(&mut self.resolver);
        let errors = take(&mut self.errors);

        ParseResult {
            syntax_tree,
            resolver,
            errors,
        }
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
        let (next_token, next_position) = self.lexer.next_token();

        if next_token.is_whitespace() {
            return self.parse_trivia();
        }

        self.previous_token = replace(&mut self.current_token, next_token);
        self.previous_position = replace(&mut self.current_position, next_position);

        info!("{} at {}", self.current_token, self.current_position);

        Ok(())
    }

    fn recover(&mut self, error: ParseError) {
        error!("{error:?}");

        self.errors.push(error);

        if self.previous_token == Token::Semicolon {
            warn!("Error recovery is continuing after a semicolon");

            return;
        }

        while !matches!(
            self.current_token,
            Token::Semicolon | Token::RightCurlyBrace | Token::Eof
        ) {
            let _ = self.advance().map_err(|error| self.recover(error));
        }

        warn!(
            "Error recovery has skipped to {} at {}",
            self.current_token, self.current_position
        );

        if self.current_token == Token::Semicolon {
            let _ = self.advance().map_err(|error| self.recover(error));
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

    fn parse_trivia(&mut self) -> Result<(), ParseError> {
        let start = self.current_position.0;

        self.advance()?;

        let end = self.previous_position.1;
        let node = SyntaxNode {
            kind: SyntaxKind::Trivia,
            position: Span(start, end),
            payload: (0, 0),
            r#type: TypeId::NONE,
        };

        self.syntax_tree.trivia.push(node);

        Ok(())
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

    fn _parse_statement(&mut self) -> Result<(), ParseError> {
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
            r#type: TypeId::NONE,
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

        let last_node_type = self
            .syntax_tree
            .last_node()
            .map(|node| node.r#type)
            .unwrap_or(TypeId::NONE);

        let first_child = self.syntax_tree.children.len() as u32;
        let child_count = children.len() as u32;

        self.syntax_tree.nodes[0] = SyntaxNode {
            kind: SyntaxKind::MainFunctionItem,
            position: Span(0, self.current_position.1),
            payload: (first_child, child_count),
            r#type: last_node_type,
        };

        self.syntax_tree.children.extend(children);
    }

    pub fn parse_module_item(&mut self) {
        let span = span!(Level::INFO, "Parsing Module");
        let _enter = span.enter();

        let end_token = if self.current_token == Token::Mod {
            let _ = self.advance().map_err(|error| self.recover(error));
            let _ = self
                .expect(Token::LeftCurlyBrace)
                .map_err(|error| self.recover(error));

            Token::RightCurlyBrace
        } else {
            Token::Eof
        };

        let node_index = self.syntax_tree.nodes.len();
        let placeholder_node = SyntaxNode {
            kind: SyntaxKind::ModuleItem,
            position: Span::default(),
            payload: (0, 0),
            r#type: TypeId::NONE,
        };

        self.syntax_tree.push_node(placeholder_node);

        let mut children = Self::new_child_buffer();

        while self.current_token != end_token {
            let _ = self.parse_item().map_err(|error| self.recover(error));
            children.push(self.syntax_tree.last_node_id());
        }

        let first_child = self.syntax_tree.children.len() as u32;
        let child_count = children.len() as u32;

        self.syntax_tree.nodes[node_index] = SyntaxNode {
            kind: SyntaxKind::ModuleItem,
            position: Span(0, self.current_position.1),
            payload: (first_child, child_count),
            r#type: TypeId::NONE,
        };

        self.syntax_tree.children.extend(children);
    }

    fn parse_function_statement_or_expression(&mut self) -> Result<(), ParseError> {
        let start = self.current_position.0;

        self.advance()?;

        match self.current_token {
            Token::Identifier => {
                let identifier_position = self.current_position;
                let identifier_text = self.current_source();

                self.advance()?;
                self.parse_function_expression()?;

                let end = self.previous_position.1;
                let declaration = Declaration {
                    kind: DeclarationKind::Function,
                    scope: self.current_scope,
                    r#type: TypeId::NONE,
                    identifier_position,
                };
                let declaration_id = self.resolver.add_declaration(identifier_text, declaration);
                let function_expression_id = self.syntax_tree.last_node_id();
                let node = SyntaxNode {
                    kind: SyntaxKind::FunctionStatement,
                    position: Span(start, end),
                    payload: (declaration_id.0, function_expression_id.0),
                    r#type: TypeId::NONE,
                };

                self.syntax_tree.push_node(node);

                Ok(())
            }
            Token::LeftParenthesis => {
                self.parse_function_expression()?;

                Ok(())
            }
            _ => Err(ParseError::ExpectedMultipleTokens {
                expected: &[Token::Identifier, Token::LeftParenthesis],
                actual: self.current_token,
                position: self.current_position,
            }),
        }
    }

    fn parse_function_expression(&mut self) -> Result<(), ParseError> {
        todo!()
    }

    fn parse_type(&mut self) -> Result<TypeId, ParseError> {
        let start = self.current_position.0;

        let (node_kind, r#type) = match self.current_token {
            Token::Bool => (SyntaxKind::BooleanType, TypeId::BOOLEAN),
            Token::Byte => (SyntaxKind::ByteType, TypeId::BYTE),
            Token::Char => (SyntaxKind::CharacterType, TypeId::CHARACTER),
            Token::Float => (SyntaxKind::FloatType, TypeId::FLOAT),
            Token::Int => (SyntaxKind::IntegerType, TypeId::INTEGER),
            Token::Str => (SyntaxKind::StringType, TypeId::STRING),
            Token::Identifier => {
                todo!()
            }
            _ => {
                return Err(ParseError::ExpectedMultipleTokens {
                    expected: &[
                        Token::Bool,
                        Token::Byte,
                        Token::Char,
                        Token::Float,
                        Token::Int,
                        Token::Str,
                        Token::Identifier,
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
            r#type: TypeId::NONE,
        };

        self.syntax_tree.push_node(node);

        Ok(r#type)
    }

    fn parse_let_statement(&mut self) -> Result<(), ParseError> {
        let start = self.current_position.0;

        self.advance()?;

        let kind = if self.allow(Token::Mut)? {
            SyntaxKind::LetMutStatement
        } else {
            SyntaxKind::LetStatement
        };

        let identifier_text = if self.current_token == Token::Identifier {
            let text = self.current_source();

            self.advance()?;

            text
        } else {
            return Err(ParseError::ExpectedToken {
                expected: Token::Identifier,
                actual: self.current_token,
                position: self.current_position,
            });
        };

        if self.allow(Token::Colon)? {
            self.parse_type()?;
        }

        self.expect(Token::Equal)?;
        self.parse_expression()?;
        self.allow(Token::Semicolon)?;

        let expression_id = self.syntax_tree.last_node_id();
        let expression_type = self
            .syntax_tree
            .get_node(expression_id)
            .map(|node| node.r#type)
            .ok_or(ParseError::MissingNode { id: expression_id })?;
        let end = self.previous_position.1;
        let declaration = Declaration {
            kind: DeclarationKind::Local,
            scope: self.current_scope,
            r#type: expression_type,
            identifier_position: self.current_position,
        };
        let declaration_id = self.resolver.add_declaration(identifier_text, declaration);
        let node = SyntaxNode {
            kind,
            position: Span(start, end),
            payload: (declaration_id.0, expression_id.0),
            r#type: expression_type,
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_boolean_expression(&mut self) -> Result<(), ParseError> {
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
            position: self.current_position,
            payload: (boolean_payload, 0),
            r#type: TypeId::BOOLEAN,
        };

        self.advance()?;
        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_byte_expression(&mut self) -> Result<(), ParseError> {
        let position = self.current_position;
        let byte_text_utf8 = &self.current_source().as_bytes()[2..]; // Skip the "0x" prefix

        self.advance()?;

        let byte_payload = u8::from_ascii_radix(byte_text_utf8, 16).unwrap_or_default() as u32;
        let node = SyntaxNode {
            kind: SyntaxKind::ByteExpression,
            position,
            payload: (byte_payload, 0),
            r#type: TypeId::BYTE,
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_character_expression(&mut self) -> Result<(), ParseError> {
        let start = self.current_position.0;
        let character_text = self.current_source();

        self.advance()?;

        let end = self.previous_position.1;
        let character_payload = character_text.chars().nth(1).unwrap_or_default() as u32;
        let node = SyntaxNode {
            kind: SyntaxKind::CharacterExpression,
            position: Span(start, end),
            payload: (character_payload, 0),
            r#type: TypeId::CHARACTER,
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
        let payload = SyntaxNode::encode_float(float);
        let node = SyntaxNode {
            kind: SyntaxKind::FloatExpression,
            position: Span(start, end),
            payload,
            r#type: TypeId::FLOAT,
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
        let (left_payload, right_payload) = SyntaxNode::encode_integer(integer);
        let node = SyntaxNode {
            kind: SyntaxKind::IntegerExpression,
            position: Span(start, end),
            payload: (left_payload, right_payload),
            r#type: TypeId::INTEGER,
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
        let position = self.current_position;
        let string_text = {
            let token_text = self.current_source();

            &token_text[1..token_text.len() - 1]
        };
        let payload = self.resolver.constants.push_str_to_string_pool(string_text);

        self.advance()?;

        let node = SyntaxNode {
            kind: SyntaxKind::StringExpression,
            position,
            payload,
            r#type: TypeId::STRING,
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
        let left_node = *self
            .syntax_tree
            .get_node(left)
            .ok_or(ParseError::MissingNode { id: left })?;
        let start = left_node.position.0;
        let operator = self.current_token;
        let node_kind = match operator {
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
                    actual: operator,
                    position: self.current_position,
                });
            }
        };
        let operator_precedence = ParseRule::from(operator).precedence;

        self.advance()?;
        self.parse_sub_expression(operator_precedence)?;

        let right = self.syntax_tree.last_node_id();
        let right_node = self
            .syntax_tree
            .get_node(right)
            .ok_or(ParseError::MissingNode { id: right })?;
        let end = self.previous_position.1;
        let r#type = match operator {
            Token::Plus => match (left_node.r#type, right_node.r#type) {
                (TypeId::BYTE, TypeId::BYTE) => TypeId::BYTE,
                (TypeId::INTEGER, TypeId::INTEGER) => TypeId::INTEGER,
                (TypeId::FLOAT, TypeId::FLOAT) => TypeId::FLOAT,
                (TypeId::CHARACTER, TypeId::CHARACTER) => TypeId::STRING,
                (TypeId::CHARACTER, TypeId::STRING) => TypeId::STRING,
                (TypeId::STRING, TypeId::CHARACTER) => TypeId::STRING,
                (TypeId::STRING, TypeId::STRING) => TypeId::STRING,
                _ => {
                    let left_type = self
                        .resolver
                        .resolve_type(left_node.r#type)
                        .unwrap_or(Type::None);
                    let right_type = self
                        .resolver
                        .resolve_type(right_node.r#type)
                        .unwrap_or(Type::None);

                    return Err(ParseError::AdditionTypeMismatch {
                        left_type,
                        left_position: left_node.position,
                        right_type,
                        right_position: right_node.position,
                        position: Span(start, end),
                    });
                }
            },
            _ => match (left_node.r#type, right_node.r#type) {
                (TypeId::BYTE, TypeId::BYTE) => TypeId::BYTE,
                (TypeId::INTEGER, TypeId::INTEGER) => TypeId::INTEGER,
                (TypeId::FLOAT, TypeId::FLOAT) => TypeId::FLOAT,
                _ => {
                    let left_type = self
                        .resolver
                        .resolve_type(left_node.r#type)
                        .unwrap_or(Type::None);
                    let right_type = self
                        .resolver
                        .resolve_type(right_node.r#type)
                        .unwrap_or(Type::None);

                    return Err(ParseError::OperandTypeMismatch {
                        operator,
                        left_type,
                        left_position: left_node.position,
                        right_type,
                        right_position: right_node.position,
                        position: Span(start, end),
                    });
                }
            },
        };

        let node = SyntaxNode {
            kind: node_kind,
            position: Span(start, end),
            payload: (left.0, right.0),
            r#type,
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
        let r#type = self
            .syntax_tree
            .get_node(expression_id)
            .map(|node| node.r#type)
            .ok_or(ParseError::MissingNode { id: expression_id })?;
        let node = SyntaxNode {
            kind: SyntaxKind::GroupedExpression,
            position: Span(start, end),
            payload: (expression_id.0, 0),
            r#type,
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

    fn parse_path_expression(&mut self) -> Result<(), ParseError> {
        let position = self.current_position;
        let identifier_text = self.current_source();

        self.advance()?;

        let Some((declaration_id, declaration)) = self
            .resolver
            .get_declaration_full(identifier_text, self.current_scope)
        else {
            return Err(ParseError::UndeclaredVariable {
                identifier: identifier_text.to_string(),
                position,
            });
        };
        let node = SyntaxNode {
            kind: SyntaxKind::PathExpression,
            position,
            payload: (declaration_id.0, 0),
            r#type: declaration.r#type,
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
                r#type: TypeId::NONE,
            }
        } else {
            let span = Span(last_node.position.0, end);
            let expression_id = self.syntax_tree.last_node_id();

            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                position: span,
                payload: (expression_id.0, 0),
                r#type: last_node.r#type,
            }
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_str(&mut self) -> Result<(), ParseError> {
        todo!()
    }
}

impl<'src> Default for Parser<'src> {
    fn default() -> Self {
        Self::new()
    }
}
