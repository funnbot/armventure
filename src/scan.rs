use std::ascii::Char as Ascii;
use std::fmt;
use std::num::NonZeroU32;
use std::str::Chars;

use crate::stream::{Stream, PeekLifetime, PeekItem, MultiPeek};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceLocation {
    pub line: NonZeroU32,
    pub column: NonZeroU32,
}

impl SourceLocation {
    #[must_use]
    #[allow(clippy::missing_panics_doc, reason = "const")]
    pub fn new() -> Self {
        Self {
            line: NonZeroU32::new(1).unwrap(),
            column: NonZeroU32::new(1).unwrap(),
        }
    }
    /// # Panics
    /// out of range
    pub fn inc_line(&mut self) {
        self.line = self.line.checked_add(1).unwrap();
        self.column = NonZeroU32::new(1).unwrap();
    }
    /// # Panics
    /// out of range
    pub fn inc_column(&mut self) {
        self.column = self.column.checked_add(1).unwrap();
    }
}

impl Default for SourceLocation {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SourceSlice<'a> {
    pub location: SourceLocation,
    pub src: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Unexpected,
    Expected(Ascii),
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
    Space,
    Comment,
    Newline,
    Other,
    Symbol(Ascii),
    Error(ErrorKind),
    Number(NumberBase),
}

pub fn variant_eq<Enum>(lhs: &Enum, rhs: &Enum) -> bool {
    std::mem::discriminant(lhs) == std::mem::discriminant(rhs)
}

#[derive(Clone, Copy)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub slice: SourceSlice<'a>,
}

impl<'s> PartialEq<TokenKind> for Token<'s> {
    fn eq(&self, other: &TokenKind) -> bool {
        self.kind == *other
    }
}

struct SourceIterator<'s> {
    inner: Chars<'s>,
    current: Option<char>,
    location: SourceLocation,
}

impl<'this, 's> PeekLifetime<'this> for SourceIterator<'s> {
    type PeekItem = char;
}

impl<'s> Stream for SourceIterator<'s> {
    type Item = char;

    fn peek(&self) -> Option<char> {
        self.current
    }
    fn next(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.current = self.inner.next();

        // works with \r\n
        if c == '\n' {
            self.location.inc_line();
        } else {
            self.location.inc_column();
        }
        Some(c)
    }
}

impl<'s> MultiPeek for SourceIterator<'s> {
    fn peek_by(&self, n: usize) -> Option<char> {
        if n == 0 {
            self.peek()
        } else {
            self.iter().nth(n - 1)
        }
    }
}

