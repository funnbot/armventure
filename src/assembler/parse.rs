use crate::{
    assembler::lex::{IntRadix, Scanner, Token, TokenKind},
    code::{self, Loc, Span},
    stream::Stream,
};
use bumpalo::{collections::Vec as BumpVec, Bump};

pub mod ast {
    use enum_variant_type::EnumVariantType;

    use super::*;

    pub mod expr {
        use super::*;

        #[derive(Debug, Clone, Copy)]
        pub struct IntLiteral {
            pub value: u64,
            pub base: IntRadix,
            pub span: Span,
        }
        #[derive(Debug, Clone, Copy)]
        pub struct IdentInt {
            pub span: Span,
            pub int: IntLiteral,
        }

        impl From<IntLiteral> for Expr<'_> {
            fn from(val: IntLiteral) -> Self {
                Expr::IntLiteral(val)
            }
        }
        impl From<IdentInt> for Expr<'_> {
            fn from(val: IdentInt) -> Self {
                Expr::IdentInt(val)
            }
        }
    }

    #[derive(Debug)]
    pub enum Expr<'bump> {
        Address {
            args: &'bump [Expr<'bump>],
            group: Span,
        },
        IntLiteral(expr::IntLiteral),
        FloatLiteral {
            int: u64,
            frac: u64,
            span: Span,
        },
        Ident {
            span: Span,
        },
        IdentInt(expr::IdentInt),
        String {
            span: Span,
        },
        Error,
    }

    #[derive(Debug)]
    pub enum Top<'bump> {
        Directive {
            name: Span,
            args: Option<&'bump [Expr<'bump>]>,
        },
        Instruction {
            mnem: Span,
            args: Option<&'bump [Expr<'bump>]>,
        },
        Label(Span),
        Error,
    }
}

use ast::{expr, Expr, Top};

struct TokenStream<'src> {
    scanner: Scanner<'src>,
    current: Option<Token>,
}

impl<'src> Stream for TokenStream<'src> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.current?;
        std::mem::replace(&mut self.current, self.scanner.next())
    }
    fn peek(&self) -> Option<&Token> {
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
    fn next_span(&mut self) -> Span {
        self.next().expect("already peeked").span
    }
}

pub struct Parser<'bump, 'src> {
    it: TokenStream<'src>,
    src: &'src code::Source,
    bump: &'bump Bump,
}

impl<'bump, 'src> Parser<'bump, 'src> {
    #[must_use]
    pub fn new_in(src: &'src code::Source, bump: &'bump Bump) -> Self {
        let scanner = Scanner::new(src);
        Self {
            it: TokenStream::new(scanner),
            src,
            bump,
        }
    }

    fn parse_int_value(src: &'src str, base: IntRadix) -> Option<u64> {
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
    fn parse_int(&mut self, span: Span, base: IntRadix) -> Option<expr::IntLiteral> {
        let int_str = self.src.span(span);
        if int_str.is_empty() || int_str.as_bytes()[0] != b'#' {
            return None;
        }
        match Self::parse_int_value(&int_str[1..], base) {
            Some(value) => ast::expr::IntLiteral { value, base, span }.into(),
            None => None,
        }
    }

    // called after consuming float
    fn parse_float(&mut self, span: Span) -> ast::Expr<'bump> {
        let mut iter = self.src.span(span).splitn(2, '.');
        let Some(int_str) = iter.next() else {
            return ast::Expr::Error;
        };
        if int_str.is_empty() || int_str.as_bytes()[0] != b'#' {
            return ast::Expr::Error;
        }
        let Some(int) = Self::parse_int_value(&int_str[1..], IntRadix::Dec) else {
            return ast::Expr::Error;
        };
        let Some(frac_str) = iter.next() else {
            return ast::Expr::Error;
        };
        let Some(frac) = Self::parse_int_value(frac_str, IntRadix::Dec) else {
            return ast::Expr::Error;
        };
        ast::Expr::FloatLiteral { int, frac, span }
    }

    // called after consuming [
    fn parse_address_arg(&mut self) -> &'bump [ast::Expr<'bump>] {
        use TokenKind as T;
        let first = match self.it.peek_kind() {
            Some(T::Identifier) => ast::Expr::Ident {
                span: self.it.next_span(),
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
                span: self.it.next_span(),
            },
            Some(T::Int(base)) => {
                let span = self.it.next_span();
                self.parse_int(span, base).unwrap().into()
            }
            Some(T::Float) => {
                let span = self.it.next_span();
                self.parse_float(span)
            }
            _ => todo!("invalid addr mode arg"),
        };
        self.bump.alloc([first, second])
    }

    fn parse_address(&mut self, span: Span) -> ast::Expr<'bump> {
        let args = self.parse_address_arg();
        if self.it.peek_kind() != Some(TokenKind::RightSquareBracket) {
            todo!("error if no right bracket");
        }
        let end = self.it.next_span();
        ast::Expr::Address {
            args,
            group: Span::group(span, end),
        }
    }

    // x1
    // #123
    // "abc",
    // [x1, 10]
    // LSL #1
    fn parse_one_arg(&mut self) -> Option<ast::Expr<'bump>> {
        use TokenKind as T;
        let Token { kind, span } = self.it.next()?;
        match kind {
            T::Identifier => {
                if let Some(T::Int(base)) = self.it.peek_kind() {
                    let int_span = self.it.next_span();
                    let int = self.parse_int(int_span, base)?;
                    Some(expr::IdentInt { span, int }.into())
                } else {
                    Some(ast::Expr::Ident { span })
                }
            }
            T::Int(base) => Some(self.parse_int(span, base)?.into()),
            T::Float => Some(self.parse_float(span)),
            T::LeftSquareBracket => Some(self.parse_address(span)),
            _ => todo!("error invalid arg"),
        }
    }

    fn parse_modif(&mut self) -> Option<Span> {
        self.it.next_if_eq(TokenKind::Dot)?;
        self.it.next_if_eq(TokenKind::Identifier).map(|t| t.span)
    }

    fn parse_args(&mut self, is_instr: bool) -> Option<&'bump [ast::Expr<'bump>]> {
        use TokenKind as T;
        if let Some(T::Newline) = self.it.peek_kind() {
            return None;
        }

        let mut args = BumpVec::<ast::Expr<'bump>>::with_capacity_in(3, self.bump);
        if is_instr {
            if let Some(modif) = self.parse_modif() {
                args.push(ast::Expr::Ident { span: modif })
            }
        }

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

    fn parse_root(&mut self) -> ast::Top<'bump> {
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
                    let args = self.parse_args(true);
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
                    let args = self.parse_args(false);
                    ast::Top::Directive { name: span, args }
                }
                _ => ast::Top::Error,
            },
            _ => ast::Top::Error,
        }
    }

    pub fn next<'a>(&'a mut self) -> Option<ast::Top<'bump>> {
        self.it.next_while(|t| t.kind == TokenKind::Newline);
        self.it.peek_kind()?;
        Some(self.parse_root())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn it_works() {
    //     let bump = Bump::new();
    //     let s = Scanner::new("ADD r1, #10\nSUB r1, #10, LSL  #10\n");
    //     let mut l = Parser::new_in(s, &bump);
    //     dbg!(l.next());
    //     dbg!(l.next());
    // }
}
