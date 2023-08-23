use crate::batch_assert_matches;
use crate::code_file::{CodeLoc, CodeSpan};
use crate::stream::{MultiPeek, Stream};
use std::ascii::Char as Ascii;

pub(crate) struct SourceIterator<'s> {
    pub inner: std::str::CharIndices<'s>,
    pub current: Option<char>,
    pub location: CodeLoc,
    pub index: usize,
}

impl<'s> Stream for SourceIterator<'s> {
    type Item = char;

    fn peek(&self) -> Option<&char> {
        self.current.as_ref()
    }
    fn next(&mut self) -> Option<char> {
        let c = *self.peek()?;
        self.index = self.inner.offset();
        self.current = self.inner.next().map(|i| i.1);

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
            self.peek().copied()
        } else {
            self.iter().nth(n - 1)
        }
    }
}

impl<'a> SourceIterator<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut inner = source.char_indices();
        let current = inner.next().map(|i| i.1);
        Self {
            inner,
            current,
            location: CodeLoc::new(),
            index: 0,
        }
    }
    pub fn iter(&self) -> std::str::Chars<'a> {
        self.inner.as_str().chars().clone()
    }
    pub fn next_till_newline(&mut self) {
        while let Some(ch) = self.peek().copied() {
            if ch == '\n' || (ch == '\r' && self.peek_by(1) == Some('\n')) {
                break;
            }
            self.next();
        }
    }
    pub const fn is_eof(&self) -> bool {
        self.current.is_none()
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
            1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4,
            4, 4,
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
