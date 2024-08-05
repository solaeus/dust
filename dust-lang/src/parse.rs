use crate::{
    identifier::Identifier,
    lex::{LexError, Lexer},
    Span, Token, Value,
};

pub fn parse(input: &str) -> Result<Instruction, ParseError> {
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);

    parser.parse()
}

#[derive(Debug, PartialEq, Clone)]
pub struct Instruction {
    pub operation: Operation,
    pub span: Span,
}

impl Instruction {
    pub fn new(operation: Operation, span: Span) -> Self {
        Self { operation, span }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operation {
    Add(Box<(Instruction, Instruction)>),
    Assign(Box<(Instruction, Instruction)>),
    Constant(Value),
    Identifier(Identifier),
    List(Vec<Instruction>),
    Multiply(Box<(Instruction, Instruction)>),
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

    pub fn parse(&mut self) -> Result<Instruction, ParseError> {
        self.parse_instruction(0)
    }

    fn next_token(&mut self) -> Result<(), ParseError> {
        self.current = self.lexer.next_token()?;

        Ok(())
    }

    fn parse_instruction(&mut self, precedence: u8) -> Result<Instruction, ParseError> {
        let left_instruction = self.parse_primary()?;
        let left_start = left_instruction.span.0;

        if precedence < self.current_precedence() {
            match &self.current {
                (Token::Plus, _) => {
                    self.next_token()?;

                    let right_instruction = self.parse_instruction(self.current_precedence())?;
                    let right_end = right_instruction.span.1;

                    return Ok(Instruction::new(
                        Operation::Add(Box::new((left_instruction, right_instruction))),
                        (left_start, right_end),
                    ));
                }
                (Token::Star, _) => {
                    self.next_token()?;

                    let right_instruction = self.parse_instruction(self.current_precedence())?;
                    let right_end = right_instruction.span.1;

                    return Ok(Instruction::new(
                        Operation::Multiply(Box::new((left_instruction, right_instruction))),
                        (left_start, right_end),
                    ));
                }
                (Token::Equal, _) => {
                    self.next_token()?;

                    let right_instruction = self.parse_instruction(self.current_precedence())?;
                    let right_end = right_instruction.span.1;

                    return Ok(Instruction::new(
                        Operation::Assign(Box::new((left_instruction, right_instruction))),
                        (left_start, right_end),
                    ));
                }
                _ => {}
            }
        }

        Ok(left_instruction)
    }

    fn parse_primary(&mut self) -> Result<Instruction, ParseError> {
        match self.current.clone() {
            (Token::Float(float), span) => {
                self.next_token()?;

                Ok(Instruction::new(
                    Operation::Constant(Value::float(float)),
                    span,
                ))
            }
            (Token::Integer(int), span) => {
                self.next_token()?;

                Ok(Instruction::new(
                    Operation::Constant(Value::integer(int)),
                    span,
                ))
            }
            (Token::Identifier(identifier), span) => {
                self.next_token()?;

                Ok(Instruction::new(Operation::Identifier(identifier), span))
            }
            (Token::LeftParenthesis, left_span) => {
                self.next_token()?;

                let instruction = self.parse_instruction(0)?;

                if let (Token::RightParenthesis, right_span) = self.current {
                    self.next_token()?;

                    Ok(Instruction::new(
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

                        return Ok(Instruction::new(
                            Operation::List(instructions),
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

    use super::*;

    #[test]
    fn complex_list() {
        let input = "[1, 1 + 1, 2 + (4 * 10)]";

        assert_eq!(
            parse(input),
            Ok(Instruction::new(
                Operation::List(vec![
                    Instruction::new(Operation::Constant(Value::integer(1)), (1, 2)),
                    Instruction::new(
                        Operation::Add(Box::new((
                            Instruction::new(Operation::Constant(Value::integer(1)), (4, 5)),
                            Instruction::new(Operation::Constant(Value::integer(1)), (8, 9)),
                        ))),
                        (4, 9)
                    ),
                    Instruction::new(
                        Operation::Add(Box::new((
                            Instruction::new(Operation::Constant(Value::integer(2)), (11, 12)),
                            Instruction::new(
                                Operation::Multiply(Box::new((
                                    Instruction::new(
                                        Operation::Constant(Value::integer(4)),
                                        (16, 17)
                                    ),
                                    Instruction::new(
                                        Operation::Constant(Value::integer(10)),
                                        (20, 22)
                                    ),
                                ))),
                                (15, 23)
                            ),
                        ))),
                        (11, 23)
                    )
                ]),
                (0, 24)
            ))
        );
    }

    #[test]
    fn list() {
        let input = "[1, 2]";

        assert_eq!(
            parse(input),
            Ok(Instruction::new(
                Operation::List(vec![
                    Instruction::new(Operation::Constant(Value::integer(1)), (1, 2)),
                    Instruction::new(Operation::Constant(Value::integer(2)), (4, 5)),
                ]),
                (0, 6)
            ))
        );
    }

    #[test]
    fn empty_list() {
        let input = "[]";

        assert_eq!(
            parse(input),
            Ok(Instruction::new(Operation::List(vec![]), (0, 2)))
        );
    }

    #[test]
    fn float() {
        let input = "42.0";

        assert_eq!(
            parse(input),
            Ok(Instruction::new(
                Operation::Constant(Value::float(42.0)),
                (0, 4)
            ))
        );
    }

    #[test]
    fn add() {
        let input = "1 + 2";

        assert_eq!(
            parse(input),
            Ok(Instruction::new(
                Operation::Add(Box::new((
                    Instruction::new(Operation::Constant(Value::integer(1)), (0, 1)),
                    Instruction::new(Operation::Constant(Value::integer(2)), (4, 5)),
                ))),
                (0, 5)
            ))
        );
    }

    #[test]
    fn multiply() {
        let input = "1 * 2";

        assert_eq!(
            parse(input),
            Ok(Instruction::new(
                Operation::Multiply(Box::new((
                    Instruction::new(Operation::Constant(Value::integer(1)), (0, 1)),
                    Instruction::new(Operation::Constant(Value::integer(2)), (4, 5)),
                ))),
                (0, 5)
            ))
        );
    }

    #[test]
    fn add_and_multiply() {
        let input = "1 + 2 * 3";

        assert_eq!(
            parse(input),
            Ok(Instruction::new(
                Operation::Add(Box::new((
                    Instruction::new(Operation::Constant(Value::integer(1)), (0, 1)),
                    Instruction::new(
                        Operation::Multiply(Box::new((
                            Instruction::new(Operation::Constant(Value::integer(2)), (4, 5)),
                            Instruction::new(Operation::Constant(Value::integer(3)), (8, 9)),
                        ))),
                        (4, 9)
                    ),
                ))),
                (0, 9)
            ))
        );
    }

    #[test]
    fn assignment() {
        let input = "a = 1 + 2 * 3";

        assert_eq!(
            parse(input),
            Ok(Instruction::new(
                Operation::Assign(Box::new((
                    Instruction::new(Operation::Identifier(Identifier::new("a")), (0, 1)),
                    Instruction::new(
                        Operation::Add(Box::new((
                            Instruction::new(Operation::Constant(Value::integer(1)), (4, 5)),
                            Instruction::new(
                                Operation::Multiply(Box::new((
                                    Instruction::new(
                                        Operation::Constant(Value::integer(2)),
                                        (8, 9)
                                    ),
                                    Instruction::new(
                                        Operation::Constant(Value::integer(3)),
                                        (12, 13)
                                    ),
                                ))),
                                (8, 13)
                            ),
                        ))),
                        (4, 13)
                    ),
                ))),
                (0, 13)
            ))
        );
    }
}
