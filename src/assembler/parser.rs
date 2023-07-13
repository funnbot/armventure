use std::ascii::Char as Ascii;
use std::mem::Discriminant;

use crate::scan::{variant_eq, Scanner, SourceSlice};
use crate::scan::{Token, TokenKind};

pub mod ast {
    use crate::scan::{SourceSlice, Token};

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
}

use ast::Node;

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

struct ScannerIterator<'a> {
    scanner: Scanner<'a>,
    current: Option<Token<'a>>,
}
impl<'a> ScannerIterator<'a> {
    fn new(mut scanner: Scanner<'a>) -> Self {
        let current = scanner.next();
        Self { scanner, current }
    }
    fn peek(&self) -> Result<&Token<'a>> {
        self.current.as_ref().ok_or(Error::EOF)
    }
    fn next(&mut self) -> Result<Token<'a>> {
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
    fn next_skip_space(&mut self) -> Result<Token<'a>> {
        self.skip_space();
        self.next()
    }
    fn peek_skip_space(&mut self) -> Result<&Token<'a>> {
        self.skip_space();
        self.peek()
    }
    fn next_if<F: Fn(TokenKind) -> bool>(&mut self, pred: F) -> Option<Token<'a>> {
        if let Some(current) = &self.current {
            if pred(current.kind) {
                let current = *current; // move out
                self.current = self.scanner.next();
                return Some(current);
            }
        }
        None
    }
    fn next_if_eq(&mut self, kind: TokenKind) -> Option<Token<'a>> {
        if let Some(current) = &self.current {
            if kind == current.kind {
                let current = *current; // move out
                self.current = self.scanner.next();
                return Some(current);
            }
        }
        None
    }
    fn expect(&mut self, kind: TokenKind) -> Result<Token<'a>> {
        match self.next() {
            Ok(token) if token.kind == kind => Ok(token),
            _ => Err(Error::Expected(kind)),
        }
    }
}

pub struct Parser<'a> {
    nodes: Vec<ast::Node<'a>>,
    it: ScannerIterator<'a>,
    node: Option<ast::Node<'a>>,
}

impl<'a> Parser<'a> {
    #[must_use]
    pub fn new(scanner: Scanner<'a>) -> Self {
        Self {
            nodes: Vec::new(),
            it: ScannerIterator::new(scanner),
            node: None,
        }
    }

    fn next_state(&mut self, state: State) -> Result<State> {
        use State as S;
        use TokenKind as T;
        let Parser { it, node, .. } = self;

        match state {
            S::ExpectAny => match it.peek()?.kind {
                T::Identifier => Ok(S::ExpectInstOrLabel),
                T::Symbol(Ascii::FullStop) => Ok(S::ExpectDirective),
                T::Newline | T::Space | T::Comment => Ok(state),
                _ => Err(Error::UnexpectedToken),
            },
            S::ExpectInstOrLabel => {
                let name = it.expect(T::Identifier)?.slice;
                match it.peek_skip_space()?.kind {
                    T::Symbol(Ascii::Colon) => {
                        // TODO: error if node is not none
                        _ = node.insert(Node::Label { name });
                        Ok(S::ExpectAny)
                    }
                    _ => {
                        *node = Some(Node::Inst {
                            mnem: name,
                            args: Vec::new(),
                        });
                        Ok(S::ExpectArgs)
                    }
                }
            }
            S::ExpectDirective => {
                it.expect(T::Symbol(Ascii::FullStop))?;
                let name = it.expect(T::Identifier)?.slice;

                todo!()
            }
            _ => todo!(),
        }
    }

    fn recover_error_state(&mut self, state: State, err: Error) -> State {
        todo!()
    }

    pub fn run(&mut self) {
        let mut cur_state = State::ExpectAny;
        loop {
            cur_state = match self.next_state(cur_state) {
                Ok(State::Exit) => break,
                Ok(state) => state,
                Err(err) => self.recover_error_state(cur_state, err),
            }
        }
    }
}
