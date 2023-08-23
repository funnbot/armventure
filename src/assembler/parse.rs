use crate::assembler::lex::{NumberBase, Scanner, Token, TokenKind};
use crate::code_file::{CodeLoc, CodeSpan};
use crate::stream::Stream;
use bumpalo::{collections::Vec as BumpVec, Bump};

#[derive(Debug)]
pub enum ErrorKind {
    InvalidString,
}

pub mod ast {
    use super::*;

    #[derive(Debug)]
    pub enum BinOp {
        Add,
        Sub,
    }
    #[derive(Debug)]
    pub enum Expr<'bump, 'src> {
        Binary {
            op: BinOp,
            lhs: &'bump Self,
            rhs: &'bump Self,
            span: CodeSpan<'src>,
        },
        Address {
            args: &'bump [Self],
            loc: CodeLoc,
            left: usize,
            right: usize,
        },
        IntLiteral {
            value: u64,
            base: NumberBase,
            span: CodeSpan<'src>,
        },
        FloatLiteral {
            int: u64,
            frac: u64,
            span: CodeSpan<'src>,
        },
        Ident {
            span: CodeSpan<'src>,
        },
        String {
            span: CodeSpan<'src>,
        },
        Error,
    }
    #[derive(Debug)]
    pub enum Top<'bump, 'src> {
        Directive {
            name: CodeSpan<'src>,
            args: Option<&'bump [Expr<'bump, 'src>]>,
        },
        Instruction {
            mnem: CodeSpan<'src>,
            args: Option<&'bump [Expr<'bump, 'src>]>,
        },
        Label(CodeSpan<'src>),
        Error,
    }

    type TopRef<'bump, 'src> = &'bump Top<'bump, 'src>;
}

struct TokenStream<'src> {
    scanner: Scanner<'src>,
    current: Option<Token<'src>>,
}

impl<'src> Stream for TokenStream<'src> {
    type Item = Token<'src>;

    fn next(&mut self) -> Option<Self::Item> {
        self.current?;
        std::mem::replace(&mut self.current, self.scanner.next())
    }
    fn peek(&self) -> Option<&Token<'src>> {
        self.current.as_ref()
    }
}

impl<'src> TokenStream<'src> {
    fn new(mut scanner: Scanner<'src>) -> Self {
        let current = scanner.next();
        Self { scanner, current }
    }

    fn peek_kind(&self) -> Option<TokenKind> {
        self.peek().map(|t| t.kind)
    }
    fn unwrap_span(&mut self) -> CodeSpan<'src> {
        self.next().expect("already peeked").span
    }
}

pub struct Parser<'bump, 'src> {
    it: TokenStream<'src>,
    bump: &'bump Bump,
}

impl<'bump, 'src> Parser<'bump, 'src> {
    #[must_use]
    pub fn new_in(it: Scanner<'src>, bump: &'bump Bump) -> Self {
        Self {
            it: TokenStream::new(it),
            bump,
        }
    }

    fn parse_int_value(src: &'src str, base: NumberBase) -> Option<u64> {
        use std::num::IntErrorKind::{NegOverflow, PosOverflow};
        if src.is_empty() {
            return None;
        }
        match u64::from_str_radix(src, base as u32).map_err(|e| e.kind().clone()) {
            Ok(val) => Some(val),
            Err(NegOverflow | PosOverflow) => None,
            _ => panic!("scanner mistyped token as {base:?}: {src:?}"),
        }
    }

    // called after consuming int
    fn parse_int(&mut self, span: CodeSpan<'src>, base: NumberBase) -> ast::Expr<'bump, 'src> {
        let int_str = span.src;
        if int_str.is_empty() || int_str.as_bytes()[0] != b'#' {
            return ast::Expr::Error;
        }
        match Self::parse_int_value(&int_str[1..], base) {
            Some(value) => ast::Expr::IntLiteral { value, base, span },
            None => ast::Expr::Error,
        }
    }

    // called after consuming float
    fn parse_float(&mut self, span: CodeSpan<'src>) -> ast::Expr<'bump, 'src> {
        let mut iter = span.src.splitn(2, '.');
        let Some(int_str) = iter.next() else {
            return ast::Expr::Error;
        };
        if int_str.is_empty() || int_str.as_bytes()[0] != b'#' {
            return ast::Expr::Error;
        }
        let Some(int) = Self::parse_int_value(&int_str[1..], NumberBase::Dec) else {
            return ast::Expr::Error;
        };
        let Some(frac_str) = iter.next() else {
            return ast::Expr::Error;
        };
        let Some(frac) = Self::parse_int_value(frac_str, NumberBase::Dec) else {
            return ast::Expr::Error;
        };
        ast::Expr::FloatLiteral { int, frac, span }
    }

