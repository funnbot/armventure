use super::{
    operand::{op, Kind, Ops},
    Error,
};
use crate::enum_str::{str_hash, str_hash_to_lower};
use std::fmt;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Param {
    Req(Kind),
    Opt(Kind),
}

pub type E = std::mem::Discriminant<Kind>;

pub(super) fn is_variant<'a, I>(mut iter: I, expect: &'static [Param]) -> bool
where
    I: Iterator<Item = &'a Ops>,
{
    // check if each argument matches the expected argument kind
    // if the iter ends early, and all the rest of expect are optional, then it matches
    // if there is an optional arg that matches, then all the rest need to be optional or none
    let mut has_optional = false;
    for kind in expect {
        if let Some(arg) = iter.next() {
            match *kind {
                Param::Opt(opt) => {
                    has_optional = true;
                    if opt != arg.kind() {
                        return false;
                    }
                }
                Param::Req(req) => {
                    if has_optional {
                        todo!("error if required after optional");
                    }
                    if req != arg.kind() {
                        return false;
                    }
                }
            }
        } else {
            return match *kind {
                Param::Opt(..) => true,
                Param::Req(..) => false,
            };
        }
    }
    // if iter.next() ends early, then this wont be reached, so iter.next().is_none() is valid here
    iter.next().is_none()
}

#[derive(Debug, PartialEq, Eq)]
pub enum NarrowError {
    Required(Kind),
    Multiple,
    None,
}

/// TODO: Is this really necessary?
pub struct NarrowVariant {
    variants: &'static [&'static [Param]],
    index: usize,
    /// bitflags of an index in args that did not match
    failed: u32,
}

impl NarrowVariant {
    pub fn new(variants: &'static [&'static [Param]]) -> Self {
        assert!(
            variants.len() <= 32,
            "arbitrary limit due to impl of failed checking"
        );
        Self {
            variants,
            index: 0,
            failed: 0,
        }
    }

    pub fn allow(&self, kind: Kind) -> bool {
        for (i, variant) in self.variants.iter().copied().enumerate() {
            if self.is_fail(i) {
                continue;
            }
            if let Some(param) = variant.get(self.index) {
                let expect = match param {
                    Param::Req(req) => *req,
                    Param::Opt(opt) => *opt,
                };
                // one of the variants matched, so this kind is allowed at this index
                if expect == kind {
                    return true;
                }
            } // else can't match this variant, will get failed on check_next()
        }
        false
    }
    pub fn check_next(&mut self, kind: Kind) {
        for (i, variant) in self.variants.iter().copied().enumerate() {
            if self.is_fail(i) {
                continue;
            }
            if let Some(param) = variant.get(self.index) {
                let expect = match param {
                    Param::Req(req) => *req,
                    Param::Opt(opt) => *opt,
                };
                if expect != kind {
                    self.fail(i);
                }
            } else {
                // can't match this variant
                self.fail(i);
            }
        }
        self.index += 1;
    }
    // only called in finish()
    fn check_end(&self) -> (usize, usize) {
        let mut count = 0;
        let mut variant_idx = 0;
        for (i, variant) in self.variants.iter().copied().enumerate() {
            if self.is_fail(i) {
                continue;
            }
            if let Some(Param::Req(req)) = variant.get(self.index) {
                // would self.fail(i) but unnecessary
            } else {
                count += 1;
                // variant_idx only gets used if count == 1
                variant_idx = i;
            }
        }
        (count, variant_idx)
    }
    pub fn finish(self) -> Result<usize, NarrowError> {
        let (count, variant_idx) = self.check_end();
        if count == 0 {
            Err(NarrowError::None)
        } else if count > 1 {
            Err(NarrowError::Multiple)
        } else if let Some(Param::Req(req)) = self.variants[variant_idx].get(self.index) {
            Err(NarrowError::Required(*req))
        } else {
            Ok(variant_idx)
        }
    }

    fn is_fail(&self, i: usize) -> bool {
        self.failed & (1 << i) != 0
    }
    fn fail(&mut self, i: usize) {
        self.failed |= 1 << i;
    }
}

pub(super) const fn rest_are_opt(args: &'static [Param]) -> bool {
    let mut i: usize = 0;
    let mut marker: bool = false;
    while i < args.len() {
        match args[i] {
            Param::Req(..) => {
                if marker {
                    return false;
                }
            }
            Param::Opt(..) => {
                marker = true;
            }
        }
        i += 1;
    }
    true
}

pub(super) fn parse_arg<O: TryFrom<Ops, Error: fmt::Debug>>(arg: Option<&Ops>) -> O {
    O::try_from(arg.expect("length already checked").clone()).expect("kind already checked")
}

pub(super) fn parse_arg_opt<O: TryFrom<Ops, Error: fmt::Debug>>(arg: Option<&Ops>) -> Option<O> {
    arg.map(|a| O::try_from(a.clone()).expect("kind already checked"))
}

pub(super) const fn binary_string(s: &'static str) -> u8 {
    let b = s.as_bytes();
    assert!(b[0] == b'0');
    assert!(b[1] == b'b' || b[1] == b'B');
    (b.len() - 2) as u8
}

pub(super) const fn bit_str_u32(s: &'static [u8]) -> u32 {
    let mut n = 0;
    let mut i: usize = 0;
    while i < s.len() {
        let b = s[i];
        assert!(b == b'0' || b == b'1');
        n <<= 1;
        n |= (b - b'0') as u32;
        i += 1;
    }
    n
}

pub(super) fn variant_eq<Enum>(lhs: &Enum, rhs: &Enum) -> bool {
    std::mem::discriminant(lhs) == std::mem::discriminant(rhs)
}

pub trait MaybeDisplay<T>
where
    T: fmt::Display,
{
    fn maybe_display(&self) -> OptionDisplay<'_, T>;
}
pub enum OptionDisplay<'a, T>
where
    T: fmt::Display,
{
    DisplaySome(&'a T),
    DisplayNone,
}
impl<T> MaybeDisplay<T> for T
where
    T: fmt::Display,
{
    default fn maybe_display(&self) -> OptionDisplay<'_, Self> {
        OptionDisplay::DisplaySome(self)
    }
}
impl<T> MaybeDisplay<T> for Option<T>
where
    T: fmt::Display,
{
    default fn maybe_display(&self) -> OptionDisplay<'_, T> {
        match self {
            Some(v) => OptionDisplay::DisplaySome(v),
            None => OptionDisplay::DisplayNone,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rest_are_optional_works() {
        use Kind::*;
        use Param::*;
        assert!(!rest_are_opt(&[Req(Gpr), Req(Gpr), Opt(Gpr), Req(Gpr)]));
        assert!(rest_are_opt(&[Req(Gpr), Req(Gpr), Opt(Gpr), Opt(Gpr)]));
        assert!(rest_are_opt(&[Opt(Gpr), Opt(Gpr), Opt(Gpr), Opt(Gpr)]));
        assert!(!rest_are_opt(&[Opt(Gpr), Opt(Gpr), Opt(Gpr), Req(Gpr)]));
    }

    #[test]
    fn bit_str_works() {
        assert_eq!(bit_str_u32(b"1000"), 8);
        assert_eq!(bit_str_u32(b"1010"), 10);
        assert_eq!(bit_str_u32(b"1111"), 15);
        assert_eq!(bit_str_u32(b"00000111110000011111"), 0b111110000011111);
    }
}
