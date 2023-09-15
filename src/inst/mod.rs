pub mod dec;
mod def;
pub mod dir;
mod meta;
mod meta_operand;
pub mod operand;
mod util;

pub use def::{get_variant_and_emit, narrow_variant, EncInstr, EncInstrSet, Mnemonic, Variant};
pub use operand::{op, Ops};
pub use util::{NarrowError, NarrowVariant};

use bit::{BitCt, Int, IntN, IntOfBits};

#[derive(Debug)]
pub enum Error {
    Required,
    Resolve,
    Unexpected,
    OutOfRange,
    NotAligned,
    InvalidGpr,
    MismatchedConstShift,
    InvalidExtendWidth,
    UnmatchedVariant,
}

/// Error and the arg index that caused error
#[derive(Debug)]
pub struct ErrorIdx(Error, u8);

/// Error and ref to static string in the macro that caused error
#[derive(Debug)]
pub struct ErrorMacro(pub Error, pub &'static str);

pub type FixupFn<E, Value> = fn(Value, &mut E) -> Result<IntN, Error>;

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

impl<E: Emitter + ?Sized, Key: Copy, Value> Clone for Fixup<E, Key, Value> {
    fn clone(&self) -> Self {
        Self {
            key: self.key,
            bit_idx: self.bit_idx,
            pc: self.pc,
            encode_fn: self.encode_fn,
        }
    }
}

crate::typed_interner! { label; Key32 }

pub fn apply_label_fixup<E: Emitter>(
    e: &mut E,
    fixup: Fixup<E, label::Key, u64>,
) -> Result<(), Error> {
    let value = e.resolve_label(fixup.key).ok_or(Error::Resolve)?;
    e.set_pc(fixup.pc);
    let encode = (fixup.encode_fn)(value, e)?;
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
    fn push_n<const N: BitCt>(&mut self, value: Int<N>);
    fn insert(&mut self, value: IntN, offset: u8);
    fn begin_instr(&mut self);
    fn end_instr(&mut self);

    fn resolve_label(&mut self, key: label::Key) -> Option<u64>;
    fn push_label_fixup(&mut self, fixup: Fixup<Self, label::Key, u64>);
}

pub struct Addr<const ALIGN: usize>(usize);

impl<const ALIGN: usize> Addr<ALIGN> {
    pub fn new(addr: usize) -> Option<Self> {
        if addr % ALIGN == 0 {
            Some(Self(addr))
        } else {
            None
        }
    }

    pub fn get(&self) -> usize {
        self.0
    }
}