    // called after consuming [
    fn parse_address_arg(&mut self) -> &'bump [ast::Expr<'bump, 'src>] {
        use TokenKind as T;
        let first = match self.it.peek_kind() {
            Some(T::Identifier) => ast::Expr::Ident {
                span: self.it.unwrap_span(),
            },
            _ => todo!("invalid addr mode arg"),
        };
        if self.it.peek_kind() == Some(T::RightSquareBracket) {
            return self.bump.alloc([first]);
        }
        if self.it.next_if_eq(T::Comma).is_none() {
            todo!("error if arg is not followed by comma");
        }
        let second = match self.it.peek_kind() {
            Some(T::Identifier) => ast::Expr::Ident {
                span: self.it.unwrap_span(),
            },
            Some(T::Int(base)) => {
                let span = self.it.unwrap_span();
                self.parse_int(span, base)
            }
            Some(T::Float) => {
                let span = self.it.unwrap_span();
                self.parse_float(span)
            }
            _ => todo!("invalid addr mode arg"),
        };
        self.bump.alloc([first, second])
    }

    fn parse_address(&mut self, span: CodeSpan<'src>) -> ast::Expr<'bump, 'src> {
        let loc = span.loc;
        let left = span.byte_index;
        let args = self.parse_address_arg();
        if self.it.peek_kind() != Some(TokenKind::RightSquareBracket) {
            todo!("error if no right bracket");
        }
        let right = self.it.unwrap_span().byte_index;
        ast::Expr::Address {
            args,
            loc,
            left,
            right,
        }
    }

    // x1
    // #123
    // "abc",
    // [x1, 10]
    fn parse_one_arg(&mut self) -> Option<ast::Expr<'bump, 'src>> {
        use TokenKind as T;
        let Token { kind, span } = self.it.next()?;
        match kind {
            T::Identifier => Some(ast::Expr::Ident { span }),
            T::Int(base) => Some(self.parse_int(span, base)),
            T::Float => Some(self.parse_float(span)),
            T::LeftSquareBracket => Some(self.parse_address(span)),
            _ => todo!("error invalid arg"),
        }
    }

    fn parse_args(&mut self) -> Option<&'bump [ast::Expr<'bump, 'src>]> {
        use TokenKind as T;
        if let Some(T::Newline) = self.it.peek_kind() {
            return None;
        }

        let mut args = BumpVec::<ast::Expr<'bump, 'src>>::with_capacity_in(3, self.bump);

        loop {
            match self.parse_one_arg() {
                Some(expr) => args.push(expr),
                None => args.push(ast::Expr::Error),
            }
            match self.it.peek_kind() {
                Some(T::Comma) => {
                    self.it.next(); // consume comma
                }
                Some(T::Newline) => {
                    self.it.next();
                    break;
                }
                Some(_) => {
                    args.push(ast::Expr::Error);
                    break;
                }
                None => {
                    break;
                }
            }
        }

        Some(args.into_bump_slice())
    }

    fn parse_root(&mut self) -> ast::Top<'bump, 'src> {
        use TokenKind as T;
        let Some(Token { kind, span }) = self.it.next() else {
            return ast::Top::Error;
        };
        match kind {
            T::Identifier => match self.it.peek_kind() {
                Some(T::Colon) => {
                    self.it.next();
                    ast::Top::Label(span)
                }
                Some(_) => {
                    let args = self.parse_args();
                    ast::Top::Instruction { mnem: span, args }
                }
                None => ast::Top::Instruction {
                    mnem: span,
                    args: None,
                },
            },
            T::Dot => match self.it.peek_kind() {
                Some(T::Identifier) => {
                    self.it.next();
                    let args = self.parse_args();
                    ast::Top::Directive { name: span, args }
                }
                _ => ast::Top::Error,
            },
            _ => ast::Top::Error,
        }
    }

    pub fn next<'a>(&'a mut self) -> ast::Top<'bump, 'src> {
        self.it.next_while(|t| t.kind == TokenKind::Newline);
        self.parse_root()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let bump = Bump::new();
        let s = Scanner::new("ADD r1, #10\nSUB r1, #10, #10\n");
        let mut l = Parser::new_in(s, &bump);
        dbg!(l.next());
        dbg!(l.next());
    }
}
