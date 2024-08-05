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
                    Err(ParseError::ExpectedClosingParenthesis)
                }
            }
            _ => Err(ParseError::UnexpectedToken(self.current.0.clone())),
        }
    }

    fn current_precedence(&self) -> u8 {
        match self.current {
            (Token::Equal, _) => 3,
            (Token::Plus, _) => 1,
            (Token::Star, _) => 2,
            _ => 0,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ParseError {
    LexError(LexError),
    ExpectedClosingParenthesis,
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
