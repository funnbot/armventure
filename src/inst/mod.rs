mod def;
pub mod operand;

mod meta;
mod meta_operand;
mod util;

pub use def::parse as set;
use lasso::Spur;
use operand::Operand;

pub mod parse {
    pub use super::{
        def::parse as set,
        def::Mnemonic,
        operand,
        operand::{Kind, Ops},
        set::{emit_instr, parse_variant, InstrSet},
        Arg, EmitError, Emitter, Fixup, ParseError, ParseErrorIdx, Resolvable,
    };
}

#[derive(Debug)]
pub enum ParseError {
    Required,
    Resolve,
    Unexpected,
    OutOfRange,
    NotAligned,
}
#[derive(Debug)]
pub enum EmitError {
    Parse(ParseError),
}

impl From<ParseError> for EmitError {
    fn from(value: ParseError) -> Self {
        Self::Parse(value)
    }
}

#[derive(Debug)]
pub struct ParseErrorIdx(ParseError, u8);

pub trait Resolvable: Sized {
    type Key: Sized + Copy;
    type Value;
    type Result: operand::Zero + operand::Operand;
    fn value_to_op(value: Self::Value) -> operand::Ops;
}

pub type FixupFn<E, Value> = fn(Value, &mut E) -> IntN;

pub struct Fixup<E: Emitter + ?Sized, Key, Value> {
    key: Key,
    bit_idx: u8,
    pc: u64,
    encode_fn: FixupFn<E, Value>,
}

impl<E: Emitter + ?Sized, Key: std::fmt::Debug, Value> std::fmt::Debug for Fixup<E, Key, Value> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Fixup(key: {:?}, bit_idx: {}, pc: {})",
            self.key, self.bit_idx, self.pc
        )
    }
}

impl<E: Emitter + ?Sized, Key: std::fmt::Debug + Copy, Value> std::clone::Clone
    for Fixup<E, Key, Value>
{
    fn clone(&self) -> Self {
        Self {
            key: self.key,
            bit_idx: self.bit_idx,
            pc: self.pc,
            encode_fn: self.encode_fn,
        }
    }
}

/// (value, bit_count)
#[derive(Debug, Clone, Copy)]
pub struct IntN(pub u32, pub u8);

impl Encode for IntN {
    fn encode(&self) -> IntN {
        *self
    }
}

pub trait Encode: Sized {
    // won't fail, the value should already be parsed and validated
    fn encode(&self) -> IntN;
}

pub fn apply_label_fixup<E: Emitter>(
    e: &mut E,
    fixup: Fixup<E, Spur, u64>,
) -> Result<(), ParseError> {
    let value = e.resolve_label(fixup.key).ok_or(ParseError::Resolve)?;
    e.set_pc(fixup.pc);
    let encode = (fixup.encode_fn)(value, e);
    e.insert(encode, fixup.bit_idx);
    Ok(())
}

pub fn create_fixup<Key, Value, E: Emitter>(
    e: &E,
    key: Key,
    fixup_fn: FixupFn<E, Value>,
) -> Fixup<E, Key, Value> {
    Fixup {
        key,
        bit_idx: e.bit_idx(),
        pc: e.pc(),
        encode_fn: fixup_fn,
    }
}

pub trait Emitter {
    fn pc(&self) -> u64;
    fn bit_idx(&self) -> u8;
    fn set_pc(&mut self, value: u64);
    fn push(&mut self, value: IntN);
    fn insert(&mut self, value: IntN, offset: u8);
    fn begin_instr(&mut self);
    fn end_instr(&mut self);

    fn resolve_label(&mut self, key: Spur) -> Option<u64>;
    fn push_label_fixup(&mut self, fixup: Fixup<Self, Spur, u64>);
}

pub trait Arg {
    fn kind(&self) -> operand::Kind;
    fn parse(&self) -> Result<operand::Ops, ParseError>;
}

pub const fn uint_in_range<const BITS: u8>(value: u64) -> bool {
    debug_assert!(BITS <= 64);
    value <= (u64::MAX >> ((u64::BITS - (BITS as u32)) as u64))
}
/// returns true if `value` would fit into the range of a twos compliment integer of `BITS` bits
pub const fn sint_in_range<const BITS: u8>(value: i64) -> bool {
    debug_assert!(BITS <= 64);
    value >= (i64::MIN >> ((i64::BITS - (BITS as u32)) as i64))
        && value <= (i64::MAX >> ((i64::BITS - (BITS as u32)) as i64))
}
pub const fn uint_is_aligned<const AMT: u8>(value: u64) -> bool {
    let mask = !(u64::MAX << AMT);
    (value & mask) == 0
}
pub const fn sint_is_aligned<const AMT: u8>(value: i64) -> bool {
    let v = value as u64;
    let mask = !(u64::MAX << AMT);
    (v & mask) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_checks_sint_in_range() {
        assert!(!sint_in_range::<2>(2));
        assert!(sint_in_range::<2>(1));
        assert!(sint_in_range::<2>(-1));
        assert!(sint_in_range::<2>(-2));
        assert!(!sint_in_range::<2>(-3));
        assert!(!sint_in_range::<2>(i64::MIN));
        assert!(!sint_in_range::<2>(i64::MAX));
        assert!(sint_in_range::<64>(i64::MIN));
        assert!(sint_in_range::<64>(i64::MAX));
    }
}
