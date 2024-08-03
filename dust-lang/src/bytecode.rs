use crate::{identifier::Identifier, Value};

pub type Span = (usize, usize);

#[derive(Debug, PartialEq, Clone)]
pub enum LexError {
    IntegerParseError(std::num::ParseIntError),
}

impl From<std::num::ParseIntError> for LexError {
    fn from(v: std::num::ParseIntError) -> Self {
        Self::IntegerParseError(v)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Eof,
    Equal,
    Identifier(Identifier),
    Integer(i64),
    Plus,
    Star,
    LeftParenthesis,
    RightParenthesis,
}

pub fn lex(input: &str) -> Result<Vec<(Token, Span)>, LexError> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();

    loop {
        let (token, span) = lexer.next_token()?;
        let is_eof = matches!(token, Token::Eof);

        tokens.push((token, span));

        if is_eof {
            break;
        }
    }

    Ok(tokens)
}

#[derive(Debug, Clone)]
pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer { input, position: 0 }
    }

    fn next_char(&mut self) -> Option<char> {
        self.input[self.position..].chars().next().map(|c| {
            self.position += c.len_utf8();
            c
        })
    }

    pub fn next_token(&mut self) -> Result<(Token, Span), LexError> {
        self.skip_whitespace();

        let (token, span) = if let Some(c) = self.peek_char() {
            match c {
                '0'..='9' => self.lex_number()?,
                'a'..='z' | 'A'..='Z' => self.lex_identifier()?,
                '+' => {
                    self.position += 1;
                    (Token::Plus, (self.position - 1, self.position))
                }
                '*' => {
                    self.position += 1;
                    (Token::Star, (self.position - 1, self.position))
                }
                '(' => {
                    self.position += 1;
                    (Token::LeftParenthesis, (self.position - 1, self.position))
                }
                ')' => {
                    self.position += 1;
                    (Token::RightParenthesis, (self.position - 1, self.position))
                }
                '=' => {
                    self.position += 1;
                    (Token::Equal, (self.position - 1, self.position))
                }
                _ => (Token::Eof, (self.position, self.position)),
            }
        } else {
            (Token::Eof, (self.position, self.position))
        };

        Ok((token, span))
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.next_char();
            } else {
                break;
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }

    fn lex_number(&mut self) -> Result<(Token, Span), LexError> {
        let start_pos = self.position;

        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                self.next_char();
            } else {
                break;
            }
        }

        let integer = self.input[start_pos..self.position].parse::<i64>()?;

        Ok((Token::Integer(integer), (start_pos, self.position)))
    }

    fn lex_identifier(&mut self) -> Result<(Token, Span), LexError> {
        let start_pos = self.position;

        while let Some(c) = self.peek_char() {
            if c.is_ascii_alphanumeric() {
                self.next_char();
            } else {
                break;
            }
        }

        let identifier = &self.input[start_pos..self.position];
        let token = Token::Identifier(Identifier::new(identifier));

        Ok((token, (start_pos, self.position)))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    Add(Box<(Instruction, Instruction)>),
    Assign(Box<(Instruction, Instruction)>),
    Constant(Value),
    Identifier(Identifier),
    Multiply(Box<(Instruction, Instruction)>),
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

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        let mut lexer = lexer;
        let current_token = lexer
            .next_token()
            .map(|(token, _)| token)
            .unwrap_or(Token::Eof);

        Parser {
            lexer,
            current_token,
        }
    }

    pub fn parse(&mut self) -> Result<Instruction, ParseError> {
        self.parse_instruction(0)
    }

    fn next_token(&mut self) -> Result<(), ParseError> {
        self.current_token = self.lexer.next_token()?.0;

        Ok(())
    }

    fn parse_instruction(&mut self, precedence: u8) -> Result<Instruction, ParseError> {
        let mut left = self.parse_primary()?;

        while precedence < self.current_precedence() {
            match &self.current_token {
                Token::Plus => {
                    self.next_token()?;

                    let right = self.parse_instruction(self.current_precedence())?;
                    left = Instruction::Add(Box::new((left, right)));
                }
                Token::Star => {
                    self.next_token()?;

                    let right = self.parse_instruction(self.current_precedence())?;
                    left = Instruction::Multiply(Box::new((left, right)));
                }
                Token::Equal => {
                    self.next_token()?;

                    let right = self.parse_instruction(self.current_precedence())?;
                    left = Instruction::Assign(Box::new((left, right)));
                }
                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Instruction, ParseError> {
        match self.current_token.clone() {
            Token::Integer(int) => {
                self.next_token()?;
                Ok(Instruction::Constant(Value::integer(int)))
            }
            Token::Identifier(identifier) => {
                self.next_token()?;
                Ok(Instruction::Identifier(identifier))
            }
            Token::LeftParenthesis => {
                self.next_token()?;

                let instruction = self.parse_instruction(0)?;

                if let Token::RightParenthesis = self.current_token {
                    self.next_token()?;
                } else {
                    return Err(ParseError::ExpectedClosingParenthesis);
                }

                Ok(instruction)
            }
            _ => Err(ParseError::UnexpectedToken(self.current_token.clone())),
        }
    }

    fn current_precedence(&self) -> u8 {
        match self.current_token {
            Token::Equal => 3,
            Token::Plus => 1,
            Token::Star => 2,
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{identifier::Identifier, Value};

    use super::{lex, Instruction, Lexer, Parser, Token};

    #[test]
    fn lex_api() {
        let input = "1 + 2 * 3";

        assert_eq!(
            lex(input),
            Ok(vec![
                (Token::Integer(1), (0, 1)),
                (Token::Plus, (2, 3)),
                (Token::Integer(2), (4, 5)),
                (Token::Star, (6, 7)),
                (Token::Integer(3), (8, 9)),
                (Token::Eof, (9, 9)),
            ])
        );
    }

    #[test]
    fn parser() {
        let input = "1 + 2 * 3";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        assert_eq!(
            parser.parse(),
            Ok(Instruction::Add(Box::new((
                Instruction::Constant(Value::integer(1)),
                Instruction::Multiply(Box::new((
                    Instruction::Constant(Value::integer(2)),
                    Instruction::Constant(Value::integer(3))
                )))
            ))))
        );
    }

    #[test]
    fn assignment() {
        let input = "a = 1 + 2 * 3";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        assert_eq!(
            parser.parse(),
            Ok(Instruction::Assign(Box::new((
                Instruction::Identifier(Identifier::new("a")),
                Instruction::Add(Box::new((
                    Instruction::Constant(Value::integer(1)),
                    Instruction::Multiply(Box::new((
                        Instruction::Constant(Value::integer(2)),
                        Instruction::Constant(Value::integer(3))
                    )))
                )))
            ))))
        );
    }
}
