use crate::{Span, Token};

pub fn tokenize(source: &str) -> Vec<(Token, Span)> {
    let mut lexer = Lexer::with_source(source);
    let mut tokens = Vec::new();

    loop {
        let (token, span) = lexer.next_token();

        tokens.push((token, span));

        if token == Token::Eof {
            break;
        }
    }

    tokens
}

#[derive(Debug, Default)]
pub struct Lexer<'src> {
    source: &'src str,
    position: usize,
    line_breaks: Vec<u32>,
}

impl<'src> Lexer<'src> {
    pub fn new() -> Self {
        Lexer {
            source: "",
            position: 0,
            line_breaks: Vec::new(),
        }
    }

    pub fn with_source(source: &'src str) -> Self {
        Lexer {
            source,
            position: 0,
            line_breaks: Vec::new(),
        }
    }

    pub fn into_line_breaks(self) -> Vec<u32> {
        self.line_breaks
    }

    pub fn initialize(&mut self, source: &'src str, offset: usize) -> (Token, Span) {
        self.source = source;
        self.position = offset;

        self.line_breaks.clear();
        self.next_token()
    }

    pub fn source(&self) -> &'src str {
        self.source
    }

    /// Produce the next token.
    pub fn next_token(&mut self) -> (Token, Span) {
        let Some(character) = self.peek_char() else {
            return (Token::Eof, Span::new(self.position, self.position));
        };

        let (token, span) = match character {
            '0'..='9' => self.lex_numeric(),
            'a'..='z' | 'A'..='Z' | '_' => self.lex_keyword_or_identifier(),
            '"' => self.lex_string(),
            '\'' => self.lex_character(),
            '+' => self.lex_plus(),
            '-' => self.lex_minus(),
            '*' => self.lex_asterisk(),
            '/' => self.lex_slash(),
            '%' => self.lex_percent(),
            '!' => self.lex_exclamation_mark(),
            '=' => self.lex_equal(),
            '<' => self.lex_less_than(),
            '>' => self.lex_greater_than(),
            '&' => self.lex_ampersand(),
            '|' => self.lex_pipe(),
            '.' => self.lex_dot(),
            ':' => self.lex_colon(),
            '(' => self.emit(Token::LeftParenthesis),
            ')' => self.emit(Token::RightParenthesis),
            '[' => self.emit(Token::LeftSquareBracket),
            ']' => self.emit(Token::RightSquareBracket),
            '{' => self.emit(Token::LeftCurlyBrace),
            '}' => self.emit(Token::RightCurlyBrace),
            ';' => self.emit(Token::Semicolon),
            ',' => self.emit(Token::Comma),
            ' ' => self.emit(Token::Space),
            '\t' => self.emit(Token::Tab),
            '\r' => self.lex_carriage_return(),
            '\n' => {
                let (token, position) = self.emit(Token::Newline);

                self.line_breaks.push(position.1);

                (token, position)
            }
            _ => self.emit(Token::Unknown),
        };

        (token, span)
    }

    /// Progress to the next character.
    fn advance(&mut self) {
        if let Some(c) = self.source[self.position..].chars().next() {
            self.position += c.len_utf8();
        }
    }

    /// Peek at the next character without consuming it.
    fn peek_char(&self) -> Option<char> {
        self.source[self.position..].chars().next()
    }

    /// Peek at the second-to-next character without consuming it.
    fn peek_second_char(&self) -> Option<char> {
        self.source[self.position..].chars().nth(1)
    }

    /// Peek the next `n` characters without consuming them.
    fn peek_chars(&self, n: usize) -> impl Iterator<Item = char> + 'src {
        self.source[self.position..].chars().take(n)
    }

    #[inline]
    fn emit(&mut self, token: Token) -> (Token, Span) {
        let start = self.position;

        self.advance();

        (token, Span::new(start, self.position))
    }

    fn lex_numeric(&mut self) -> (Token, Span) {
        let start = self.position;
        let mut is_float = false;
        let peek_char = self.peek_char();

        if peek_char == Some('-') {
            self.advance();
        }

        if peek_char == Some('0') {
            self.advance();

            if self.peek_char() == Some('x') {
                self.advance();

                for character in self.peek_chars(2) {
                    if character.is_ascii_hexdigit() {
                        self.advance();
                    } else {
                        return (Token::Unknown, Span::new(start, self.position));
                    }
                }

                return (Token::ByteValue, Span::new(start, self.position));
            }
        }

        while let Some(c) = self.peek_char() {
            if c == '.' {
                if matches!(
                    self.peek_second_char(),
                    Some('0'..='9' | 'e' | 'E' | '+' | '-' | '_')
                ) {
                    if !is_float {
                        self.advance(); // Consume the dot
                    }

                    is_float = true;

                    self.advance();
                } else {
                    break;
                }

                while let Some(peek_char) = self.peek_char() {
                    if peek_char.is_ascii_digit() || peek_char == '_' {
                        self.advance();

                        continue;
                    }

                    let peek_second_char = self.peek_second_char();

                    match (peek_char, peek_second_char) {
                        ('e' | 'E', Some('0'..='9')) => {
                            self.advance();
                            self.advance();

                            continue;
                        }
                        ('e' | 'E', Some('+' | '-')) => {
                            self.advance();
                            self.advance();

                            continue;
                        }
                        _ => break,
                    }
                }
            }

            if c.is_ascii_digit() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }

        if is_float {
            (Token::FloatValue, Span::new(start, self.position))
        } else {
            (Token::IntegerValue, Span::new(start, self.position))
        }
    }

    #[inline]
    fn lex_keyword_or_identifier(&mut self) -> (Token, Span) {
        let start = self.position;

        while let Some(char) = self.peek_char() {
            if char.is_ascii_alphanumeric() || char == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let token = match &self.source[start..self.position] {
            "Infinity" => Token::FloatValue,
            "NaN" => Token::FloatValue,
            "any" => Token::Any,
            "async" => Token::Async,
            "bool" => Token::Bool,
            "break" => Token::Break,
            "byte" => Token::Byte,
            "cell" => Token::Cell,
            "char" => Token::Char,
            "const" => Token::Const,
            "else" => Token::Else,
            "false" => Token::FalseValue,
            "float" => Token::Float,
            "fn" => Token::Fn,
            "if" => Token::If,
            "int" => Token::Int,
            "let" => Token::Let,
            "list" => Token::List,
            "loop" => Token::Loop,
            "map" => Token::Map,
            "mod" => Token::Mod,
            "mut" => Token::Mut,
            "pub" => Token::Pub,
            "return" => Token::Return,
            "str" => Token::Str,
            "struct" => Token::Struct,
            "true" => Token::TrueValue,
            "use" => Token::Use,
            "while" => Token::While,
            _ => Token::Identifier,
        };

        (token, Span::new(start, self.position))
    }

    #[inline]
    fn lex_string(&mut self) -> (Token, Span) {
        let start = self.position;

        self.advance();

        while let Some(c) = self.peek_char()
            && c != '"'
        {
            self.advance();
        }

        self.advance();

        let end = self.position;

        (Token::StringValue, Span::new(start, end))
    }

    #[inline]
    fn lex_character(&mut self) -> (Token, Span) {
        let start = self.position;

        self.advance();
        self.advance();

        if self.peek_char() == Some('\'') {
            self.advance();

            (Token::CharacterValue, Span::new(start, self.position))
        } else {
            (Token::Unknown, Span::new(start, self.position))
        }
    }

    #[inline]
    fn lex_plus(&mut self) -> (Token, Span) {
        let start = self.position;

        self.advance();

        if self.peek_char() == Some('=') {
            self.advance();

            (Token::PlusEqual, Span::new(start, self.position))
        } else {
            (Token::Plus, Span::new(start, self.position))
        }
    }

    #[inline]
    fn lex_minus(&mut self) -> (Token, Span) {
        let start = self.position;

        if self
            .peek_second_char()
            .is_some_and(|char| char.is_ascii_digit())
        {
            return self.lex_numeric();
        }

        self.advance();

        match self.peek_char() {
            Some('=') => {
                self.advance();

                return (Token::MinusEqual, Span::new(start, self.position));
            }
            Some('>') => {
                self.advance();

                return (Token::ArrowThin, Span::new(start, self.position));
            }
            _ => {}
        }

        if self.peek_chars(8).eq("Infinity".chars()) {
            self.position += 8;

            return (Token::FloatValue, Span::new(start, self.position));
        }

        (Token::Minus, Span::new(start, self.position))
    }

    #[inline]
    fn lex_asterisk(&mut self) -> (Token, Span) {
        let start = self.position;

        self.advance();

        if self.peek_char() == Some('=') {
            self.advance();

            (Token::AsteriskEqual, Span::new(start, self.position))
        } else {
            (Token::Asterisk, Span::new(start, self.position))
        }
    }

    #[inline]
    fn lex_slash(&mut self) -> (Token, Span) {
        let start = self.position;

        self.advance();

        if self.peek_char() == Some('=') {
            self.advance();

            (Token::SlashEqual, Span::new(start, self.position))
        } else {
            (Token::Slash, Span::new(start, self.position))
        }
    }

    #[inline]
    fn lex_percent(&mut self) -> (Token, Span) {
        let start = self.position;

        self.advance();

        if self.peek_char() == Some('=') {
            self.advance();

            (Token::PercentEqual, Span::new(start, self.position))
        } else {
            (Token::Percent, Span::new(start, self.position))
        }
    }

    #[inline]
    fn lex_exclamation_mark(&mut self) -> (Token, Span) {
        let start = self.position;

        self.advance();

        if self.peek_char() == Some('=') {
            self.advance();

            (Token::BangEqual, Span::new(start, self.position))
        } else {
            (Token::Bang, Span::new(start, self.position))
        }
    }

    #[inline]
    fn lex_equal(&mut self) -> (Token, Span) {
        let start = self.position;

        self.advance();

        if self.peek_char() == Some('=') {
            self.advance();

            (Token::DoubleEqual, Span::new(start, self.position))
        } else {
            (Token::Equal, Span::new(start, self.position))
        }
    }

    #[inline]
    fn lex_less_than(&mut self) -> (Token, Span) {
        let start = self.position;

        self.advance();

        if self.peek_char() == Some('=') {
            self.advance();

            (Token::LessEqual, Span::new(start, self.position))
        } else {
            (Token::Less, Span::new(start, self.position))
        }
    }

    #[inline]
    fn lex_greater_than(&mut self) -> (Token, Span) {
        let start = self.position;

        self.advance();

        if self.peek_char() == Some('=') {
            self.advance();

            (Token::GreaterEqual, Span::new(start, self.position))
        } else {
            (Token::Greater, Span::new(start, self.position))
        }
    }

    #[inline]
    fn lex_ampersand(&mut self) -> (Token, Span) {
        let start = self.position;

        self.advance();

        if self.peek_char() == Some('&') {
            self.advance();

            (Token::DoubleAmpersand, Span::new(start, self.position))
        } else {
            (Token::Unknown, Span::new(start, self.position))
        }
    }

    #[inline]
    fn lex_pipe(&mut self) -> (Token, Span) {
        let start = self.position;

        self.advance();

        let peek_char = self.peek_char();

        if let Some('|') = peek_char {
            self.advance();

            (Token::DoublePipe, Span::new(start, self.position))
        } else {
            (Token::Unknown, Span::new(start, self.position))
        }
    }

    #[inline]
    fn lex_dot(&mut self) -> (Token, Span) {
        let start = self.position;

        self.advance();

        if self.peek_char() == Some('.') {
            self.advance();

            (Token::DoubleDot, Span::new(start, self.position))
        } else {
            (Token::Dot, Span::new(start, self.position))
        }
    }

    #[inline]
    fn lex_colon(&mut self) -> (Token, Span) {
        let start = self.position;

        self.advance();

        if self.peek_char() == Some(':') {
            self.advance();

            (Token::DoubleColon, Span::new(start, self.position))
        } else {
            (Token::Colon, Span::new(start, self.position))
        }
    }

    #[inline]
    fn lex_carriage_return(&mut self) -> (Token, Span) {
        let start = self.position;

        self.advance();

        if self.peek_char() == Some('\n') {
            self.advance();
        }

        self.line_breaks.push(self.position as u32);

        (Token::Newline, Span::new(start, self.position))
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn character() {
//         let input = "'a'";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Character('a'), Span(0, 3)),
//                 (Token::Eof, Span(3, 3)),
//             ])
//         );
//     }

//     #[test]
//     fn map_expression() {
//         let input = "map { x = \"1\", y = 2, z = 3.0 }";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Map, Span(0, 3)),
//                 (Token::LeftBrace, Span(4, 5)),
//                 (Token::Identifier("x"), Span(6, 7)),
//                 (Token::Equal, Span(8, 9)),
//                 (Token::String("1"), Span(10, 13)),
//                 (Token::Comma, Span(13, 14)),
//                 (Token::Identifier("y"), Span(15, 16)),
//                 (Token::Equal, Span(17, 18)),
//                 (Token::Integer("2"), Span(19, 20)),
//                 (Token::Comma, Span(20, 21)),
//                 (Token::Identifier("z"), Span(22, 23)),
//                 (Token::Equal, Span(24, 25)),
//                 (Token::Float("3.0"), Span(26, 29)),
//                 (Token::RightBrace, Span(30, 31)),
//                 (Token::Eof, Span(31, 31)),
//             ])
//         );
//     }

//     #[test]
//     fn let_statement() {
//         let input = "let x = 42";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Let, Span(0, 3)),
//                 (Token::Identifier("x"), Span(4, 5)),
//                 (Token::Equal, Span(6, 7)),
//                 (Token::Integer("42"), Span(8, 10)),
//                 (Token::Eof, Span(10, 10)),
//             ])
//         );
//     }

//     #[test]
//     fn unit_struct() {
//         let input = "struct Foo";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Struct, Span(0, 6)),
//                 (Token::Identifier("Foo"), Span(7, 10)),
//                 (Token::Eof, Span(10, 10)),
//             ])
//         );
//     }

//     #[test]
//     fn tuple_struct() {
//         let input = "struct Foo(int, float)";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Struct, Span(0, 6)),
//                 (Token::Identifier("Foo"), Span(7, 10)),
//                 (Token::LeftParenthesis, Span(10, 11)),
//                 (Token::Int, Span(11, 14)),
//                 (Token::Comma, Span(14, 15)),
//                 (Token::FloatKeyword, Span(16, 21)),
//                 (Token::RightParenthesis, Span(21, 22)),
//                 (Token::Eof, Span(22, 22))
//             ])
//         );
//     }

//     #[test]
//     fn fields_struct() {
//         let input = "struct FooBar { foo: int, bar: float }";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Struct, Span(0, 6)),
//                 (Token::Identifier("FooBar"), Span(7, 13)),
//                 (Token::LeftBrace, Span(14, 15)),
//                 (Token::Identifier("foo"), Span(16, 19)),
//                 (Token::Colon, Span(19, 20)),
//                 (Token::Int, Span(21, 24)),
//                 (Token::Comma, Span(24, 25)),
//                 (Token::Identifier("bar"), Span(26, 29)),
//                 (Token::Colon, Span(29, 30)),
//                 (Token::FloatKeyword, Span(31, 36)),
//                 (Token::RightBrace, Span(37, 38)),
//                 (Token::Eof, Span(38, 38))
//             ])
//         );
//     }

//     #[test]
//     fn list_index() {
//         let input = "[1, 2, 3][1]";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::LeftBracket, Span(0, 1)),
//                 (Token::Integer("1"), Span(1, 2)),
//                 (Token::Comma, Span(2, 3)),
//                 (Token::Integer("2"), Span(4, 5)),
//                 (Token::Comma, Span(5, 6)),
//                 (Token::Integer("3"), Span(7, 8)),
//                 (Token::RightBracket, Span(8, 9)),
//                 (Token::LeftBracket, Span(9, 10)),
//                 (Token::Integer("1"), Span(10, 11)),
//                 (Token::RightBracket, Span(11, 12)),
//                 (Token::Eof, Span(12, 12)),
//             ])
//         )
//     }

//     #[test]
//     fn list() {
//         let input = "[1, 2, 3]";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::LeftBracket, Span(0, 1)),
//                 (Token::Integer("1"), Span(1, 2)),
//                 (Token::Comma, Span(2, 3)),
//                 (Token::Integer("2"), Span(4, 5)),
//                 (Token::Comma, Span(5, 6)),
//                 (Token::Integer("3"), Span(7, 8)),
//                 (Token::RightBracket, Span(8, 9)),
//                 (Token::Eof, Span(9, 9)),
//             ])
//         )
//     }

//     #[test]
//     fn map_field_access() {
//         let input = "{a = 1, b = 2, c = 3}.c";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::LeftBrace, Span(0, 1)),
//                 (Token::Identifier("a"), Span(1, 2)),
//                 (Token::Equal, Span(3, 4)),
//                 (Token::Integer("1"), Span(5, 6)),
//                 (Token::Comma, Span(6, 7)),
//                 (Token::Identifier("b"), Span(8, 9)),
//                 (Token::Equal, Span(10, 11)),
//                 (Token::Integer("2"), Span(12, 13)),
//                 (Token::Comma, Span(13, 14)),
//                 (Token::Identifier("c"), Span(15, 16)),
//                 (Token::Equal, Span(17, 18)),
//                 (Token::Integer("3"), Span(19, 20)),
//                 (Token::RightBrace, Span(20, 21)),
//                 (Token::Dot, Span(21, 22)),
//                 (Token::Identifier("c"), Span(22, 23)),
//                 (Token::Eof, Span(23, 23)),
//             ])
//         )
//     }

//     #[test]
//     fn range() {
//         let input = "0..42";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Integer("0"), Span(0, 1)),
//                 (Token::DoubleDot, Span(1, 3)),
//                 (Token::Integer("42"), Span(3, 5)),
//                 (Token::Eof, Span(5, 5))
//             ])
//         );
//     }

//     #[test]
//     fn negate_expression() {
//         let input = "x = -42; -x";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Identifier("x"), Span(0, 1)),
//                 (Token::Equal, Span(2, 3)),
//                 (Token::Integer("-42"), Span(4, 7)),
//                 (Token::Semicolon, Span(7, 8)),
//                 (Token::Minus, Span(9, 10)),
//                 (Token::Identifier("x"), Span(10, 11)),
//                 (Token::Eof, Span(11, 11))
//             ])
//         );
//     }

//     #[test]
//     fn not_expression() {
//         let input = "!true; !false";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Bang, Span(0, 1)),
//                 (Token::Boolean("true"), Span(1, 5)),
//                 (Token::Semicolon, Span(5, 6)),
//                 (Token::Bang, Span(7, 8)),
//                 (Token::Boolean("false"), Span(8, 13)),
//                 (Token::Eof, Span(13, 13))
//             ])
//         );
//     }

//     #[test]
//     fn if_else() {
//         let input = "if x < 10 { x + 1 } else { x }";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::If, Span(0, 2)),
//                 (Token::Identifier("x"), Span(3, 4)),
//                 (Token::Less, Span(5, 6)),
//                 (Token::Integer("10"), Span(7, 9)),
//                 (Token::LeftBrace, Span(10, 11)),
//                 (Token::Identifier("x"), Span(12, 13)),
//                 (Token::Plus, Span(14, 15)),
//                 (Token::Integer("1"), Span(16, 17)),
//                 (Token::RightBrace, Span(18, 19)),
//                 (Token::Else, Span(20, 24)),
//                 (Token::LeftBrace, Span(25, 26)),
//                 (Token::Identifier("x"), Span(27, 28)),
//                 (Token::RightBrace, Span(29, 30)),
//                 (Token::Eof, Span(30, 30)),
//             ])
//         )
//     }

//     #[test]
//     fn while_loop() {
//         let input = "while x < 10 { x += 1 }";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::While, Span(0, 5)),
//                 (Token::Identifier("x"), Span(6, 7)),
//                 (Token::Less, Span(8, 9)),
//                 (Token::Integer("10"), Span(10, 12)),
//                 (Token::LeftBrace, Span(13, 14)),
//                 (Token::Identifier("x"), Span(15, 16)),
//                 (Token::PlusEqual, Span(17, 19)),
//                 (Token::Integer("1"), Span(20, 21)),
//                 (Token::RightBrace, Span(22, 23)),
//                 (Token::Eof, Span(23, 23)),
//             ])
//         )
//     }

//     #[test]
//     fn add_assign() {
//         let input = "x += 42";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Identifier("x"), Span(0, 1)),
//                 (Token::PlusEqual, Span(2, 4)),
//                 (Token::Integer("42"), Span(5, 7)),
//                 (Token::Eof, Span(7, 7)),
//             ])
//         )
//     }

//     #[test]
//     fn or() {
//         let input = "true || false";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Boolean("true"), Span(0, 4)),
//                 (Token::DoublePipe, Span(5, 7)),
//                 (Token::Boolean("false"), Span(8, 13)),
//                 (Token::Eof, Span(13, 13)),
//             ])
//         )
//     }

//     #[test]
//     fn block() {
//         let input = "{ x = 42; y = \"foobar\" }";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::LeftBrace, Span(0, 1)),
//                 (Token::Identifier("x"), Span(2, 3)),
//                 (Token::Equal, Span(4, 5)),
//                 (Token::Integer("42"), Span(6, 8)),
//                 (Token::Semicolon, Span(8, 9)),
//                 (Token::Identifier("y"), Span(10, 11)),
//                 (Token::Equal, Span(12, 13)),
//                 (Token::String("foobar"), Span(14, 22)),
//                 (Token::RightBrace, Span(23, 24)),
//                 (Token::Eof, Span(24, 24)),
//             ])
//         )
//     }

//     #[test]
//     fn not_equal() {
//         let input = "42 != 42";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Integer("42"), Span(0, 2)),
//                 (Token::BangEqual, Span(3, 5)),
//                 (Token::Integer("42"), Span(6, 8)),
//                 (Token::Eof, Span(8, 8)),
//             ])
//         )
//     }

//     #[test]
//     fn equal() {
//         let input = "42 == 42";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Integer("42"), Span(0, 2)),
//                 (Token::DoubleEqual, Span(3, 5)),
//                 (Token::Integer("42"), Span(6, 8)),
//                 (Token::Eof, Span(8, 8)),
//             ])
//         )
//     }

//     #[test]
//     fn modulo() {
//         let input = "42 % 2";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Integer("42"), Span(0, 2)),
//                 (Token::Percent, Span(3, 4)),
//                 (Token::Integer("2"), Span(5, 6)),
//                 (Token::Eof, Span(6, 6)),
//             ])
//         )
//     }

//     #[test]
//     fn divide() {
//         let input = "42 / 2";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Integer("42"), Span(0, 2)),
//                 (Token::Slash, Span(3, 4)),
//                 (Token::Integer("2"), Span(5, 6)),
//                 (Token::Eof, Span(6, 6)),
//             ])
//         )
//     }

//     #[test]
//     fn greater_than() {
//         let input = ">";

//         assert_eq!(
//             lex(input),
//             Ok(vec![(Token::Greater, Span(0, 1)), (Token::Eof, Span(1, 1))])
//         )
//     }

//     #[test]
//     fn greater_than_or_equal() {
//         let input = ">=";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::GreaterEqual, Span(0, 2)),
//                 (Token::Eof, Span(2, 2))
//             ])
//         )
//     }

//     #[test]
//     fn less_than() {
//         let input = "<";

//         assert_eq!(
//             lex(input),
//             Ok(vec![(Token::Less, Span(0, 1)), (Token::Eof, Span(1, 1))])
//         )
//     }

//     #[test]
//     fn less_than_or_equal() {
//         let input = "<=";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::LessEqual, Span(0, 2)),
//                 (Token::Eof, Span(2, 2))
//             ])
//         )
//     }

//     #[test]
//     fn infinity() {
//         let input = "Infinity";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Float("Infinity"), Span(0, 8)),
//                 (Token::Eof, Span(8, 8)),
//             ])
//         )
//     }

//     #[test]
//     fn negative_infinity() {
//         let input = "-Infinity";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Float("-Infinity"), Span(0, 9)),
//                 (Token::Eof, Span(9, 9)),
//             ])
//         )
//     }

//     #[test]
//     fn nan() {
//         let input = "NaN";

//         assert!(lex(input).is_ok_and(|tokens| tokens[0].0 == Token::Float("NaN")));
//     }

//     #[test]
//     fn complex_float() {
//         let input = "42.42e42";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Float("42.42e42"), Span(0, 8)),
//                 (Token::Eof, Span(8, 8)),
//             ])
//         )
//     }

//     #[test]
//     fn max_integer() {
//         let input = "9223372036854775807";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Integer("9223372036854775807"), Span(0, 19)),
//                 (Token::Eof, Span(19, 19)),
//             ])
//         )
//     }

//     #[test]
//     fn min_integer() {
//         let input = "-9223372036854775808";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Integer("-9223372036854775808"), Span(0, 20)),
//                 (Token::Eof, Span(20, 20)),
//             ])
//         )
//     }

//     #[test]
//     fn subtract_negative_integers() {
//         let input = "-42 - -42";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Integer("-42"), Span(0, 3)),
//                 (Token::Minus, Span(4, 5)),
//                 (Token::Integer("-42"), Span(6, 9)),
//                 (Token::Eof, Span(9, 9)),
//             ])
//         )
//     }

//     #[test]
//     fn negative_integer() {
//         let input = "-42";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Integer("-42"), Span(0, 3)),
//                 (Token::Eof, Span(3, 3))
//             ])
//         )
//     }

//     #[test]
//     fn read_line() {
//         let input = "read_line()";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Identifier("read_line"), Span(0, 9)),
//                 (Token::LeftParenthesis, Span(9, 10)),
//                 (Token::RightParenthesis, Span(10, 11)),
//                 (Token::Eof, Span(11, 11)),
//             ])
//         )
//     }

//     #[test]
//     fn write_line() {
//         let input = "write_line(\"Hello, world!\")";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Identifier("write_line"), Span(0, 10)),
//                 (Token::LeftParenthesis, Span(10, 11)),
//                 (Token::String("Hello, world!"), Span(11, 26)),
//                 (Token::RightParenthesis, Span(26, 27)),
//                 (Token::Eof, Span(27, 27)),
//             ])
//         )
//     }

//     #[test]
//     fn string_concatenation() {
//         let input = "\"Hello, \" + \"world!\"";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::String("Hello, "), Span(0, 9)),
//                 (Token::Plus, Span(10, 11)),
//                 (Token::String("world!"), Span(12, 20)),
//                 (Token::Eof, Span(20, 20)),
//             ])
//         )
//     }

//     #[test]
//     fn string() {
//         let input = "\"Hello, world!\"";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::String("Hello, world!"), Span(0, 15)),
//                 (Token::Eof, Span(15, 15)),
//             ])
//         )
//     }

//     #[test]
//     fn r#true() {
//         let input = "true";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Boolean("true"), Span(0, 4)),
//                 (Token::Eof, Span(4, 4)),
//             ])
//         )
//     }

//     #[test]
//     fn r#false() {
//         let input = "false";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Boolean("false"), Span(0, 5)),
//                 (Token::Eof, Span(5, 5))
//             ])
//         )
//     }

//     #[test]
//     fn property_access_function_call() {
//         let input = "42.is_even()";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Integer("42"), Span(0, 2)),
//                 (Token::Dot, Span(2, 3)),
//                 (Token::Identifier("is_even"), Span(3, 10)),
//                 (Token::LeftParenthesis, Span(10, 11)),
//                 (Token::RightParenthesis, Span(11, 12)),
//                 (Token::Eof, Span(12, 12)),
//             ])
//         )
//     }

//     #[test]
//     fn empty() {
//         let input = "";

//         assert_eq!(lex(input), Ok(vec![(Token::Eof, Span(0, 0))]))
//     }

//     #[test]
//     fn reserved_identifier() {
//         let input = "length";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Identifier("length"), Span(0, 6)),
//                 (Token::Eof, Span(6, 6)),
//             ])
//         )
//     }

//     #[test]
//     fn square_braces() {
//         let input = "[]";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::LeftBracket, Span(0, 1)),
//                 (Token::RightBracket, Span(1, 2)),
//                 (Token::Eof, Span(2, 2)),
//             ])
//         )
//     }

//     #[test]
//     fn small_float() {
//         let input = "1.23";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Float("1.23"), Span(0, 4)),
//                 (Token::Eof, Span(4, 4)),
//             ])
//         )
//     }

//     #[test]
//     #[allow(clippy::excessive_precision)]
//     fn big_float() {
//         let input = "123456789.123456789";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Float("123456789.123456789"), Span(0, 19)),
//                 (Token::Eof, Span(19, 19)),
//             ])
//         )
//     }

//     #[test]
//     fn float_with_exponent() {
//         let input = "1.23e4";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Float("1.23e4"), Span(0, 6)),
//                 (Token::Eof, Span(6, 6)),
//             ])
//         )
//     }

//     #[test]
//     fn float_with_negative_exponent() {
//         let input = "1.23e-4";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Float("1.23e-4"), Span(0, 7)),
//                 (Token::Eof, Span(7, 7)),
//             ])
//         )
//     }

//     #[test]
//     fn float_infinity_and_nan() {
//         let input = "Infinity -Infinity NaN";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Float("Infinity"), Span(0, 8)),
//                 (Token::Float("-Infinity"), Span(9, 18)),
//                 (Token::Float("NaN"), Span(19, 22)),
//                 (Token::Eof, Span(22, 22)),
//             ])
//         )
//     }

//     #[test]
//     fn add() {
//         let input = "1 + 2";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Integer("1"), Span(0, 1)),
//                 (Token::Plus, Span(2, 3)),
//                 (Token::Integer("2"), Span(4, 5)),
//                 (Token::Eof, Span(5, 5)),
//             ])
//         )
//     }

//     #[test]
//     fn multiply() {
//         let input = "1 * 2";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Integer("1"), Span(0, 1)),
//                 (Token::Star, Span(2, 3)),
//                 (Token::Integer("2"), Span(4, 5)),
//                 (Token::Eof, Span(5, 5)),
//             ])
//         )
//     }

//     #[test]
//     fn add_and_multiply() {
//         let input = "1 + 2 * 3";

//         assert_eq!(
//             lex(input),
//             Ok(vec![
//                 (Token::Integer("1"), Span(0, 1)),
//                 (Token::Plus, Span(2, 3)),
//                 (Token::Integer("2"), Span(4, 5)),
//                 (Token::Star, Span(6, 7)),
//                 (Token::Integer("3"), Span(8, 9)),
//                 (Token::Eof, Span(9, 9)),
//             ])
//         );
//     }

//     #[test]
//     fn assignment() {
//         let input = "a = 1 + 2 * 3";

//         assert_eq!(
//             lex(input,),
//             Ok(vec![
//                 (Token::Identifier("a"), Span(0, 1)),
//                 (Token::Equal, Span(2, 3)),
//                 (Token::Integer("1"), Span(4, 5)),
//                 (Token::Plus, Span(6, 7)),
//                 (Token::Integer("2"), Span(8, 9)),
//                 (Token::Star, Span(10, 11)),
//                 (Token::Integer("3"), Span(12, 13)),
//                 (Token::Eof, Span(13, 13)),
//             ])
//         );
//     }
// }
