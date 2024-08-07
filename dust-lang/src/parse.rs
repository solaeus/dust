/// Parsing tools.
///
/// This module provides two parsing options:
/// - `parse` convenience function
/// - `Parser` struct, which parses the input a statement at a time
use std::collections::VecDeque;

use crate::{
    abstract_tree::BuiltInFunction, AbstractSyntaxTree, LexError, Lexer, Node, Span, Statement,
    Token, Value,
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
///                 statement: Statement::Assign(
///                     Box::new(Node {
///                         statement: Statement::Identifier("x".into()),
///                         position: (0, 1),
///                     }),
///                     Box::new(Node {
///                         statement: Statement::Constant(Value::integer(42)),
///                         position: (4, 6),
///                     })
///                 ),
///                 position: (0, 6),
///             }
///         ].into(),
///     }),
/// );
/// ```
pub fn parse(input: &str) -> Result<AbstractSyntaxTree, ParseError> {
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
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
/// let lexer = Lexer::new(input);
/// let mut parser = Parser::new(lexer);
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
///     Into::<VecDeque<Node>>::into([
///         Node {
///             statement: Statement::Assign(
///                 Box::new(Node {
///                     statement: Statement::Identifier("x".into()),
///                     position: (0, 1),
///                 }),
///                 Box::new(Node {
///                     statement: Statement::Constant(Value::integer(42)),
///                     position: (4, 6),
///                 })
///             ),
///             position: (0, 6),
///         }
///     ]),
/// );
/// ```
pub struct Parser<'src> {
    lexer: Lexer<'src>,
    current: (Token, Span),
}

impl<'src> Parser<'src> {
    pub fn new(lexer: Lexer<'src>) -> Self {
        let mut lexer = lexer;
        let current = lexer.next_token().unwrap_or((Token::Eof, (0, 0)));

        Parser { lexer, current }
    }

    pub fn parse(&mut self) -> Result<Node, ParseError> {
        self.parse_node(0)
    }

    pub fn current(&self) -> &(Token, Span) {
        &self.current
    }

    fn next_token(&mut self) -> Result<(), ParseError> {
        self.current = self.lexer.next_token()?;

        Ok(())
    }

    fn parse_node(&mut self, precedence: u8) -> Result<Node, ParseError> {
        let left_node = self.parse_primary()?;
        let left_start = left_node.position.0;

        if precedence < self.current_precedence() {
            match &self.current {
                (Token::Plus, _) => {
                    self.next_token()?;

                    let right_node = self.parse_node(self.current_precedence())?;
                    let right_end = right_node.position.1;

                    return Ok(Node::new(
                        Statement::Add(Box::new(left_node), Box::new(right_node)),
                        (left_start, right_end),
                    ));
                }
                (Token::Star, _) => {
                    self.next_token()?;

                    let right_node = self.parse_node(self.current_precedence())?;
                    let right_end = right_node.position.1;

                    return Ok(Node::new(
                        Statement::Multiply(Box::new(left_node), Box::new(right_node)),
                        (left_start, right_end),
                    ));
                }
                (Token::Equal, _) => {
                    self.next_token()?;

                    let right_node = self.parse_node(self.current_precedence())?;
                    let right_end = right_node.position.1;

                    return Ok(Node::new(
                        Statement::Assign(Box::new(left_node), Box::new(right_node)),
                        (left_start, right_end),
                    ));
                }
                (Token::Dot, _) => {
                    self.next_token()?;

                    let right_node = self.parse_node(self.current_precedence())?;
                    let right_end = right_node.position.1;

                    return Ok(Node::new(
                        Statement::PropertyAccess(Box::new(left_node), Box::new(right_node)),
                        (left_start, right_end),
                    ));
                }
                _ => {}
            }
        }

        Ok(left_node)
    }

    fn parse_primary(&mut self) -> Result<Node, ParseError> {
        match self.current.clone() {
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
            (Token::Identifier(identifier), span) => {
                self.next_token()?;

                Ok(Node::new(Statement::Identifier(identifier), span))
            }
            (Token::LeftParenthesis, left_span) => {
                self.next_token()?;

                let node = self.parse_node(0)?;

                if let (Token::RightParenthesis, right_span) = self.current {
                    self.next_token()?;

                    Ok(Node::new(node.statement, (left_span.0, right_span.1)))
                } else {
                    Err(ParseError::ExpectedClosingParenthesis {
                        actual: self.current.0.clone(),
                        span: self.current.1,
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
                        return Err(ParseError::ExpectedClosingSquareBrace {
                            actual: self.current.0.clone(),
                            span: self.current.1,
                        });
                    }
                }
            }
            (Token::IsEven, left_span) => {
                self.next_token()?;

                if let (Token::LeftParenthesis, _) = self.current {
                    self.next_token()?;
                } else {
                    return Err(ParseError::ExpectedOpeningParenthesis {
                        actual: self.current.0.clone(),
                        span: self.current.1,
                    });
                }

                if let (Token::RightParenthesis, _) = self.current {
                    self.next_token()?;
                } else {
                    return Err(ParseError::ExpectedClosingParenthesis {
                        actual: self.current.0.clone(),
                        span: self.current.1,
                    });
                }

                Ok(Node::new(
                    Statement::BuiltInFunctionCall {
                        function: BuiltInFunction::IsEven,
                        type_arguments: None,
                        value_arguments: None,
                    },
                    left_span,
                ))
            }
            (Token::IsOdd, left_span) => {
                self.next_token()?;

                if let (Token::LeftParenthesis, _) = self.current {
                    self.next_token()?;
                } else {
                    return Err(ParseError::ExpectedOpeningParenthesis {
                        actual: self.current.0.clone(),
                        span: self.current.1,
                    });
                }

                if let (Token::RightParenthesis, _) = self.current {
                    self.next_token()?;
                } else {
                    return Err(ParseError::ExpectedClosingParenthesis {
                        actual: self.current.0.clone(),
                        span: self.current.1,
                    });
                }

                Ok(Node::new(
                    Statement::BuiltInFunctionCall {
                        function: BuiltInFunction::IsOdd,
                        type_arguments: None,
                        value_arguments: None,
                    },
                    left_span,
                ))
            }
            (Token::Length, left_span) => {
                self.next_token()?;

                if let (Token::LeftParenthesis, _) = self.current {
                    self.next_token()?;
                } else {
                    return Err(ParseError::ExpectedOpeningParenthesis {
                        actual: self.current.0.clone(),
                        span: self.current.1,
                    });
                }

                if let (Token::RightParenthesis, _) = self.current {
                    self.next_token()?;
                } else {
                    return Err(ParseError::ExpectedClosingParenthesis {
                        actual: self.current.0.clone(),
                        span: self.current.1,
                    });
                }

                Ok(Node::new(
                    Statement::BuiltInFunctionCall {
                        function: BuiltInFunction::Length,
                        type_arguments: None,
                        value_arguments: None,
                    },
                    left_span,
                ))
            }
            _ => Err(ParseError::UnexpectedToken(self.current.0.clone())),
        }
    }

    fn current_precedence(&self) -> u8 {
        match self.current.0 {
            Token::Dot => 4,
            Token::Equal => 3,
            Token::Plus => 1,
            Token::Star => 2,
            _ => 0,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ParseError {
    ExpectedClosingParenthesis { actual: Token, span: Span },
    ExpectedClosingSquareBrace { actual: Token, span: Span },
    ExpectedOpeningParenthesis { actual: Token, span: Span },
    LexError(LexError),
    UnexpectedToken(Token),
}

impl From<LexError> for ParseError {
    fn from(v: LexError) -> Self {
        Self::LexError(v)
    }
}

#[cfg(test)]
mod tests {
    use crate::Identifier;

    use super::*;

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
                            Statement::Add(
                                Box::new(Node::new(Statement::Constant(Value::integer(1)), (4, 5))),
                                Box::new(Node::new(Statement::Constant(Value::integer(1)), (8, 9))),
                            ),
                            (4, 9),
                        ),
                        Node::new(
                            Statement::Add(
                                Box::new(Node::new(
                                    Statement::Constant(Value::integer(2)),
                                    (11, 12)
                                )),
                                Box::new(Node::new(
                                    Statement::Multiply(
                                        Box::new(Node::new(
                                            Statement::Constant(Value::integer(4)),
                                            (16, 17)
                                        )),
                                        Box::new(Node::new(
                                            Statement::Constant(Value::integer(10)),
                                            (20, 22)
                                        )),
                                    ),
                                    (15, 23),
                                ),),
                            ),
                            (11, 23),
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
                    Statement::Add(
                        Box::new(Node::new(Statement::Constant(Value::integer(1)), (0, 1))),
                        Box::new(Node::new(Statement::Constant(Value::integer(2)), (4, 5))),
                    ),
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
                    Statement::Multiply(
                        Box::new(Node::new(Statement::Constant(Value::integer(1)), (0, 1))),
                        Box::new(Node::new(Statement::Constant(Value::integer(2)), (4, 5))),
                    ),
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
                    Statement::Add(
                        Box::new(Node::new(Statement::Constant(Value::integer(1)), (0, 1))),
                        Box::new(Node::new(
                            Statement::Multiply(
                                Box::new(Node::new(Statement::Constant(Value::integer(2)), (4, 5))),
                                Box::new(Node::new(Statement::Constant(Value::integer(3)), (8, 9))),
                            ),
                            (4, 9),
                        )),
                    ),
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
                    Statement::Assign(
                        Box::new(Node::new(
                            Statement::Identifier(Identifier::new("a")),
                            (0, 1)
                        )),
                        Box::new(Node::new(
                            Statement::Add(
                                Box::new(Node::new(Statement::Constant(Value::integer(1)), (4, 5))),
                                Box::new(Node::new(
                                    Statement::Multiply(
                                        Box::new(Node::new(
                                            Statement::Constant(Value::integer(2)),
                                            (8, 9)
                                        )),
                                        Box::new(Node::new(
                                            Statement::Constant(Value::integer(3)),
                                            (12, 13)
                                        )),
                                    ),
                                    (8, 13),
                                )),
                            ),
                            (4, 13),
                        )),
                    ),
                    (0, 13),
                )]
                .into()
            })
        );
    }
}
