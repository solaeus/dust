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
    AbstractSyntaxTree, AssignmentOperator, BinaryOperator, BuiltInFunction, DustError, Identifier,
    LexError, Lexer, Node, Span, Statement, StructDefinition, Token, TokenKind, TokenOwned, Type,
    UnaryOperator, Value,
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
            .parse()
            .map_err(|parse_error| DustError::ParseError {
                parse_error,
                source,
            })?;

        nodes.push_back(node);

        if let Token::Eof = parser.current.0 {
            break;
        }
    }

    Ok(AbstractSyntaxTree { nodes })
}

pub fn parse_into<'src>(
    source: &'src str,
    tree: &mut AbstractSyntaxTree,
) -> Result<(), DustError<'src>> {
    let lexer = Lexer::new();
    let mut parser = Parser::new(source, lexer);

    loop {
        let node = parser
            .parse()
            .map_err(|parse_error| DustError::ParseError {
                parse_error,
                source,
            })?;

        tree.nodes.push_back(node);

        if let Token::Eof = parser.current.0 {
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
pub struct Parser<'src> {
    source: &'src str,
    lexer: Lexer,
    current: (Token<'src>, Span),
    mode: ParserMode,
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str, lexer: Lexer) -> Self {
        let mut lexer = lexer;
        let current = lexer.next_token(source).unwrap_or((Token::Eof, (0, 0)));

        Parser {
            source,
            lexer,
            current,
            mode: ParserMode::None,
        }
    }

    pub fn current(&self) -> &(Token, Span) {
        &self.current
    }

    pub fn parse(&mut self) -> Result<Node<Statement>, ParseError> {
        self.parse_statement(0)
    }

    fn next_token(&mut self) -> Result<(), ParseError> {
        self.current = self.lexer.next_token(self.source)?;

        Ok(())
    }

    fn parse_statement(&mut self, mut precedence: u8) -> Result<Node<Statement>, ParseError> {
        // Parse a statement starting from the current node.
        let mut left = if self.current.0.is_prefix() {
            self.parse_prefix()?
        } else {
            self.parse_primary()?
        };

        // While the current token has a higher precedence than the given precedence
        while precedence < self.current.0.precedence() {
            // Give precedence to postfix operations
            left = if self.current.0.is_postfix() {
                let statement = self.parse_postfix(left)?;

                precedence = self.current.0.precedence();

                // Replace the left-hand side with the postfix operation
                statement
            } else {
                // Replace the left-hand side with the infix operation
                self.parse_infix(left)?
            };
        }

        log::trace!(
            "{}'s precedence is lower than or equal to {}",
            self.current.0,
            precedence
        );

        Ok(left)
    }

    fn parse_statement_in_mode(
        &mut self,
        mode: ParserMode,
        precedence: u8,
    ) -> Result<Node<Statement>, ParseError> {
        let old_mode = self.mode;
        self.mode = mode;

        let result = self.parse_statement(precedence);

        self.mode = old_mode;

        result
    }

    fn parse_prefix(&mut self) -> Result<Node<Statement>, ParseError> {
        log::trace!("Parsing {} as prefix operator", self.current.0);

        match self.current {
            (Token::Bang, position) => {
                self.next_token()?;

                let operand = Box::new(self.parse_statement(0)?);
                let operand_end = operand.position.1;

                Ok(Node::new(
                    Statement::UnaryOperation {
                        operator: Node::new(UnaryOperator::Not, position),
                        operand,
                    },
                    (position.0, operand_end),
                ))
            }
            (Token::Minus, position) => {
                self.next_token()?;

                let operand = Box::new(self.parse_statement(0)?);
                let operand_end = operand.position.1;

                Ok(Node::new(
                    Statement::UnaryOperation {
                        operator: Node::new(UnaryOperator::Negate, position),
                        operand,
                    },
                    (position.0, operand_end),
                ))
            }
            _ => Err(ParseError::UnexpectedToken {
                actual: self.current.0.to_owned(),
                position: self.current.1,
            }),
        }
    }

    fn parse_primary(&mut self) -> Result<Node<Statement>, ParseError> {
        log::trace!("Parsing {} as primary", self.current.0);

        match self.current {
            (Token::Async, position) => {
                self.next_token()?;

                if let Token::LeftCurlyBrace = self.current.0 {
                    self.next_token()?;
                } else {
                    return Err(ParseError::UnexpectedToken {
                        actual: self.current.0.to_owned(),
                        position: self.current.1,
                    });
                }

                let mut statements = Vec::new();

                loop {
                    if let Token::RightCurlyBrace = self.current.0 {
                        let right_end = self.current.1 .1;

                        self.next_token()?;

                        return Ok(Node::new(
                            Statement::AsyncBlock(statements),
                            (position.0, right_end),
                        ));
                    }

                    let statement = self.parse_statement(0)?;

                    statements.push(statement);
                }
            }
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
            (Token::Identifier(text), position) => {
                self.next_token()?;

                if let ParserMode::Condition = self.mode {
                    return Ok(Node::new(
                        Statement::Identifier(Identifier::new(text)),
                        position,
                    ));
                }

                if let Token::LeftCurlyBrace = self.current.0 {
                    self.next_token()?;

                    let mut fields = Vec::new();

                    loop {
                        if let Token::RightCurlyBrace = self.current.0 {
                            let right_end = self.current.1 .1;

                            self.next_token()?;

                            return Ok(Node::new(
                                Statement::FieldsStructInstantiation {
                                    name: Node::new(Identifier::new(text), position),
                                    fields,
                                },
                                (position.0, right_end),
                            ));
                        }

                        let field_name = self.parse_identifier()?;

                        if let Token::Equal = self.current.0 {
                            self.next_token()?;
                        } else {
                            return Err(ParseError::ExpectedToken {
                                expected: TokenKind::Equal,
                                actual: self.current.0.to_owned(),
                                position: self.current.1,
                            });
                        }

                        let field_value = self.parse_statement(0)?;

                        fields.push((field_name, field_value));

                        if let Token::Comma = self.current.0 {
                            self.next_token()?;
                        }
                    }
                }

                Ok(Node::new(
                    Statement::Identifier(Identifier::new(text)),
                    position,
                ))
            }
            (Token::Integer(text), position) => {
                self.next_token()?;

                let integer = text
                    .parse::<i64>()
                    .map_err(|error| ParseError::IntegerError { error, position })?;

                if let Token::DoubleDot = self.current.0 {
                    self.next_token()?;

                    if let Token::Integer(range_end) = self.current.0 {
                        self.next_token()?;

                        let range_end = range_end
                            .parse::<i64>()
                            .map_err(|error| ParseError::IntegerError { error, position })?;

                        Ok(Node::new(
                            Statement::Constant(Value::range(integer..range_end)),
                            (position.0, self.current.1 .1),
                        ))
                    } else {
                        Err(ParseError::ExpectedToken {
                            expected: TokenKind::Integer,
                            actual: self.current.0.to_owned(),
                            position: (position.0, self.current.1 .1),
                        })
                    }
                } else {
                    Ok(Node::new(
                        Statement::Constant(Value::integer(integer)),
                        position,
                    ))
                }
            }
            (Token::If, position) => {
                self.next_token()?;

                let condition = Box::new(self.parse_statement_in_mode(ParserMode::Condition, 0)?);
                let if_body = Box::new(self.parse_block()?);

                if let Token::Else = self.current.0 {
                    self.next_token()?;

                    if let Token::If = self.current.0 {
                        self.next_token()?;

                        let first_else_if = (
                            self.parse_statement_in_mode(ParserMode::Condition, 0)?,
                            self.parse_statement(0)?,
                        );
                        let mut else_ifs = vec![first_else_if];

                        loop {
                            if let Token::Else = self.current.0 {
                                self.next_token()?;
                            } else {
                                return Ok(Node::new(
                                    Statement::IfElseIf {
                                        condition,
                                        if_body,
                                        else_ifs,
                                    },
                                    position,
                                ));
                            }

                            if let Token::If = self.current.0 {
                                self.next_token()?;

                                let else_if = (
                                    self.parse_statement_in_mode(ParserMode::Condition, 0)?,
                                    self.parse_statement(0)?,
                                );

                                else_ifs.push(else_if);
                            } else {
                                let else_body = Box::new(self.parse_block()?);
                                let else_end = else_body.position.1;

                                return Ok(Node::new(
                                    Statement::IfElseIfElse {
                                        condition,
                                        if_body,
                                        else_ifs,
                                        else_body,
                                    },
                                    (position.0, else_end),
                                ));
                            }
                        }
                    } else {
                        let else_body = Box::new(self.parse_block()?);
                        let else_end = else_body.position.1;

                        Ok(Node::new(
                            Statement::IfElse {
                                condition,
                                if_body,
                                else_body,
                            },
                            (position.0, else_end),
                        ))
                    }
                } else {
                    let if_end = if_body.position.1;

                    self.mode = ParserMode::None;

                    Ok(Node::new(
                        Statement::If {
                            condition,
                            body: if_body,
                        },
                        (position.0, if_end),
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

                let first_node = self.parse_statement(0)?;

                // Determine whether the new statement is a block or a map
                //
                // If the first node is an assignment, this might be a map
                let mut statement = if let Statement::Assignment {
                    identifier: left,
                    operator:
                        Node {
                            inner: AssignmentOperator::Assign,
                            position: operator_position,
                        },
                    value: right,
                } = first_node.inner
                {
                    // If the current token is a comma or closing brace
                    if self.current.0 == Token::Comma || self.current.0 == Token::RightCurlyBrace {
                        // Allow commas after properties
                        if let Token::Comma = self.current.0 {
                            self.next_token()?;
                        }

                        // The new statement is a map
                        Statement::Map(vec![(left, *right)])
                    } else {
                        // Otherwise, the new statement is a block
                        Statement::Block(vec![Node::new(
                            Statement::Assignment {
                                identifier: left,
                                operator: Node::new(AssignmentOperator::Assign, operator_position),
                                value: right,
                            },
                            first_node.position,
                        )])
                    }
                // If the next node is not an assignment, the new statement is a block
                } else {
                    Statement::Block(vec![first_node])
                };

                loop {
                    // If a closing brace is found, return the new statement
                    if let (Token::RightCurlyBrace, right_position) = self.current {
                        self.next_token()?;

                        return Ok(Node::new(statement, (left_position.0, right_position.1)));
                    }

                    let next_node = self.parse_statement(0)?;

                    // If the new statement is already a block, add the next node to it
                    if let Some(block_statements) = statement.block_statements_mut() {
                        block_statements.push(next_node);

                        continue;
                    }

                    // If the new statement is already a map
                    if let Some(map_properties) = statement.map_properties_mut() {
                        // Expect the next node to be an assignment
                        if let Statement::Assignment {
                            identifier,
                            operator:
                                Node {
                                    inner: AssignmentOperator::Assign,
                                    ..
                                },
                            value,
                        } = next_node.inner
                        {
                            // Add the new property to the map
                            map_properties.push((identifier, *value));

                            // Allow commas after properties
                            if let Token::Comma = self.current.0 {
                                self.next_token()?;
                            }

                            continue;
                        } else {
                            return Err(ParseError::ExpectedAssignment { actual: next_node });
                        }
                    }
                }
            }
            (Token::LeftParenthesis, left_position) => {
                self.next_token()?;

                let node = self.parse_statement(0)?;

                if let (Token::RightParenthesis, right_position) = self.current {
                    self.next_token()?;

                    Ok(Node::new(node.inner, (left_position.0, right_position.1)))
                } else {
                    Err(ParseError::ExpectedToken {
                        expected: TokenKind::RightParenthesis,
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

                    let statement = self.parse_statement(0)?;

                    nodes.push(statement);
                }
            }
            (
                Token::IsEven
                | Token::IsOdd
                | Token::Length
                | Token::ReadLine
                | Token::ToString
                | Token::WriteLine,
                left_position,
            ) => {
                let function = match self.current.0 {
                    Token::IsEven => BuiltInFunction::IsEven,
                    Token::IsOdd => BuiltInFunction::IsOdd,
                    Token::Length => BuiltInFunction::Length,
                    Token::ReadLine => BuiltInFunction::ReadLine,
                    Token::ToString => BuiltInFunction::ToString,
                    Token::WriteLine => BuiltInFunction::WriteLine,
                    _ => unreachable!(),
                };

                self.next_token()?;

                if let (Token::LeftParenthesis, _) = self.current {
                    self.next_token()?;
                } else {
                    return Err(ParseError::ExpectedToken {
                        expected: TokenKind::LeftParenthesis,
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

                    if let Ok(node) = self.parse_statement(0) {
                        if let Some(ref mut arguments) = value_arguments {
                            arguments.push(node);
                        } else {
                            value_arguments = Some(vec![node]);
                        }
                    } else {
                        return Err(ParseError::ExpectedToken {
                            expected: TokenKind::RightParenthesis,
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
            (Token::Mut, left_position) => {
                self.next_token()?;

                let identifier = self.parse_identifier()?;

                if let (Token::Equal, _) = self.current {
                    self.next_token()?;
                } else {
                    return Err(ParseError::ExpectedToken {
                        expected: TokenKind::Equal,
                        actual: self.current.0.to_owned(),
                        position: self.current.1,
                    });
                }

                let value = Box::new(self.parse_statement(0)?);
                let value_end = value.position.1;

                Ok(Node::new(
                    Statement::MutAssignment { identifier, value },
                    (left_position.0, value_end),
                ))
            }
            (Token::Struct, left_position) => {
                self.next_token()?;

                let (name, name_end) = if let Token::Identifier(_) = self.current.0 {
                    let position = self.current.1 .1;

                    (self.parse_identifier()?, position)
                } else {
                    return Err(ParseError::ExpectedToken {
                        expected: TokenKind::Identifier,
                        actual: self.current.0.to_owned(),
                        position: self.current.1,
                    });
                };

                if let Token::LeftParenthesis = self.current.0 {
                    self.next_token()?;

                    let mut types = Vec::new();

                    loop {
                        if let (Token::RightParenthesis, right_position) = self.current {
                            self.next_token()?;

                            return Ok(Node::new(
                                Statement::StructDefinition(StructDefinition::Tuple {
                                    name,
                                    items: types,
                                }),
                                (left_position.0, right_position.1),
                            ));
                        }

                        if let (Token::Comma, _) = self.current {
                            self.next_token()?;
                            continue;
                        }

                        let type_node = self.parse_type()?;

                        types.push(type_node);
                    }
                }

                if let Token::LeftCurlyBrace = self.current.0 {
                    self.next_token()?;

                    let mut fields = Vec::new();

                    loop {
                        if let (Token::RightCurlyBrace, right_position) = self.current {
                            self.next_token()?;

                            return Ok(Node::new(
                                Statement::StructDefinition(StructDefinition::Fields {
                                    name,
                                    fields,
                                }),
                                (left_position.0, right_position.1),
                            ));
                        }

                        if let (Token::Comma, _) = self.current {
                            self.next_token()?;
                            continue;
                        }

                        let field_name = self.parse_identifier()?;

                        if let (Token::Colon, _) = self.current {
                            self.next_token()?;
                        } else {
                            return Err(ParseError::ExpectedToken {
                                expected: TokenKind::Colon,
                                actual: self.current.0.to_owned(),
                                position: self.current.1,
                            });
                        }

                        let field_type = self.parse_type()?;

                        fields.push((field_name, field_type));
                    }
                }

                Ok(Node::new(
                    Statement::StructDefinition(StructDefinition::Unit { name }),
                    (left_position.0, name_end),
                ))
            }
            (Token::While, left_position) => {
                self.next_token()?;

                let condition = self.parse_statement_in_mode(ParserMode::Condition, 0)?;

                let body = self.parse_block()?;
                let body_end = body.position.1;

                Ok(Node::new(
                    Statement::While {
                        condition: Box::new(condition),
                        body: Box::new(body),
                    },
                    (left_position.0, body_end),
                ))
            }
            _ => Err(ParseError::UnexpectedToken {
                actual: self.current.0.to_owned(),
                position: self.current.1,
            }),
        }
    }

    fn parse_infix(&mut self, left: Node<Statement>) -> Result<Node<Statement>, ParseError> {
        log::trace!("Parsing {} as infix operator", self.current.0);

        let operator_precedence = self.current.0.precedence()
            - if self.current.0.is_right_associative() {
                1
            } else {
                0
            };
        let left_start = left.position.0;

        if let Token::Equal | Token::PlusEqual | Token::MinusEqual = &self.current.0 {
            let operator = match self.current.0 {
                Token::Equal => AssignmentOperator::Assign,
                Token::PlusEqual => AssignmentOperator::AddAssign,
                Token::MinusEqual => AssignmentOperator::SubtractAssign,
                _ => unreachable!(),
            };
            let operator_position = self.current.1;

            self.next_token()?;

            let identifier = if let Statement::Identifier(identifier) = left.inner {
                Node::new(identifier, left.position)
            } else {
                return Err(ParseError::ExpectedToken {
                    expected: TokenKind::Identifier,
                    actual: self.current.0.to_owned(),
                    position: self.current.1,
                });
            };
            let right = self.parse_statement(operator_precedence)?;
            let right_end = right.position.1;

            return Ok(Node::new(
                Statement::Assignment {
                    identifier,
                    operator: Node::new(operator, operator_position),
                    value: Box::new(right),
                },
                (left_start, right_end),
            ));
        }

        if let Token::Dot = &self.current.0 {
            let operator_position = self.current.1;

            self.next_token()?;

            let right = self.parse_statement(operator_precedence)?;
            let right_end = right.position.1;

            if let Statement::BuiltInFunctionCall {
                function,
                type_arguments,
                value_arguments,
            } = right.inner
            {
                let value_arguments = if let Some(mut arguments) = value_arguments {
                    arguments.insert(0, left);

                    Some(arguments)
                } else {
                    Some(vec![left])
                };

                return Ok(Node::new(
                    Statement::BuiltInFunctionCall {
                        function,
                        type_arguments,
                        value_arguments,
                    },
                    (left_start, right_end),
                ));
            }

            if let Statement::Invokation {
                invokee: function,
                type_arguments,
                value_arguments,
            } = right.inner
            {
                let value_arguments = if let Some(mut arguments) = value_arguments {
                    arguments.insert(0, left);

                    Some(arguments)
                } else {
                    Some(vec![left])
                };

                return Ok(Node::new(
                    Statement::Invokation {
                        invokee: function,
                        type_arguments,
                        value_arguments,
                    },
                    (left_start, right_end),
                ));
            }

            return Ok(Node::new(
                Statement::BinaryOperation {
                    left: Box::new(left),
                    operator: Node::new(BinaryOperator::FieldAccess, operator_position),
                    right: Box::new(right),
                },
                (left_start, right_end),
            ));
        }

        let binary_operator = match &self.current.0 {
            Token::DoubleAmpersand => Node::new(BinaryOperator::And, self.current.1),
            Token::DoubleEqual => Node::new(BinaryOperator::Equal, self.current.1),
            Token::DoublePipe => Node::new(BinaryOperator::Or, self.current.1),
            Token::Greater => Node::new(BinaryOperator::Greater, self.current.1),
            Token::GreaterEqual => Node::new(BinaryOperator::GreaterOrEqual, self.current.1),
            Token::Less => Node::new(BinaryOperator::Less, self.current.1),
            Token::LessEqual => Node::new(BinaryOperator::LessOrEqual, self.current.1),
            Token::Minus => Node::new(BinaryOperator::Subtract, self.current.1),
            Token::Plus => Node::new(BinaryOperator::Add, self.current.1),
            Token::Star => Node::new(BinaryOperator::Multiply, self.current.1),
            Token::Slash => Node::new(BinaryOperator::Divide, self.current.1),
            Token::Percent => Node::new(BinaryOperator::Modulo, self.current.1),
            _ => {
                return Err(ParseError::UnexpectedToken {
                    actual: self.current.0.to_owned(),
                    position: self.current.1,
                });
            }
        };

        self.next_token()?;

        let left_start = left.position.0;
        let right = self.parse_statement(operator_precedence)?;
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

    fn parse_postfix(&mut self, left: Node<Statement>) -> Result<Node<Statement>, ParseError> {
        log::trace!("Parsing {} as postfix operator", self.current.0);

        let left_start = left.position.0;

        let statement = match &self.current.0 {
            Token::LeftParenthesis => {
                self.next_token()?;

                let mut arguments = Vec::new();

                while self.current.0 != Token::RightParenthesis {
                    let argument = self.parse_statement(0)?;

                    arguments.push(argument);

                    if let Token::Comma = self.current.0 {
                        self.next_token()?;
                    } else {
                        break;
                    }
                }

                self.next_token()?;

                let right_end = self.current.1 .1;

                Node::new(
                    Statement::Invokation {
                        invokee: Box::new(left),
                        type_arguments: None,
                        value_arguments: Some(arguments),
                    },
                    (left_start, right_end),
                )
            }
            Token::LeftSquareBrace => {
                let operator_start = self.current.1 .0;

                self.next_token()?;

                let index = self.parse_statement(0)?;

                let operator_end = if let Token::RightSquareBrace = self.current.0 {
                    let end = self.current.1 .1;

                    self.next_token()?;

                    end
                } else {
                    return Err(ParseError::ExpectedToken {
                        expected: TokenKind::RightSquareBrace,
                        actual: self.current.0.to_owned(),
                        position: self.current.1,
                    });
                };

                let right_end = self.current.1 .1;

                Node::new(
                    Statement::BinaryOperation {
                        left: Box::new(left),
                        operator: Node::new(
                            BinaryOperator::ListIndex,
                            (operator_start, operator_end),
                        ),
                        right: Box::new(index),
                    },
                    (left_start, right_end),
                )
            }
            Token::Semicolon => {
                let operator_end = self.current.1 .1;

                self.next_token()?;

                Node::new(Statement::Nil(Box::new(left)), (left_start, operator_end))
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    actual: self.current.0.to_owned(),
                    position: self.current.1,
                });
            }
        };

        if self.current.0.is_postfix() {
            self.parse_postfix(statement)
        } else {
            Ok(statement)
        }
    }

    fn parse_identifier(&mut self) -> Result<Node<Identifier>, ParseError> {
        let identifier = if let Token::Identifier(text) = &self.current.0 {
            Node::new(Identifier::new(text), self.current.1)
        } else {
            return Err(ParseError::ExpectedToken {
                expected: TokenKind::Identifier,
                actual: self.current.0.to_owned(),
                position: self.current.1,
            });
        };

        self.next_token()?;

        Ok(identifier)
    }

    fn parse_block(&mut self) -> Result<Node<Statement>, ParseError> {
        let left_start = self.current.1 .0;

        if let Token::LeftCurlyBrace = self.current.0 {
            self.next_token()?;
        } else {
            return Err(ParseError::ExpectedToken {
                expected: TokenKind::LeftCurlyBrace,
                actual: self.current.0.to_owned(),
                position: self.current.1,
            });
        }

        let mut statements = Vec::new();

        loop {
            if let Token::RightCurlyBrace = self.current.0 {
                let right_end = self.current.1 .1;

                self.next_token()?;

                return Ok(Node::new(
                    Statement::Block(statements),
                    (left_start, right_end),
                ));
            }

            let statement = self.parse_statement(0)?;

            statements.push(statement);
        }
    }

    fn parse_type(&mut self) -> Result<Node<Type>, ParseError> {
        let r#type = match self.current.0 {
            Token::Bool => Type::Boolean,
            Token::FloatKeyword => Type::Float,
            Token::Int => Type::Integer,
            _ => {
                return Err(ParseError::ExpectedTokenMultiple {
                    expected: vec![TokenKind::Bool, TokenKind::FloatKeyword, TokenKind::Int],
                    actual: self.current.0.to_owned(),
                    position: self.current.1,
                });
            }
        };
        let position = self.current.1;

        self.next_token()?;

        Ok(Node::new(r#type, position))
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum ParserMode {
    Condition,
    None,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ParseError {
    BooleanError {
        error: ParseBoolError,
        position: Span,
    },
    LexError(LexError),
    ExpectedAssignment {
        actual: Node<Statement>,
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
    FloatError {
        error: ParseFloatError,
        position: Span,
    },
    IntegerError {
        error: ParseIntError,
        position: Span,
    },
}

impl From<LexError> for ParseError {
    fn from(v: LexError) -> Self {
        Self::LexError(v)
    }
}

impl ParseError {
    pub fn position(&self) -> Span {
        match self {
            ParseError::BooleanError { position, .. } => *position,
            ParseError::ExpectedAssignment { actual } => actual.position,
            ParseError::ExpectedIdentifier { position, .. } => *position,
            ParseError::ExpectedToken { position, .. } => *position,
            ParseError::ExpectedTokenMultiple { position, .. } => *position,
            ParseError::FloatError { position, .. } => *position,
            ParseError::IntegerError { position, .. } => *position,
            ParseError::LexError(error) => error.position(),
            ParseError::UnexpectedToken { position, .. } => *position,
        }
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LexError(error) => Some(error),
            _ => None,
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::BooleanError { error, .. } => write!(f, "{}", error),
            Self::ExpectedAssignment { .. } => write!(f, "Expected assignment"),
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
            Self::FloatError { error, .. } => write!(f, "{}", error),
            Self::IntegerError { error, .. } => write!(f, "{}", error),
            Self::LexError(error) => write!(f, "{}", error),
            Self::UnexpectedToken { actual, .. } => write!(f, "Unexpected token {actual}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{BinaryOperator, Identifier, StructDefinition, Type, UnaryOperator};

    use super::*;

    #[test]
    fn mutable_variable() {
        let input = "mut x = 42";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                nodes: [Node::new(
                    Statement::MutAssignment {
                        identifier: Node::new(Identifier::new("x"), (4, 5)),
                        value: Box::new(Node::new(
                            Statement::Constant(Value::integer(42)),
                            (8, 10)
                        )),
                    },
                    (0, 10)
                )]
                .into()
            })
        );
    }

    #[test]
    fn async_block() {
        let input = "async { x = 42; y = 4.0 }";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                nodes: [Node::new(
                    Statement::AsyncBlock(vec![
                        Node::new(
                            Statement::Nil(Box::new(Node::new(
                                Statement::Assignment {
                                    identifier: Node::new(Identifier::new("x"), (8, 9)),
                                    operator: Node::new(AssignmentOperator::Assign, (10, 11)),
                                    value: Box::new(Node::new(
                                        Statement::Constant(Value::integer(42)),
                                        (12, 14)
                                    )),
                                },
                                (8, 14)
                            ))),
                            (8, 15)
                        ),
                        Node::new(
                            Statement::Assignment {
                                identifier: Node::new(Identifier::new("y"), (16, 17)),
                                operator: Node::new(AssignmentOperator::Assign, (18, 19)),
                                value: Box::new(Node::new(
                                    Statement::Constant(Value::float(4.0)),
                                    (20, 23)
                                )),
                            },
                            (16, 23)
                        )
                    ]),
                    (0, 25)
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
                nodes: [Node::new(
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
                nodes: [Node::new(
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
                nodes: [Node::new(
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
                nodes: [
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
                nodes: [Node::new(
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
                nodes: [Node::new(
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
                nodes: [Node::new(
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
                nodes: [Node::new(
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
                nodes: [Node::new(Statement::Constant(Value::range(0..42)), (0, 5))].into()
            })
        );
    }

    #[test]
    fn negate_variable() {
        let input = "a = 1; -a";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                nodes: [
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
                nodes: [Node::new(
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
                nodes: [Node::new(
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
                nodes: [
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
                nodes: [Node::new(
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
                nodes: [Node::new(
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
                nodes: [Node::new(
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
                nodes: [Node::new(
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
                nodes: [Node::new(
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
    fn map_with_two_fields() {
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
    fn map_with_one_field() {
        let input = "{ x = 42 }";

        assert_eq!(
            parse(input),
            Ok(AbstractSyntaxTree {
                nodes: [Node::new(
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
                nodes: [Node::new(
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
                nodes: [Node::new(
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
