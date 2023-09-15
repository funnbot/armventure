#![allow(internal_features)]
#![feature(rustc_attrs)]
#![feature(specialization)]
#![feature(never_type)]

pub mod int;

use bit::{BitCt, CreateMask};

#[rustfmt::skip]
mod detail {
    use super::BitCt;

    pub struct TypeOf;

    pub trait SIntOf<const N: BitCt> { type Type; }
    pub trait UIntOf<const N: BitCt> { type Type; }
    pub trait PrimUIntOf<const N: BitCt> { type Type; }
    pub trait PrimSIntOf<const N: BitCt> { type Type; }

    impl<const N: BitCt> UIntOf<N> for TypeOf {
        default type Type = !;
    }
    impl<const N: BitCt> SIntOf<N> for TypeOf {
        default type Type = !;
    }
    impl<const N: BitCt> PrimUIntOf<N> for TypeOf {
        default type Type = !;
    }
    impl<const N: BitCt> PrimSIntOf<N> for TypeOf {
        default type Type = !;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BitPat<Int: UInt> {
    pub value: Int,
    pub mask: Int,
}
impl<Int: UInt> BitPat<Int> {
    pub const BITS: BitCt = Int::BITS;
}

#[allow(clippy::wrong_self_convention)]
pub trait Integer
where
    Self: PartialEq + Eq + PartialOrd + Ord + Sized + From<Self::Prim>,
    Self::Prim: From<Self>,
{
    type Prim;
    const BITS: BitCt;
    const MIN: Self;
    const MAX: Self;

    fn as_isize(self) -> isize;
    fn as_i64(self) -> i64;
    fn as_i32(self) -> i32;
    fn as_i16(self) -> i16;
    fn as_i8(self) -> i8;

    fn as_usize(self) -> usize;
    fn as_u64(self) -> u64;
    fn as_u32(self) -> u32;
    fn as_u16(self) -> u16;
    fn as_u8(self) -> u8;
}

pub trait UInt: Integer {}
pub trait SInt: Integer {}

macro_rules! const_map {
    (UInt $prim:ident $max:tt |$value:ident| $body:expr) => {{
        const MAX_0: $prim = $max + 1;
        const MAX_1: $prim = $prim::MAX;
        match $value {
            (MAX_0..=MAX_1) => unsafe { unreachable_unchecked() },
            $value => $body,
        }
    }};
    (SInt $prim:ident $max:tt $min:tt |$value:ident| $body:expr) => {{
        const MIN_0: $prim = $prim::MIN;
        const MIN_1: $prim = $min - 1;
        const MAX_0: $prim = $max + 1;
        const MAX_1: $prim = $prim::MAX;
        match $value {
            (MIN_0..=MIN_1) | (MAX_0..=MAX_1) => unsafe { unreachable_unchecked() },
            $value => $body,
        }
    }};
}

#[rustfmt::skip]
macro_rules! impl_int_trait {
    ($trait:ident $name:ident $ty:ident $bits:literal : $max:tt : $min:tt) => {
        impl Integer for $name {
            type Prim = $ty;
            const BITS: BitCt = $bits;
            #[allow(unused_unsafe)]
            const MIN: Self = unsafe { Self($min) };
            #[allow(unused_unsafe)]
            const MAX: Self = unsafe { Self($max) };

            #[inline] fn as_isize(self) -> isize { self.0 as isize }
            #[inline] fn as_i64(self) -> i64 { self.0 as i64 }
            #[inline] fn as_i32(self) -> i32 { self.0 as i32 }
            #[inline] fn as_i16(self) -> i16 { self.0 as i16 }
            #[inline] fn as_i8(self) -> i8 { self.0 as i8 }

            #[inline] fn as_usize(self) -> usize { self.0 as usize }
            #[inline] fn as_u64(self) -> u64 { self.0 as u64 }
            #[inline] fn as_u32(self) -> u32 { self.0 as u32 }
            #[inline] fn as_u16(self) -> u16 { self.0 as u16 }
            #[inline] fn as_u8(self) -> u8 { self.0 as u8 }
        }
        impl $trait for $name {}
    };
}

macro_rules! def_uint {
    { $name:ident $ty:ident $bits:literal : $max:tt } => {
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
        #[repr(transparent)]
        #[rustc_layout_scalar_valid_range_end($max)]
        pub struct $name($ty);

        impl_int_trait! { UInt $name $ty $bits:$max:0 }

        impl $name {
            /// initialize from next real integer type
            /// panics if out of range
            #[inline]
            pub const fn new(value: $ty) -> Self {
                assert!(uint_in_range::<$bits>(value as u64));
                unsafe { Self(value) }
            }
            /// returns None if out of range
            #[inline]
            pub const fn try_new(value: $ty) -> Option<Self> {
                if uint_in_range::<$bits>(value as u64) {
                    Some(unsafe { Self(value) })
                } else {
                    None
                }
            }
            /// clamps the value if out of range
            #[inline]
            pub const fn new_clamp(value: $ty) -> Self {
                if $bits == $ty::BITS {
                    return unsafe { Self(value) };
                }
                unsafe { Self(value & !($ty::MAX << $bits)) }
            }
            /// # Safety
            /// value must be in range
            #[inline]
            pub const unsafe fn new_unchecked(value: $ty) -> Self {
                debug_assert!(uint_in_range::<$bits>(value as u64));
                Self(value)
            }
            #[inline]
            pub const fn get(self) -> $ty {
                self.0
            }
            pub fn map<R, F: FnOnce($ty) -> R>(self, f: F) -> R {
                let v = self.0;
                const_map!(UInt $ty $max |v| f(v))
            }
        }

        impl BitPat<$name> {
            #[inline]
            pub const fn test(self, value: $name) -> bool {
                let v = value.0;
                const_map!(UInt $ty $max |v|
                    self.value.0 == (v & self.mask.0))
            }
        }
    }
}

macro_rules! def_sint {
    { $name:ident $ty:ident $bits:literal : $max:tt : $min:tt : $layout_end:literal } => {
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
        #[repr(transparent)]
        #[rustc_layout_scalar_valid_range_end($layout_end)]
        pub struct $name($ty);

        impl_int_trait! { SInt $name $ty $bits:$max:$min }

        impl $name {
            /// initialize from next real integer type
            /// panics if out of range
            #[inline]
            pub const fn new(value: $ty) -> Self {
                assert!(sint_in_range::<$bits>(value as i64));
                unsafe { Self(value) }
            }
            /// returns None if out of range
            #[inline]
            pub const fn try_new(value: $ty) -> Option<Self> {
                if sint_in_range::<$bits>(value as i64) {
                    Some(unsafe { Self(value) })
                } else {
                    None
                }
            }
            /// clamps the value if out of range
            #[inline]
            pub const fn new_clamp(value: $ty) -> Self {
                if $bits == $ty::BITS {
                    return unsafe { Self(value) };
                }
                let uval = value as $ty;
                let mask = $ty::MAX << ($bits - 1);
                if value >= 0 {
                    unsafe { Self((uval & !mask) as $ty) }
                } else {
                    unsafe { Self((uval | mask) as $ty) }
                }
            }
            /// # Safety
            /// value must be in range
            #[inline]
            pub const unsafe fn new_unchecked(value: $ty) -> Self {
                debug_assert!(sint_in_range::<$bits>(value as i64));
                Self(value)
            }
            #[inline]
            pub const fn get(self) -> $ty {
                self.0
            }
            pub fn map<R, F: FnOnce($ty) -> R>(self, f: F) -> R {
                let v = self.0;
                const_map!(SInt $ty $max $min |v| f(v))
            }
        }
    }
}

macro_rules! def_wrap_prim_int {
    ( $name:ident $ty:ident $bits:literal $kind:ident ) => {
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
        #[repr(transparent)]
        pub struct $name($ty);

        impl_int_trait! { $kind $name $ty $bits:($ty::MAX):($ty::MIN) }

        impl $name {
            /// never panics
            #[inline]
            pub const fn new(value: $ty) -> Self {
                Self(value)
            }
            /// equivalent to new()
            /// always returns Some
            #[inline]
            pub const fn try_new(value: $ty) -> Option<Self> {
                Some(Self(value))
            }
            /// equivalent to new()
            #[inline]
            pub const fn new_clamp(value: $ty) -> Self {
                Self(value)
            }
            /// # Safety
            /// not unsafe, here to match other integer types
            #[inline]
            pub const unsafe fn new_unchecked(value: $ty) -> Self {
                Self(value)
            }
            #[inline]
            pub const fn get(self) -> $ty {
                self.0
            }
            pub fn map<R, F: FnOnce($ty) -> R>(self, f: F) -> R {
                f(self.0)
            }
        }
    };
}

macro_rules! impl_int {
    { $name:ident $ty:ident $typeof_trait:ident $typeof_prim_trait:ident $bits:literal } => {
        impl detail::$typeof_prim_trait<$bits> for detail::TypeOf {
            type Type = $ty;
        }
        impl detail::$typeof_trait<$bits> for detail::TypeOf {
            type Type = $name;
        }

        impl ::std::convert::From<$ty> for $name {
            #[inline]
            fn from(value: $ty) -> Self {
                Self::new(value)
            }
        }
        impl ::std::convert::From<$name> for $ty {
            #[inline]
            fn from(value: $name) -> $ty {
                value.0
            }
        }
        impl ::std::default::Default for $name {
            fn default() -> Self {
                #[allow(unused_unsafe)]
                unsafe { Self(0) }
            }
        }
        impl ::std::fmt::Display for $name {
            #[inline]
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
        impl ::std::cmp::PartialEq<$ty> for $name {
            fn eq(&self, rhs: & $ty) -> bool {
                self.0 == *rhs
            }
        }
    }
}

pub(crate) use const_map;
pub(crate) use def_sint;
pub(crate) use def_uint;
pub(crate) use def_wrap_prim_int;
pub(crate) use impl_int;
pub(crate) use impl_int_trait;

pub type PrimUIntOf<const N: BitCt> = <detail::TypeOf as detail::PrimUIntOf<N>>::Type;
pub type PrimSIntOf<const N: BitCt> = <detail::TypeOf as detail::PrimSIntOf<N>>::Type;

pub type UIntOf<const N: BitCt> = <detail::TypeOf as detail::UIntOf<N>>::Type;
pub type SIntOf<const N: BitCt> = <detail::TypeOf as detail::SIntOf<N>>::Type;

fn gen_macro(max_bits: u32) {
    const PRIMS: &[(&str, &str, u32)] = &[
        ("u8", "i8", 8),
        ("u16", "i16", 16),
        ("u32", "i32", 32),
        ("u64", "i64", 64),
    ];

    for bits in 1..=max_bits {
        let (uprim, sprim, prim_bits) = PRIMS[if bits <= 8 {
            0
        } else if bits <= 16 {
            1
        } else if bits <= 32 {
            2
        } else {
            3
        }];
        let rs = 64 - bits;
        let umax = u64::MAX >> rs;
        let imax = i64::MAX >> rs;
        let imin = i64::MIN >> rs;

        let ilaymin = imin as u64 & u64::mask_lo_1s(prim_bits);

        if bits == 8 || bits == 16 || bits == 32 || bits == 64 {
            println!("def_wrap_prim_int!(U{bits} {uprim} {bits} UInt);");
            println!("def_wrap_prim_int!(I{bits} {sprim} {bits} SInt);");
        } else {
            println!("def_uint!(U{bits} {uprim} {bits}:{umax});");
            println!("def_sint!(I{bits} {sprim} {bits}:{imax}:({imin}):{ilaymin});");
        }
        println!("impl_int!(U{bits} {uprim} UIntOf PrimUIntOf {bits});");
        println!("impl_int!(I{bits} {sprim} SIntOf PrimSIntOf {bits});");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gen_macro_test() {
        gen_macro(64);
    }
}
