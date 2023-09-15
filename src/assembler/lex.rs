use crate::{code, code::Span, code_stream::SourceStream, stream::Stream};
use std::{borrow::BorrowMut, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Unexpected,
    ExpectedDigit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntRadix {
    Bin = 2,
    Dec = 10,
    Hex = 16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Identifier,
    String,
    Newline,
    Error(ErrorKind),
    Int(IntRadix),
    Float,
    Colon,
    Dot,
    Comma,
    LeftSquareBracket,
    RightSquareBracket,
}

impl TokenKind {
    pub const fn is_int(&self) -> bool {
        matches!(self, Self::Int(..))
    }
}

pub fn variant_eq<Enum>(lhs: &Enum, rhs: &Enum) -> bool {
    std::mem::discriminant(lhs) == std::mem::discriminant(rhs)
}

#[derive(Clone, Copy)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl PartialEq<TokenKind> for Token {
    fn eq(&self, other: &TokenKind) -> bool {
        self.kind == *other
    }
}

pub struct Scanner<'a> {
    source: &'a code::Source,
    it: SourceStream<'a>,
}

impl<'a> Scanner<'a> {
    #[must_use]
    pub fn new(source: &'a code::Source) -> Self {
        Scanner {
            source,
            it: SourceStream::new(source.str()),
        }
    }
    // called after consuming '#'
    fn parse_number(&mut self) -> TokenKind {
        if self.it.next_if(|&c| c.is_ascii_digit()).is_none() {
            return TokenKind::Error(ErrorKind::ExpectedDigit);
        }
        let base = match self.it.peek() {
            Some('x') => {
                self.it.next();
                IntRadix::Hex
            }
            Some('b') => {
                self.it.next();
                IntRadix::Bin
            }
            Some('.') => IntRadix::Dec,
            Some('0'..='9' | '_') => IntRadix::Dec,
            _ => return TokenKind::Int(IntRadix::Dec),
        };
        self.it.next_while(|&c| c.is_digit(base as u32) || c == '_');

        if self.it.next_if_eq('.').is_some() {
            self.it.next_while(|&c| c.is_ascii_digit() || c == '_');
            TokenKind::Float
        } else {
            TokenKind::Int(base)
        }
    }
    // called after consuming if c.is_ascii_alphabetic() || c == '_'
    fn parse_ident(&mut self) -> TokenKind {
        self.it
            .next_while(|&c| c.is_ascii_alphanumeric() || c == '_');
        TokenKind::Identifier
    }
    // called after consuming '"'|'\''
    fn parse_string(&mut self, delim: char) -> TokenKind {
        let mut escaped = false;
        while let Some(&c) = self.it.peek() {
            if c == '\\' {
                escaped = !escaped;
            } else if c == delim && !escaped {
                break;
            } else {
                escaped = false;
            }
            self.it.next();
        }
        assert_eq!(self.it.next(), Some(delim));
        TokenKind::String
    }
    fn skip_space(&mut self) {
        while let Some(ch) = self.it.peek().copied() {
            match ch {
                ' ' | '\t' => {
                    self.it.next();
                }
                '/' if self.it.next_if_eq('/').is_some() => {
                    self.it.next_till_newline();
                    break;
                }
                _ => break,
            }
        }
    }
    fn parse_token(&mut self) -> TokenKind {
        match self.it.next().expect("eof") {
            '\n' => TokenKind::Newline,
            ':' => TokenKind::Colon,
            '.' => TokenKind::Dot,
            '[' => TokenKind::LeftSquareBracket,
            ']' => TokenKind::RightSquareBracket,
            ',' => TokenKind::Comma,
            '\r' if self.it.next_if_eq('\n').is_some() => TokenKind::Newline,
            '#' => self.parse_number(),
            c @ ('"' | '\'') => self.parse_string(c),
            c if c.is_ascii_alphabetic() || c == '_' => self.parse_ident(),
            _ => todo!("error unexpected token"),
        }
    }

    pub fn next(&mut self) -> Option<Token> {
        if self.it.is_eof() {
            return None;
        }
        self.skip_space();

        let start_location = self.it.location;
        let start_index = self.it.index;
        let kind = self.parse_token();
        let span = self
            .source
            .create_span(start_location, start_index, self.it.index);

        Some(Token { span, kind })
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Token:{:?}{:?}", self.kind, self.span.loc())
    }
}
