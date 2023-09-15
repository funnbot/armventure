#![feature(proc_macro_diagnostic)]
// TODO: temporary
#![allow(warnings)]

extern crate proc_macro;

use instr_set_internal_items::*;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{
    self, bracketed, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token, Index, LitInt, LitStr, Token,
};

struct BitRange {
    high: LitInt,
    low: Option<LitInt>,
}
impl Parse for BitRange {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let bracket = bracketed!(content in input);
        let high = content.parse::<LitInt>()?;
        let colon = content.parse::<Option<Token![:]>>()?;
        let low = if colon.is_some() {
            Some(content.parse::<LitInt>()?)
        } else {
            None
        };
        Ok(Self { high, low })
    }
}

struct BitPattern {
    lit: LitStr,
}
impl Parse for BitPattern {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            lit: input.parse()?,
        })
    }
}

struct NamedBitRange {
    ident: Ident,
    range: BitRange,
}
impl Parse for NamedBitRange {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            ident: input.parse()?,
            range: input.parse()?,
        })
    }
}

struct AssignBitRanges {
    ranges: Punctuated<NamedBitRange, Token![,]>,
}
impl Parse for AssignBitRanges {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![let]>()?;
        let content;
        let paren = parenthesized!(content in input);
        Ok(Self {
            ranges: content.parse_terminated(NamedBitRange::parse)?,
        })
    }
}

struct InstrSet {}

impl Parse for InstrSet {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        todo!()
    }
}

struct InstrOperation {}

impl Parse for InstrOperation {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {})
    }
}

#[proc_macro]
pub fn instr_set(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as InstrSet);

    todo!()
}

macro_rules! any_syntax {
    ($($tt:tt)*) => {};
}

any_syntax! {
    Group when
}

any_syntax! {
    |e: Executor|

    let is_64[31], main_group[28:25];
    if true {
        group "Data Processing - Immediate":
        if main_group == "100x" {
            let op0[25:23];

            class "Add/subtract (immediate)":
            if op0 == "010" {
                let opc[30], S[29];
                args: (Rd[4:0]: GprOrSp, Rn[9:5]: GprOrSp, imm[21:10]: UImm<Align4>, sh[22]: ConstShift<LSL, 12>)
                encoding: (is_64 opc S "100010" sh imm Rn Rd)
                mnems: [ADD(is_64, 0, 0), ADDS(is_64, 0, 1), SUB(is_64, 1, 0), SUBS(is_64, 1, 1)]
                exec: (is_64, opc, S) {
                    let imm = if sh == 1 { imm << 12 } else { imm };
                    let res = if opc == 0 {
                        e.reg(Rn) + imm
                    } else {
                        e.reg(Rn) - imm
                    }
                    if S == 1 {
                        e.set_flags(res)
                    }
                    e.set_reg(Rd, res)
                }
            }

            use data_processing;
        }
    }
}

/*
```
def_matcher! {
    decode_instr (sf(31), main_group(28:25)) {
        Class (sf("0"), main_group("0000")) => (op0(30:29), op1(24:16)) {
            (op0("00"), op1("000000000")) => ADD Immediate ()
        }
        Group (main_group("100x")) => (op0(25:23)) {
            Class (op0("010")) => (opc(30), s(29), sh(22), imm12(12:10), Rn(9:5), Rd(4:0)) {
                (opc("0"), s("0")) => ADD Immediate ()
            }
        }
    }
}
    ADD Immediate
        (Gpr() Gpr() Imm() Opt(Shift()))
        (Sf():0 B(0b00100010) ShiftConst(LSL, 12):3 Gpr(AllowSp):1 UImm(12):2 Gpr(AllowSp):0),
        ShiftedRegister
        // docs say Shift is optional, but that conflicts with ExtendedRegister,
        // and need to favor that since it allows special registers in the common `ADD Gpr, Gpr, Gpr`
        (Gpr() Gpr() Gpr() Shift())
        (Sf():0 B(0b0001011) Shift(Kind):3 B(0b0) Gpr():2 Shift(Amount(6)):3 Gpr():1 Gpr():0),
        ExtendedRegister
        (Gpr() Gpr() Gpr() Opt(Extend()))
        (Sf():2 B(0b0001011001) Gpr(AllowZr):2 Extend(Kind):3 Extend(Shift):3 Gpr(AllowSp):1 Gpr(AllowSp):0);
```
*/

trait Alignment {
    const BYTES: usize;
    const BITS: u32;
}
struct Align1;
impl Alignment for Align1 {
    const BYTES: usize = 1;
    const BITS: u32 = 0;
}
struct Align4;
impl Alignment for Align4 {
    const BYTES: usize = 4;
    const BITS: u32 = 2;
}
