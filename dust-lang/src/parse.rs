use crate::{
    lex::{LexError, Lexer},
    Node, Span, Statement, Token, Value,
};

pub fn parse(input: &str) -> Result<Vec<Node>, ParseError> {
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let mut instructions = Vec::new();

    loop {
        let instruction = parser.parse()?;

        instructions.push(instruction);

        if let Token::Eof = parser.current.0 {
            break;
        }
    }

    Ok(instructions)
}

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
        self.parse_instruction(0)
    }

    fn next_token(&mut self) -> Result<(), ParseError> {
        self.current = self.lexer.next_token()?;

        Ok(())
    }

    fn parse_instruction(&mut self, precedence: u8) -> Result<Node, ParseError> {
        let left_instruction = self.parse_primary()?;
        let left_start = left_instruction.span.0;

        if precedence < self.current_precedence() {
            match &self.current {
                (Token::Plus, _) => {
                    self.next_token()?;

                    let right_instruction = self.parse_instruction(self.current_precedence())?;
                    let right_end = right_instruction.span.1;

                    return Ok(Node::new(
                        Statement::Add(Box::new((left_instruction, right_instruction))),
                        (left_start, right_end),
                    ));
                }
                (Token::Star, _) => {
                    self.next_token()?;

                    let right_instruction = self.parse_instruction(self.current_precedence())?;
                    let right_end = right_instruction.span.1;

                    return Ok(Node::new(
                        Statement::Multiply(Box::new((left_instruction, right_instruction))),
                        (left_start, right_end),
                    ));
                }
                (Token::Equal, _) => {
                    self.next_token()?;

                    let right_instruction = self.parse_instruction(self.current_precedence())?;
                    let right_end = right_instruction.span.1;

                    return Ok(Node::new(
                        Statement::Assign(Box::new((left_instruction, right_instruction))),
                        (left_start, right_end),
                    ));
                }
                _ => {}
            }
        }

        Ok(left_instruction)
    }

    fn parse_primary(&mut self) -> Result<Node, ParseError> {
        match self.current.clone() {
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

                let instruction = self.parse_instruction(0)?;

                if let (Token::RightParenthesis, right_span) = self.current {
                    self.next_token()?;

                    Ok(Node::new(
                        instruction.operation,
                        (left_span.0, right_span.1),
                    ))
                } else {
                    Err(ParseError::ExpectedClosingParenthesis {
                        actual: self.current.0.clone(),
                        span: self.current.1,
                    })
                }
            }
            (Token::LeftSquareBrace, left_span) => {
                self.next_token()?;

                let mut instructions = Vec::new();

                loop {
                    if let (Token::RightSquareBrace, right_span) = self.current {
                        self.next_token()?;

                        return Ok(Node::new(
                            Statement::List(instructions),
                            (left_span.0, right_span.1),
                        ));
                    }

                    if let (Token::Comma, _) = self.current {
                        self.next_token()?;

                        continue;
                    }

                    if let Ok(instruction) = self.parse_instruction(0) {
                        instructions.push(instruction);
                    } else {
                        return Err(ParseError::ExpectedClosingSquareBrace {
                            actual: self.current.0.clone(),
                            span: self.current.1,
                        });
                    }
                }
            }
            _ => Err(ParseError::UnexpectedToken(self.current.0.clone())),
        }
    }

    fn current_precedence(&self) -> u8 {
        match self.current.0 {
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
    fn complex_list() {
        let input = "[1, 1 + 1, 2 + (4 * 10)]";

        assert_eq!(
            parse(input),
            Ok(vec![Node::new(
                Statement::List(vec![
                    Node::new(Statement::Constant(Value::integer(1)), (1, 2)),
                    Node::new(
                        Statement::Add(Box::new((
                            Node::new(Statement::Constant(Value::integer(1)), (4, 5)),
                            Node::new(Statement::Constant(Value::integer(1)), (8, 9)),
                        ))),
                        (4, 9)
                    ),
                    Node::new(
                        Statement::Add(Box::new((
                            Node::new(Statement::Constant(Value::integer(2)), (11, 12)),
                            Node::new(
                                Statement::Multiply(Box::new((
                                    Node::new(Statement::Constant(Value::integer(4)), (16, 17)),
                                    Node::new(Statement::Constant(Value::integer(10)), (20, 22)),
                                ))),
                                (15, 23)
                            ),
                        ))),
                        (11, 23)
                    )
                ]),
                (0, 24)
            )])
        );
    }

    #[test]
    fn list() {
        let input = "[1, 2]";

        assert_eq!(
            parse(input),
            Ok(vec![Node::new(
                Statement::List(vec![
                    Node::new(Statement::Constant(Value::integer(1)), (1, 2)),
                    Node::new(Statement::Constant(Value::integer(2)), (4, 5)),
                ]),
                (0, 6)
            )])
        );
    }

    #[test]
    fn empty_list() {
        let input = "[]";

        assert_eq!(
            parse(input),
            Ok(vec![Node::new(Statement::List(vec![]), (0, 2))])
        );
    }

    #[test]
    fn float() {
        let input = "42.0";

        assert_eq!(
            parse(input),
            Ok(vec![Node::new(
                Statement::Constant(Value::float(42.0)),
                (0, 4)
            )])
        );
    }

    #[test]
    fn add() {
        let input = "1 + 2";

        assert_eq!(
            parse(input),
            Ok(vec![Node::new(
                Statement::Add(Box::new((
                    Node::new(Statement::Constant(Value::integer(1)), (0, 1)),
                    Node::new(Statement::Constant(Value::integer(2)), (4, 5)),
                ))),
                (0, 5)
            )])
        );
    }

    #[test]
    fn multiply() {
        let input = "1 * 2";

        assert_eq!(
            parse(input),
            Ok(vec![Node::new(
                Statement::Multiply(Box::new((
                    Node::new(Statement::Constant(Value::integer(1)), (0, 1)),
                    Node::new(Statement::Constant(Value::integer(2)), (4, 5)),
                ))),
                (0, 5)
            )])
        );
    }

    #[test]
    fn add_and_multiply() {
        let input = "1 + 2 * 3";

        assert_eq!(
            parse(input),
            Ok(vec![Node::new(
                Statement::Add(Box::new((
                    Node::new(Statement::Constant(Value::integer(1)), (0, 1)),
                    Node::new(
                        Statement::Multiply(Box::new((
                            Node::new(Statement::Constant(Value::integer(2)), (4, 5)),
                            Node::new(Statement::Constant(Value::integer(3)), (8, 9)),
                        ))),
                        (4, 9)
                    ),
                ))),
                (0, 9)
            )])
        );
    }

    #[test]
    fn assignment() {
        let input = "a = 1 + 2 * 3";

        assert_eq!(
            parse(input),
            Ok(vec![Node::new(
                Statement::Assign(Box::new((
                    Node::new(Statement::Identifier(Identifier::new("a")), (0, 1)),
                    Node::new(
                        Statement::Add(Box::new((
                            Node::new(Statement::Constant(Value::integer(1)), (4, 5)),
                            Node::new(
                                Statement::Multiply(Box::new((
                                    Node::new(Statement::Constant(Value::integer(2)), (8, 9)),
                                    Node::new(Statement::Constant(Value::integer(3)), (12, 13)),
                                ))),
                                (8, 13)
                            ),
                        ))),
                        (4, 13)
                    ),
                ))),
                (0, 13)
            )])
        );
    }
}
