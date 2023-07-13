#![feature(assert_matches)]

use armventure::scan::{Scanner, TokenKind, NumberBase};
use std::assert_matches::assert_matches;
use TokenKind::*;

const SRC_1: &'static str = "100_000_000 Hello";
const KINDS_1: &'static [TokenKind] = &[Number(NumberBase::Dec), Space, Identifier];

fn assert_kinds(src: &str, expected: &[TokenKind]) {
    let mut scanner = Scanner::new(src);
    for kind in expected.iter() {
        assert_matches!(scanner.next(), Some(t) if t.kind == *kind);
    }
    assert_matches!(scanner.next(), None);
}
fn assert_chars(src: &str, expected: &[&str]) {
    let mut scanner = Scanner::new(src);
    for &s in expected.iter() {
        assert_matches!(scanner.next(),
                Some(t) if t.slice.src.eq(s));
    }
    assert_matches!(scanner.next(), None);
}

#[test]
fn scanner_test() {
    assert_kinds(SRC_1, KINDS_1);
}
