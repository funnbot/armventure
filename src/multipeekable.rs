use std::cmp::min;
use std::collections::VecDeque;

pub struct MultiPeekable<Iter: Iterator> {
    inner: Iter,
    peeked: VecDeque<Iter::Item>,
}

impl<Iter: Iterator> Iterator for MultiPeekable<Iter> {
    type Item = Iter::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.peeked.is_empty() {
            return self.inner.next();
        }
        return self.peeked.pop_front();
    }
}

impl<Iter: Iterator> MultiPeekable<Iter> {
    pub fn new(iter: Iter) -> Self {
        Self {
            inner: iter,
            peeked: VecDeque::new(),
        }
    }
    pub fn peek(&mut self) -> Option<&Iter::Item> {
        if !self.peeked.is_empty() {
            return self.peeked.front();
        }
        if let Some(item) = self.inner.next() {
            self.peeked.push_back(item);
            debug_assert!(self.peeked.len() == 1);
            return self.peeked.front();
        }
        None
    }
    pub fn peek_by(&mut self, n: usize) -> Option<&Iter::Item> {
        assert!(n != 0, "peek_by(0) is a past value");
        let to_advance = n - min(self.peeked.len(), n);
        for _ in 0..to_advance {
            if let Some(item) = self.inner.next() {
                self.peeked.push_back(item);
            } else {
                return None;
            }
        }
        Some(&self.peeked[n - 1])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_lookahead() {
        let str = "012345678";
        let mut iter = MultiPeekable::new(str.chars());
        assert_eq!(iter.next(), Some('0'));
        assert_eq!(iter.peek(), Some(&'1'));
        assert_eq!(iter.peek_by(1), Some(&'1'));
        assert_eq!(iter.peek_by(5), Some(&'5'));
        assert_eq!(iter.next(), Some('1'));
        assert_eq!(iter.peek(), Some(&'2'));
    }
}
