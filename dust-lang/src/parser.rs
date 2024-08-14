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
    abstract_tree::*, DustError, Identifier, LexError, Lexer, Span, Token, TokenKind, TokenOwned,
    Type,
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
    let mut parser = Parser::new(source, lexer);
    let mut nodes = VecDeque::new();

    loop {
        let node = parser
            .parse_statement()
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
    let mut parser = Parser::new(source, lexer);

    loop {
        let node = parser
            .parse_statement()
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
/// let source = "x = 42";
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
pub struct Parser<'src> {
    source: &'src str,
    lexer: Lexer,
    current_token: Token<'src>,
    current_position: Span,
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str, lexer: Lexer) -> Self {
        let mut lexer = lexer;
        let (current_token, current_position) =
            lexer.next_token(source).unwrap_or((Token::Eof, (0, 0)));

        Parser {
            source,
            lexer,
            current_token,
            current_position,
        }
    }

    pub fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        let start_position = self.current_position;

        if let Token::Struct = self.current_token {
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

            return Ok(Statement::struct_definition(
                StructDefinition::Unit { name },
                (start_position.0, name_end),
            ));
        }

        let expression = self.parse_expression(0)?;

        Ok(Statement::Expression(expression))
    }

    fn next_token(&mut self) -> Result<(), ParseError> {
        let (token, position) = self.lexer.next_token(self.source)?;

        self.current_token = token;
        self.current_position = position;

        Ok(())
    }

    fn parse_expression(&mut self, mut precedence: u8) -> Result<Expression, ParseError> {
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

    fn parse_prefix(&mut self) -> Result<Expression, ParseError> {
        log::trace!("Parsing {} as prefix operator", self.current_token);

        let operator_start = self.current_position.0;

        match self.current_token {
            Token::Bang => {
                self.next_token()?;

                let operand = self.parse_expression(0)?;
                let position = (operator_start, self.current_position.1);

                Ok(Expression::operator(
                    OperatorExpression::Not(operand),
                    position,
                ))
            }
            Token::Minus => {
                self.next_token()?;

                let operand = self.parse_expression(0)?;
                let position = (operator_start, self.current_position.1);

                Ok(Expression::operator(
                    OperatorExpression::Negation(operand),
                    position,
                ))
            }
            _ => Err(ParseError::UnexpectedToken {
                actual: self.current_token.to_owned(),
                position: self.current_position,
            }),
        }
    }

    fn parse_primary(&mut self) -> Result<Expression, ParseError> {
        log::trace!("Parsing {} as primary", self.current_token);

        let start_position = self.current_position;

        match self.current_token {
            Token::Async => {
                let block = self.parse_block()?;
                let position = (start_position.0, self.current_position.1);

                return Ok(Expression::block(block.inner, position));
            }
            Token::Boolean(text) => {
                self.next_token()?;

                let boolean = text.parse().map_err(|error| ParseError::Boolean {
                    error,
                    position: start_position,
                })?;
                let right_end = self.current_position.1;
                let statement = Expression::literal(
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

                Ok(Expression::literal(
                    LiteralExpression::Float(float),
                    position,
                ))
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

                            return Ok(Expression::r#struct(
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

                Ok(Expression::identifier(identifier, identifier_position))
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

                        Ok(Expression::literal(
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
                    Ok(Expression::literal(
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

                Ok(Expression::r#if(
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

                Ok(Expression::literal(
                    LiteralExpression::String(text.to_string()),
                    start_position,
                ))
            }
            Token::LeftCurlyBrace => {
                let block_node = self.parse_block()?;

                Ok(Expression::block(block_node.inner, block_node.position))
            }
            Token::LeftParenthesis => {
                self.next_token()?;

                let node = self.parse_expression(0)?;

                if let Token::RightParenthesis = self.current_token {
                    let position = (start_position.0, self.current_position.1);

                    self.next_token()?;

                    Ok(Expression::grouped(node, position))
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

                if let Token::RightSquareBrace = self.current_token {
                    let position = (start_position.0, self.current_position.1);

                    self.next_token()?;

                    return Ok(Expression::list(
                        ListExpression::Ordered(Vec::new()),
                        position,
                    ));
                }

                let first_expression = self.parse_expression(0)?;

                if let Token::Semicolon = self.current_token {
                    self.next_token()?;

                    let repeat_operand = self.parse_expression(0)?;

                    if let Token::RightSquareBrace = self.current_token {
                        let position = (start_position.0, self.current_position.1);

                        self.next_token()?;

                        return Ok(Expression::list(
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

                        return Ok(Expression::list(
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
            Token::While => {
                self.next_token()?;

                let condition = self.parse_expression(0)?;
                let block = self.parse_block()?;
                let position = (start_position.0, self.current_position.1);

                Ok(Expression::r#loop(
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

    fn parse_infix(&mut self, left: Expression) -> Result<Expression, ParseError> {
        log::trace!("Parsing {} as infix operator", self.current_token);

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

            return Ok(Expression::operator(
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

            return Ok(Expression::operator(
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

            return Ok(Expression::field_access(
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

        Ok(Expression::operator(
            OperatorExpression::Math {
                left,
                operator: math_operator,
                right,
            },
            position,
        ))
    }

    fn parse_postfix(&mut self, left: Expression) -> Result<Expression, ParseError> {
        log::trace!("Parsing {} as postfix operator", self.current_token);

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

                Expression::call(
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

                Expression::list_index(ListIndex { list: left, index }, position)
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

            let statement = self.parse_statement()?;

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
    use fmt::Write;

    use crate::{Identifier, Type};

    use super::*;

    #[test]
    fn mutable_variable() {
        let source = "mut x = false";

        assert_eq!(parse(source), todo!());
    }

    #[test]
    fn async_block() {
        let source = "async { x = 42; y = 4.0 }";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree {
                statements: [Statement::Expression(Expression::block(
                    Block::Async(vec![
                        Statement::Expression(Expression::operator(
                            OperatorExpression::Assignment {
                                assignee: Expression::identifier(Identifier::new("x"), (0, 0)),
                                value: Expression::literal(LiteralExpression::Integer(42), (0, 0)),
                            },
                            (0, 0)
                        )),
                        Statement::Expression(Expression::operator(
                            OperatorExpression::Assignment {
                                assignee: Expression::identifier(Identifier::new("y"), (0, 0)),
                                value: Expression::literal(LiteralExpression::Float(4.0), (0, 0)),
                            },
                            (0, 0)
                        ))
                    ]),
                    (0, 0)
                ),)]
                .into()
            })
        );
    }

    #[test]
    fn tuple_struct_access() {
        let source = "(Foo(42, 'bar')).0";

        assert_eq!(parse(source), todo!());
    }

    #[test]
    fn fields_struct_instantiation() {
        let source = "Foo { a = 42, b = 4.0 }";

        assert_eq!(parse(source), todo!());
    }

    #[test]
    fn fields_struct() {
        let source = "struct Foo { a: int, b: float }";

        assert_eq!(parse(source), todo!());
    }

    #[test]
    fn tuple_struct_instantiation() {
        let source = "struct Foo(int, float) Foo(1, 2.0)";

        assert_eq!(parse(source), todo!());
    }

    #[test]
    fn tuple_struct() {
        let source = "struct Foo(int, float)";

        assert_eq!(parse(source), todo!());
    }

    #[test]
    fn unit_struct() {
        let source = "struct Foo";

        assert_eq!(parse(source), todo!());
    }

    #[test]
    fn list_index_nested() {
        let source = "[1, [2], 3][1][0]";

        assert_eq!(parse(source), todo!());
    }

    #[test]
    fn map_property_nested() {
        let source = "{ x = { y = 42 } }.x.y";
    }

    #[test]
    fn range() {
        let source = "0..42";

        assert_eq!(parse(source), todo!());
    }

    #[test]
    fn negate_variable() {
        let source = "a = 1; -a";

        assert_eq!(parse(source), todo!());
    }

    #[test]
    fn negate_expression() {
        let source = "-(1 + 1)";

        assert_eq!(parse(source), todo!());
    }

    #[test]
    fn not_expression() {
        let source = "!(1 > 42)";

        assert_eq!(parse(source), todo!());
    }

    #[test]
    fn not_variable() {
        let source = "a = false; !a";

        assert_eq!(parse(source), todo!());
    }

    #[test]
    fn r#if() {
        let source = "if x { y }";

        assert_eq!(parse(source), todo!());
    }

    #[test]
    fn if_else() {
        let source = "if x { y } else { z }";

        assert_eq!(parse(source), todo!())
    }

    #[test]
    fn if_else_if_else() {
        let source = "if x { y } else if z { a } else { b }";

        assert_eq!(parse(source), todo!())
    }

    #[test]
    fn malformed_map() {
        let source = "{ x = 1, y = 2, z = 3; }";

        assert_eq!(parse(source), todo!())
    }

    #[test]
    fn while_loop() {
        let source = "while x < 10 { x += 1 }";

        assert_eq!(parse(source), todo!())
    }

    #[test]
    fn add_assign() {
        let source = "a += 1";

        assert_eq!(parse(source), todo!())
    }

    #[test]
    fn or() {
        let source = "true || false";

        assert_eq!(parse(source), todo!())
    }

    #[test]
    fn misplaced_semicolon() {
        let source = ";";

        assert_eq!(parse(source), todo!())
    }

    #[test]
    fn block_with_one_statement() {
        let source = "{ 40 + 2 }";

        assert_eq!(parse(source), todo!())
    }

    #[test]
    fn block_with_assignment() {
        let source = "{ foo = 42; bar = 42; baz = '42' }";

        assert_eq!(parse(source), todo!())
    }

    #[test]
    fn empty_map() {
        let source = "{}";

        assert_eq!(parse(source), todo!());
    }

    #[test]
    fn map_with_trailing_comma() {
        let source = "{ foo = 42, bar = 42, baz = '42', }";

        assert_eq!(parse(source), todo!());
    }

    #[test]
    fn map_with_two_fields() {
        let source = "{ x = 42, y = 'foobar' }";

        assert_eq!(parse(source), todo!());
    }

    #[test]
    fn map_with_one_field() {
        let source = "{ x = 42 }";

        assert_eq!(parse(source), todo!());
    }

    #[test]
    fn equal() {
        let source = "42 == 42";

        assert_eq!(parse(source), todo!());
    }

    #[test]
    fn modulo() {
        let source = "42 % 2";

        assert_eq!(parse(source), todo!());
    }

    #[test]
    fn divide() {
        let source = "42 / 2";

        assert_eq!(parse(source), todo!());
    }

    #[test]
    fn less_than() {
        let source = "1 < 2";

        assert_eq!(parse(source), todo!());
    }

    #[test]
    fn less_than_or_equal() {
        let source = "1 <= 2";

        assert_eq!(parse(source), todo!());
    }

    #[test]
    fn greater_than_or_equal() {
        let source = "1 >= 2";

        assert_eq!(parse(source), todo!());
    }

    #[test]
    fn greater_than() {
        let source = "1 > 2";

        assert_eq!(parse(source), todo!());
    }

    #[test]
    fn subtract_negative_integers() {
        let source = "-1 - -2";

        assert_eq!(parse(source), todo!());
    }

    #[test]
    fn string_concatenation() {
        let source = "\"Hello, \" + \"World!\"";

        assert_eq!(parse(source), todo!());
    }

    #[test]
    fn string() {
        let source = "\"Hello, World!\"";

        assert_eq!(parse(source), todo!());
    }

    #[test]
    fn boolean() {
        let source = "true";

        assert_eq!(parse(source), todo!());
    }

    #[test]
    fn property_access_function_call() {
        let source = "42.is_even()";

        assert_eq!(parse(source), todo!());
    }

    #[test]
    fn list_index() {
        let source = "[1, 2, 3][0]";

        assert_eq!(parse(source), todo!());
    }

    #[test]
    fn property_access() {
        let source = "a.b";

        assert_eq!(parse(source), todo!());
    }

    #[test]
    fn complex_list() {
        let source = "[1, 1 + 1, 2 + (4 * 10)]";

        assert_eq!(parse(source), todo!());
    }

    #[test]
    fn list() {
        let source = "[1, 2]";

        assert_eq!(parse(source), todo!());
    }

    #[test]
    fn empty_list() {
        let source = "[]";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::list(ListExpression::Ordered(vec![]), (0, 2)))
            ]))
        );
    }

    #[test]
    fn float() {
        let source = "42.0";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::literal(LiteralExpression::Float(42.0), (0, 4)))
            ]))
        );
    }

    #[test]
    fn add() {
        let source = "1 + 2";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::operator(
                    OperatorExpression::Math {
                        left: Expression::literal(LiteralExpression::Integer(1), (0, 1)),
                        operator: Node::new(MathOperator::Add, (2, 3)),
                        right: Expression::literal(LiteralExpression::Integer(2), (4, 5))
                    },
                    (0, 5)
                ))
            ]))
        );
    }

    #[test]
    fn multiply() {
        let source = "1 * 2";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::operator(
                    OperatorExpression::Math {
                        left: Expression::literal(LiteralExpression::Integer(1), (0, 1)),
                        operator: Node::new(MathOperator::Multiply, (2, 3)),
                        right: Expression::literal(LiteralExpression::Integer(2), (4, 5))
                    },
                    (0, 5)
                ))
            ]))
        );
    }

    #[test]
    fn add_and_multiply() {
        let source = "1 + 2 * 3";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::operator(
                    OperatorExpression::Math {
                        left: Expression::literal(LiteralExpression::Integer(1), (0, 1)),
                        operator: Node::new(MathOperator::Add, (2, 3)),
                        right: Expression::operator(
                            OperatorExpression::Math {
                                left: Expression::literal(LiteralExpression::Integer(2), (4, 5)),
                                operator: Node::new(MathOperator::Multiply, (6, 7)),
                                right: Expression::literal(LiteralExpression::Integer(3), (8, 9))
                            },
                            (4, 9)
                        )
                    },
                    (0, 5)
                ))
            ]))
        );
    }

    #[test]
    fn assignment() {
        let source = "a = 1 + 2 * 3";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::operator(
                    OperatorExpression::Assignment {
                        assignee: Expression::identifier(Identifier::new("a"), (0, 1)),
                        value: Expression::operator(
                            OperatorExpression::Math {
                                left: Expression::literal(LiteralExpression::Integer(1), (4, 5)),
                                operator: Node::new(MathOperator::Add, (6, 7)),
                                right: Expression::operator(
                                    OperatorExpression::Math {
                                        left: Expression::literal(
                                            LiteralExpression::Integer(2),
                                            (8, 9)
                                        ),
                                        operator: Node::new(MathOperator::Multiply, (10, 11)),
                                        right: Expression::literal(
                                            LiteralExpression::Integer(3),
                                            (12, 13)
                                        )
                                    },
                                    (8, 13)
                                )
                            },
                            (4, 13)
                        )
                    },
                    (0, 13)
                ))
            ]))
        );
    }
}
