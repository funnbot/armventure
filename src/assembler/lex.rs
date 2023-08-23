use crate::code_file::CodeSpan;
use crate::code_stream::SourceIterator;
use crate::stream::Stream;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Unexpected,
    ExpectedDigit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberBase {
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
    Int(NumberBase),
    Float,
    Colon,
    Dot,
    Comma,
    LeftSquareBracket,
    RightSquareBracket,
}

pub fn variant_eq<Enum>(lhs: &Enum, rhs: &Enum) -> bool {
    std::mem::discriminant(lhs) == std::mem::discriminant(rhs)
}

#[derive(Clone, Copy)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub span: CodeSpan<'a>,
}

impl<'s> PartialEq<TokenKind> for Token<'s> {
    fn eq(&self, other: &TokenKind) -> bool {
        self.kind == *other
    }
}

pub struct Scanner<'a> {
    source: &'a str,
    it: SourceIterator<'a>,
}

impl<'a> Scanner<'a> {
    #[must_use]
    pub fn new(source: &'a str) -> Self {
        Scanner {
            source,
            it: SourceIterator::new(source),
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
                NumberBase::Hex
            }
            Some('b') => {
                self.it.next();
                NumberBase::Bin
            }
            Some('.') => NumberBase::Dec,
            Some('0'..='9' | '_') => NumberBase::Dec,
            _ => return TokenKind::Int(NumberBase::Dec),
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
    #[expect(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<Token<'a>> {
        if self.it.is_eof() {
            return None;
        }
        self.skip_space();

        let start_location = self.it.location;
        let start_index = self.it.index;
        let kind = self.parse_token();
        let token_src = &self.source[start_index..self.it.index];

        Some(Token {
            span: CodeSpan {
                loc: start_location,
                src: token_src,
                byte_index: start_index,
            },
            kind,
        })
    }
}

impl fmt::Debug for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Token:{:?} {:?}", self.kind, self.span)
    }
}

#[cfg(test)]
mod tests {
    
    use super::*;
    use crate::probably::Probably;
    use crate::batch_assert_matches;
    #[test]
    fn scanner() -> Probably {
        let mut s = Scanner::new("hello #12345");
        batch_assert_matches! {
            s.next()?.span.src => "hello",
            s.next()?.span.src => "#12345",
            s.next() => None,
        }
        Probably
    }
}
