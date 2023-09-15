use std::marker::PhantomData;

/// number of bits
pub type BitCt = u32;

/// runtime type for alignment
pub type Align = usize;

// TODO: rename to not IntN, a name that is descriptive that it is a tuple with (value, bit_count), like
/// (value, bit_count)
#[derive(Debug, Clone, Copy)]
pub struct IntN(pub u32, pub u8);

pub trait IntOfBits<Num>: Sized {
    const N: BitCt;
    fn new(value: Num) -> Self;
    fn get(self) -> Num;
}

#[derive(Debug, Clone, Copy)]
pub struct Int<const N: BitCt>(pub u32);

impl<const N: BitCt> IntOfBits<u32> for Int<N> {
    const N: BitCt = N;
    fn new(value: u32) -> Self {
        Self(value)
    }
    fn get(self) -> u32 {
        self.0
    }
}

/// returns true if `value` fits into range of unsigned integer of `BITS` bit count
pub const fn uint_in_range<const BITS: BitCt>(value: u64) -> bool {
    debug_assert!(BITS <= 64);
    value <= (u64::MAX >> ((u64::BITS - BITS) as u64))
}
/// returns true if `value` would fit into the range of a twos compliment integer of `BITS` bits
pub const fn sint_in_range<const BITS: BitCt>(value: i64) -> bool {
    debug_assert!(BITS <= 64);
    value >= (i64::MIN >> ((i64::BITS - BITS) as i64))
        && value <= (i64::MAX >> ((i64::BITS - BITS) as i64))
}

pub const fn uint_low_zeros<const ALIGN: BitCt>(value: u64) -> bool {
    value & (((ALIGN * ALIGN) as u64) - 1) == 0
}
pub const fn sint_low_zeros<const ALIGN: BitCt>(value: i64) -> bool {
    let uval = value as u64;
    uval & (((ALIGN * ALIGN) as u64) - 1) == 0
}

pub const fn uint_max_value<const BITS: BitCt>() -> u64 {
    debug_assert!(BITS <= 64);
    u64::MAX >> (u64::BITS - BITS)
}
pub const fn sint_max_value<const BITS: BitCt>() -> i64 {
    debug_assert!(BITS <= 64);
    i64::MAX >> (i64::BITS - BITS)
}
pub const fn sint_min_value<const BITS: BitCt>() -> i64 {
    debug_assert!(BITS <= 64);
    i64::MIN >> (i64::BITS - BITS)
}

const fn log2_floor(n: u64) -> u32 {
    debug_assert!(n != 0);
    (u64::BITS - 1) - n.leading_zeros()
}

pub trait BitIndex<Num>: Sized {
    /// bit range index
    /// zero indexed, inclusive high, inclusive low, `(high >= low)`
    /// bit len of returned value is `(high - low) + 1`
    fn bi(self, high: BitCt, low: BitCt) -> Num;
}

macro_rules! impl_bit_index {
    ($( $main:ident $( $result:ident )+ ; )+) => {
        $( $(
        impl BitIndex<$result> for $main {
            #[inline]
            fn bi(self, high: BitCt, low: BitCt) -> $result {
                debug_assert!(high < $result::BITS && high >= low);
                let shifted = (self >> low) as $result;
                let mask = !($result::MAX << (high - low + 1));
                shifted & mask
            }
        }
        )+ )+
    }
}

impl_bit_index! {
    u8 u8;
    u16 u16 u8;
    u32 u32 u16 u8;
    u64 u64 u32 u16 u8;
}

#[macro_export]
macro_rules! bi {
    ($tt:tt [$high:tt:$low:tt]) => {
        $tt.bits($high, $low)
    };
}

pub trait CreateMask: Sized {
    fn mask_lo_0s(n: BitCt) -> Self;
    fn mask_lo_1s(n: BitCt) -> Self;
    fn mask_hi_0s(n: BitCt) -> Self;
    fn mask_hi_1s(n: BitCt) -> Self;
}

macro_rules! impl_create_mask {
    ( $( $num:ident )* ) => {
        $(
        impl CreateMask for $num {
            fn mask_lo_0s(n: BitCt) -> $num {
                if n >= $num::BITS {
                    0
                } else {
                    $num::MAX << n
                }
            }
            fn mask_lo_1s(n: BitCt) -> $num {
                !Self::mask_lo_0s(n)
            }
            fn mask_hi_0s(n: BitCt) -> $num {
                if n >= $num::BITS {
                    0
                } else {
                    $num::MAX >> n
                }
            }
            fn mask_hi_1s(n: BitCt) -> $num {
                !Self::mask_hi_0s(n)
            }
        }
        )*
    };
}

impl_create_mask! { u8 u16 u32 u64 usize }

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Debug, PartialEq, Eq)]
    struct U32(u32);
    #[test]
    fn bit_range_works() {
        assert_eq!(0b1011u8.bi(2, 1), 0b01u8);
        assert_eq!(U32(0b1111u32.bi(1, 1)), U32(0b1u32));
        assert_eq!(0b1111u8.bi(1, 0), 0b11u8);
        assert_eq!(0b1111u8.bi(0, 0), 0b1u8);
    }

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
