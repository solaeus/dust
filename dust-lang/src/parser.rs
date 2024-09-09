use std::{
    fmt::{self, Display, Formatter},
    mem,
    num::ParseIntError,
};

use crate::{
    Chunk, ChunkError, DustError, Identifier, Instruction, LexError, Lexer, Span, Token, TokenKind,
    TokenOwned, Value,
};

pub fn parse(source: &str) -> Result<Chunk, DustError> {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);

    while !parser.is_eof() {
        parser
            .parse_statement()
            .map_err(|error| DustError::Parse { error, source })?;
    }

    Ok(parser.chunk)
}

#[derive(Debug)]
pub struct Parser<'src> {
    lexer: Lexer<'src>,
    chunk: Chunk,
    previous_token: Token<'src>,
    previous_position: Span,
    current_token: Token<'src>,
    current_position: Span,
}

impl<'src> Parser<'src> {
    pub fn new(mut lexer: Lexer<'src>) -> Self {
        let (current_token, current_position) =
            lexer.next_token().unwrap_or((Token::Eof, Span(0, 0)));

        log::trace!("Starting parser with token {current_token} at {current_position}");

        Parser {
            lexer,
            chunk: Chunk::new(),
            previous_token: Token::Eof,
            previous_position: Span(0, 0),
            current_token,
            current_position,
        }
    }

    fn is_eof(&self) -> bool {
        matches!(self.current_token, Token::Eof)
    }

    fn advance(&mut self) -> Result<(), ParseError> {
        let (new_token, position) = self.lexer.next_token()?;

        log::trace!("Advancing to token {new_token} at {position}");

        self.previous_token = mem::replace(&mut self.current_token, new_token);
        self.previous_position = mem::replace(&mut self.current_position, position);

        Ok(())
    }

