//! Parsing tools.
//!
//! This module provides two parsing options:
//! - `parse` convenience function
//! - `Parser` struct, which parses the input a statement at a time
use std::{
    collections::VecDeque,
    error::Error,
    fmt::{self, Display, Formatter},
    num::{ParseFloatError, ParseIntError},
    str::ParseBoolError,
};

use crate::{
    abstract_tree::BinaryOperator, built_in_function::BuiltInFunction, token::TokenOwned,
    AbstractSyntaxTree, Identifier, LexError, Lexer, Node, Span, Statement, Token, Value,
};

/// Parses the input into an abstract syntax tree.
///
/// # Examples
/// ```
/// # use dust_lang::*;
/// let input = "x = 42";
/// let result = parse(input);
///
/// assert_eq!(
///     result,
///     Ok(AbstractSyntaxTree {
///         nodes: [
///             Node {
///                 inner: Statement::Assignment {
///                     identifier: Node {
///                         inner: Identifier::new("x"),
///                         position: (0, 1),
///                     },
///                     value_node: Box::new(Node {
///                         inner: Statement::Constant(Value::integer(42)),
///                         position: (4, 6),
///                     })
///                 },
///                 position: (0, 6),
///             }
///         ].into(),
///     }),
/// );
/// ```
pub fn parse(input: &str) -> Result<AbstractSyntaxTree, ParseError> {
    let lexer = Lexer::new();
    let mut parser = Parser::new(input, lexer);
    let mut nodes = VecDeque::new();

    loop {
        let node = parser.parse()?;

        nodes.push_back(node);

        if let Token::Eof = parser.current.0 {
            break;
        }
    }

    Ok(AbstractSyntaxTree { nodes })
}

