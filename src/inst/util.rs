use super::{
    operand::{Kind, Operand, Zero},
    Arg, ParseError,
};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Param {
    Req(Kind),
    Opt(Kind),
}

pub(super) fn is_variant<'a, I, A>(mut iter: I, expect: &'static [Param]) -> bool
where
    I: Iterator<Item = &'a A>,
    A: Arg,
    A: 'a,
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

pub(super) fn parse_arg<T: Arg, O: Operand>(arg: Option<&T>) -> Result<O, ParseError> {
    arg.ok_or(ParseError::Required)
        .and_then(|value| Arg::parse(value))
        .and_then(|op| Operand::from_op(op))
}

pub(super) fn parse_arg_opt<T: Arg, O: Operand + Zero>(arg: Option<&T>) -> Result<O, ParseError> {
    match arg {
        Some(value) => Arg::parse(value).and_then(|op| Operand::from_op(op)),
        None => Ok(Zero::zero()),
    }
}

pub(super) const fn binary_string(s: &'static str) -> u8 {
    let b = s.as_bytes();
    assert!(b[0] == b'0');
    assert!(b[1] == b'b' || b[1] == b'B');
    (b.len() - 2) as u8
}

const fn short_str_hash_rec(s: &[u8], i: usize, hash: u32) -> u32 {
    const FNV_PRIME: u32 = 16777619u32;
    const MAKE_UPPER: u8 = !0x20u8;
    if i < s.len() {
        short_str_hash_rec(
            s,
            i + 1,
            (hash ^ ((s[i] & MAKE_UPPER) as u32)).wrapping_mul(FNV_PRIME),
        )
    } else {
        hash
    }
}

/// case insensitive
pub const fn str_hash(s: &str) -> u32 {
    const FNV_OFFSET: u32 = 2166136261u32;
    short_str_hash_rec(s.as_bytes(), 0, FNV_OFFSET)
}
