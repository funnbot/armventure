#![feature(assert_matches)]
#![feature(ascii_char)]
#![feature(ascii_char_variants)]

use armventure::assembler::lex::{NumberBase, Scanner, TokenKind};
use std::ascii::Char as Ascii;
use std::assert_matches::assert_matches;
use TokenKind::*;

const SRC_1: &str = "#100_000_000 Hello
SUB r1, r1, #0xF10 // foo
";
const KINDS_1: &[TokenKind] = &[
    Int(NumberBase::Dec),
    Identifier,
    Newline,
    Identifier,
    Identifier,
    Comma,
    Identifier,
    Comma,
    Int(NumberBase::Hex),
    Newline,
];
const STRS_1: &[&str] = &[
    "#100_000_000",
    "Hello",
    "\n",
    "SUB",
    "r1",
    ",",
    "r1",
    ",",
    "#0xF10",
    "\n",
];

fn assert_kinds(src: &str, expected: &[TokenKind]) {
    let mut scanner = Scanner::new(src);
    for kind in expected.iter() {
        assert_matches!(scanner.next(), Some(t) if t.kind == *kind, "expected: {:?}", *kind);
    }
    assert_matches!(scanner.next(), None);
}
fn assert_chars(src: &str, expected: &[&str]) {
    let mut scanner = Scanner::new(src);
    for &s in expected.iter() {
        assert_matches!(scanner.next(),
                Some(t) if t.span.src.eq(s), "expected: \"{s}\"");
    }
    assert_matches!(scanner.next(), None);
}

#[test]
fn scanner_test() {
    assert_kinds(SRC_1, KINDS_1);
    assert_chars(SRC_1, STRS_1);
}