/// Low-level tool for parsing the input a statement at a time.
///
/// # Examples
/// ```
/// # use std::collections::VecDeque;
/// # use dust_lang::*;
/// let input = "x = 42";
/// let lexer = Lexer::new();
/// let mut parser = Parser::new(input, lexer);
/// let mut nodes = VecDeque::new();
///
/// loop {
///     let node = parser.parse().unwrap();
///
///     nodes.push_back(node);
///
///     if let Token::Eof = parser.current().0 {
///         break;
///     }
/// }
///
/// assert_eq!(
///     nodes,
///     Into::<VecDeque<Node<Statement>>>::into([
///         Node {
///             inner: Statement::Assignment {
///                 identifier: Node {
///                     inner: Identifier::new("x"),
///                     position: (0, 1),
///                 },
///                 value_node: Box::new(Node {
///                     inner: Statement::Constant(Value::integer(42)),
///                     position: (4, 6),
///                 }),
///             },
///             position: (0, 6),
///         }
///     ]),
/// );
/// ```
pub struct Parser<'src> {
    source: &'src str,
    lexer: Lexer,
    current: (Token<'src>, Span),
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str, lexer: Lexer) -> Self {
        let mut lexer = lexer;
        let current = lexer.next_token(source).unwrap_or((Token::Eof, (0, 0)));

        Parser {
            source,
            lexer,
            current,
        }
    }

    pub fn parse(&mut self) -> Result<Node<Statement>, ParseError> {
        self.parse_node(0)
    }

    pub fn current(&self) -> &(Token, Span) {
        &self.current
    }

    fn next_token(&mut self) -> Result<(), ParseError> {
        let next = self.lexer.next_token(self.source);

        self.current = match next {
            Ok((token, position)) => (token, position),
            Err(lex_error) => {
                let position = {
                    self.next_token()?;

                    self.current.1
                };

                return Err(ParseError::LexError {
                    error: lex_error,
                    position,
                });
            }
        };

        Ok(())
    }

    fn parse_node(&mut self, precedence: u8) -> Result<Node<Statement>, ParseError> {
        let left = self.parse_primary()?;

        if precedence < self.current_precedence() {
            self.parse_infix(left)
        } else {
            Ok(left)
        }
    }

    fn parse_primary(&mut self) -> Result<Node<Statement>, ParseError> {
        match self.current {
            (Token::Boolean(text), position) => {
                self.next_token()?;

                let boolean = text
                    .parse()
                    .map_err(|error| ParseError::BooleanError { error, position })?;

                Ok(Node::new(
                    Statement::Constant(Value::boolean(boolean)),
                    position,
                ))
            }
            (Token::Float(text), position) => {
                self.next_token()?;

                let float = text
                    .parse()
                    .map_err(|error| ParseError::FloatError { error, position })?;

                Ok(Node::new(
                    Statement::Constant(Value::float(float)),
                    position,
                ))
            }
            (Token::Integer(text), position) => {
                self.next_token()?;

                let integer = text
                    .parse()
                    .map_err(|error| ParseError::IntegerError { error, position })?;

                Ok(Node::new(
                    Statement::Constant(Value::integer(integer)),
                    position,
                ))
            }
            (Token::Identifier(text), position) => {
                self.next_token()?;

                if let (Token::Equal, _) = self.current {
                    self.next_token()?;

                    let value_node = self.parse_node(0)?;
                    let right_end = value_node.position.1;

                    Ok(Node::new(
                        Statement::Assignment {
                            identifier: Node::new(Identifier::new(text), position),
                            value_node: Box::new(value_node),
                        },
                        (position.0, right_end),
                    ))
                } else {
                    Ok(Node::new(
                        Statement::Identifier(Identifier::new(text)),
                        position,
                    ))
                }
            }
            (Token::String(string), position) => {
                self.next_token()?;

                Ok(Node::new(
                    Statement::Constant(Value::string(string)),
                    position,
                ))
            }
            (Token::LeftCurlyBrace, left_position) => {
                self.next_token()?;

                // If the next token is a right curly brace, this is an empty map
                if let (Token::RightCurlyBrace, right_position) = self.current {
                    self.next_token()?;

                    return Ok(Node::new(
                        Statement::Map(Vec::new()),
                        (left_position.0, right_position.1),
                    ));
                }

                let mut statement = None;

                loop {
                    // If a closing brace is found, return the new statement
                    if let (Token::RightCurlyBrace, right_position) = self.current {
                        self.next_token()?;

                        return Ok(Node::new(
                            statement.unwrap(),
                            (left_position.0, right_position.1),
                        ));
                    }

                    let next_node = self.parse_node(0)?;

                    // If the next node is an assignment, this might be a map
                    if let Statement::Assignment {
                        identifier,
                        value_node,
                    } = next_node.inner
                    {
                        // If the current token is a comma, right curly brace, or the new
                        // statement is already a map
                        if self.current.0 == Token::Comma
                            || statement
                                .as_ref()
                                .is_some_and(|statement| matches!(statement, Statement::Map(_)))
                        {
                            // The new statement is a map
                            if let Statement::Map(map_properties) =
                                statement.get_or_insert_with(|| Statement::Map(Vec::new()))
                            {
                                // Ignore commas after properties
                                if let Token::Comma = self.current.0 {
                                    self.next_token()?;
                                }

                                // Add the new property to the map
                                map_properties.push((identifier, *value_node));
                            }
                        // Otherwise, the new statement is a block
                        } else if let Statement::Block(statements) =
                            statement.get_or_insert_with(|| Statement::Block(Vec::new()))
                        {
                            if self.current.0 == Token::Semicolon {
                                self.next_token()?;

                                statements.push(Node::new(
                                    Statement::Nil(Box::new(Node::new(
                                        Statement::Assignment {
                                            identifier,
                                            value_node,
                                        },
                                        next_node.position,
                                    ))),
                                    (next_node.position.0, self.current.1 .1),
                                ));

                                continue;
                            } else {
                                statements.push(Node::new(
                                    Statement::Assignment {
                                        identifier,
                                        value_node,
                                    },
                                    next_node.position,
                                ));

                                continue;
                            }
                        }
                    } else if let Statement::Block(statements) =
                        statement.get_or_insert_with(|| Statement::Block(Vec::new()))
                    {
                        // Add the assignment statement to the block
                        statements.push(next_node);
                    }
                }
            }
            (Token::LeftParenthesis, left_position) => {
                self.next_token()?;

                let node = self.parse_node(0)?;

                if let (Token::RightParenthesis, right_position) = self.current {
                    self.next_token()?;

                    Ok(Node::new(node.inner, (left_position.0, right_position.1)))
                } else {
                    Err(ParseError::ExpectedToken {
                        expected: TokenOwned::RightParenthesis,
                        actual: self.current.0.to_owned(),
                        position: self.current.1,
                    })
                }
            }
            (Token::LeftSquareBrace, left_position) => {
                self.next_token()?;

                let mut nodes = Vec::new();

                loop {
                    if let (Token::RightSquareBrace, right_position) = self.current {
                        self.next_token()?;

                        return Ok(Node::new(
                            Statement::List(nodes),
                            (left_position.0, right_position.1),
                        ));
                    }

                    if let (Token::Comma, _) = self.current {
                        self.next_token()?;

                        continue;
                    }

                    if let Ok(instruction) = self.parse_node(0) {
                        nodes.push(instruction);
                    } else {
                        return Err(ParseError::ExpectedToken {
                            expected: TokenOwned::RightSquareBrace,
                            actual: self.current.0.to_owned(),
                            position: self.current.1,
                        });
                    }
                }
            }
            (
                Token::IsEven | Token::IsOdd | Token::Length | Token::ReadLine | Token::WriteLine,
                left_position,
            ) => {
                let function = match self.current.0 {
                    Token::IsEven => BuiltInFunction::IsEven,
                    Token::IsOdd => BuiltInFunction::IsOdd,
                    Token::Length => BuiltInFunction::Length,
                    Token::ReadLine => BuiltInFunction::ReadLine,
                    Token::WriteLine => BuiltInFunction::WriteLine,
                    _ => unreachable!(),
                };

                self.next_token()?;

                if let (Token::LeftParenthesis, _) = self.current {
                    self.next_token()?;
                } else {
                    return Err(ParseError::ExpectedToken {
                        expected: TokenOwned::LeftParenthesis,
                        actual: self.current.0.to_owned(),
                        position: self.current.1,
                    });
                }

                let mut value_arguments: Option<Vec<Node<Statement>>> = None;

                loop {
                    if let (Token::RightParenthesis, _) = self.current {
                        self.next_token()?;
                        break;
                    }

                    if let (Token::Comma, _) = self.current {
                        self.next_token()?;
                        continue;
                    }

                    if let Ok(node) = self.parse_node(0) {
                        if let Some(ref mut arguments) = value_arguments {
                            arguments.push(node);
                        } else {
                            value_arguments = Some(vec![node]);
                        }
                    } else {
                        return Err(ParseError::ExpectedToken {
                            expected: TokenOwned::RightParenthesis,
                            actual: self.current.0.to_owned(),
                            position: self.current.1,
                        });
                    }
                }

                Ok(Node::new(
                    Statement::BuiltInFunctionCall {
                        function,
                        type_arguments: None,
                        value_arguments,
                    },
                    left_position,
                ))
            }
            _ => Err(ParseError::UnexpectedToken {
                actual: self.current.0.to_owned(),
                position: self.current.1,
            }),
        }
    }

    fn parse_infix(&mut self, left: Node<Statement>) -> Result<Node<Statement>, ParseError> {
        let left_start = left.position.0;

        let binary_operator = match &self.current {
            (Token::Dot, _) => {
                self.next_token()?;

                let right_node = self.parse_node(0)?;
                let right_end = right_node.position.1;

                return Ok(Node::new(
                    Statement::PropertyAccess(Box::new(left), Box::new(right_node)),
                    (left_start, right_end),
                ));
            }
            (Token::DoubleAmpersand, _) => Node::new(BinaryOperator::And, self.current.1),
            (Token::DoubleEqual, _) => Node::new(BinaryOperator::Equal, self.current.1),
            (Token::DoublePipe, _) => Node::new(BinaryOperator::Or, self.current.1),
            (Token::Greater, _) => Node::new(BinaryOperator::Greater, self.current.1),
            (Token::GreaterEqual, _) => Node::new(BinaryOperator::GreaterOrEqual, self.current.1),
            (Token::Less, _) => Node::new(BinaryOperator::Less, self.current.1),
            (Token::LessEqual, _) => Node::new(BinaryOperator::LessOrEqual, self.current.1),
            (Token::Minus, _) => Node::new(BinaryOperator::Subtract, self.current.1),
            (Token::Plus, _) => Node::new(BinaryOperator::Add, self.current.1),
            (Token::Star, _) => Node::new(BinaryOperator::Multiply, self.current.1),
            (Token::Slash, _) => Node::new(BinaryOperator::Divide, self.current.1),
            (Token::Percent, _) => Node::new(BinaryOperator::Modulo, self.current.1),
            _ => {
                self.next_token()?;

                return Err(ParseError::UnexpectedToken {
                    actual: self.current.0.to_owned(),
                    position: self.current.1,
                });
            }
        };

        self.next_token()?;

        let right = self.parse_node(0)?;
        let right_end = right.position.1;

        Ok(Node::new(
            Statement::BinaryOperation {
                left: Box::new(left),
                operator: binary_operator,
                right: Box::new(right),
            },
            (left_start, right_end),
        ))
    }

    fn current_precedence(&self) -> u8 {
        match self.current.0 {
            Token::DoubleEqual => 7,
            Token::DoubleAmpersand | Token::DoublePipe => 6,
            Token::Greater | Token::GreaterEqual | Token::Less | Token::LessEqual => 5,
            Token::Dot => 4,
            Token::Percent => 3,
            Token::Star => 2,
            Token::Slash => 2,
            Token::Plus => 1,
            Token::Minus => 1,
            _ => 0,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ParseError {
    BooleanError {
        error: ParseBoolError,
        position: Span,
    },
    LexError {
        error: LexError,
        position: Span,
    },
    ExpectedIdentifier {
        actual: TokenOwned,
        position: Span,
    },
    ExpectedToken {
        expected: TokenOwned,
        actual: TokenOwned,
        position: Span,
    },
    UnexpectedToken {
        actual: TokenOwned,
        position: Span,
    },
    FloatError {
        error: ParseFloatError,
        position: Span,
    },
    IntegerError {
        error: ParseIntError,
        position: Span,
    },
}

impl ParseError {
    pub fn position(&self) -> Span {
        match self {
            ParseError::BooleanError { position, .. } => *position,
            ParseError::ExpectedIdentifier { position, .. } => *position,
            ParseError::ExpectedToken { position, .. } => *position,
            ParseError::FloatError { position, .. } => *position,
            ParseError::IntegerError { position, .. } => *position,
            ParseError::LexError { position, .. } => *position,
            ParseError::UnexpectedToken { position, .. } => *position,
        }
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LexError { error, .. } => Some(error),
            _ => None,
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::BooleanError { error, .. } => write!(f, "{}", error),
            Self::ExpectedIdentifier { actual, .. } => {
                write!(f, "Expected identifier, found {actual}")
            }
            Self::ExpectedToken {
                expected, actual, ..
            } => write!(f, "Expected token {expected}, found {actual}"),
            Self::FloatError { error, .. } => write!(f, "{}", error),
            Self::IntegerError { error, .. } => write!(f, "{}", error),
            Self::LexError { error, .. } => write!(f, "{}", error),
            Self::UnexpectedToken { actual, .. } => write!(f, "Unexpected token {actual}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{abstract_tree::BinaryOperator, Identifier};

    use super::*;

    #[test]
    fn or() {
        let input = "true || false";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                nodes: [Node::new(
                    Statement::BinaryOperation {
                        left: Box::new(Node::new(
                            Statement::Constant(Value::boolean(true)),
                            (0, 4)
                        )),
                        operator: Node::new(BinaryOperator::Or, (5, 7)),
                        right: Box::new(Node::new(
                            Statement::Constant(Value::boolean(false)),
                            (8, 13)
                        )),
                    },
                    (0, 13)
                )]
                .into()
            })
        );
    }

    #[test]
    fn misplaced_semicolon() {
        let input = ";";

        assert_eq!(
            parse(input),
            Err(ParseError::UnexpectedToken {
                actual: TokenOwned::Semicolon,
                position: (0, 1)
            })
        );
    }

    #[test]
    fn block_with_one_statement() {
        let input = "{ 40 + 2 }";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                nodes: [Node::new(
                    Statement::Block(vec![Node::new(
                        Statement::BinaryOperation {
                            left: Box::new(Node::new(
                                Statement::Constant(Value::integer(40)),
                                (2, 4)
                            )),
                            operator: Node::new(BinaryOperator::Add, (5, 6)),
                            right: Box::new(Node::new(
                                Statement::Constant(Value::integer(2)),
                                (7, 8)
                            )),
                        },
                        (2, 8)
                    )]),
                    (0, 10)
                )]
                .into()
            })
        );
    }

    #[test]
    fn block_with_assignment() {
        let input = "{ foo = 42; bar = 42; baz = '42' }";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                nodes: [Node::new(
                    Statement::Block(vec![
                        Node::new(
                            Statement::Nil(Box::new(Node::new(
                                Statement::Assignment {
                                    identifier: Node::new(Identifier::new("foo"), (2, 5)),
                                    value_node: Box::new(Node::new(
                                        Statement::Constant(Value::integer(42)),
                                        (8, 10)
                                    ))
                                },
                                (2, 10)
                            ),)),
                            (2, 15)
                        ),
                        Node::new(
                            Statement::Nil(Box::new(Node::new(
                                Statement::Assignment {
                                    identifier: Node::new(Identifier::new("bar"), (12, 15)),
                                    value_node: Box::new(Node::new(
                                        Statement::Constant(Value::integer(42)),
                                        (18, 20)
                                    ))
                                },
                                (12, 20)
                            ),)),
                            (12, 25)
                        ),
                        Node::new(
                            Statement::Assignment {
                                identifier: Node::new(Identifier::new("baz"), (22, 25)),
                                value_node: Box::new(Node::new(
                                    Statement::Constant(Value::string("42")),
                                    (28, 32)
                                ))
                            },
                            (22, 32)
                        )
                    ]),
                    (0, 34)
                )]
                .into()
            })
        );
    }

    #[test]
    fn empty_map() {
        let input = "{}";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                nodes: [Node::new(Statement::Map(vec![]), (0, 2))].into()
            })
        );
    }

    #[test]
    fn map_with_trailing_comma() {
        let input = "{ foo = 42, bar = 42, baz = '42', }";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                nodes: [Node::new(
                    Statement::Map(vec![
                        (
                            Node::new(Identifier::new("foo"), (2, 5)),
                            Node::new(Statement::Constant(Value::integer(42)), (8, 10))
                        ),
                        (
                            Node::new(Identifier::new("bar"), (12, 15)),
                            Node::new(Statement::Constant(Value::integer(42)), (18, 20))
                        ),
                        (
                            Node::new(Identifier::new("baz"), (22, 25)),
                            Node::new(Statement::Constant(Value::string("42")), (28, 32))
                        ),
                    ]),
                    (0, 35)
                )]
                .into()
            })
        );
    }

    #[test]
    fn map_with_two_properties() {
        let input = "{ x = 42, y = 'foobar' }";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                nodes: [Node::new(
                    Statement::Map(vec![
                        (
                            Node::new(Identifier::new("x"), (2, 3)),
                            Node::new(Statement::Constant(Value::integer(42)), (6, 8))
                        ),
                        (
                            Node::new(Identifier::new("y"), (10, 11)),
                            Node::new(Statement::Constant(Value::string("foobar")), (14, 22))
                        )
                    ]),
                    (0, 24)
                )]
                .into()
            })
        );
    }

    #[test]
    fn map_with_one_property() {
        let input = "{ x = 42, }";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                nodes: [Node::new(
                    Statement::Map(vec![(
                        Node::new(Identifier::new("x"), (2, 3)),
                        Node::new(Statement::Constant(Value::integer(42)), (6, 8))
                    )]),
                    (0, 11)
                )]
                .into()
            })
        );
    }

    #[test]
    fn equal() {
        let input = "42 == 42";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                nodes: [Node::new(
                    Statement::BinaryOperation {
                        left: Box::new(Node::new(Statement::Constant(Value::integer(42)), (0, 2))),
                        operator: Node::new(BinaryOperator::Equal, (3, 5)),
                        right: Box::new(Node::new(Statement::Constant(Value::integer(42)), (6, 8)))
                    },
                    (0, 8)
                )]
                .into()
            })
        );
    }

    #[test]
    fn modulo() {
        let input = "42 % 2";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                nodes: [Node::new(
                    Statement::BinaryOperation {
                        left: Box::new(Node::new(Statement::Constant(Value::integer(42)), (0, 2))),
                        operator: Node::new(BinaryOperator::Modulo, (3, 4)),
                        right: Box::new(Node::new(Statement::Constant(Value::integer(2)), (5, 6)))
                    },
                    (0, 6)
                )]
                .into()
            })
        );
    }

    #[test]
    fn divide() {
        let input = "42 / 2";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                nodes: [Node::new(
                    Statement::BinaryOperation {
                        left: Box::new(Node::new(Statement::Constant(Value::integer(42)), (0, 2))),
                        operator: Node::new(BinaryOperator::Divide, (3, 4)),
                        right: Box::new(Node::new(Statement::Constant(Value::integer(2)), (5, 6)))
                    },
                    (0, 6)
                )]
                .into()
            })
        );
    }

    #[test]
    fn malformed_assignment() {
        let input = "false = 1";

        assert_eq!(
            parse(input),
            Err(ParseError::UnexpectedToken {
                actual: TokenOwned::Equal,
                position: (6, 7)
            })
        );
    }

    #[test]
    fn less_than() {
        let input = "1 < 2";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                nodes: [Node::new(
                    Statement::BinaryOperation {
                        left: Box::new(Node::new(Statement::Constant(Value::integer(1)), (0, 1))),
                        operator: Node::new(BinaryOperator::Less, (2, 3)),
                        right: Box::new(Node::new(Statement::Constant(Value::integer(2)), (4, 5))),
                    },
                    (0, 5)
                )]
                .into()
            })
        );
    }

    #[test]
    fn less_than_or_equal() {
        let input = "1 <= 2";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                nodes: [Node::new(
                    Statement::BinaryOperation {
                        left: Box::new(Node::new(Statement::Constant(Value::integer(1)), (0, 1))),
                        operator: Node::new(BinaryOperator::LessOrEqual, (2, 4)),
                        right: Box::new(Node::new(Statement::Constant(Value::integer(2)), (5, 6))),
                    },
                    (0, 6)
                )]
                .into()
            })
        );
    }

    #[test]
    fn greater_than_or_equal() {
        let input = "1 >= 2";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                nodes: [Node::new(
                    Statement::BinaryOperation {
                        left: Box::new(Node::new(Statement::Constant(Value::integer(1)), (0, 1))),
                        operator: Node::new(BinaryOperator::GreaterOrEqual, (2, 4)),
                        right: Box::new(Node::new(Statement::Constant(Value::integer(2)), (5, 6))),
                    },
                    (0, 6)
                )]
                .into()
            })
        );
    }

    #[test]
    fn greater_than() {
        let input = "1 > 2";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                nodes: [Node::new(
                    Statement::BinaryOperation {
                        left: Box::new(Node::new(Statement::Constant(Value::integer(1)), (0, 1))),
                        operator: Node::new(BinaryOperator::Greater, (2, 3)),
                        right: Box::new(Node::new(Statement::Constant(Value::integer(2)), (4, 5))),
                    },
                    (0, 5)
                )]
                .into()
            })
        );
    }

    #[test]
    fn subtract_negative_integers() {
        let input = "-1 - -2";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                nodes: [Node::new(
                    Statement::BinaryOperation {
                        left: Node::new(Statement::Constant(Value::integer(-1)), (0, 2)).into(),
                        operator: Node::new(BinaryOperator::Subtract, (3, 4)),
                        right: Node::new(Statement::Constant(Value::integer(-2)), (5, 7)).into()
                    },
                    (0, 7)
                )]
                .into()
            })
        );
    }

    #[test]
    fn string_concatenation() {
        let input = "\"Hello, \" + \"World!\"";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                nodes: [Node::new(
                    Statement::BinaryOperation {
                        left: Box::new(Node::new(
                            Statement::Constant(Value::string("Hello, ")),
                            (0, 9)
                        )),
                        operator: Node::new(BinaryOperator::Add, (10, 11)),
                        right: Box::new(Node::new(
                            Statement::Constant(Value::string("World!")),
                            (12, 20)
                        ))
                    },
                    (0, 20)
                )]
                .into()
            })
        );
    }

    #[test]
    fn string() {
        let input = "\"Hello, World!\"";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                nodes: [Node::new(
                    Statement::Constant(Value::string("Hello, World!")),
                    (0, 15)
                )]
                .into()
            })
        );
    }

    #[test]
    fn boolean() {
        let input = "true";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                nodes: [Node::new(Statement::Constant(Value::boolean(true)), (0, 4))].into()
            })
        );
    }

    #[test]
    fn property_access_function_call() {
        let input = "42.is_even()";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                nodes: [Node::new(
                    Statement::PropertyAccess(
                        Box::new(Node::new(Statement::Constant(Value::integer(42)), (0, 2))),
                        Box::new(Node::new(
                            Statement::BuiltInFunctionCall {
                                function: BuiltInFunction::IsEven,
                                type_arguments: None,
                                value_arguments: None
                            },
                            (3, 10)
                        )),
                    ),
                    (0, 10),
                )]
                .into()
            })
        );
    }

    #[test]
    fn list_access() {
        let input = "[1, 2, 3].0";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                nodes: [Node::new(
                    Statement::PropertyAccess(
                        Box::new(Node::new(
                            Statement::List(vec![
                                Node::new(Statement::Constant(Value::integer(1)), (1, 2)),
                                Node::new(Statement::Constant(Value::integer(2)), (4, 5)),
                                Node::new(Statement::Constant(Value::integer(3)), (7, 8)),
                            ]),
                            (0, 9)
                        )),
                        Box::new(Node::new(Statement::Constant(Value::integer(0)), (10, 11))),
                    ),
                    (0, 11),
                )]
                .into()
            })
        );
    }

    #[test]
    fn property_access() {
        let input = "a.b";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                nodes: [Node::new(
                    Statement::PropertyAccess(
                        Box::new(Node::new(
                            Statement::Identifier(Identifier::new("a")),
                            (0, 1)
                        )),
                        Box::new(Node::new(
                            Statement::Identifier(Identifier::new("b")),
                            (2, 3)
                        )),
                    ),
                    (0, 3),
                )]
                .into()
            })
        );
    }

    #[test]
    fn complex_list() {
        let input = "[1, 1 + 1, 2 + (4 * 10)]";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                nodes: [Node::new(
                    Statement::List(vec![
                        Node::new(Statement::Constant(Value::integer(1)), (1, 2)),
                        Node::new(
                            Statement::BinaryOperation {
                                left: Box::new(Node::new(
                                    Statement::Constant(Value::integer(1)),
                                    (4, 5)
                                )),
                                operator: Node::new(BinaryOperator::Add, (6, 7)),
                                right: Box::new(Node::new(
                                    Statement::Constant(Value::integer(1)),
                                    (8, 9)
                                ))
                            },
                            (4, 9)
                        ),
                        Node::new(
                            Statement::BinaryOperation {
                                left: Box::new(Node::new(
                                    Statement::Constant(Value::integer(2)),
                                    (11, 12)
                                )),
                                operator: Node::new(BinaryOperator::Add, (13, 14)),
                                right: Box::new(Node::new(
                                    Statement::BinaryOperation {
                                        left: Box::new(Node::new(
                                            Statement::Constant(Value::integer(4)),
                                            (16, 17)
                                        )),
                                        operator: Node::new(BinaryOperator::Multiply, (18, 19)),
                                        right: Box::new(Node::new(
                                            Statement::Constant(Value::integer(10)),
                                            (20, 22)
                                        ))
                                    },
                                    (15, 23)
                                ))
                            },
                            (11, 23)
                        ),
                    ]),
                    (0, 24),
                )]
                .into()
            })
        );
    }

    #[test]
    fn list() {
        let input = "[1, 2]";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                nodes: [Node::new(
                    Statement::List(vec![
                        Node::new(Statement::Constant(Value::integer(1)), (1, 2)),
                        Node::new(Statement::Constant(Value::integer(2)), (4, 5)),
                    ]),
                    (0, 6),
                )]
                .into()
            })
        );
    }

    #[test]
    fn empty_list() {
        let input = "[]";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                nodes: [Node::new(Statement::List(vec![]), (0, 2))].into()
            })
        );
    }

    #[test]
    fn float() {
        let input = "42.0";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                nodes: [Node::new(Statement::Constant(Value::float(42.0)), (0, 4))].into()
            })
        );
    }

    #[test]
    fn add() {
        let input = "1 + 2";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                nodes: [Node::new(
                    Statement::BinaryOperation {
                        left: Box::new(Node::new(Statement::Constant(Value::integer(1)), (0, 1))),
                        operator: Node::new(BinaryOperator::Add, (2, 3)),
                        right: Box::new(Node::new(Statement::Constant(Value::integer(2)), (4, 5)),)
                    },
                    (0, 5),
                )]
                .into()
            })
        );
    }

    #[test]
    fn multiply() {
        let input = "1 * 2";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                nodes: [Node::new(
                    Statement::BinaryOperation {
                        left: Box::new(Node::new(Statement::Constant(Value::integer(1)), (0, 1))),
                        operator: Node::new(BinaryOperator::Multiply, (2, 3)),
                        right: Box::new(Node::new(Statement::Constant(Value::integer(2)), (4, 5)),)
                    },
                    (0, 5),
                )]
                .into()
            })
        );
    }

    #[test]
    fn add_and_multiply() {
        let input = "1 + 2 * 3";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                nodes: [Node::new(
                    Statement::BinaryOperation {
                        left: Box::new(Node::new(Statement::Constant(Value::integer(1)), (0, 1))),
                        operator: Node::new(BinaryOperator::Add, (2, 3)),
                        right: Box::new(Node::new(
                            Statement::BinaryOperation {
                                left: Box::new(Node::new(
                                    Statement::Constant(Value::integer(2)),
                                    (4, 5)
                                )),
                                operator: Node::new(BinaryOperator::Multiply, (6, 7)),
                                right: Box::new(Node::new(
                                    Statement::Constant(Value::integer(3)),
                                    (8, 9)
                                ),)
                            },
                            (4, 9)
                        ),)
                    },
                    (0, 9),
                )]
                .into()
            })
        );
    }

    #[test]
    fn assignment() {
        let input = "a = 1 + 2 * 3";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                nodes: [Node::new(
                    Statement::Assignment {
                        identifier: Node::new(Identifier::new("a"), (0, 1)),
                        value_node: Box::new(Node::new(
                            Statement::BinaryOperation {
                                left: Box::new(Node::new(
                                    Statement::Constant(Value::integer(1)),
                                    (4, 5)
                                )),
                                operator: Node::new(BinaryOperator::Add, (6, 7)),
                                right: Box::new(Node::new(
                                    Statement::BinaryOperation {
                                        left: Box::new(Node::new(
                                            Statement::Constant(Value::integer(2)),
                                            (8, 9)
                                        )),
                                        operator: Node::new(BinaryOperator::Multiply, (10, 11)),
                                        right: Box::new(Node::new(
                                            Statement::Constant(Value::integer(3)),
                                            (12, 13)
                                        ),)
                                    },
                                    (8, 13)
                                ),)
                            },
                            (4, 13)
                        ),)
                    },
                    (0, 13),
                )]
                .into()
            })
        );
    }
}
