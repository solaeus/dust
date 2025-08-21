mod parse_rule;

use std::{
    fmt::{self, Display, Formatter},
    mem::replace,
};

use tracing::{Level, debug, error, info, span, warn};

use crate::{
    LexError, Lexer, Span, Token, Value,
    dust_error::{AnnotatedError, DustError, ErrorMessage},
    parser::parse_rule::{ParseRule, Precedence},
    syntax_tree::{Local, Scope, SyntaxKind, SyntaxNode, SyntaxTree},
};

pub fn parse(source: &'_ str) -> (SyntaxTree, Option<DustError<'_>>) {
    let lexer = Lexer::new(source);
    let parser = Parser::new(lexer);
    let (syntax_tree, errors) = parser.parse();
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

    current_scope: Scope,

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
            current_scope: Scope::default(),
            errors,
        }
    }

    pub fn parse(mut self) -> (SyntaxTree, Vec<ParseError>) {
        let span = span!(Level::INFO, "Parsing");
        let _enter = span.enter();

        let placeholder_node = SyntaxNode {
            kind: SyntaxKind::MainFunctionStatement,
            span: Span::default(),
            child: 0,
            payload: 0,
        };

        self.syntax_tree.push_node(placeholder_node);

        let mut children = Vec::new();

        while self.current_token != Token::Eof {
            if let Err(error) = self.pratt(Precedence::None) {
                self.recover(error);
            } else {
                let child_index = self.syntax_tree.node_count() - 1;

                children.push(child_index);
            }
        }

        if let Some(last_child) = self.syntax_tree.last_node()
            && last_child.kind == SyntaxKind::ExpressionStatement
        {
            let expression_index = last_child.child;

            if let Some(index_in_children) = children
                .iter()
                .rposition(|child| *child == expression_index)
            {
                children.remove(index_in_children);
            }
        }

        let first_child = self.syntax_tree.children.len();

        self.syntax_tree.nodes[0] = SyntaxNode {
            kind: SyntaxKind::MainFunctionStatement,
            span: Span(0, self.current_position.1),
            child: first_child as u32,
            payload: children.len() as u32,
        };

        self.syntax_tree.children.extend(children);

        (self.syntax_tree, self.errors)
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

        while !matches!(self.current_token, Token::Semicolon | Token::Eof) {
            if let Err(err) = self.advance() {
                error!("{err}");
            }
        }

        warn!(
            "Error recovery has skipped to {} at {}",
            self.current_token, self.current_position
        );

        if self.current_token == Token::Semicolon {
            let _ = self.advance();
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

    fn parse_expression(&mut self) -> Result<(), ParseError> {
        self.pratt(Precedence::None)?;

        match self.syntax_tree.last_node() {
            Some(node) if !node.kind.is_expression() => Err(ParseError::ExpectedExpression {
                actual: node.kind,
                position: node.span,
            }),
            None => Err(ParseError::UnexpectedToken {
                actual: self.current_token,
                position: self.current_position,
            }),
            _ => Ok(()),
        }
    }

    fn parse_sub_expression(&mut self, precedence: Precedence) -> Result<(), ParseError> {
        self.pratt(precedence.increment())?;

        match self.syntax_tree.last_node() {
            Some(node) if !node.kind.is_expression() => Err(ParseError::ExpectedExpression {
                actual: node.kind,
                position: self.current_position,
            }),
            None => Err(ParseError::UnexpectedToken {
                actual: self.current_token,
                position: self.current_position,
            }),
            _ => Ok(()),
        }
    }

    fn parse_unexpected(&mut self) -> Result<(), ParseError> {
        Err(ParseError::UnexpectedToken {
            actual: self.current_token,
            position: self.current_position,
        })
    }

    fn parse_boolean(&mut self) -> Result<(), ParseError> {
        todo!()
    }

    fn parse_byte(&mut self) -> Result<(), ParseError> {
        todo!()
    }

    fn parse_character(&mut self) -> Result<(), ParseError> {
        todo!()
    }

    fn parse_float(&mut self) -> Result<(), ParseError> {
        todo!()
    }

    fn parse_integer_expression(&mut self) -> Result<(), ParseError> {
        let start = self.current_position.0;
        let integer_text = self.current_source();

        self.advance()?;

        let end = self.previous_position.1;
        let integer = self.parse_integer(integer_text);
        let integer_index = self.syntax_tree.push_constant(Value::integer(integer));
        let node = SyntaxNode {
            kind: SyntaxKind::IntegerExpression,
            span: Span(start, end),
            child: 0,
            payload: integer_index,
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_integer(&mut self, text: &str) -> i64 {
        let mut integer = 0_i64;
        let mut chars = text.chars().peekable();

        let is_positive = if chars.peek() == Some(&'-') {
            chars.next();

            false
        } else {
            true
        };

        for (index, digit) in chars.enumerate() {
            let Some(digit) = digit.to_digit(10) else {
                continue;
            };
            let digit_place = text.len() - index - 1;
            let place_value = 10_i64.pow(digit_place as u32);
            let digit_value = digit as i64 * place_value;

            integer += digit_value;
        }

        if is_positive { integer } else { -integer }
    }

    fn parse_string_expression(&mut self) -> Result<(), ParseError> {
        let start = self.current_position.0;
        let text = self.current_source();

        self.advance()?;

        let end = self.previous_position.1;
        let string_index = self.syntax_tree.push_constant(Value::string(text));
        let node = SyntaxNode {
            kind: SyntaxKind::StringExpression,
            span: Span(start, end),
            child: 0,
            payload: string_index,
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_let_statement(&mut self) -> Result<(), ParseError> {
        let start = self.current_position.0;

        self.advance()?;

        let is_mutable = self.allow(Token::Mut)?;
        let identifier_position = self.current_position;

        self.expect(Token::Identifier)?;
        self.expect(Token::Equal)?;
        self.parse_expression()?;
        self.allow(Token::Semicolon)?;

        let end = self.previous_position.1;
        let expression_index = self.syntax_tree.node_count() - 1;
        let local = Local {
            identifier_position,
            is_mutable,
            scope: self.current_scope,
        };
        let local_index = self.syntax_tree.push_local(local).map_err(|local_index| {
            ParseError::DuplicateLocal {
                local_index,
                identifier_position,
            }
        })?;
        let node = SyntaxNode {
            kind: SyntaxKind::LetStatement,
            span: Span(start, end),
            child: expression_index,
            payload: local_index,
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
        let left = self.syntax_tree.node_count() - 1;
        let left_node = self.syntax_tree.get_node(left);
        let start = left_node.map(|node| node.span).unwrap_or_default().0;
        let node_kind = match self.current_token {
            Token::Plus => SyntaxKind::AdditionExpression,
            Token::Minus => SyntaxKind::SubtractionExpression,
            Token::Asterisk => SyntaxKind::MultiplicationExpression,
            Token::Slash => SyntaxKind::DivisionExpression,
            Token::Percent => SyntaxKind::ModuloExpression,
            _ => {
                return Err(ParseError::ExpectedMultipleTokens {
                    expected: vec![
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

        let right = self.syntax_tree.node_count() - 1;
        let end = self.current_position.0;
        let node = SyntaxNode {
            kind: node_kind,
            span: Span(start, end),
            child: left,
            payload: right,
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
        let expression_index = self.syntax_tree.node_count() - 1;
        let node = SyntaxNode {
            kind: SyntaxKind::GroupedExpression,
            span: Span(start, end),
            child: expression_index,
            payload: 0,
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_function(&mut self) -> Result<(), ParseError> {
        todo!()
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
        let identifier_text = self.current_source();

        self.advance()?;

        let end = self.previous_position.1;

        let local_index = self
            .syntax_tree
            .find_local_index(identifier_text, self.lexer.source())
            .ok_or_else(|| ParseError::UndeclaredVariable {
                position: Span::new(start, end),
            })?;

        let node = SyntaxNode {
            kind: SyntaxKind::PathExpression,
            span: Span(start, end),
            child: 0,
            payload: local_index,
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
                span: Span(start, end),
                child: 0,
                payload: is_optional as u32,
            }
        } else {
            let span = Span(last_node.span.0, end);

            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                span,
                child: self.syntax_tree.node_count() - 1,
                payload: 0,
            }
        };

        self.syntax_tree.push_node(node);

        Ok(())
    }

    fn parse_str(&mut self) -> Result<(), ParseError> {
        todo!()
    }
}

#[derive(Debug)]
pub enum ParseError {
    DuplicateLocal {
        local_index: u32,
        identifier_position: Span,
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
        expected: Vec<Token>,
        position: Span,
    },
    UnexpectedToken {
        actual: Token,
        position: Span,
    },
    LexError {
        error: LexError,
    },
    UndeclaredVariable {
        position: Span,
    },
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ParseError::DuplicateLocal {
                local_index,
                identifier_position,
            } => {
                write!(
                    f,
                    "Duplicate local at index {local_index} at {identifier_position}"
                )
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
            ParseError::UndeclaredVariable { position } => {
                write!(f, "Undeclared variable at {position}")
            }
        }
    }
}

impl AnnotatedError for ParseError {
    fn annotated_error(&self) -> ErrorMessage {
        let title = "Parsing Error";

        match self {
            ParseError::DuplicateLocal {
                identifier_position,
                ..
            } => ErrorMessage {
                title,
                description: "Duplicate variable declaration",
                detail_snippets: vec![(
                    "This variable already exists in this scope.".to_string(),
                    *identifier_position,
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
            ParseError::UndeclaredVariable { position } => ErrorMessage {
                title: "Undeclared Variable",
                description: "Variable used before declaration",
                detail_snippets: vec![("Variable used here".to_string(), *position)],
                help_snippet: None,
            },
        }
    }
}