impl<'a> SourceIterator<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut inner = source.chars();
        let current = inner.next();
        Self {
            inner,
            current,
            location: SourceLocation::new(),
        }
    }
    pub fn iter(&self) -> std::str::Chars<'a> {
        self.inner.clone()
    }
    pub fn next_till_newline(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\n' || (ch == '\r' && self.peek_by(1) == Some('\n')) {
                break;
            }
            self.next();
        }
    }
    // sizeof peek() is not included
    pub fn num_bytes(&self) -> usize {
        self.inner.as_str().len()
    }
    pub const fn is_eof(&self) -> bool {
        self.current.is_none()
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

    // called after consuming '0'..='9'
    fn parse_number(&mut self) -> TokenKind {
        let base = match self.it.peek() {
            Some('x') => NumberBase::Hex,
            Some('b') => NumberBase::Bin,
            Some('0'..='9' | '_') => NumberBase::Dec,
            _ => return TokenKind::Number(NumberBase::Dec),
        };
        self.it.next_while(|c| c.is_digit(base as u32) || c == '_');
        TokenKind::Number(base)
    }
    // called after consuming if c.is_alphabetic() || c == '_'
    fn parse_ident(&mut self) -> TokenKind {
        self.it.next_while(|c| c.is_alphanumeric() || c == '_');
        TokenKind::Identifier
    }
    // called after consuming '"'|'\''
    fn parse_string(&mut self, delim: char) -> TokenKind {
        let mut escaped = false;
        while let Some(c) = self.it.peek() {
            if c == '\\' {
                escaped = !escaped;
            } else if c == delim && !escaped {
                break;
            }
            self.it.next();
        }
        assert_eq!(self.it.next(), Some(delim));
        TokenKind::String
    }
    // called after consuming ' '|'\t'
    fn parse_space(&mut self) -> TokenKind {
        self.it.next_while(|c| c == ' ' || c == '\t');
        TokenKind::Space
    }
    // called after consuming "//"
    fn parse_comment(&mut self) -> TokenKind {
        self.it.next_till_newline();
        TokenKind::Comment
    }
    fn parse_token(&mut self) -> TokenKind {
        match self.it.next().expect("eof") {
            '\n' => TokenKind::Newline,
            '\r' if self.it.next_if_eq('\n').is_some() => TokenKind::Newline,
            '0'..='9' => self.parse_number(),
            c @ ('"' | '\'') => self.parse_string(c),
            '/' if self.it.next_if_eq('/').is_some() => self.parse_comment(),
            ' ' | '\t' => self.parse_space(),
            c if c.is_alphabetic() || c == '_' => self.parse_ident(),
            c if c.is_ascii() => TokenKind::Symbol(Ascii::from_u8(c as u8).unwrap()),
            c => TokenKind::Other,
        }
    }
    // given the next character, produce a token, possibly consuming some proceeding characters as well
    // should be as agnostic to the final grammar as possible
    //
    // symbols: one character, all other unicode values are a single character symbol \
    // strings: find the closing symbol if it isn't escaped. don't escape others \
    // numbers: `123  123.123  123.  0b101  0x1FFF  100_000` \
    // identifiers: starts with _ or any unicode alphabet, rest is _ or any unicode alphanumeric \
    // newline: \n or \r\n
    #[expect(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<Token<'a>> {
        if self.it.is_eof() {
            return None;
        }

        //let start_index = self.it.current_index().unwrap_or(0);
        let start_location = self.it.location;

        let src_rest = self.it.inner.as_str();
        let bytes_before = self.it.num_bytes();

        let kind = self.parse_token();

        let bytes_read = bytes_before - self.it.num_bytes();
        let token_src = &src_rest[..bytes_read];

        Some(Token {
            slice: SourceSlice {
                location: start_location,
                src: token_src,
            },
            kind,
        })
    }
}

macro_rules! batch_assert_matches {
    { $( $value:expr => $pattern:pat),+ $(,)? } =>
    {
        $(
            match $value {
                $pattern => {},
                ref left_value => std::panic!("{}, {}:{}:{}", ${index()}, std::file!(), std::line!(), std::column!()),
            }
        );+
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

impl fmt::Debug for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{{:?}:{} \"{}\"}}",
            self.kind, self.slice.location, self.slice.src
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probably::Probably;

    #[test]
    fn scanner_iterator_newline() {
        const src: &str = "123\n1234\n12345\r\n123456\r\n";
        const exp: &[usize; 24] = &[
            1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4,
        ];
        let a: Option<usize> = None;
        let mut iter = SourceIterator::new(src);
        for (i, e) in exp.iter().enumerate() {
            assert_eq!(
                *e,
                iter.location.line.get() as usize,
                "char[{}]: {}",
                i,
                iter.peek().unwrap().escape_default()
            );
            iter.next();
        }
    }

    #[test]
    fn peek_by() -> Probably {
        let mut s = SourceIterator::new("01234567");
        batch_assert_matches! {
            s.next()? => '0',
            s.next()? => '1',
            s.peek()? => '2',
            s.peek_by(2)? => '4',
            s.next()? => '2',
            s.next()? => '3',
            s.next()? => '4',
        }
        Probably
    }
}
