use std::ops;

pub trait Num: Sized {
    const BITS: u32;
    const MAX: Self;
    const MIN: Self;
}
pub trait Int:
    Num
    + ops::Add<Self, Output = Self>
    + ops::AddAssign<Self>
    + ops::Sub<Self, Output = Self>
    + ops::SubAssign<Self>
    + ops::Mul<Self, Output = Self>
    + ops::MulAssign<Self>
    + ops::Div<Self, Output = Self>
    + ops::DivAssign<Self>
    + ops::BitAnd<Self, Output = Self>
    + ops::BitAndAssign<Self>
    + ops::BitOr<Self, Output = Self>
    + ops::BitOrAssign<Self>
    + ops::BitXor<Self, Output = Self>
    + ops::BitXorAssign<Self>
    + ops::Shl<Self, Output = Self>
    + ops::ShlAssign<Self>
    + ops::Shr<Self, Output = Self>
    + ops::ShrAssign<Self>
    + ops::Not<Output = Self>
    + PartialOrd
    + Ord
    + PartialEq
    + Eq
    + Clone
    + Copy
{
    type Signed;
    type Unsigned;
    const IS_SIGNED: bool;
}

macro_rules! impl_num {
    ($($ut:ident $st:ident)+) => {
        $(
            impl Num for $ut {
                const BITS: u32 = $ut::BITS;
                const MAX: $ut = $ut::MAX;
                const MIN: $ut = $ut::MIN;
            }
            impl Int for $ut {
                type Signed = $st;
                type Unsigned = $ut;
                const IS_SIGNED: bool = false;
            }
            impl Num for $st {
                const BITS: u32 = $st::BITS;
                const MAX: $st = $st::MAX;
                const MIN: $st = $st::MIN;
            }
            impl Int for $st {
                type Signed = $st;
                type Unsigned = $ut;
                const IS_SIGNED: bool = true;
            }
        )+
    }
}

impl_num! {
    u8 i8
    u16 i16
    u32 i32
    u64 i64
    u128 i128
}
