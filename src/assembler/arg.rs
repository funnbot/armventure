use super::parse::ast::{
    self,
    expr::{self, IdentInt},
};
use crate::{
    code::{self, Span},
    enum_str::EnumStr,
    inst::{
        label,
        operand::{op, CondKind, ExtendKind, GprKind, GprSize, Kind, Ops, ShiftKind},
        NarrowError, NarrowVariant,
    },
};

pub struct ArgParser<'src, 'i> {
    src: &'src code::Source,
    intern: &'i mut label::Intern,
    narrow: NarrowVariant,
}

impl<'ast, 'src, 'i> ArgParser<'src, 'i> {
    pub fn new(
        src: &'src code::Source,
        intern: &'i mut label::Intern,
        narrow: NarrowVariant,
    ) -> Self {
        Self {
            src,
            intern,
            narrow,
        }
    }

    fn parse_cond(&mut self, s: Span) -> Option<op::Cond> {
        if !self.allow(Kind::Cond) {
            return None;
        }
        CondKind::from_str_lower_or_upper(self.src.span(s)).map(op::Cond)
    }

    fn parse_gpr(&self, span: Span) -> Option<op::Gpr> {
        if !self.allow(Kind::Gpr) {
            return None;
        }
        let s = self.src.span(span);
        match s.as_bytes() {
            b"SP" | b"sp" | b"ZR" | b"zr" => Some(op::Gpr {
                reg: GprKind::SP,
                size: GprSize::B8,
            }),
            b"WSP" | b"wsp" | b"WZR" | b"wzr" => Some(op::Gpr {
                reg: GprKind::ZR,
                size: GprSize::B4,
            }),
            [b'X', rest @ ..] | [b'x', rest @ ..] => {
                let reg = byte_str_to_u8(rest)?;
                if reg > 30 {
                    return None;
                }
                Some(op::Gpr {
                    reg: GprKind::R(reg.try_into().unwrap()),
                    size: GprSize::B8,
                })
            }
            [b'W', rest @ ..] | [b'w', rest @ ..] => {
                let reg = byte_str_to_u8(rest)?;
                if reg > 30 {
                    return None;
                }
                Some(op::Gpr {
                    reg: GprKind::R(reg.try_into().unwrap()),
                    size: GprSize::B4,
                })
            }
            _ => None,
        }
    }

    fn parse_shift(&self, expr: ast::expr::IdentInt) -> Option<op::Shift> {
        if !self.allow(Kind::Shift) {
            return None;
        }
        let s = self.src.span(expr.span);
        let kind = ShiftKind::from_str_lower_or_upper(s)?;
        let amount = u8::try_from(expr.int.value).unwrap();
        Some(op::Shift { kind, amount })
    }

    fn parse_extend(&self, span: Span, int: Option<ast::expr::IntLiteral>) -> Option<op::Extend> {
        if !self.allow(Kind::Extend) {
            return None;
        }
        let s = self.src.span(span);
        let kind = ExtendKind::from_str_lower_or_upper(s).or_else(|| {
            if s.as_bytes() == b"LSL" {
                Some(ExtendKind::UXTX)
            } else {
                None
            }
        })?;
        let left_shift_amount = int.map(|i| u8::try_from(i.value).unwrap());
        Some(op::Extend {
            kind,
            left_shift_amount,
        })
    }

    fn parse_label(&mut self, span: Span) -> Option<op::Label> {
        if !self.allow(Kind::Label) {
            return None;
        }
        let s = self.src.span(span);
        Some(op::Label(self.intern.get_or_intern(s)))
    }

    fn parse_ident(&mut self, span: Span) -> Ops {
        if let Some(op) = self.parse_gpr(span) {
            return op.into();
        }
        if let Some(op) = self.parse_extend(span, None) {
            return op.into();
        }
        if let Some(op) = self.parse_cond(span) {
            return op.into();
        }
        if let Some(op) = self.parse_label(span) {
            return op.into();
        }
        todo!()
    }

    fn parse_ident_int(&self, expr: &expr::IdentInt) -> Ops {
        if let Some(op) = self.parse_shift(*expr) {
            return op.into();
        }
        if let Some(op) = self.parse_extend(expr.span, Some(expr.int)) {
            return op.into();
        }
        todo!()
    }

    fn allow(&self, kind: Kind) -> bool {
        self.narrow.allow(kind)
    }

    pub fn parse_args(&mut self, args: &'ast [ast::Expr<'ast>], vec: &mut Vec<Ops>) {
        use ast::Expr;
        for arg in args {
            let op = match arg {
                Expr::Ident { span } => self.parse_ident(*span),
                Expr::IdentInt(expr) => self.parse_ident_int(expr),
                _ => todo!(),
            };
            self.narrow.check_next(op.kind());
            vec.push(op);
        }
    }

    pub fn finish(self) -> Result<usize, NarrowError> {
        self.narrow.finish()
    }
}

fn byte_str_to_u8(s: &[u8]) -> Option<u8> {
    let mut acc: u8 = 0;
    for b in s.iter().copied() {
        let v = b.checked_sub(b'0')?;
        if v < 10 {
            acc *= 10;
            acc += v;
        } else {
            return None;
        }
    }
    Some(acc)
}
