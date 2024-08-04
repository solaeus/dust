use crate::{
    identifier::Identifier,
    lex::{LexError, Lexer},
    Span, Token, Value,
};

pub fn parse(input: &str) -> Result<(Instruction, Span), ParseError> {
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);

    parser.parse()
}

#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
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
        let current_token = lexer
            .next_token()
            .map(|(token, _)| token)
            .unwrap_or(Token::Eof);

        Parser {
            lexer,
            current: (current_token, (0, 0)),
        }
    }

    pub fn parse(&mut self) -> Result<(Instruction, Span), ParseError> {
        self.parse_instruction(0)
    }

    fn next_token(&mut self) -> Result<(), ParseError> {
        self.current = self.lexer.next_token()?;

        Ok(())
    }

    fn parse_instruction(&mut self, precedence: u8) -> Result<(Instruction, Span), ParseError> {
        let (left_instruction, left_span) = self.parse_primary()?;

        if precedence < self.current_precedence() {
            match &self.current {
                (Token::Plus, _) => {
                    self.next_token()?;

                    let (right_instruction, right_span) =
                        self.parse_instruction(self.current_precedence())?;

                    return Ok((
                        Instruction::Add(Box::new((left_instruction, right_instruction))),
                        (left_span.0, right_span.1),
                    ));
                }
                (Token::Star, _) => {
                    self.next_token()?;

                    let (right_instruction, right_span) =
                        self.parse_instruction(self.current_precedence())?;

                    return Ok((
                        Instruction::Multiply(Box::new((left_instruction, right_instruction))),
                        (left_span.0, right_span.1),
                    ));
                }
                (Token::Equal, _) => {
                    self.next_token()?;

                    let (right_instruction, right_span) =
                        self.parse_instruction(self.current_precedence())?;

                    return Ok((
                        Instruction::Assign(Box::new((left_instruction, right_instruction))),
                        (left_span.0, right_span.1),
                    ));
                }
                _ => {}
            }
        }

        Ok((left_instruction, left_span))
    }

    fn parse_primary(&mut self) -> Result<(Instruction, Span), ParseError> {
        match self.current.clone() {
            (Token::Integer(int), span) => {
                self.next_token()?;
                Ok((Instruction::Constant(Value::integer(int)), span))
            }
            (Token::Identifier(identifier), span) => {
                self.next_token()?;
                Ok((Instruction::Identifier(identifier), span))
            }
            (Token::LeftParenthesis, left_span) => {
                self.next_token()?;

                let (instruction, _) = self.parse_instruction(0)?;

                if let (Token::RightParenthesis, right_span) = self.current {
                    self.next_token()?;

                    Ok((instruction, (left_span.0, right_span.1)))
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
    fn add() {
        let input = "1 + 2";

        assert_eq!(
            parse(input),
            Ok((
                Instruction::Add(Box::new((
                    Instruction::Constant(Value::integer(1)),
                    Instruction::Constant(Value::integer(2))
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
            Ok((
                Instruction::Multiply(Box::new((
                    Instruction::Constant(Value::integer(1)),
                    Instruction::Constant(Value::integer(2))
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
            Ok((
                Instruction::Add(Box::new((
                    Instruction::Constant(Value::integer(1)),
                    Instruction::Multiply(Box::new((
                        Instruction::Constant(Value::integer(2)),
                        Instruction::Constant(Value::integer(3))
                    )))
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
            Ok((
                Instruction::Assign(Box::new((
                    Instruction::Identifier(Identifier::new("a")),
                    Instruction::Add(Box::new((
                        Instruction::Constant(Value::integer(1)),
                        Instruction::Multiply(Box::new((
                            Instruction::Constant(Value::integer(2)),
                            Instruction::Constant(Value::integer(3))
                        )))
                    )))
                ))),
                (0, 13)
            ))
        );
    }
}
