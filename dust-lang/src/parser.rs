//! Parsing tools.
//!
//! This module provides two parsing options:
//! - `parse` convenience function
//! - `Parser` struct, which parses the input a statement at a time
use std::{
    collections::VecDeque,
    error::Error,
    fmt::{self, Display, Formatter},
    marker::PhantomData,
    num::{ParseFloatError, ParseIntError},
    str::ParseBoolError,
};

use crate::{
    abstract_tree::*, AbstractSyntaxTree, BuiltInFunction, DustError, Identifier, LexError, Lexer,
    Node, Span, Statement, StructDefinition, Token, TokenKind, TokenOwned, Type, Value,
};

/// Parses the input into an abstract syntax tree.
///
/// # Examples
/// ```
/// # use dust_lang::*;
/// let tree = parse("x + 42").unwrap();
///
/// assert_eq!(
///     tree,
///     AbstractSyntaxTree {
///         nodes: [
///             Node::new(
///                 Statement::BinaryOperation {
///                     left: Box::new(Node::new(
///                         Statement::Identifier(Identifier::new("x")),
///                         (0, 1),
///                     )),
///                     operator: Node::new(
///                         BinaryOperator::Add,
///                         (2, 3)
///                     ),
///                     right: Box::new(Node::new(
///                         Statement::Constant(Value::integer(42)),
///                         (4, 6),
///                     ))
///                 },
///                 (0, 6),
///             )
///         ].into(),
///     },
/// );
/// ```
pub fn parse(source: &str) -> Result<AbstractSyntaxTree, DustError> {
    let lexer = Lexer::new();
    let mut parser = Parser::<Statement>::new(source, lexer);
    let mut nodes = VecDeque::new();

    loop {
        let node = parser
            .parse()
            .map_err(|parse_error| DustError::ParseError {
                parse_error,
                source,
            })?;

        nodes.push_back(node);

        if let Token::Eof = parser.current_token {
            break;
        }
    }

    Ok(AbstractSyntaxTree { statements: nodes })
}

pub fn parse_into<'src>(
    source: &'src str,
    tree: &mut AbstractSyntaxTree,
) -> Result<(), DustError<'src>> {
    let lexer = Lexer::new();
    let mut parser = Parser::<Statement>::new(source, lexer);

    loop {
        let node = parser
            .parse()
            .map_err(|parse_error| DustError::ParseError {
                parse_error,
                source,
            })?;

        tree.statements.push_back(node);

        if let Token::Eof = parser.current_token {
            break;
        }
    }

    Ok(())
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
/// let tree = AbstractSyntaxTree { nodes };
///
/// ```
pub struct Parser<'src, T> {
    source: &'src str,
    lexer: Lexer,
    current_token: Token<'src>,
    current_position: Span,
    product: PhantomData<T>,
}

impl<'src, T> Parser<'src, T> {
    pub fn new(source: &'src str, lexer: Lexer) -> Self {
        let mut lexer = lexer;
        let (current_token, current_position) =
            lexer.next_token(source).unwrap_or((Token::Eof, (0, 0)));

        Parser {
            source,
            lexer,
            current_token,
            current_position,
            product: PhantomData,
        }
    }

    pub fn parse(&mut self) -> Result<Statement, ParseError> {
        self.parse_next(0)
    }

    fn next_token(&mut self) -> Result<(), ParseError> {
        let (token, position) = self.lexer.next_token(self.source)?;

        self.current_token = token;
        self.current_position = position;

        Ok(())
    }

