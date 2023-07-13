use std::fmt::{self, Write};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceLocation {
    pub line: u32,
    pub column: u32,
}

impl SourceLocation {
    pub fn default() -> Self {
        SourceLocation { line: 1, column: 1 }
    }
}

pub struct Instruction {
    pub location: SourceLocation,
    pub operands: Vec<Operand>,
}

pub struct Operand {
    pub location: SourceLocation,
}

macro_rules! DefineInstruction {
    ($name:ident) => {};
}

pub const MOV: Instruction = Instruction {
    location: SourceLocation { line: 1, column: 2 },
    operands: Vec::new(),
};

pub struct TokenStream<'a> {
    src: &'a str,
    iter: std::str::CharIndices<'a>,
}

impl<'a> TokenStream<'a> {
    pub fn next(&mut self) -> Option<Token<'a>> {
        let (cur_index, cur_char) = self.iter.next()?;

        todo!();
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TokenKind {
    Alpha,
    Digit,
    Punct,
    Newline,
    Space,
    Other,
}

pub struct Token<'a> {
    str: &'a str,
    location: SourceLocation,
    kind: TokenKind,
}

impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{{:?}:{}:{} \"{}\"}}",
            self.kind, self.location.line, self.location.column, self.str
        )
    }
}

impl<'a> fmt::Debug for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{{:?}:{}:{} \"{}\"}}",
            self.kind, self.location.line, self.location.column, self.str
        )
    }
}

fn char_kind(c: char) -> TokenKind {
    if c == ' ' || c == '\t' {
        TokenKind::Space
    } else if c.is_ascii_digit() {
        TokenKind::Digit
    } else if c.is_ascii_punctuation() {
        TokenKind::Punct
    } else if c.is_ascii_alphabetic() {
        TokenKind::Alpha
    } else {
        TokenKind::Other
    }
}

struct TokenStart {
    index: usize,
    location: SourceLocation,
    kind: TokenKind,
}

fn generate_token_kinds_for_line<'a>(
    line: &'a str,
    result: &mut Vec<Token<'a>>,
    line_number: usize,
) {
    let mut current: Option<TokenStart> = None;
    // char type is 4 bytes, chars() is an iterator of utf8 code points
    let mut iter = line.char_indices().enumerate();

    while let Some((column_index, (byte_index, ch))) = iter.next() {
        let kind = char_kind(ch);

        if let Some(start) = current.as_ref() {
            if start.kind != kind || start.kind == TokenKind::Punct {
                result.push(Token {
                    str: &line[start.index..byte_index],
                    location: start.location,
                    kind: start.kind,
                });
                current = None;
            }
        }

        if current.is_none() {
            current = Some(TokenStart {
                index: byte_index,
                location: SourceLocation {
                    line: line_number as u32,
                    column: (column_index + 1) as u32,
                },
                kind,
            });
        }
    }

    if let Some(start) = current.as_ref() {
        result.push(Token {
            str: &line[start.index..],
            location: start.location,
            kind: start.kind,
        });
    }

    if let Some(last) = result.last() {
        if last.str == "\r" {}
    }

    //result.push(Token { str: &line[] });
}

pub fn generate_token_kinds<'a>(src: &'a str) -> Vec<Token<'a>> {
    let mut result = Vec::<Token<'a>>::new();
    for (line_index, line) in src.split_inclusive('\n').enumerate() {
        let line_number = line_index + 1;
        generate_token_kinds_for_line(line, &mut result, line_number);
    }

    result
}

pub fn assemble_string(txt: &str, line_number: usize, column_number: usize) -> usize {
    todo!();
}

pub fn assemble_text_from_file(text: &str) {
    if text.is_empty() {
        return;
    }

    // line ending agnostic
    for (line_number, line) in text.lines().enumerate() {
        // char type is 4 bytes, chars() is an iterator of utf8 code points
        for (column_number, chara) in line.chars().enumerate() {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_generates_tokens_for_lines() {
        let ln = "TEXT1TEXT1Something{}[]";
        let mut result = Vec::<Token<'static>>::new();
        generate_token_kinds_for_line(ln, &mut result, 1);
        println!("{:?}", result);
    }
}