    fn allow(&mut self, allowed: TokenKind) -> Result<bool, ParseError> {
        if self.current_token.kind() == allowed {
            self.advance()?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn expect(&mut self, expected: TokenKind) -> Result<(), ParseError> {
        if self.current_token.kind() == expected {
            self.advance()
        } else {
            Err(ParseError::ExpectedToken {
                expected,
                found: self.current_token.to_owned(),
                position: self.current_position,
            })
        }
    }

    fn emit_byte(&mut self, byte: u8, position: Span) {
        self.chunk.write(byte, position);
    }

    fn emit_constant(&mut self, value: Value) -> Result<(), ParseError> {
        let constant_index = self.chunk.push_constant(value)?;
        let position = self.previous_position;

        self.emit_byte(Instruction::Constant as u8, position);
        self.emit_byte(constant_index, position);

        Ok(())
    }

    fn parse_boolean(&mut self, _allow_assignment: bool) -> Result<(), ParseError> {
        if let Token::Boolean(text) = self.previous_token {
            let boolean = text.parse::<bool>().unwrap();
            let value = Value::boolean(boolean);

            self.emit_constant(value)?;
        }

        Ok(())
    }

    fn parse_float(&mut self, _allow_assignment: bool) -> Result<(), ParseError> {
        if let Token::Float(text) = self.previous_token {
            let float = text.parse::<f64>().unwrap();
            let value = Value::float(float);

            self.emit_constant(value)?;
        }

        Ok(())
    }

    fn parse_integer(&mut self, _allow_assignment: bool) -> Result<(), ParseError> {
        if let Token::Integer(text) = self.previous_token {
            let integer = text.parse::<i64>().unwrap();
            let value = Value::integer(integer);

            self.emit_constant(value)?;
        }

        Ok(())
    }

    fn parse_string(&mut self, _allow_assignment: bool) -> Result<(), ParseError> {
        if let Token::String(text) = self.previous_token {
            let value = Value::string(text);

            self.emit_constant(value)?;
        }

        Ok(())
    }

    fn parse_grouped(&mut self, _allow_assignment: bool) -> Result<(), ParseError> {
        self.parse_expression()?;
        self.expect(TokenKind::RightParenthesis)
    }

    fn parse_unary(&mut self, _allow_assignment: bool) -> Result<(), ParseError> {
        let operator_position = self.previous_position;
        let byte = match self.previous_token.kind() {
            TokenKind::Minus => Instruction::Negate as u8,
            _ => {
                return Err(ParseError::ExpectedTokenMultiple {
                    expected: vec![TokenKind::Minus],
                    found: self.previous_token.to_owned(),
                    position: operator_position,
                })
            }
        };

        self.parse_expression()?;
        self.emit_byte(byte, operator_position);

        Ok(())
    }

    fn parse_binary(&mut self) -> Result<(), ParseError> {
        let operator_position = self.previous_position;
        let operator = self.previous_token.kind();
        let rule = ParseRule::from(&operator);

        self.parse(rule.precedence.increment())?;

        let byte = match operator {
            TokenKind::Plus => Instruction::Add as u8,
            TokenKind::Minus => Instruction::Subtract as u8,
            TokenKind::Star => Instruction::Multiply as u8,
            TokenKind::Slash => Instruction::Divide as u8,
            _ => {
                return Err(ParseError::ExpectedTokenMultiple {
                    expected: vec![
                        TokenKind::Plus,
                        TokenKind::Minus,
                        TokenKind::Star,
                        TokenKind::Slash,
                    ],
                    found: self.previous_token.to_owned(),
                    position: operator_position,
                })
            }
        };

        self.emit_byte(byte, operator_position);

        Ok(())
    }

    fn parse_variable(&mut self, allow_assignment: bool) -> Result<(), ParseError> {
        self.parse_named_variable_from(self.previous_token.to_owned(), allow_assignment)
    }

    fn parse_named_variable_from(
        &mut self,
        token: TokenOwned,
        allow_assignment: bool,
    ) -> Result<(), ParseError> {
        let identifier_index = self.parse_identifier_from(token)?;

        if allow_assignment && self.allow(TokenKind::Equal)? {
            self.parse_expression()?;
            self.emit_byte(Instruction::SetVariable as u8, self.previous_position);
            self.emit_byte(identifier_index, self.previous_position);
        } else {
            self.emit_byte(Instruction::GetVariable as u8, self.previous_position);
            self.emit_byte(identifier_index, self.previous_position);
        }

        Ok(())
    }

    fn parse_identifier_from(&mut self, token: TokenOwned) -> Result<u8, ParseError> {
        if let TokenOwned::Identifier(text) = token {
            self.advance()?;

            let identifier = Identifier::new(text);
            let identifier_index = self.chunk.get_identifier_index(&identifier)?;

            Ok(identifier_index)
        } else {
            Err(ParseError::ExpectedToken {
                expected: TokenKind::Identifier,
                found: self.current_token.to_owned(),
                position: self.current_position,
            })
        }
    }

    fn parse_expression(&mut self) -> Result<(), ParseError> {
        self.parse(Precedence::None)
    }

    fn parse_statement(&mut self) -> Result<(), ParseError> {
        let start = self.current_position.0;
        let is_expression_statement = match self.current_token {
            Token::Let => {
                self.parse_let_assignment(true)?;

                false
            }
            _ => {
                self.parse_expression()?;

                true
            }
        };
        let has_semicolon = self.allow(TokenKind::Semicolon)?;

        if is_expression_statement && has_semicolon {
            let end = self.previous_position.1;

            self.emit_byte(Instruction::Pop as u8, Span(start, end));
        }

        Ok(())
    }

    fn parse_let_assignment(&mut self, _allow_assignment: bool) -> Result<(), ParseError> {
        self.expect(TokenKind::Let)?;

        let position = self.current_position;

        let identifier_index = if let Token::Identifier(text) = self.current_token {
            self.advance()?;

            let identifier = Identifier::new(text);

            self.chunk.push_identifier(identifier)?
        } else {
            return Err(ParseError::ExpectedToken {
                expected: TokenKind::Identifier,
                found: self.current_token.to_owned(),
                position: self.current_position,
            });
        };

        self.emit_byte(Instruction::DefineVariable as u8, position);
        self.emit_byte(identifier_index, position);
        self.expect(TokenKind::Equal)?;
        self.parse_expression()?;

        Ok(())
    }

    fn parse(&mut self, precedence: Precedence) -> Result<(), ParseError> {
        self.advance()?;

        let prefix_rule = if let Some(prefix) = ParseRule::from(&self.previous_token.kind()).prefix
        {
            log::trace!(
                "Parsing {} as prefix with precedence {precedence}",
                self.previous_token,
            );

            prefix
        } else {
            return Err(ParseError::ExpectedExpression {
                found: self.previous_token.to_owned(),
                position: self.previous_position,
            });
        };

        let allow_assignment = precedence <= Precedence::Assignment;

        prefix_rule(self, allow_assignment)?;

        while precedence < ParseRule::from(&self.current_token.kind()).precedence {
            self.advance()?;

            let infix_rule = ParseRule::from(&self.previous_token.kind()).infix;

            if let Some(infix) = infix_rule {
                log::trace!(
                    "Parsing {} as infix with precedence {precedence}",
                    self.previous_token,
                );

                if allow_assignment && self.allow(TokenKind::Equal)? {
                    return Err(ParseError::InvalidAssignmentTarget {
                        found: self.previous_token.to_owned(),
                        position: self.previous_position,
                    });
                }

                infix(self)?;
            } else {
                break;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    None = 0,
    Assignment = 1,
    Conditional = 2,
    LogicalOr = 3,
    LogicalAnd = 4,
    Equality = 5,
    Comparison = 6,
    Term = 7,
    Factor = 8,
    Unary = 9,
    Call = 10,
    Primary = 11,
}

impl Precedence {
    fn from_byte(byte: u8) -> Self {
        match byte {
            0 => Self::None,
            1 => Self::Assignment,
            2 => Self::Conditional,
            3 => Self::LogicalOr,
            4 => Self::LogicalAnd,
            5 => Self::Equality,
            6 => Self::Comparison,
            7 => Self::Term,
            8 => Self::Factor,
            9 => Self::Unary,
            10 => Self::Call,
            _ => Self::Primary,
        }
    }

    fn increment(&self) -> Self {
        Self::from_byte(*self as u8 + 1)
    }
}

impl Display for Precedence {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

type PrefixFunction<'a> = fn(&mut Parser<'a>, bool) -> Result<(), ParseError>;
type InfixFunction<'a> = fn(&mut Parser<'a>) -> Result<(), ParseError>;

#[derive(Debug, Clone, Copy)]
pub struct ParseRule<'a> {
    pub prefix: Option<PrefixFunction<'a>>,
    pub infix: Option<InfixFunction<'a>>,
    pub precedence: Precedence,
}

impl From<&TokenKind> for ParseRule<'_> {
    fn from(token_kind: &TokenKind) -> Self {
        match token_kind {
            TokenKind::Eof => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Identifier => ParseRule {
                prefix: Some(Parser::parse_variable),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Boolean => ParseRule {
                prefix: Some(Parser::parse_boolean),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Character => todo!(),
            TokenKind::Float => ParseRule {
                prefix: Some(Parser::parse_float),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Integer => ParseRule {
                prefix: Some(Parser::parse_integer),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::String => ParseRule {
                prefix: Some(Parser::parse_string),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Async => todo!(),
            TokenKind::Bool => todo!(),
            TokenKind::Break => todo!(),
            TokenKind::Else => todo!(),
            TokenKind::FloatKeyword => todo!(),
            TokenKind::If => todo!(),
            TokenKind::Int => todo!(),
            TokenKind::Let => ParseRule {
                prefix: Some(Parser::parse_let_assignment),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Loop => todo!(),
            TokenKind::Map => todo!(),
            TokenKind::Str => todo!(),
            TokenKind::While => todo!(),
            TokenKind::BangEqual => todo!(),
            TokenKind::Bang => todo!(),
            TokenKind::Colon => todo!(),
            TokenKind::Comma => todo!(),
            TokenKind::Dot => todo!(),
            TokenKind::DoubleAmpersand => todo!(),
            TokenKind::DoubleDot => todo!(),
            TokenKind::DoubleEqual => todo!(),
            TokenKind::DoublePipe => todo!(),
            TokenKind::Equal => todo!(),
            TokenKind::Greater => todo!(),
            TokenKind::GreaterOrEqual => todo!(),
            TokenKind::LeftCurlyBrace => todo!(),
            TokenKind::LeftParenthesis => ParseRule {
                prefix: Some(Parser::parse_grouped),
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::LeftSquareBrace => todo!(),
            TokenKind::Less => todo!(),
            TokenKind::LessOrEqual => todo!(),
            TokenKind::Minus => ParseRule {
                prefix: Some(Parser::parse_unary),
                infix: Some(Parser::parse_binary),
                precedence: Precedence::Term,
            },
            TokenKind::MinusEqual => todo!(),
            TokenKind::Mut => todo!(),
            TokenKind::Percent => todo!(),
            TokenKind::Plus => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary),
                precedence: Precedence::Term,
            },
            TokenKind::PlusEqual => todo!(),
            TokenKind::RightCurlyBrace => todo!(),
            TokenKind::RightParenthesis => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::RightSquareBrace => todo!(),
            TokenKind::Semicolon => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
            TokenKind::Star => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary),
                precedence: Precedence::Factor,
            },
            TokenKind::Struct => todo!(),
            TokenKind::Slash => ParseRule {
                prefix: None,
                infix: Some(Parser::parse_binary),
                precedence: Precedence::Factor,
            },
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    ExpectedExpression {
        found: TokenOwned,
        position: Span,
    },
    ExpectedToken {
        expected: TokenKind,
        found: TokenOwned,
        position: Span,
    },
    ExpectedTokenMultiple {
        expected: Vec<TokenKind>,
        found: TokenOwned,
        position: Span,
    },
    InvalidAssignmentTarget {
        found: TokenOwned,
        position: Span,
    },

    // Wrappers around foreign errors
    Chunk(ChunkError),
    Lex(LexError),
    ParseIntError(ParseIntError),
}

impl From<ParseIntError> for ParseError {
    fn from(error: ParseIntError) -> Self {
        Self::ParseIntError(error)
    }
}

impl From<LexError> for ParseError {
    fn from(error: LexError) -> Self {
        Self::Lex(error)
    }
}

impl From<ChunkError> for ParseError {
    fn from(error: ChunkError) -> Self {
        Self::Chunk(error)
    }
}

#[cfg(test)]
mod tests {
    use crate::identifier_stack::Local;

    use super::*;

    #[test]
    fn add_variables() {
        let source = "
            let x = 42;
            let y = 42;
            x + y
        ";
        let test_chunk = parse(source);

        assert_eq!(
            test_chunk,
            Ok(Chunk::with_data(
                vec![
                    (Instruction::Constant as u8, Span(21, 23)),
                    (0, Span(21, 23)),
                    (Instruction::DefineVariable as u8, Span(17, 18)),
                    (0, Span(17, 18)),
                    (Instruction::Constant as u8, Span(44, 46)),
                    (1, Span(44, 46)),
                    (Instruction::DefineVariable as u8, Span(40, 41)),
                    (1, Span(40, 41)),
                    (Instruction::GetVariable as u8, Span(61, 62)),
                    (0, Span(61, 62)),
                    (Instruction::GetVariable as u8, Span(52, 53)),
                    (1, Span(52, 53)),
                    (Instruction::Add as u8, Span(48, 53))
                ],
                vec![Value::integer(42), Value::integer(42)],
                vec![
                    Local {
                        identifier: Identifier::new("x"),
                        depth: 0
                    },
                    Local {
                        identifier: Identifier::new("y"),
                        depth: 0
                    },
                    Local {
                        identifier: Identifier::new("x"),
                        depth: 0
                    },
                    Local {
                        identifier: Identifier::new("y"),
                        depth: 0
                    },
                ],
            ))
        );
    }

    #[test]
    fn let_statement() {
        let source = "let x = 42;";
        let test_chunk = parse(source);

        assert_eq!(
            test_chunk,
            Ok(Chunk::with_data(
                vec![
                    (Instruction::DefineVariable as u8, Span(4, 5)),
                    (0, Span(4, 5)),
                    (Instruction::Constant as u8, Span(8, 10)),
                    (0, Span(8, 10)),
                ],
                vec![Value::integer(42)],
                vec![Local {
                    identifier: Identifier::new("x"),
                    depth: 0
                }],
            ))
        );
    }

    #[test]
    fn string() {
        let source = "\"Hello, World!\"";
        let test_chunk = parse(source);

        assert_eq!(
            test_chunk,
            Ok(Chunk::with_data(
                vec![(Instruction::Constant as u8, Span(0, 15)), (0, Span(0, 15))],
                vec![Value::string("Hello, World!")],
                vec![],
            ))
        );
    }

    #[test]
    fn integer() {
        let source = "42";
        let test_chunk = parse(source);

        assert_eq!(
            test_chunk,
            Ok(Chunk::with_data(
                vec![(Instruction::Constant as u8, Span(0, 2)), (0, Span(0, 2))],
                vec![Value::integer(42)],
                vec![],
            ))
        );
    }

    #[test]
    fn boolean() {
        let source = "true";
        let test_chunk = parse(source);

        assert_eq!(
            test_chunk,
            Ok(Chunk::with_data(
                vec![(Instruction::Constant as u8, Span(0, 4)), (0, Span(0, 4))],
                vec![Value::boolean(true)],
                vec![],
            ))
        );
    }

    #[test]
    fn grouping() {
        let source = "(42 + 42) * 2";
        let test_chunk = parse(source);

        assert_eq!(
            test_chunk,
            Ok(Chunk::with_data(
                vec![
                    (Instruction::Constant as u8, Span(1, 3)),
                    (0, Span(1, 3)),
                    (Instruction::Constant as u8, Span(6, 8)),
                    (1, Span(6, 8)),
                    (Instruction::Add as u8, Span(4, 5)),
                    (Instruction::Constant as u8, Span(12, 13)),
                    (2, Span(12, 13)),
                    (Instruction::Multiply as u8, Span(10, 11)),
                ],
                vec![Value::integer(42), Value::integer(42), Value::integer(2)],
                vec![],
            ))
        );
    }

    #[test]
    fn negation() {
        let source = "-(42)";
        let test_chunk = parse(source);

        assert_eq!(
            test_chunk,
            Ok(Chunk::with_data(
                vec![
                    (Instruction::Constant as u8, Span(2, 4)),
                    (0, Span(2, 4)),
                    (Instruction::Negate as u8, Span(0, 1)),
                ],
                vec![Value::integer(42)],
                vec![],
            ))
        );
    }

    #[test]
    fn addition() {
        let source = "42 + 42";
        let test_chunk = parse(source);

        assert_eq!(
            test_chunk,
            Ok(Chunk::with_data(
                vec![
                    (Instruction::Constant as u8, Span(0, 2)),
                    (0, Span(0, 2)),
                    (Instruction::Constant as u8, Span(5, 7)),
                    (1, Span(5, 7)),
                    (Instruction::Add as u8, Span(3, 4)),
                ],
                vec![Value::integer(42), Value::integer(42)],
                vec![],
            ))
        );
    }

    #[test]
    fn subtraction() {
        let source = "42 - 42";
        let test_chunk = parse(source);

        assert_eq!(
            test_chunk,
            Ok(Chunk::with_data(
                vec![
                    (Instruction::Constant as u8, Span(0, 2)),
                    (0, Span(0, 2)),
                    (Instruction::Constant as u8, Span(5, 7)),
                    (1, Span(5, 7)),
                    (Instruction::Subtract as u8, Span(3, 4)),
                ],
                vec![Value::integer(42), Value::integer(42)],
                vec![],
            ))
        );
    }

    #[test]
    fn multiplication() {
        let source = "42 * 42";
        let test_chunk = parse(source);

        assert_eq!(
            test_chunk,
            Ok(Chunk::with_data(
                vec![
                    (Instruction::Constant as u8, Span(0, 2)),
                    (0, Span(0, 2)),
                    (Instruction::Constant as u8, Span(5, 7)),
                    (1, Span(5, 7)),
                    (Instruction::Multiply as u8, Span(3, 4)),
                ],
                vec![Value::integer(42), Value::integer(42)],
                vec![],
            ))
        );
    }

    #[test]
    fn division() {
        let source = "42 / 42";
        let test_chunk = parse(source);

        assert_eq!(
            test_chunk,
            Ok(Chunk::with_data(
                vec![
                    (Instruction::Constant as u8, Span(0, 2)),
                    (0, Span(0, 2)),
                    (Instruction::Constant as u8, Span(5, 7)),
                    (1, Span(5, 7)),
                    (Instruction::Divide as u8, Span(3, 4)),
                ],
                vec![Value::integer(42), Value::integer(42)],
                vec![],
            ))
        );
    }
}
