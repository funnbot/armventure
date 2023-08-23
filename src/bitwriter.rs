use crate::num::Int;
use std::ops;

fn set_bits<T: Int>(current: T, value: T, index: u8, len: u8) -> T
where
    T: From<u32> + From<u8> + std::fmt::Debug,
{
    debug_assert!(
        T::from(index + len) <= T::from(T::BITS),
        "not enough room for bits, expected {:?}, required {:?}.",
        T::from(T::BITS) - T::from(index),
        index + len
    );
    // mask to zero the bits after len
    let mask_clear_high = !((!(T::from(0u8))) << T::from(len));
    let masked = value & mask_clear_high;
    // start of value is now aligned with the bits to bitor with in current
    // if len + index > MAX_BITS then it will run off the end
    let shifted = masked << T::from(index);
    // apply value to current
    current | shifted
}

pub trait BitWriter {
    type Num: Int;

    fn push(&mut self, value: Self::Num, bit_len: u8) -> &mut Self;
    fn insert(&mut self, value: Self::Num, bit_len: u8, bit_index: u8) -> &mut Self;
    fn skip(&mut self, bit_len: u8) -> &mut Self;
    fn all_bits_written(&self) -> bool;
    fn bit_idx(&self) -> u8;
    fn value(&self) -> Self::Num;
}

pub fn bit_write_u32(current: u32, value: u32, bit_len: u8, bit_index: u8) -> u32 {
    assert!((bit_index - bit_len) <= 32);
    println!(
        "set bits: {:#4x} {:#4x} {:#4x} {:#4x}",
        current,
        value,
        bit_index - bit_len,
        bit_len
    );
    set_bits::<u32>(current, value, bit_index - bit_len, bit_len)
}

pub struct BitWriterU32 {
    value: u32,
    bit_index: u8,
}

impl BitWriter for BitWriterU32 {
    type Num = u32;
    fn push(&mut self, value: Self::Num, bit_len: u8) -> &mut Self {
        assert!((self.bit_index - bit_len) <= 32);
        self.bit_index -= bit_len;
        println!(
            "set bits: {:#4x} {:#4x} {:#4x} {:#4x}",
            self.value, value, self.bit_index, bit_len
        );
        self.value = set_bits::<u32>(self.value, value, self.bit_index, bit_len);
        self
    }
    fn insert(&mut self, value: Self::Num, bit_len: u8, bit_index: u8) -> &mut Self {
        assert!((bit_index - bit_len) <= 32);
        println!(
            "set bits: {:#4x} {:#4x} {:#4x} {:#4x}",
            self.value,
            value,
            bit_index - bit_len,
            bit_len
        );
        self.value = set_bits::<u32>(self.value, value, bit_index - bit_len, bit_len);
        self
    }
    fn skip(&mut self, bit_len: u8) -> &mut Self {
        assert!((self.bit_index - bit_len) <= 32);
        self.bit_index -= bit_len;
        self
    }
    fn all_bits_written(&self) -> bool {
        self.bit_index == 0
    }
    fn bit_idx(&self) -> u8 {
        self.bit_index
    }
    fn value(&self) -> Self::Num {
        self.value
    }
}

impl BitWriterU32 {
    pub fn new() -> Self {
        Self {
            value: Default::default(),
            bit_index: 32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_set_bits() {
        assert_eq!(
            set_bits::<u32>(0b1111000011110000, 0b0110, 8, 4),
            0b1111011011110000u32
        );
    }
}
