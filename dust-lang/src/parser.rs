//! Parsing tools.
//!
//! This module provides two parsing options:
//! - `parse` convenience function
//! - `Parser` struct, which parses the input a statement at a time
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    num::{ParseFloatError, ParseIntError},
    str::ParseBoolError,
};

use crate::{ast::*, DustError, Identifier, LexError, Lexer, Token, TokenKind, TokenOwned, Type};

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
    let mut tree = AbstractSyntaxTree::new();

    parse_into(source, &mut tree)?;

    Ok(tree)
}

pub fn parse_into<'src>(
    source: &'src str,
    tree: &mut AbstractSyntaxTree,
) -> Result<(), DustError<'src>> {
    let mut parser = Parser::new(source);

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
    lexer: Lexer<'src>,
    current_token: Token<'src>,
    current_position: Span,
    mode: ParserMode,
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str) -> Self {
        let mut lexer = Lexer::new(source);
        let (current_token, current_position) = lexer.next_token().unwrap_or((Token::Eof, (0, 0)));

        Parser {
            lexer,
            current_token,
            current_position,
            mode: ParserMode::Normal,
        }
    }

    pub fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        let start_position = self.current_position;

        if let Token::Let = self.current_token {
            log::trace!("Parsing let statement");

            self.next_token()?;

            let is_mutable = if let Token::Mut = self.current_token {
                self.next_token()?;

                true
            } else {
                false
            };

            let identifier = self.parse_identifier()?;

            if let Token::Equal = self.current_token {
                self.next_token()?;
            } else {
                return Err(ParseError::ExpectedToken {
                    expected: TokenKind::Equal,
                    actual: self.current_token.to_owned(),
                    position: self.current_position,
                });
            }

            let value = self.parse_expression(0)?;

            let end = if let Token::Semicolon = self.current_token {
                let end = self.current_position.1;

                self.next_token()?;

                end
            } else {
                return Err(ParseError::ExpectedToken {
                    expected: TokenKind::Semicolon,
                    actual: self.current_token.to_owned(),
                    position: self.current_position,
                });
            };

            let r#let = if is_mutable {
                LetStatement::LetMut { identifier, value }
            } else {
                LetStatement::Let { identifier, value }
            };
            let position = (start_position.0, end);

            return Ok(Statement::Let(Node::new(r#let, position)));
        }

        if let Token::Struct = self.current_token {
            log::trace!("Parsing struct definition");

            self.next_token()?;

            let name = if let Token::Identifier(_) = self.current_token {
                self.parse_identifier()?
            } else {
                return Err(ParseError::ExpectedToken {
                    expected: TokenKind::Identifier,
                    actual: self.current_token.to_owned(),
                    position: self.current_position,
                });
            };

            if let Token::Semicolon = self.current_token {
                let end = self.current_position.1;

                self.next_token()?;

                return Ok(Statement::struct_definition(
                    StructDefinition::Unit { name },
                    (start_position.0, end),
                ));
            }

            if let Token::LeftParenthesis = self.current_token {
                self.next_token()?;

                let mut types = Vec::new();

                loop {
                    if let Token::RightParenthesis = self.current_token {
                        self.next_token()?;

                        if let Token::Semicolon = self.current_token {
                            break;
                        } else {
                            return Err(ParseError::ExpectedToken {
                                expected: TokenKind::Semicolon,
                                actual: self.current_token.to_owned(),
                                position: self.current_position,
                            });
                        }
                    }

                    let type_node = self.parse_type()?;

                    types.push(type_node);

                    if let Token::Comma = self.current_token {
                        self.next_token()?;

                        continue;
                    }
                }

                let position = (start_position.0, self.current_position.1);

                self.next_token()?;

                return if types.is_empty() {
                    Ok(Statement::struct_definition(
                        StructDefinition::Unit { name },
                        position,
                    ))
                } else {
                    Ok(Statement::struct_definition(
                        StructDefinition::Tuple { name, items: types },
                        position,
                    ))
                };
            }

            if let Token::LeftCurlyBrace = self.current_token {
                self.next_token()?;

                let mut fields = Vec::new();

                loop {
                    if let Token::RightCurlyBrace = self.current_token {
                        if let Token::Semicolon = self.current_token {
                            self.next_token()?;
                        }

                        break;
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

                    if let Token::Comma = self.current_token {
                        self.next_token()?;

                        continue;
                    }
                }

                let position = (start_position.0, self.current_position.1);

                self.next_token()?;

                return if fields.is_empty() {
                    Ok(Statement::struct_definition(
                        StructDefinition::Unit { name },
                        position,
                    ))
                } else {
                    Ok(Statement::struct_definition(
                        StructDefinition::Fields { name, fields },
                        position,
                    ))
                };
            }

            return Err(ParseError::ExpectedTokenMultiple {
                expected: vec![
                    TokenKind::LeftParenthesis,
                    TokenKind::LeftCurlyBrace,
                    TokenKind::Semicolon,
                ],
                actual: self.current_token.to_owned(),
                position: self.current_position,
            });
        }

        let expression = self.parse_expression(0)?;

        if let Token::Semicolon = self.current_token {
            let position = (start_position.0, self.current_position.1);

            self.next_token()?;

            Ok(Statement::ExpressionNullified(Node::new(
                expression, position,
            )))
        } else {
            Ok(Statement::Expression(expression))
        }
    }

    fn next_token(&mut self) -> Result<(), ParseError> {
        let (token, position) = self.lexer.next_token()?;

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

                Ok(Expression::not(operand, position))
            }
            Token::Minus => {
                self.next_token()?;

                let operand = self.parse_expression(0)?;
                let position = (operator_start, self.current_position.1);

                Ok(Expression::negation(operand, position))
            }
            _ => Err(ParseError::ExpectedTokenMultiple {
                expected: vec![TokenKind::Bang, TokenKind::Minus],
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
                let position = (start_position.0, block.position.1);

                Ok(Expression::block(block.inner, position))
            }
            Token::Boolean(text) => {
                self.next_token()?;

                let boolean = text.parse::<bool>().map_err(|error| ParseError::Boolean {
                    error,
                    position: start_position,
                })?;
                let statement = Expression::literal(boolean, start_position);

                Ok(statement)
            }
            Token::Float(text) => {
                self.next_token()?;

                let float = text.parse::<f64>().map_err(|error| ParseError::Float {
                    error,
                    position: start_position,
                })?;

                Ok(Expression::literal(float, start_position))
            }
            Token::Identifier(text) => {
                self.next_token()?;

                let identifier = Identifier::new(text);

                if let ParserMode::Condition = self.mode {
                    return Ok(Expression::identifier(identifier, start_position));
                }

                if let Token::LeftCurlyBrace = self.current_token {
                    self.next_token()?;

                    let name = Node::new(identifier, start_position);
                    let mut fields = Vec::new();

                    loop {
                        if let Token::RightCurlyBrace = self.current_token {
                            self.next_token()?;

                            break;
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

                        let field_value = self.parse_expression(0)?;

                        fields.push((field_name, field_value));

                        if let Token::Comma = self.current_token {
                            self.next_token()?;
                        }
                    }

                    let position = (start_position.0, self.current_position.1);

                    return Ok(Expression::r#struct(
                        StructExpression::Fields { name, fields },
                        position,
                    ));
                }

                Ok(Expression::identifier(identifier, start_position))
            }
            Token::Integer(text) => {
                self.next_token()?;

                let integer = text.parse::<i64>().map_err(|error| ParseError::Integer {
                    error,
                    position: start_position,
                })?;

                Ok(Expression::literal(integer, start_position))
            }
            Token::If => {
                self.next_token()?;

                let r#if = self.parse_if()?;
                let position = (start_position.0, self.current_position.1);

                Ok(Expression::r#if(r#if, position))
            }
            Token::String(text) => {
                self.next_token()?;

                Ok(Expression::literal(text.to_string(), start_position))
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

                    return Ok(Expression::list(Vec::new(), position));
                }

                let first_expression = self.parse_expression(0)?;

                if let Token::Semicolon = self.current_token {
                    self.next_token()?;

                    let repeat_operand = self.parse_expression(0)?;

                    if let Token::RightSquareBrace = self.current_token {
                        let position = (start_position.0, self.current_position.1);

                        self.next_token()?;

                        return Ok(Expression::auto_fill_list(
                            first_expression,
                            repeat_operand,
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

                        return Ok(Expression::list(expressions, position));
                    }

                    if let Token::Comma = self.current_token {
                        self.next_token()?;

                        continue;
                    }

                    let expression = self.parse_expression(0)?;

                    expressions.push(expression);
                }
            }
            Token::Map => {
                self.next_token()?;

                if let Token::LeftCurlyBrace = self.current_token {
                    self.next_token()?;
                } else {
                    return Err(ParseError::ExpectedToken {
                        expected: TokenKind::LeftCurlyBrace,
                        actual: self.current_token.to_owned(),
                        position: self.current_position,
                    });
                }

                let mut fields = Vec::new();

                loop {
                    if let Token::RightCurlyBrace = self.current_token {
                        let position = (start_position.0, self.current_position.1);

                        self.next_token()?;

                        return Ok(Expression::map(fields, position));
                    }

                    let field_name = self.parse_identifier()?;

                    if let Token::Equal = self.current_token {
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
            Token::While => {
                self.next_token()?;

                let condition = self.parse_expression(0)?;
                let block = self.parse_block()?;
                let position = (start_position.0, self.current_position.1);

                Ok(Expression::while_loop(condition, block, position))
            }
            _ => Err(ParseError::ExpectedTokenMultiple {
                expected: vec![
                    TokenKind::Async,
                    TokenKind::Boolean,
                    TokenKind::Float,
                    TokenKind::Identifier,
                    TokenKind::Integer,
                    TokenKind::If,
                    TokenKind::LeftCurlyBrace,
                    TokenKind::LeftParenthesis,
                    TokenKind::LeftSquareBrace,
                    TokenKind::String,
                    TokenKind::While,
                ],
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

            return Ok(Expression::assignment(left, value, position));
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
                    modifier: value,
                },
                position,
            ));
        }

        if let Token::DoubleDot = &self.current_token {
            self.next_token()?;

            let end = self.parse_expression(operator_precedence)?;
            let position = (left_start, end.position().1);

            return Ok(Expression::exclusive_range(left, end, position));
        }

        if let Token::Minus | Token::Plus | Token::Star | Token::Slash | Token::Percent =
            &self.current_token
        {
            let math_operator = match &self.current_token {
                Token::Minus => Node::new(MathOperator::Subtract, self.current_position),
                Token::Plus => Node::new(MathOperator::Add, self.current_position),
                Token::Star => Node::new(MathOperator::Multiply, self.current_position),
                Token::Slash => Node::new(MathOperator::Divide, self.current_position),
                Token::Percent => Node::new(MathOperator::Modulo, self.current_position),
                _ => unreachable!(),
            };

            self.next_token()?;

            let right = self.parse_expression(operator_precedence)?;
            let position = (left_start, right.position().1);

            return Ok(Expression::operator(
                OperatorExpression::Math {
                    left,
                    operator: math_operator,
                    right,
                },
                position,
            ));
        }

        if let Token::DoubleEqual
        | Token::BangEqual
        | Token::Less
        | Token::LessEqual
        | Token::Greater
        | Token::GreaterEqual = &self.current_token
        {
            let comparison_operator = match &self.current_token {
                Token::DoubleEqual => Node::new(ComparisonOperator::Equal, self.current_position),
                Token::BangEqual => Node::new(ComparisonOperator::NotEqual, self.current_position),
                Token::Less => Node::new(ComparisonOperator::LessThan, self.current_position),
                Token::LessEqual => {
                    Node::new(ComparisonOperator::LessThanOrEqual, self.current_position)
                }
                Token::Greater => Node::new(ComparisonOperator::GreaterThan, self.current_position),
                Token::GreaterEqual => Node::new(
                    ComparisonOperator::GreaterThanOrEqual,
                    self.current_position,
                ),
                _ => unreachable!(),
            };

            self.next_token()?;

            let right = self.parse_expression(operator_precedence)?;
            let position = (left_start, right.position().1);

            return Ok(Expression::operator(
                OperatorExpression::Comparison {
                    left,
                    operator: comparison_operator,
                    right,
                },
                position,
            ));
        }

        let logic_operator = match &self.current_token {
            Token::DoubleAmpersand => Node::new(LogicOperator::And, self.current_position),
            Token::DoublePipe => Node::new(LogicOperator::Or, self.current_position),
            _ => {
                return Err(ParseError::UnexpectedToken {
                    actual: self.current_token.to_owned(),
                    position: self.current_position,
                })
            }
        };

        self.next_token()?;

        let right = self.parse_expression(operator_precedence)?;
        let position = (left_start, right.position().1);

        Ok(Expression::operator(
            OperatorExpression::Logic {
                left,
                operator: logic_operator,
                right,
            },
            position,
        ))
    }

    fn parse_postfix(&mut self, left: Expression) -> Result<Expression, ParseError> {
        log::trace!("Parsing {} as postfix operator", self.current_token);

        let expression = match &self.current_token {
            Token::Dot => {
                self.next_token()?;

                if let Token::Integer(text) = &self.current_token {
                    let index = text.parse::<usize>().map_err(|error| ParseError::Integer {
                        error,
                        position: self.current_position,
                    })?;
                    let index_node = Node::new(index, self.current_position);
                    let position = (left.position().0, self.current_position.1);

                    self.next_token()?;

                    Expression::tuple_access(left, index_node, position)
                } else {
                    let field = self.parse_identifier()?;
                    let position = (left.position().0, self.current_position.1);

                    Expression::field_access(left, field, position)
                }
            }
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

                Expression::call(left, arguments, position)
            }
            Token::LeftSquareBrace => {
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

                Expression::list_index(left, index, position)
            }
            _ => {
                return Err(ParseError::ExpectedTokenMultiple {
                    expected: vec![
                        TokenKind::Dot,
                        TokenKind::LeftParenthesis,
                        TokenKind::LeftSquareBrace,
                    ],
                    actual: self.current_token.to_owned(),
                    position: self.current_position,
                });
            }
        };

        if self.current_token.is_postfix() {
            self.parse_postfix(expression)
        } else {
            Ok(expression)
        }
    }

    fn parse_if(&mut self) -> Result<IfExpression, ParseError> {
        // Assume that the "if" token has already been consumed

        self.mode = ParserMode::Condition;

        let condition = self.parse_expression(0)?;

        self.mode = ParserMode::Normal;

        let if_block = self.parse_block()?;

        if let Token::Else = self.current_token {
            self.next_token()?;

            let if_keyword_start = self.current_position.0;

            if let Token::If = self.current_token {
                self.next_token()?;

                let if_expression = self.parse_if()?;
                let position = (if_keyword_start, self.current_position.1);

                Ok(IfExpression::IfElse {
                    condition,
                    if_block,
                    r#else: ElseExpression::If(Node::new(Box::new(if_expression), position)),
                })
            } else {
                let else_block = self.parse_block()?;

                Ok(IfExpression::IfElse {
                    condition,
                    if_block,
                    r#else: ElseExpression::Block(else_block),
                })
            }
        } else {
            Ok(IfExpression::If {
                condition,
                if_block,
            })
        }
    }

    fn parse_identifier(&mut self) -> Result<Node<Identifier>, ParseError> {
        if let Token::Identifier(text) = self.current_token {
            let position = self.current_position;

            self.next_token()?;

            Ok(Node::new(Identifier::new(text), position))
        } else {
            Err(ParseError::ExpectedToken {
                expected: TokenKind::Identifier,
                actual: self.current_token.to_owned(),
                position: self.current_position,
            })
        }
    }

    fn parse_block(&mut self) -> Result<Node<BlockExpression>, ParseError> {
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
                    Ok(Node::new(BlockExpression::Async(statements), position))
                } else {
                    Ok(Node::new(BlockExpression::Sync(statements), position))
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
pub enum ParserMode {
    Condition,
    Normal,
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
    ExpectedIdentifierNode {
        actual: Expression,
    },
    ExpectedIdentifierToken {
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
            ParseError::ExpectedIdentifierNode { actual } => actual.position(),
            ParseError::ExpectedIdentifierToken { position, .. } => *position,
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
            Self::ExpectedIdentifierNode { actual } => {
                write!(f, "Expected identifier, found {actual}")
            }
            Self::ExpectedIdentifierToken { actual, .. } => {
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

    use crate::{Identifier, Type};

    use super::*;

    #[test]
    fn map_expression() {
        let source = "map { x = '1', y = 2, z = 3.0 }";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree {
                statements: [Statement::Expression(Expression::map(
                    vec![
                        (
                            Node::new(Identifier::new("x"), (6, 7)),
                            Expression::literal("1".to_string(), (10, 13)),
                        ),
                        (
                            Node::new(Identifier::new("y"), (15, 16)),
                            Expression::literal(2, (19, 20)),
                        ),
                        (
                            Node::new(Identifier::new("z"), (22, 23)),
                            Expression::literal(3.0, (26, 29)),
                        ),
                    ],
                    (0, 31),
                ))]
                .into(),
            })
        );
    }

    #[test]
    fn let_mut_while_loop() {
        let source = "let mut x = 0; while x < 10 { x += 1 }; x";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree {
                statements: [
                    Statement::Let(Node::new(
                        LetStatement::LetMut {
                            identifier: Node::new(Identifier::new("x"), (8, 9)),
                            value: Expression::literal(0, (12, 13)),
                        },
                        (0, 14),
                    )),
                    Statement::ExpressionNullified(Node::new(
                        Expression::while_loop(
                            Expression::comparison(
                                Expression::identifier(Identifier::new("x"), (21, 22)),
                                Node::new(ComparisonOperator::LessThan, (23, 24)),
                                Expression::literal(10, (25, 27)),
                                (21, 27),
                            ),
                            Node::new(
                                BlockExpression::Sync(vec![Statement::Expression(
                                    Expression::compound_assignment(
                                        Expression::identifier(Identifier::new("x"), (30, 31)),
                                        Node::new(MathOperator::Add, (32, 34)),
                                        Expression::literal(1, (35, 36)),
                                        (30, 36),
                                    ),
                                )]),
                                (28, 38),
                            ),
                            (15, 39),
                        ),
                        (15, 39)
                    )),
                    Statement::Expression(Expression::identifier(Identifier::new("x"), (40, 41)),),
                ]
                .into()
            })
        );
    }

    #[test]
    fn let_statement() {
        let source = "let x = 42;";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree {
                statements: [Statement::Let(Node::new(
                    LetStatement::Let {
                        identifier: Node::new(Identifier::new("x"), (4, 5)),
                        value: Expression::literal(42, (8, 10)),
                    },
                    (0, 11),
                ))]
                .into()
            })
        );
    }

    #[test]
    fn let_mut_statement() {
        let source = "let mut x = false;";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree {
                statements: [Statement::Let(Node::new(
                    LetStatement::LetMut {
                        identifier: Node::new(Identifier::new("x"), (8, 9)),
                        value: Expression::literal(false, (12, 17)),
                    },
                    (0, 18),
                ))]
                .into()
            })
        );
    }

    #[test]
    fn async_block() {
        let source = "async { x = 42; y = 4.0 }";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree {
                statements: [Statement::Expression(Expression::block(
                    BlockExpression::Async(vec![
                        Statement::ExpressionNullified(Node::new(
                            Expression::operator(
                                OperatorExpression::Assignment {
                                    assignee: Expression::identifier(Identifier::new("x"), (8, 9)),
                                    value: Expression::literal(42, (12, 14)),
                                },
                                (8, 14)
                            ),
                            (8, 15)
                        )),
                        Statement::Expression(Expression::operator(
                            OperatorExpression::Assignment {
                                assignee: Expression::identifier(Identifier::new("y"), (16, 17)),
                                value: Expression::literal(4.0, (20, 23)),
                            },
                            (16, 23)
                        ))
                    ]),
                    (0, 25)
                ))]
                .into()
            })
        );
    }

    #[test]
    fn tuple_struct_access() {
        let source = "Foo(42, 'bar').0";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::tuple_access(
                    Expression::call(
                        Expression::identifier(Identifier::new("Foo"), (0, 3)),
                        vec![
                            Expression::literal(42, (4, 6)),
                            Expression::literal("bar".to_string(), (8, 13)),
                        ],
                        (0, 15)
                    ),
                    Node::new(0, (15, 16)),
                    (0, 16)
                ))
            ]))
        );
    }

    #[test]
    fn fields_struct_instantiation() {
        let source = "Foo { a: 42, b: 4.0 }";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::r#struct(
                    StructExpression::Fields {
                        name: Node::new(Identifier::new("Foo"), (0, 3)),
                        fields: vec![
                            (
                                Node::new(Identifier::new("a"), (6, 7)),
                                Expression::literal(42, (9, 11)),
                            ),
                            (
                                Node::new(Identifier::new("b"), (13, 14)),
                                Expression::literal(4.0, (16, 19))
                            )
                        ]
                    },
                    (0, 21)
                ))
            ]))
        );
    }

    #[test]
    fn fields_struct() {
        let source = "struct Foo { a: int, b: float }";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::struct_definition(
                    StructDefinition::Fields {
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
                    },
                    (0, 31)
                )
            ]))
        );
    }

    #[test]
    fn tuple_struct_instantiation() {
        let source = "Foo(1, 2.0)";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::call(
                    Expression::identifier(Identifier::new("Foo"), (0, 3)),
                    vec![
                        Expression::literal(1, (4, 5)),
                        Expression::literal(2.0, (7, 10))
                    ],
                    (0, 11)
                ))
            ]))
        );
    }

    #[test]
    fn tuple_struct_definition() {
        let source = "struct Foo(int, float);";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::StructDefinition(Node::new(
                    StructDefinition::Tuple {
                        name: Node::new(Identifier::new("Foo"), (7, 10)),
                        items: vec![
                            Node::new(Type::Integer, (11, 14)),
                            Node::new(Type::Float, (16, 21)),
                        ],
                    },
                    (0, 23)
                ))
            ]))
        );
    }

    #[test]
    fn unit_struct() {
        let source = "struct Foo;";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::StructDefinition(Node::new(
                    StructDefinition::Unit {
                        name: Node::new(Identifier::new("Foo"), (7, 10)),
                    },
                    (0, 11)
                ))
            ]))
        );
    }

    #[test]
    fn list_index_nested() {
        let source = "[1, [2], 3][1][0]";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::list_index(
                    Expression::list_index(
                        Expression::list(
                            [
                                Expression::literal(1, (1, 2)),
                                Expression::list([Expression::literal(2, (5, 6))], (4, 7)),
                                Expression::literal(3, (9, 10)),
                            ],
                            (0, 11)
                        ),
                        Expression::literal(1, (12, 13)),
                        (0, 14)
                    ),
                    Expression::literal(0, (15, 16)),
                    (0, 17)
                ),)
            ]))
        );
    }

    #[test]
    fn range() {
        let source = "0..42";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::exclusive_range(
                    Expression::literal(0, (0, 1)),
                    Expression::literal(42, (3, 5)),
                    (0, 5)
                ))
            ]))
        );
    }

    #[test]
    fn negate_variable() {
        let source = "a = 1; -a";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::ExpressionNullified(Node::new(
                    Expression::operator(
                        OperatorExpression::Assignment {
                            assignee: Expression::identifier(Identifier::new("a"), (0, 1)),
                            value: Expression::literal(1, (4, 5)),
                        },
                        (0, 5)
                    ),
                    (0, 6)
                )),
                Statement::Expression(Expression::operator(
                    OperatorExpression::Negation(Expression::identifier(
                        Identifier::new("a"),
                        (8, 9)
                    )),
                    (7, 9)
                ))
            ]))
        );
    }

    #[test]
    fn negate_expression() {
        let source = "-(1 + 1)";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::operator(
                    OperatorExpression::Negation(Expression::grouped(
                        Expression::operator(
                            OperatorExpression::Math {
                                left: Expression::literal(1, (2, 3)),
                                operator: Node::new(MathOperator::Add, (4, 5)),
                                right: Expression::literal(1, (6, 7)),
                            },
                            (2, 7)
                        ),
                        (1, 8)
                    )),
                    (0, 8)
                ))
            ]))
        );
    }

    #[test]
    fn not_expression() {
        let source = "!(1 > 42)";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::operator(
                    OperatorExpression::Not(Expression::grouped(
                        Expression::operator(
                            OperatorExpression::Comparison {
                                left: Expression::literal(1, (2, 3)),
                                operator: Node::new(ComparisonOperator::GreaterThan, (4, 5)),
                                right: Expression::literal(42, (6, 8)),
                            },
                            (2, 8)
                        ),
                        (1, 9)
                    )),
                    (0, 9)
                ))
            ]))
        );
    }

    #[test]
    fn not_variable() {
        let source = "a = false; !a";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::ExpressionNullified(Node::new(
                    Expression::operator(
                        OperatorExpression::Assignment {
                            assignee: Expression::identifier(Identifier::new("a"), (0, 1)),
                            value: Expression::literal(false, (4, 9)),
                        },
                        (0, 9)
                    ),
                    (0, 10)
                )),
                Statement::Expression(Expression::operator(
                    OperatorExpression::Not(Expression::identifier(Identifier::new("a"), (12, 13))),
                    (11, 13)
                )),
            ]))
        );
    }

    #[test]
    fn r#if() {
        let source = "if x { y }";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::r#if(
                    IfExpression::If {
                        condition: Expression::identifier(Identifier::new("x"), (3, 4)),
                        if_block: Node::new(
                            BlockExpression::Sync(vec![Statement::Expression(
                                Expression::identifier(Identifier::new("y"), (7, 8))
                            )]),
                            (5, 10)
                        )
                    },
                    (0, 10)
                ))
            ]))
        );
    }

    #[test]
    fn if_else() {
        let source = "if x { y } else { z }";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::r#if(
                    IfExpression::IfElse {
                        condition: Expression::identifier(Identifier::new("x"), (3, 4)),
                        if_block: Node::new(
                            BlockExpression::Sync(vec![Statement::Expression(
                                Expression::identifier(Identifier::new("y"), (7, 8))
                            )]),
                            (5, 10)
                        ),
                        r#else: ElseExpression::Block(Node::new(
                            BlockExpression::Sync(vec![Statement::Expression(
                                Expression::identifier(Identifier::new("z"), (18, 19))
                            )]),
                            (16, 21)
                        ))
                    },
                    (0, 21)
                ))
            ]))
        );
    }

    #[test]
    fn if_else_if_else() {
        let source = "if x { y } else if z { a } else { b }";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::r#if(
                    IfExpression::IfElse {
                        condition: Expression::identifier(Identifier::new("x"), (3, 4)),
                        if_block: Node::new(
                            BlockExpression::Sync(vec![Statement::Expression(
                                Expression::identifier(Identifier::new("y"), (7, 8))
                            )]),
                            (5, 10)
                        ),
                        r#else: ElseExpression::If(Node::new(
                            Box::new(IfExpression::IfElse {
                                condition: Expression::identifier(Identifier::new("z"), (19, 20)),
                                if_block: Node::new(
                                    BlockExpression::Sync(vec![Statement::Expression(
                                        Expression::identifier(Identifier::new("a"), (23, 24))
                                    )]),
                                    (21, 26)
                                ),
                                r#else: ElseExpression::Block(Node::new(
                                    BlockExpression::Sync(vec![Statement::Expression(
                                        Expression::identifier(Identifier::new("b"), (34, 35))
                                    )]),
                                    (32, 37)
                                )),
                            }),
                            (16, 37)
                        )),
                    },
                    (0, 37)
                ))
            ]))
        )
    }

    #[test]
    fn while_loop() {
        let source = "while x < 10 { x += 1 }";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::while_loop(
                    Expression::operator(
                        OperatorExpression::Comparison {
                            left: Expression::identifier(Identifier::new("x"), (6, 7)),
                            operator: Node::new(ComparisonOperator::LessThan, (8, 9)),
                            right: Expression::literal(10, (10, 12)),
                        },
                        (6, 12)
                    ),
                    Node::new(
                        BlockExpression::Sync(vec![Statement::Expression(Expression::operator(
                            OperatorExpression::CompoundAssignment {
                                assignee: Expression::identifier(Identifier::new("x"), (15, 16)),
                                operator: Node::new(MathOperator::Add, (17, 19)),
                                modifier: Expression::literal(1, (20, 21)),
                            },
                            (15, 21)
                        ))]),
                        (13, 23)
                    ),
                    (0, 23)
                ))
            ]))
        )
    }

    #[test]
    fn add_assign() {
        let source = "a += 1";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::operator(
                    OperatorExpression::CompoundAssignment {
                        assignee: Expression::identifier(Identifier::new("a"), (0, 1)),
                        operator: Node::new(MathOperator::Add, (2, 4)),
                        modifier: Expression::literal(1, (5, 6)),
                    },
                    (0, 6)
                ))
            ]))
        )
    }

    #[test]
    fn or() {
        let source = "true || false";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::operator(
                    OperatorExpression::Logic {
                        left: Expression::literal(true, (0, 4)),
                        operator: Node::new(LogicOperator::Or, (5, 7)),
                        right: Expression::literal(false, (8, 13)),
                    },
                    (0, 13)
                ))
            ]))
        )
    }

    #[test]
    fn block_with_one_statement() {
        let source = "{ 40 + 2 }";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::block(
                    BlockExpression::Sync(vec![Statement::Expression(Expression::operator(
                        OperatorExpression::Math {
                            left: Expression::literal(40, (2, 4)),
                            operator: Node::new(MathOperator::Add, (5, 6)),
                            right: Expression::literal(2, (7, 8)),
                        },
                        (2, 8)
                    ))]),
                    (0, 10)
                ))
            ]))
        )
    }

    #[test]
    fn block_with_assignment() {
        let source = "{ foo = 42; bar = 42; baz = '42' }";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::block(
                    BlockExpression::Sync(vec![
                        Statement::ExpressionNullified(Node::new(
                            Expression::assignment(
                                Expression::identifier("foo", (2, 5)),
                                Expression::literal(42, (8, 10)),
                                (2, 10)
                            ),
                            (2, 11)
                        )),
                        Statement::ExpressionNullified(Node::new(
                            Expression::assignment(
                                Expression::identifier("bar", (12, 15)),
                                Expression::literal(42, (18, 20)),
                                (12, 20)
                            ),
                            (12, 21)
                        )),
                        Statement::Expression(Expression::assignment(
                            Expression::identifier("baz", (22, 25)),
                            Expression::literal("42", (28, 32)),
                            (22, 32)
                        ))
                    ]),
                    (0, 34)
                ))
            ]))
        )
    }

    #[test]
    fn equal() {
        let source = "42 == 42";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::comparison(
                    Expression::literal(42, (0, 2)),
                    Node::new(ComparisonOperator::Equal, (3, 5)),
                    Expression::literal(42, (6, 8)),
                    (0, 8)
                ))
            ]))
        );
    }

    #[test]
    fn less_than() {
        let source = "1 < 2";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::comparison(
                    Expression::literal(1, (0, 1)),
                    Node::new(ComparisonOperator::LessThan, (2, 3)),
                    Expression::literal(2, (4, 5)),
                    (0, 5)
                ))
            ]))
        );
    }

    #[test]
    fn less_than_or_equal() {
        let source = "1 <= 2";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::comparison(
                    Expression::literal(1, (0, 1)),
                    Node::new(ComparisonOperator::LessThanOrEqual, (2, 4)),
                    Expression::literal(2, (5, 6)),
                    (0, 6)
                ))
            ]))
        );
    }

    #[test]
    fn greater_than_or_equal() {
        let source = "1 >= 2";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::comparison(
                    Expression::literal(1, (0, 1)),
                    Node::new(ComparisonOperator::GreaterThanOrEqual, (2, 4)),
                    Expression::literal(2, (5, 6)),
                    (0, 6)
                ))
            ]))
        );
    }

    #[test]
    fn greater_than() {
        let source = "1 > 2";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::comparison(
                    Expression::literal(1, (0, 1)),
                    Node::new(ComparisonOperator::GreaterThan, (2, 3)),
                    Expression::literal(2, (4, 5)),
                    (0, 5)
                ))
            ]))
        );
    }

    #[test]
    fn subtract_negative_integers() {
        let source = "-1 - -2";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::math(
                    Expression::literal(-1, (0, 2)),
                    MathOperator::subtract((3, 4)),
                    Expression::literal(-2, (5, 7)),
                    (0, 7)
                ))
            ]))
        );
    }

    #[test]
    fn modulo() {
        let source = "42 % 2";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::math(
                    Expression::literal(42, (0, 2)),
                    MathOperator::modulo((3, 4)),
                    Expression::literal(2, (5, 6)),
                    (0, 6)
                ))
            ]))
        );
    }

    #[test]
    fn divide() {
        let source = "42 / 2";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::math(
                    Expression::literal(42, (0, 2)),
                    MathOperator::divide((3, 4)),
                    Expression::literal(2, (5, 6)),
                    (0, 6)
                ))
            ]))
        );
    }

    #[test]
    fn string_concatenation() {
        let source = "'Hello, ' + 'World!'";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::math(
                    Expression::literal("Hello, ", (0, 9)),
                    MathOperator::add((10, 11)),
                    Expression::literal("World!", (12, 20)),
                    (0, 20)
                ))
            ]))
        );
    }

    #[test]
    fn string() {
        let source = "\"Hello, World!\"";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::literal("Hello, World!", (0, 15)))
            ]))
        );
    }

    #[test]
    fn boolean() {
        let source = "true";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::literal(true, (0, 4)))
            ]))
        );
    }

    #[test]
    fn list_index() {
        let source = "[1, 2, 3][0]";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::list_index(
                    Expression::list(
                        [
                            Expression::literal(1, (1, 2)),
                            Expression::literal(2, (4, 5)),
                            Expression::literal(3, (7, 8)),
                        ],
                        (0, 9)
                    ),
                    Expression::literal(0, (10, 11)),
                    (0, 12)
                ))
            ]))
        );
    }

    #[test]
    fn property_access() {
        let source = "a.b";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::field_access(
                    Expression::identifier(Identifier::new("a"), (0, 1)),
                    Node::new(Identifier::new("b"), (2, 3)),
                    (0, 3)
                ))
            ]))
        );
    }

    #[test]
    fn complex_list() {
        let source = "[1, 1 + 1, 2 + (4 * 10)]";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::list(
                    [
                        Expression::literal(1, (1, 2)),
                        Expression::math(
                            Expression::literal(1, (4, 5)),
                            MathOperator::add((6, 7)),
                            Expression::literal(1, (8, 9)),
                            (4, 9)
                        ),
                        Expression::math(
                            Expression::literal(2, (11, 12)),
                            Node::new(MathOperator::Add, (13, 14)),
                            Expression::grouped(
                                Expression::math(
                                    Expression::literal(4, (16, 17)),
                                    MathOperator::multiply((18, 19)),
                                    Expression::literal(10, (20, 22)),
                                    (16, 22)
                                ),
                                (15, 23)
                            ),
                            (11, 23)
                        ),
                    ],
                    (0, 24)
                ))
            ]))
        );
    }

    #[test]
    fn list() {
        let source = "[1, 2]";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::list(
                    [
                        Expression::literal(1, (1, 2)),
                        Expression::literal(2, (4, 5))
                    ],
                    (0, 6)
                ))
            ]))
        );
    }

    #[test]
    fn empty_list() {
        let source = "[]";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::list(vec![], (0, 2)))
            ]))
        );
    }

    #[test]
    fn float() {
        let source = "42.0";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::literal(42.0, (0, 4)))
            ]))
        );
    }

    #[test]
    fn add() {
        let source = "1 + 2";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::math(
                    Expression::literal(1, (0, 1)),
                    MathOperator::add((2, 3)),
                    Expression::literal(2, (4, 5)),
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
                Statement::Expression(Expression::math(
                    Expression::literal(1, (0, 1)),
                    MathOperator::multiply((2, 3)),
                    Expression::literal(2, (4, 5)),
                    (0, 5)
                ),)
            ]))
        );
    }

    #[test]
    fn add_and_multiply() {
        let source = "1 + 2 * 3";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::math(
                    Expression::literal(1, (0, 1)),
                    MathOperator::add((2, 3)),
                    Expression::math(
                        Expression::literal(2, (4, 5)),
                        MathOperator::multiply((6, 7)),
                        Expression::literal(3, (8, 9)),
                        (4, 9)
                    ),
                    (0, 9)
                )),
            ]))
        );
    }

    #[test]
    fn assignment() {
        let source = "a = 1 + 2 * 3";

        assert_eq!(
            parse(source),
            Ok(AbstractSyntaxTree::with_statements([
                Statement::Expression(Expression::assignment(
                    Expression::identifier("a", (0, 1)),
                    Expression::math(
                        Expression::literal(1, (4, 5)),
                        MathOperator::add((6, 7)),
                        Expression::math(
                            Expression::literal(2, (8, 9)),
                            MathOperator::multiply((10, 11)),
                            Expression::literal(3, (12, 13)),
                            (8, 13)
                        ),
                        (4, 13)
                    ),
                    (0, 13)
                ))
            ]))
        );
    }
}
