use crate::scan::{Scanner, SourceSlice};
use crate::scan::{Token, TokenKind};
use crate::stream::{Stream, PeekLifetime, PeekItem};

pub enum ErrorKind {
    Undefined,
}

pub enum ArgKind {
    Reg { name: char, index: u8 },
    NumLiteral { value: usize, base: u8 },
    StrLiteral { value: String },
    Error(ErrorKind),
}

pub struct Arg<'a> {
    pub token: Token<'a>,
    pub kind: ArgKind,
}

pub enum Node<'a> {
    Direct {
        name: SourceSlice<'a>,
        args: Vec<Arg<'a>>,
    },
    Inst {
        mnem: SourceSlice<'a>,
        args: Vec<Arg<'a>>,
    },
    Label {
        name: SourceSlice<'a>,
    },
    Error(ErrorKind),
    Undefined,
}

#[derive(Debug, Clone, Copy)]
enum State {
    ExpectAny,
    ExpectInstOrLabel,
    ExpectInst,
    ExpectLabel,
    ExpectDirective,
    ExpectArgs,
    ExpectOneArg,
    Exit,
}

#[derive(Debug, Clone, Copy)]
enum Error {
    UnexpectedToken,
    Expected(TokenKind),
    EOF,
}

type Result<T> = std::result::Result<T, Error>;

struct TokenStream<'src> {
    scanner: Scanner<'src>,
    current: Option<Token<'src>>,
}

impl<'this, 'src> PeekLifetime<'this> for TokenStream<'src> {
    type PeekItem = &'this Token<'src>;
}

impl<'src> Stream for TokenStream<'src> {
    type Item = Token<'src>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }

    fn peek(&self) -> Option<&Token<'src>> {
        todo!()
    }
}

impl<'src> TokenStream<'src> {
    fn new(mut scanner: Scanner<'src>) -> Self {
        let current = scanner.next();
        Self { scanner, current }
    }

    fn skip_space(&mut self) -> bool {
        let mut skipped = false;
        while let Some(token) = self.current {
            if token.kind == TokenKind::Space {
                skipped = true;
                self.next();
            } else {
                return skipped;
            }
        }
        false
    }

    fn next_skip_space(&mut self) -> Option<Token<'src>> {
        self.skip_space();
        self.next()
    }
    fn peek_skip_space(&mut self) -> Option<&Token<'src>> {
        self.skip_space();
        self.peek()
    }
    fn expect(&mut self, kind: TokenKind) -> Result<Token<'src>> {
        match self.next() {
            Some(token) if token.kind == kind => Ok(token),
            _ => Err(Error::Expected(kind)),
        }
    }
}

struct ScannerIterator<'a> {
    scanner: Scanner<'a>,
    current: Option<Token<'a>>,
}
impl<'src> ScannerIterator<'src> {
    fn new(mut scanner: Scanner<'src>) -> Self {
        let current = scanner.next();
        Self { scanner, current }
    }
    fn peek(&self) -> Result<&Token<'src>> {
        self.current.as_ref().ok_or(Error::EOF)
    }
    fn next(&mut self) -> Result<Token<'src>> {
        match self.current {
            Some(token) => {
                self.current = self.scanner.next();
                Ok(token)
            }
            None => Err(Error::EOF),
        }
    }
    fn skip_space(&mut self) -> bool {
        let mut skipped = false;
        while let Some(token) = self.current {
            if matches!(token.kind, TokenKind::Space { .. }) {
                skipped = true;
                _ = self.next();
            } else {
                return skipped;
            }
        }
        false
    }

    fn next_if<F: Fn(TokenKind) -> bool>(&mut self, pred: F) -> Option<Token<'src>> {
        if let Some(current) = &self.current {
            if pred(current.kind) {
                let current = *current; // move out
                self.current = self.scanner.next();
                return Some(current);
            }
        }
        None
    }
    fn next_if_eq(&mut self, kind: TokenKind) -> Option<Token<'src>> {
        if let Some(current) = &self.current {
            if kind == current.kind {
                let current = *current; // move out
                self.current = self.scanner.next();
                return Some(current);
            }
        }
        None
    }
}
