use crate::{Identifier, Span, Token};

#[derive(Debug, PartialEq, Clone)]
pub enum LexError {
    IntegerParseError(std::num::ParseIntError),
}

impl From<std::num::ParseIntError> for LexError {
    fn from(v: std::num::ParseIntError) -> Self {
        Self::IntegerParseError(v)
    }
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
