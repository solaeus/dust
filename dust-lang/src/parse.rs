//! Parsing tools.
//!
//! This module provides two parsing options:
//! - `parse` convenience function
//! - `Parser` struct, which parses the input a statement at a time
use std::{
    collections::VecDeque,
    error::Error,
    fmt::{self, Display, Formatter},
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
        let left_node = self.parse_primary()?;
        let left_start = left_node.position.0;

        if precedence < self.current_precedence() {
            match &self.current {
                (Token::Dot, _) => {
                    self.next_token()?;

                    let right_node = self.parse_node(self.current_precedence())?;
                    let right_end = right_node.position.1;

                    return Ok(Node::new(
                        Statement::PropertyAccess(Box::new(left_node), Box::new(right_node)),
                        (left_start, right_end),
                    ));
                }
                (Token::Greater, _) => {
                    let operator = Node::new(BinaryOperator::Greater, self.current.1);

                    self.next_token()?;

                    let right_node = self.parse_node(self.current_precedence())?;
                    let right_end = right_node.position.1;

                    return Ok(Node::new(
                        Statement::BinaryOperation {
                            left: Box::new(left_node),
                            operator,
                            right: Box::new(right_node),
                        },
                        (left_start, right_end),
                    ));
                }
                (Token::GreaterEqual, _) => {
                    let operator = Node::new(BinaryOperator::GreaterOrEqual, self.current.1);

                    self.next_token()?;

                    let right_node = self.parse_node(self.current_precedence())?;
                    let right_end = right_node.position.1;

                    return Ok(Node::new(
                        Statement::BinaryOperation {
                            left: Box::new(left_node),
                            operator,
                            right: Box::new(right_node),
                        },
                        (left_start, right_end),
                    ));
                }
                (Token::Less, _) => {
                    let operator = Node::new(BinaryOperator::Less, self.current.1);

                    self.next_token()?;

                    let right_node = self.parse_node(self.current_precedence())?;
                    let right_end = right_node.position.1;

                    return Ok(Node::new(
                        Statement::BinaryOperation {
                            left: Box::new(left_node),
                            operator,
                            right: Box::new(right_node),
                        },
                        (left_start, right_end),
                    ));
                }
                (Token::LessEqual, _) => {
                    let operator = Node::new(BinaryOperator::LessOrEqual, self.current.1);

                    self.next_token()?;

                    let right_node = self.parse_node(self.current_precedence())?;
                    let right_end = right_node.position.1;

                    return Ok(Node::new(
                        Statement::BinaryOperation {
                            left: Box::new(left_node),
                            operator,
                            right: Box::new(right_node),
                        },
                        (left_start, right_end),
                    ));
                }
                (Token::Minus, _) => {
                    let operator = Node::new(BinaryOperator::Subtract, self.current.1);

                    self.next_token()?;

                    let right_node = self.parse_node(self.current_precedence())?;
                    let right_end = right_node.position.1;

                    return Ok(Node::new(
                        Statement::BinaryOperation {
                            left: Box::new(left_node),
                            operator,
                            right: Box::new(right_node),
                        },
                        (left_start, right_end),
                    ));
                }
                (Token::Plus, _) => {
                    let operator = Node::new(BinaryOperator::Add, self.current.1);

                    self.next_token()?;

                    let right_node = self.parse_node(self.current_precedence())?;
                    let right_end = right_node.position.1;

                    return Ok(Node::new(
                        Statement::BinaryOperation {
                            left: Box::new(left_node),
                            operator,
                            right: Box::new(right_node),
                        },
                        (left_start, right_end),
                    ));
                }
                (Token::Star, _) => {
                    let operator = Node::new(BinaryOperator::Multiply, self.current.1);

                    self.next_token()?;

                    let right_node = self.parse_node(self.current_precedence())?;
                    let right_end = right_node.position.1;

                    return Ok(Node::new(
                        Statement::BinaryOperation {
                            left: Box::new(left_node),
                            operator,
                            right: Box::new(right_node),
                        },
                        (left_start, right_end),
                    ));
                }
                (Token::Slash, _) => {
                    let operator = Node::new(BinaryOperator::Divide, self.current.1);

                    self.next_token()?;

                    let right_node = self.parse_node(self.current_precedence())?;
                    let right_end = right_node.position.1;

                    return Ok(Node::new(
                        Statement::BinaryOperation {
                            left: Box::new(left_node),
                            operator,
                            right: Box::new(right_node),
                        },
                        (left_start, right_end),
                    ));
                }
                _ => {}
            }
        }

        Ok(left_node)
    }

    fn parse_primary(&mut self) -> Result<Node<Statement>, ParseError> {
        match self.current {
            (Token::Boolean(boolean), span) => {
                self.next_token()?;

                Ok(Node::new(
                    Statement::Constant(Value::boolean(boolean)),
                    span,
                ))
            }
            (Token::Float(float), span) => {
                self.next_token()?;

                Ok(Node::new(Statement::Constant(Value::float(float)), span))
            }
            (Token::Integer(int), span) => {
                self.next_token()?;

                Ok(Node::new(Statement::Constant(Value::integer(int)), span))
            }
            (Token::Identifier(text), span) => {
                self.next_token()?;

                if let (Token::Equal, _) = self.current {
                    self.next_token()?;

                    let value = self.parse_node(0)?;
                    let right_end = value.position.1;

                    Ok(Node::new(
                        Statement::Assignment {
                            identifier: Node::new(Identifier::new(text), span),
                            value_node: Box::new(value),
                        },
                        (span.0, right_end),
                    ))
                } else {
                    Ok(Node::new(
                        Statement::Identifier(Identifier::new(text)),
                        span,
                    ))
                }
            }
            (Token::String(string), span) => {
                self.next_token()?;

                Ok(Node::new(Statement::Constant(Value::string(string)), span))
            }
            (Token::LeftCurlyBrace, left_span) => {
                self.next_token()?;

                let mut nodes = Vec::new();

                loop {
                    if let (Token::RightCurlyBrace, right_span) = self.current {
                        self.next_token()?;

                        return Ok(Node::new(
                            Statement::Map(nodes),
                            (left_span.0, right_span.1),
                        ));
                    }

                    let identifier = if let (Token::Identifier(text), right_span) = self.current {
                        self.next_token()?;

                        Node::new(Identifier::new(text), right_span)
                    } else {
                        return Err(ParseError::ExpectedIdentifier {
                            actual: self.current.0.to_owned(),
                            position: self.current.1,
                        });
                    };

                    if let Token::Equal = self.current.0 {
                        self.next_token()?;
                    } else {
                        return Err(ParseError::ExpectedToken {
                            expected: TokenOwned::Equal,
                            actual: self.current.0.to_owned(),
                            position: self.current.1,
                        });
                    }

                    let current_value_node = self.parse_node(0)?;

                    nodes.push((identifier, current_value_node));

                    if let Token::Comma = self.current.0 {
                        self.next_token()?;
                    }
                }
            }
            (Token::LeftParenthesis, left_span) => {
                self.next_token()?;

                let node = self.parse_node(0)?;

                if let (Token::RightParenthesis, right_span) = self.current {
                    self.next_token()?;

                    Ok(Node::new(node.inner, (left_span.0, right_span.1)))
                } else {
                    Err(ParseError::ExpectedToken {
                        expected: TokenOwned::RightParenthesis,
                        actual: self.current.0.to_owned(),
                        position: self.current.1,
                    })
                }
            }
            (Token::LeftSquareBrace, left_span) => {
                self.next_token()?;

                let mut nodes = Vec::new();

                loop {
                    if let (Token::RightSquareBrace, right_span) = self.current {
                        self.next_token()?;

                        return Ok(Node::new(
                            Statement::List(nodes),
                            (left_span.0, right_span.1),
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
                left_span,
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
                    left_span,
                ))
            }
            _ => Err(ParseError::UnexpectedToken {
                actual: self.current.0.to_owned(),
                position: self.current.1,
            }),
        }
    }

    fn current_precedence(&self) -> u8 {
        match self.current.0 {
            Token::Greater | Token::GreaterEqual | Token::Less | Token::LessEqual => 5,
            Token::Dot => 4,
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
    LexError {
        error: LexError,
        position: Span,
    },
    ExpectedIdentifier {
        actual: TokenOwned,
        position: (usize, usize),
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
}

impl ParseError {
    pub fn position(&self) -> Span {
        match self {
            Self::LexError { position, .. } => *position,
            Self::ExpectedIdentifier { position, .. } => *position,
            Self::ExpectedToken { position, .. } => *position,
            Self::UnexpectedToken { position, .. } => *position,
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
            Self::LexError { error, .. } => write!(f, "{}", error),
            Self::ExpectedIdentifier { actual, .. } => {
                write!(f, "Expected identifier, found {actual}")
            }
            Self::ExpectedToken {
                expected, actual, ..
            } => write!(f, "Expected token {expected}, found {actual}"),
            Self::UnexpectedToken { actual, .. } => write!(f, "Unexpected token {actual}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{abstract_tree::BinaryOperator, Identifier};

    use super::*;

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
    fn map() {
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