    fn parse_next(&mut self, mut precedence: u8) -> Result<Statement, ParseError> {
        // Parse a statement starting from the current node.
        let mut left = if self.current_token.is_prefix() {
            self.parse_prefix()?
        } else {
            self.parse_primary()?
        };

        // While the current token has a higher precedence than the given precedence
        while precedence < self.current_token.precedence() {
            // Give precedence to postfix operations
            left = if self.current_token.is_postfix() {
                let statement = self.parse_postfix(left)?;

                precedence = self.current_token.precedence();

                // Replace the left-hand side with the postfix operation
                statement
            } else {
                // Replace the left-hand side with the infix operation
                self.parse_infix(left)?
            };
        }

        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<Statement, ParseError> {
        log::trace!("Parsing {} as prefix operator", self.current_token);

        let operator_start = self.current_position.0;

        match self.current_token {
            Token::Bang => {
                self.next_token()?;

                let operand = self.parse_expression(0)?;
                let position = (operator_start, self.current_position.1);

                Ok(Statement::Expression(Expression::operator_expression(
                    OperatorExpression::Not(operand),
                    position,
                )))
            }
            Token::Minus => {
                self.next_token()?;

                let operand = self.parse_expression(0)?;
                let position = (operator_start, self.current_position.1);

                Ok(Statement::Expression(Expression::operator_expression(
                    OperatorExpression::Negation(operand),
                    position,
                )))
            }
            _ => Err(ParseError::UnexpectedToken {
                actual: self.current_token.to_owned(),
                position: self.current_position,
            }),
        }
    }

    fn parse_primary(&mut self) -> Result<Statement, ParseError> {
        log::trace!("Parsing {} as primary", self.current_token);

        let start_position = self.current_position;

        match self.current_token {
            Token::Async => {
                let block = self.parse_block()?;
                let position = (start_position.0, self.current_position.1);

                return Ok(Statement::block(block.inner, position));
            }
            Token::Boolean(text) => {
                self.next_token()?;

                let boolean = text.parse().map_err(|error| ParseError::Boolean {
                    error,
                    position: start_position,
                })?;
                let right_end = self.current_position.1;
                let statement = Statement::literal(
                    LiteralExpression::Boolean(boolean),
                    (start_position.0, right_end),
                );

                return Ok(statement);
            }
            Token::Float(text) => {
                self.next_token()?;

                let float = text.parse().map_err(|error| ParseError::Float {
                    error,
                    position: start_position,
                })?;
                let position = (start_position.0, self.current_position.1);

                return Ok(Statement::literal(
                    LiteralExpression::Float(float),
                    position,
                ));
            }
            Token::Identifier(text) => {
                let identifier = Identifier::new(text);
                let identifier_position = self.current_position;

                self.next_token()?;

                if let Token::LeftCurlyBrace = self.current_token {
                    self.next_token()?;

                    let mut fields = Vec::new();

                    loop {
                        if let Token::RightCurlyBrace = self.current_token {
                            let position = (start_position.0, self.current_position.1);

                            self.next_token()?;

                            return Ok(Statement::struct_expression(
                                StructExpression::Fields {
                                    name: Node::new(identifier, identifier_position),
                                    fields,
                                },
                                position,
                            ));
                        }

                        let field_name = self.parse_identifier()?;

                        if let Token::Colon = self.current_token {
                            self.next_token()?;
                        } else {
                            return Err(ParseError::ExpectedToken {
                                expected: TokenKind::Equal,
                                actual: self.current_token.to_owned(),
                                position: self.current_position,
                            });
                        }

                        let field_value = self.parse_expression(0)?;

                        fields.push((field_name, field_value));

                        if let Token::Comma = self.current_token {
                            self.next_token()?;
                        }
                    }
                }

                Ok(Statement::identifier_expression(
                    identifier,
                    identifier_position,
                ))
            }
            Token::Integer(text) => {
                self.next_token()?;

                let integer = text.parse::<i64>().map_err(|error| ParseError::Integer {
                    error,
                    position: start_position,
                })?;

                if let Token::DoubleDot = self.current_token {
                    self.next_token()?;

                    if let Token::Integer(range_end) = self.current_token {
                        let end_position = self.current_position;

                        self.next_token()?;

                        let range_end =
                            range_end
                                .parse::<i64>()
                                .map_err(|error| ParseError::Integer {
                                    error,
                                    position: end_position,
                                })?;

                        Ok(Statement::literal(
                            LiteralExpression::Range(integer, range_end),
                            (start_position.0, end_position.1),
                        ))
                    } else {
                        Err(ParseError::ExpectedToken {
                            expected: TokenKind::Integer,
                            actual: self.current_token.to_owned(),
                            position: (start_position.0, self.current_position.1),
                        })
                    }
                } else {
                    Ok(Statement::literal(
                        LiteralExpression::Integer(integer),
                        start_position,
                    ))
                }
            }
            Token::If => {
                self.next_token()?;

                let condition = self.parse_expression(0)?;
                let if_block = self.parse_block()?;
                let else_block = if let Token::Else = self.current_token {
                    self.next_token()?;

                    Some(self.parse_block()?)
                } else {
                    None
                };
                let position = (start_position.0, self.current_position.1);

                Ok(Statement::r#if(
                    If {
                        condition,
                        if_block,
                        else_block,
                    },
                    position,
                ))
            }
            Token::String(text) => {
                self.next_token()?;

                Ok(Statement::literal(
                    LiteralExpression::String(text.to_string()),
                    start_position,
                ))
            }
            Token::LeftCurlyBrace => {
                let block_node = self.parse_block()?;

                Ok(Statement::block(block_node.inner, block_node.position))
            }
            Token::LeftParenthesis => {
                self.next_token()?;

                let node = self.parse_expression(0)?;

                if let Token::RightParenthesis = self.current_token {
                    let position = (start_position.0, self.current_position.1);

                    self.next_token()?;

                    Ok(Statement::grouped(node, position))
                } else {
                    Err(ParseError::ExpectedToken {
                        expected: TokenKind::RightParenthesis,
                        actual: self.current_token.to_owned(),
                        position: self.current_position,
                    })
                }
            }
            Token::LeftSquareBrace => {
                self.next_token()?;

                let first_expression = self.parse_expression(0)?;

                if let Token::Semicolon = self.current_token {
                    self.next_token()?;

                    let repeat_operand = self.parse_expression(0)?;

                    if let Token::RightSquareBrace = self.current_token {
                        let position = (start_position.0, self.current_position.1);

                        self.next_token()?;

                        return Ok(Statement::list(
                            ListExpression::AutoFill {
                                length_operand: first_expression,
                                repeat_operand,
                            },
                            position,
                        ));
                    } else {
                        return Err(ParseError::ExpectedToken {
                            expected: TokenKind::RightSquareBrace,
                            actual: self.current_token.to_owned(),
                            position: self.current_position,
                        });
                    }
                }

                let mut expressions = vec![first_expression];

                loop {
                    if let Token::RightSquareBrace = self.current_token {
                        let position = (start_position.0, self.current_position.1);

                        self.next_token()?;

                        return Ok(Statement::list(
                            ListExpression::Ordered(expressions),
                            position,
                        ));
                    }

                    if let Token::Comma = self.current_token {
                        self.next_token()?;

                        continue;
                    }

                    let expression = self.parse_expression(0)?;

                    expressions.push(expression);
                }
            }
            Token::Struct => {
                self.next_token()?;

                let (name, name_end) = if let Token::Identifier(_) = self.current_token {
                    let end = self.current_position.1;

                    (self.parse_identifier()?, end)
                } else {
                    return Err(ParseError::ExpectedToken {
                        expected: TokenKind::Identifier,
                        actual: self.current_token.to_owned(),
                        position: self.current_position,
                    });
                };

                if let Token::LeftParenthesis = self.current_token {
                    self.next_token()?;

                    let mut types = Vec::new();

                    loop {
                        if let Token::RightParenthesis = self.current_token {
                            let position = (start_position.0, self.current_position.1);

                            self.next_token()?;

                            return Ok(Statement::struct_definition(
                                StructDefinition::Tuple { name, items: types },
                                position,
                            ));
                        }

                        if let Token::Comma = self.current_token {
                            self.next_token()?;

                            continue;
                        }

                        let type_node = self.parse_type()?;

                        types.push(type_node);
                    }
                }

                if let Token::LeftCurlyBrace = self.current_token {
                    self.next_token()?;

                    let mut fields = Vec::new();

                    loop {
                        if let Token::RightCurlyBrace = self.current_token {
                            let position = (start_position.0, self.current_position.1);

                            self.next_token()?;

                            return Ok(Statement::struct_definition(
                                StructDefinition::Fields { name, fields },
                                position,
                            ));
                        }

                        if let Token::Comma = self.current_token {
                            self.next_token()?;

                            continue;
                        }

                        let field_name = self.parse_identifier()?;

                        if let Token::Colon = self.current_token {
                            self.next_token()?;
                        } else {
                            return Err(ParseError::ExpectedToken {
                                expected: TokenKind::Colon,
                                actual: self.current_token.to_owned(),
                                position: self.current_position,
                            });
                        }

                        let field_type = self.parse_type()?;

                        fields.push((field_name, field_type));
                    }
                }

                Ok(Statement::struct_definition(
                    StructDefinition::Unit { name },
                    (start_position.0, name_end),
                ))
            }
            Token::While => {
                self.next_token()?;

                let condition = self.parse_expression(0)?;
                let block = self.parse_block()?;
                let position = (start_position.0, self.current_position.1);

                Ok(Statement::r#loop(
                    Loop::While { condition, block },
                    position,
                ))
            }
            _ => Err(ParseError::UnexpectedToken {
                actual: self.current_token.to_owned(),
                position: self.current_position,
            }),
        }
    }

    fn parse_infix(&mut self, left: Statement) -> Result<Statement, ParseError> {
        log::trace!("Parsing {} as infix operator", self.current_token);

        let left = if let Statement::Expression(expression) = left {
            expression
        } else {
            return Err(ParseError::ExpectedExpression { actual: left });
        };
        let operator_precedence = self.current_token.precedence()
            - if self.current_token.is_right_associative() {
                1
            } else {
                0
            };
        let left_start = left.position().0;

        if let Token::Equal = &self.current_token {
            self.next_token()?;

            let value = self.parse_expression(operator_precedence)?;
            let position = (left_start, value.position().1);

            return Ok(Statement::operator_expression(
                OperatorExpression::Assignment {
                    assignee: left,
                    value,
                },
                position,
            ));
        }

        if let Token::PlusEqual | Token::MinusEqual = &self.current_token {
            let math_operator = match self.current_token {
                Token::PlusEqual => MathOperator::Add,
                Token::MinusEqual => MathOperator::Subtract,
                _ => unreachable!(),
            };
            let operator = Node::new(math_operator, self.current_position);

            self.next_token()?;

            let value = self.parse_expression(operator_precedence)?;
            let position = (left_start, value.position().1);

            return Ok(Statement::operator_expression(
                OperatorExpression::CompoundAssignment {
                    assignee: left,
                    operator,
                    value,
                },
                position,
            ));
        }

        if let Token::Dot = &self.current_token {
            self.next_token()?;

            let field = self.parse_identifier()?;
            let position = (left_start, self.current_position.1);

            return Ok(Statement::field_access(
                FieldAccess {
                    container: left,
                    field,
                },
                position,
            ));
        }

        let math_operator = match &self.current_token {
            Token::Minus => Node::new(MathOperator::Subtract, self.current_position),
            Token::Plus => Node::new(MathOperator::Add, self.current_position),
            Token::Star => Node::new(MathOperator::Multiply, self.current_position),
            Token::Slash => Node::new(MathOperator::Divide, self.current_position),
            Token::Percent => Node::new(MathOperator::Modulo, self.current_position),
            _ => {
                return Err(ParseError::UnexpectedToken {
                    actual: self.current_token.to_owned(),
                    position: self.current_position,
                });
            }
        };

        self.next_token()?;

        let right = self.parse_expression(operator_precedence)?;
        let position = (left_start, right.position().1);

        Ok(Statement::operator_expression(
            OperatorExpression::Math {
                left,
                operator: math_operator,
                right,
            },
            position,
        ))
    }

    fn parse_postfix(&mut self, left: Statement) -> Result<Statement, ParseError> {
        log::trace!("Parsing {} as postfix operator", self.current_token);

        let left = if let Statement::Expression(expression) = left {
            expression
        } else {
            return Err(ParseError::ExpectedExpression { actual: left });
        };

        let statement = match &self.current_token {
            Token::LeftParenthesis => {
                self.next_token()?;

                let mut arguments = Vec::new();

                while self.current_token != Token::RightParenthesis {
                    let argument = self.parse_expression(0)?;

                    arguments.push(argument);

                    if let Token::Comma = self.current_token {
                        self.next_token()?;
                    } else {
                        break;
                    }
                }

                self.next_token()?;

                let position = (left.position().0, self.current_position.1);

                Statement::call_expression(
                    CallExpression {
                        function: left,
                        arguments,
                    },
                    position,
                )
            }
            Token::LeftSquareBrace => {
                let operator_start = self.current_position.0;

                self.next_token()?;

                let index = self.parse_expression(0)?;

                let operator_end = if let Token::RightSquareBrace = self.current_token {
                    let end = self.current_position.1;

                    self.next_token()?;

                    end
                } else {
                    return Err(ParseError::ExpectedToken {
                        expected: TokenKind::RightSquareBrace,
                        actual: self.current_token.to_owned(),
                        position: self.current_position,
                    });
                };

                let position = (left.position().0, operator_end);

                Statement::list_index(ListIndex { list: left, index }, position)
            }
            Token::Semicolon => {
                let position = (left.position().0, self.current_position.1);

                self.next_token()?;

                Statement::ExpressionNullified(Node::new(left, position))
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    actual: self.current_token.to_owned(),
                    position: self.current_position,
                });
            }
        };

        if self.current_token.is_postfix() {
            self.parse_postfix(statement)
        } else {
            Ok(statement)
        }
    }

    fn parse_expression(&mut self, precedence: u8) -> Result<Expression, ParseError> {
        log::trace!("Parsing expression");

        let statement = self.parse_next(precedence)?;

        if let Statement::Expression(expression) = statement {
            Ok(expression)
        } else {
            Err(ParseError::ExpectedExpression { actual: statement })
        }
    }

    fn parse_identifier(&mut self) -> Result<Node<Identifier>, ParseError> {
        if let Token::Identifier(text) = self.current_token {
            self.next_token()?;

            Ok(Node::new(Identifier::new(text), self.current_position))
        } else {
            Err(ParseError::ExpectedToken {
                expected: TokenKind::Identifier,
                actual: self.current_token.to_owned(),
                position: self.current_position,
            })
        }
    }

    fn parse_block(&mut self) -> Result<Node<Block>, ParseError> {
        let left_start = self.current_position.0;
        let is_async = if let Token::Async = self.current_token {
            self.next_token()?;

            true
        } else {
            false
        };

        if let Token::LeftCurlyBrace = self.current_token {
            self.next_token()?;
        } else {
            return Err(ParseError::ExpectedToken {
                expected: TokenKind::LeftCurlyBrace,
                actual: self.current_token.to_owned(),
                position: self.current_position,
            });
        }

        let mut statements = Vec::new();

        loop {
            if let Token::RightCurlyBrace = self.current_token {
                let position = (left_start, self.current_position.1);

                self.next_token()?;

                return if is_async {
                    Ok(Node::new(Block::Async(statements), position))
                } else {
                    Ok(Node::new(Block::Sync(statements), position))
                };
            }

            let statement = self.parse_next(0)?;

            statements.push(statement);
        }
    }

    fn parse_type(&mut self) -> Result<Node<Type>, ParseError> {
        let r#type = match self.current_token {
            Token::Bool => Type::Boolean,
            Token::FloatKeyword => Type::Float,
            Token::Int => Type::Integer,
            _ => {
                return Err(ParseError::ExpectedTokenMultiple {
                    expected: vec![TokenKind::Bool, TokenKind::FloatKeyword, TokenKind::Int],
                    actual: self.current_token.to_owned(),
                    position: self.current_position,
                });
            }
        };
        let position = self.current_position;

        self.next_token()?;

        Ok(Node::new(r#type, position))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ParseError {
    Boolean {
        error: ParseBoolError,
        position: Span,
    },
    Lex(LexError),
    ExpectedAssignment {
        actual: Statement,
    },
    ExpectedExpression {
        actual: Statement,
    },
    ExpectedIdentifier {
        actual: TokenOwned,
        position: Span,
    },
    ExpectedToken {
        expected: TokenKind,
        actual: TokenOwned,
        position: Span,
    },
    ExpectedTokenMultiple {
        expected: Vec<TokenKind>,
        actual: TokenOwned,
        position: Span,
    },
    UnexpectedToken {
        actual: TokenOwned,
        position: Span,
    },
    Float {
        error: ParseFloatError,
        position: Span,
    },
    Integer {
        error: ParseIntError,
        position: Span,
    },
}

impl From<LexError> for ParseError {
    fn from(v: LexError) -> Self {
        Self::Lex(v)
    }
}

impl ParseError {
    pub fn position(&self) -> Span {
        match self {
            ParseError::Boolean { position, .. } => *position,
            ParseError::ExpectedAssignment { actual } => actual.position(),
            ParseError::ExpectedExpression { actual } => actual.position(),
            ParseError::ExpectedIdentifier { position, .. } => *position,
            ParseError::ExpectedToken { position, .. } => *position,
            ParseError::ExpectedTokenMultiple { position, .. } => *position,
            ParseError::Float { position, .. } => *position,
            ParseError::Integer { position, .. } => *position,
            ParseError::Lex(error) => error.position(),
            ParseError::UnexpectedToken { position, .. } => *position,
        }
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Lex(error) => Some(error),
            _ => None,
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Boolean { error, .. } => write!(f, "{}", error),
            Self::ExpectedAssignment { .. } => write!(f, "Expected assignment"),
            Self::ExpectedExpression { .. } => write!(f, "Expected expression"),
            Self::ExpectedIdentifier { actual, .. } => {
                write!(f, "Expected identifier, found {actual}")
            }
            Self::ExpectedToken {
                expected, actual, ..
            } => write!(f, "Expected token {expected}, found {actual}"),
            Self::ExpectedTokenMultiple {
                expected, actual, ..
            } => {
                write!(f, "Expected one of")?;

                for (i, token_kind) in expected.iter().enumerate() {
                    if i == 0 {
                        write!(f, " {token_kind}")?;
                    } else if i == expected.len() - 1 {
                        write!(f, " or {token_kind}")?;
                    } else {
                        write!(f, ", {token_kind}")?;
                    }
                }

                write!(f, ", found {actual}")
            }
            Self::Float { error, .. } => write!(f, "{}", error),
            Self::Integer { error, .. } => write!(f, "{}", error),
            Self::Lex(error) => write!(f, "{}", error),
            Self::UnexpectedToken { actual, .. } => write!(f, "Unexpected token {actual}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Identifier, StructDefinition, Type};

    use super::*;

    #[test]
    fn mutable_variable() {
        let input = "mut x = false";

        assert_eq!(parse(input), todo!());
    }

    #[test]
    fn async_block() {
        let input = "async { x = 42; y = 4.0 }";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                statements: [Statement::block(
                    Block::Async(vec![Statement::operator_expression(
                        OperatorExpression::Assignment {
                            assignee: Expression::WithoutBlock(()),
                            value: ()
                        },
                        position
                    )]),
                    position
                )]
                .into()
            })
        );
    }

    #[test]
    fn tuple_struct_access() {
        let input = "(Foo(42, 'bar')).0";
        let mut tree = AbstractSyntaxTree::new();

        if parse_into(input, &mut tree).is_err() {
            println!("{tree:?}")
        }

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                statements: [Node::new(
                    Statement::BinaryOperation {
                        left: Box::new(Node::new(
                            Statement::Invokation {
                                invokee: Box::new(Node::new(
                                    Statement::Identifier(Identifier::new("Foo")),
                                    (1, 4)
                                )),
                                type_arguments: None,
                                value_arguments: Some(vec![
                                    Node::new(Statement::Constant(Value::integer(42)), (5, 7)),
                                    Node::new(Statement::Constant(Value::string("bar")), (9, 14))
                                ]),
                            },
                            (0, 16)
                        )),
                        operator: Node::new(BinaryOperator::FieldAccess, (16, 17)),
                        right: Box::new(Node::new(
                            Statement::Constant(Value::integer(0)),
                            (17, 18)
                        ))
                    },
                    (0, 18)
                )]
                .into()
            })
        );
    }

    #[test]
    fn fields_struct_instantiation() {
        let input = "Foo { a = 42, b = 4.0 }";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                statements: [Node::new(
                    Statement::FieldsStructInstantiation {
                        name: Node::new(Identifier::new("Foo"), (0, 3)),
                        fields: vec![
                            (
                                Node::new(Identifier::new("a"), (6, 7)),
                                Node::new(Statement::Constant(Value::integer(42)), (10, 12))
                            ),
                            (
                                Node::new(Identifier::new("b"), (14, 15)),
                                Node::new(Statement::Constant(Value::float(4.0)), (18, 21))
                            )
                        ]
                    },
                    (0, 23)
                )]
                .into()
            })
        );
    }

    #[test]
    fn fields_struct() {
        let input = "struct Foo { a: int, b: float }";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                statements: [Node::new(
                    Statement::StructDefinition(StructDefinition::Fields {
                        name: Node::new(Identifier::new("Foo"), (7, 10)),
                        fields: vec![
                            (
                                Node::new(Identifier::new("a"), (13, 14)),
                                Node::new(Type::Integer, (16, 19))
                            ),
                            (
                                Node::new(Identifier::new("b"), (21, 22)),
                                Node::new(Type::Float, (24, 29))
                            )
                        ]
                    }),
                    (0, 31)
                )]
                .into()
            })
        );
    }

    #[test]
    fn tuple_struct_instantiation() {
        let input = "struct Foo(int, float) Foo(1, 2.0)";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                statements: [
                    Node::new(
                        Statement::StructDefinition(StructDefinition::Tuple {
                            name: Node::new(Identifier::new("Foo"), (7, 10)),
                            items: vec![
                                Node::new(Type::Integer, (11, 14)),
                                Node::new(Type::Float, (16, 21))
                            ]
                        }),
                        (0, 22)
                    ),
                    Node::new(
                        Statement::Invokation {
                            invokee: Box::new(Node::new(
                                Statement::Identifier(Identifier::new("Foo")),
                                (23, 26)
                            )),
                            type_arguments: None,
                            value_arguments: Some(vec![
                                Node::new(Statement::Constant(Value::integer(1)), (27, 28)),
                                Node::new(Statement::Constant(Value::float(2.0)), (30, 33))
                            ])
                        },
                        (23, 34)
                    )
                ]
                .into()
            })
        );
    }

    #[test]
    fn tuple_struct() {
        let input = "struct Foo(int, float)";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                statements: [Node::new(
                    Statement::StructDefinition(StructDefinition::Tuple {
                        name: Node::new(Identifier::new("Foo"), (7, 10)),
                        items: vec![
                            Node::new(Type::Integer, (11, 14)),
                            Node::new(Type::Float, (16, 21))
                        ]
                    }),
                    (0, 22)
                )]
                .into()
            })
        );
    }

    #[test]
    fn unit_struct() {
        let input = "struct Foo";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                statements: [Node::new(
                    Statement::StructDefinition(StructDefinition::Unit {
                        name: Node::new(Identifier::new("Foo"), (7, 10)),
                    }),
                    (0, 10)
                )]
                .into()
            })
        );
    }

    #[test]
    fn list_index_nested() {
        let input = "[1, [2], 3][1][0]";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                statements: [Node::new(
                    Statement::BinaryOperation {
                        left: Box::new(Node::new(
                            Statement::BinaryOperation {
                                left: Box::new(Node::new(
                                    Statement::List(vec![
                                        Node::new(Statement::Constant(Value::integer(1)), (1, 2)),
                                        Node::new(
                                            Statement::List(vec![Node::new(
                                                Statement::Constant(Value::integer(2)),
                                                (5, 6)
                                            )]),
                                            (4, 7)
                                        ),
                                        Node::new(Statement::Constant(Value::integer(3)), (9, 10))
                                    ]),
                                    (0, 11)
                                )),
                                operator: Node::new(BinaryOperator::ListIndex, (11, 14)),
                                right: Box::new(Node::new(
                                    Statement::Constant(Value::integer(1)),
                                    (12, 13)
                                ))
                            },
                            (0, 15)
                        )),
                        operator: Node::new(BinaryOperator::ListIndex, (14, 17)),
                        right: Box::new(Node::new(
                            Statement::Constant(Value::integer(0)),
                            (15, 16)
                        ))
                    },
                    (0, 17)
                ),]
                .into()
            })
        );
    }

    #[test]
    fn map_property_nested() {
        let input = "{ x = { y = 42 } }.x.y";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                statements: [Node::new(
                    Statement::BinaryOperation {
                        left: Box::new(Node::new(
                            Statement::BinaryOperation {
                                left: Box::new(Node::new(
                                    Statement::Map(vec![(
                                        Node::new(Identifier::new("x"), (2, 3)),
                                        Node::new(
                                            Statement::Map(vec![(
                                                Node::new(Identifier::new("y"), (8, 9)),
                                                Node::new(
                                                    Statement::Constant(Value::integer(42)),
                                                    (12, 14)
                                                )
                                            )]),
                                            (6, 16)
                                        )
                                    )]),
                                    (0, 18)
                                )),
                                operator: Node::new(BinaryOperator::FieldAccess, (18, 19)),
                                right: Box::new(Node::new(
                                    Statement::Identifier(Identifier::new("x")),
                                    (19, 20)
                                ))
                            },
                            (0, 20)
                        )),
                        operator: Node::new(BinaryOperator::FieldAccess, (20, 21)),
                        right: Box::new(Node::new(
                            Statement::Identifier(Identifier::new("y")),
                            (21, 22)
                        ))
                    },
                    (0, 22)
                )]
                .into()
            })
        )
    }

    #[test]
    fn range() {
        let input = "0..42";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                statements: [Node::new(Statement::Constant(Value::range(0..42)), (0, 5))].into()
            })
        );
    }

    #[test]
    fn negate_variable() {
        let input = "a = 1; -a";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                statements: [
                    Node::new(
                        Statement::Nil(Box::new(Node::new(
                            Statement::Assignment {
                                identifier: Node::new(Identifier::new("a"), (0, 1)),
                                operator: Node::new(AssignmentOperator::Assign, (2, 3)),
                                value: Box::new(Node::new(
                                    Statement::Constant(Value::integer(1)),
                                    (4, 5)
                                )),
                            },
                            (0, 5)
                        ))),
                        (0, 6)
                    ),
                    Node::new(
                        Statement::UnaryOperation {
                            operator: Node::new(UnaryOperator::Negate, (7, 8)),
                            operand: Box::new(Node::new(
                                Statement::Identifier(Identifier::new("a")),
                                (8, 9)
                            )),
                        },
                        (7, 9)
                    )
                ]
                .into()
            })
        );
    }

    #[test]
    fn negate_expression() {
        let input = "-(1 + 1)";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                statements: [Node::new(
                    Statement::UnaryOperation {
                        operator: Node::new(UnaryOperator::Negate, (0, 1)),
                        operand: Box::new(Node::new(
                            Statement::BinaryOperation {
                                left: Box::new(Node::new(
                                    Statement::Constant(Value::integer(1)),
                                    (2, 3)
                                )),
                                operator: Node::new(BinaryOperator::Add, (4, 5)),
                                right: Box::new(Node::new(
                                    Statement::Constant(Value::integer(1)),
                                    (6, 7)
                                )),
                            },
                            (1, 8)
                        )),
                    },
                    (0, 8)
                )]
                .into()
            })
        );
    }

    #[test]
    fn not_expression() {
        let input = "!(1 > 42)";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                statements: [Node::new(
                    Statement::UnaryOperation {
                        operator: Node::new(UnaryOperator::Not, (0, 1)),
                        operand: Box::new(Node::new(
                            Statement::BinaryOperation {
                                left: Box::new(Node::new(
                                    Statement::Constant(Value::integer(1)),
                                    (2, 3)
                                )),
                                operator: Node::new(BinaryOperator::Greater, (4, 5)),
                                right: Box::new(Node::new(
                                    Statement::Constant(Value::integer(42)),
                                    (6, 8)
                                )),
                            },
                            (1, 9)
                        )),
                    },
                    (0, 9)
                )]
                .into()
            })
        );
    }

    #[test]
    fn not_variable() {
        let input = "a = false; !a";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                statements: [
                    Node::new(
                        Statement::Nil(Box::new(Node::new(
                            Statement::Assignment {
                                identifier: Node::new(Identifier::new("a"), (0, 1)),
                                operator: Node::new(AssignmentOperator::Assign, (2, 3)),
                                value: Box::new(Node::new(
                                    Statement::Constant(Value::boolean(false)),
                                    (4, 9)
                                )),
                            },
                            (0, 9)
                        ))),
                        (0, 10)
                    ),
                    Node::new(
                        Statement::UnaryOperation {
                            operator: Node::new(UnaryOperator::Not, (11, 12)),
                            operand: Box::new(Node::new(
                                Statement::Identifier(Identifier::new("a")),
                                (12, 13)
                            )),
                        },
                        (11, 13)
                    )
                ]
                .into()
            })
        );
    }

    #[test]
    fn r#if() {
        let input = "if x { y }";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                statements: [Node::new(
                    Statement::If {
                        condition: Box::new(Node::new(
                            Statement::Identifier(Identifier::new("x")),
                            (3, 4)
                        )),
                        body: Box::new(Node::new(
                            Statement::Block(vec![Node::new(
                                Statement::Identifier(Identifier::new("y")),
                                (7, 8)
                            )]),
                            (5, 10)
                        )),
                    },
                    (0, 10)
                )]
                .into()
            })
        );
    }

    #[test]
    fn if_else() {
        let input = "if x { y } else { z }";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                statements: [Node::new(
                    Statement::IfElse {
                        condition: Box::new(Node::new(
                            Statement::Identifier(Identifier::new("x")),
                            (3, 4)
                        )),
                        if_body: Box::new(Node::new(
                            Statement::Block(vec![Node::new(
                                Statement::Identifier(Identifier::new("y")),
                                (7, 8)
                            )]),
                            (5, 10)
                        )),
                        else_body: Box::new(Node::new(
                            Statement::Block(vec![Node::new(
                                Statement::Identifier(Identifier::new("z")),
                                (18, 19)
                            )]),
                            (16, 21)
                        )),
                    },
                    (0, 21)
                )]
                .into()
            })
        );
    }

    #[test]
    fn if_else_if_else() {
        let input = "if x { y } else if z { a } else { b }";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                statements: [Node::new(
                    Statement::IfElseIfElse {
                        condition: Box::new(Node::new(
                            Statement::Identifier(Identifier::new("x")),
                            (3, 4)
                        )),
                        if_body: Box::new(Node::new(
                            Statement::Block(vec![Node::new(
                                Statement::Identifier(Identifier::new("y")),
                                (7, 8)
                            )]),
                            (5, 10)
                        )),
                        else_ifs: vec![(
                            Node::new(Statement::Identifier(Identifier::new("z")), (19, 20)),
                            Node::new(
                                Statement::Block(vec![Node::new(
                                    Statement::Identifier(Identifier::new("a")),
                                    (23, 24)
                                )]),
                                (21, 26)
                            ),
                        )],
                        else_body: Box::new(Node::new(
                            Statement::Block(vec![Node::new(
                                Statement::Identifier(Identifier::new("b")),
                                (34, 35)
                            )]),
                            (32, 37)
                        )),
                    },
                    (0, 37)
                )]
                .into()
            })
        );
    }

    #[test]
    fn malformed_map() {
        let input = "{ x = 1, y = 2, z = 3; }";

        assert_eq!(
            parse(input),
            Err(DustError::ParseError {
                source: input,
                parse_error: ParseError::ExpectedAssignment {
                    actual: Node::new(
                        Statement::Nil(Box::new(Node::new(
                            Statement::Assignment {
                                identifier: Node::new(Identifier::new("z"), (16, 17)),
                                operator: Node::new(AssignmentOperator::Assign, (18, 19)),
                                value: Box::new(Node::new(
                                    Statement::Constant(Value::integer(3)),
                                    (20, 21)
                                )),
                            },
                            (16, 21)
                        ))),
                        (16, 22)
                    ),
                }
            })
        );
    }

    #[test]
    fn while_loop() {
        let input = "while x < 10 { x += 1 }";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                statements: [Node::new(
                    Statement::While {
                        condition: Box::new(Node::new(
                            Statement::BinaryOperation {
                                left: Box::new(Node::new(
                                    Statement::Identifier(Identifier::new("x")),
                                    (6, 7)
                                )),
                                operator: Node::new(BinaryOperator::Less, (8, 9)),
                                right: Box::new(Node::new(
                                    Statement::Constant(Value::integer(10)),
                                    (10, 12)
                                )),
                            },
                            (6, 12)
                        )),
                        body: Box::new(Node::new(
                            Statement::Block(vec![Node::new(
                                Statement::Assignment {
                                    identifier: Node::new(Identifier::new("x"), (15, 16)),
                                    operator: Node::new(AssignmentOperator::AddAssign, (17, 19)),
                                    value: Box::new(Node::new(
                                        Statement::Constant(Value::integer(1)),
                                        (20, 21)
                                    )),
                                },
                                (15, 21)
                            )]),
                            (13, 23)
                        )),
                    },
                    (0, 23)
                )]
                .into()
            })
        );
    }

    #[test]
    fn add_assign() {
        let input = "a += 1";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                statements: [Node::new(
                    Statement::Assignment {
                        identifier: Node::new(Identifier::new("a"), (0, 1)),
                        operator: Node::new(AssignmentOperator::AddAssign, (2, 4)),
                        value: Box::new(Node::new(Statement::Constant(Value::integer(1)), (5, 6))),
                    },
                    (0, 6)
                )]
                .into()
            })
        );
    }

    #[test]
    fn or() {
        let input = "true || false";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                statements: [Node::new(
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
            Err(DustError::ParseError {
                source: input,
                parse_error: ParseError::UnexpectedToken {
                    actual: TokenOwned::Semicolon,
                    position: (0, 1)
                }
            })
        );
    }

    #[test]
    fn block_with_one_statement() {
        let input = "{ 40 + 2 }";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                statements: [Node::new(
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
                statements: [Node::new(
                    Statement::Block(vec![
                        Node::new(
                            Statement::Nil(Box::new(Node::new(
                                Statement::Assignment {
                                    identifier: Node::new(Identifier::new("foo"), (2, 5)),
                                    operator: Node::new(AssignmentOperator::Assign, (6, 7)),
                                    value: Box::new(Node::new(
                                        Statement::Constant(Value::integer(42)),
                                        (8, 10)
                                    )),
                                },
                                (2, 10)
                            ),)),
                            (2, 11)
                        ),
                        Node::new(
                            Statement::Nil(Box::new(Node::new(
                                Statement::Assignment {
                                    identifier: Node::new(Identifier::new("bar"), (12, 15)),
                                    operator: Node::new(AssignmentOperator::Assign, (16, 17)),
                                    value: Box::new(Node::new(
                                        Statement::Constant(Value::integer(42)),
                                        (18, 20)
                                    )),
                                },
                                (12, 20)
                            ),)),
                            (12, 21)
                        ),
                        Node::new(
                            Statement::Assignment {
                                identifier: Node::new(Identifier::new("baz"), (22, 25)),
                                operator: Node::new(AssignmentOperator::Assign, (26, 27)),
                                value: Box::new(Node::new(
                                    Statement::Constant(Value::string("42")),
                                    (28, 32)
                                )),
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
                statements: [Node::new(Statement::Map(vec![]), (0, 2))].into()
            })
        );
    }

    #[test]
    fn map_with_trailing_comma() {
        let input = "{ foo = 42, bar = 42, baz = '42', }";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                statements: [Node::new(
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
    fn map_with_two_fields() {
        let input = "{ x = 42, y = 'foobar' }";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                statements: [Node::new(
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
    fn map_with_one_field() {
        let input = "{ x = 42 }";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                statements: [Node::new(
                    Statement::Map(vec![(
                        Node::new(Identifier::new("x"), (2, 3)),
                        Node::new(Statement::Constant(Value::integer(42)), (6, 8))
                    )]),
                    (0, 10)
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
                statements: [Node::new(
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
                statements: [Node::new(
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
                statements: [Node::new(
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
    fn less_than() {
        let input = "1 < 2";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                statements: [Node::new(
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
                statements: [Node::new(
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
                statements: [Node::new(
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
                statements: [Node::new(
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
                statements: [Node::new(
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
                statements: [Node::new(
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
                statements: [Node::new(
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
                statements: [Node::new(Statement::Constant(Value::boolean(true)), (0, 4))].into()
            })
        );
    }

    #[test]
    fn property_access_function_call() {
        let input = "42.is_even()";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                statements: [Node::new(
                    Statement::BuiltInFunctionCall {
                        function: BuiltInFunction::IsEven,
                        type_arguments: None,
                        value_arguments: Some(vec![Node::new(
                            Statement::Constant(Value::integer(42)),
                            (0, 2)
                        )])
                    },
                    (0, 10),
                )]
                .into()
            })
        );
    }

    #[test]
    fn list_index() {
        let input = "[1, 2, 3][0]";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                statements: [Node::new(
                    Statement::BinaryOperation {
                        left: Box::new(Node::new(
                            Statement::List(vec![
                                Node::new(Statement::Constant(Value::integer(1)), (1, 2)),
                                Node::new(Statement::Constant(Value::integer(2)), (4, 5)),
                                Node::new(Statement::Constant(Value::integer(3)), (7, 8)),
                            ]),
                            (0, 9)
                        )),
                        operator: Node::new(BinaryOperator::ListIndex, (9, 12)),
                        right: Box::new(Node::new(
                            Statement::Constant(Value::integer(0)),
                            (10, 11)
                        )),
                    },
                    (0, 12),
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
                statements: [Node::new(
                    Statement::BinaryOperation {
                        left: Box::new(Node::new(
                            Statement::Identifier(Identifier::new("a")),
                            (0, 1)
                        )),
                        operator: Node::new(BinaryOperator::FieldAccess, (1, 2)),
                        right: Box::new(Node::new(
                            Statement::Identifier(Identifier::new("b")),
                            (2, 3)
                        )),
                    },
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
                statements: [Node::new(
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
                statements: [Node::new(
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
                statements: [Node::new(Statement::List(vec![]), (0, 2))].into()
            })
        );
    }

    #[test]
    fn float() {
        let input = "42.0";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                statements: [Node::new(Statement::Constant(Value::float(42.0)), (0, 4))].into()
            })
        );
    }

    #[test]
    fn add() {
        let input = "1 + 2";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                statements: [Node::new(
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
                statements: [Node::new(
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
                statements: [Node::new(
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
                statements: [Node::new(
                    Statement::Assignment {
                        identifier: Node::new(Identifier::new("a"), (0, 1)),
                        operator: Node::new(AssignmentOperator::Assign, (2, 3)),
                        value: Box::new(Node::new(
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
