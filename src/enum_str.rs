// #[macro_export]
// macro_rules! enum_str_kind {
//     {$( #[ $attr:meta ] )* $vis:vis enum $name:ident : $kind_name:ident $( <( $($generics:tt)+ ) for ( $($impls:tt)+ )> )?
//         {
//             $( $variant:ident $( ( $($tuple:tt)* ) )? $( { $($struct:tt)* } )? $( = $expr:expr)? ),* $(,)?
//         }
//     } => {
//         $( #[$attr] )*
//         $vis enum $name  $( < $( $generics )+ > )? {
//             $( $variant $( ( $($tuple)* ) )? $( { $($struct)* } )? $( = $expr )?, )*
//         }
//         todo!()
//     };
// }

#[macro_export]
macro_rules! enum_str {
    {$( #[ $attr:meta ] )* $vis:vis enum $name:ident
        {
            $(
                $( #[ $var_attr:meta ] )*
                $variant:ident $( = $expr:expr)?
            ),* $(,)?
        }
    } => {
        $( #[$attr] )*
        $vis enum $name {
            $(
                $( #[$var_attr] )*
                $variant $( = $expr )?,
            )*
        }
        #[allow(dead_code)]
        impl $crate::enum_str::EnumStr for $name {
            fn from_str(s: &str) -> ::std::option::Option<Self> {
                let hash = $crate::enum_str::str_hash(s);
                $(
                    if $crate::enum_str::str_hash( stringify!( $variant) ) == hash {
                        return Some( Self::$variant )
                    }
                )+
                None
            }
            fn from_str_lower(s: &str) -> ::std::option::Option<Self> {
                let hash = $crate::enum_str::str_hash(s);
                $(
                    if $crate::enum_str::str_hash_to_lower( stringify!( $variant) ) == hash {
                        return Some( Self::$variant )
                    }
                )+
                None
            }
            fn from_str_upper(s: &str) -> ::std::option::Option<Self> {
                let hash = $crate::enum_str::str_hash(s);
                $(
                    if $crate::enum_str::str_hash_to_upper( stringify!( $variant) ) == hash {
                        return Some( Self::$variant )
                    }
                )+
                None
            }
            #[inline]
            fn from_str_lower_or_upper(s: &str) -> Option<Self> {
                Self::from_str_lower(s).or_else(|| Self::from_str_upper(s))
            }
            fn from_str_mixed(s: &str) -> ::std::option::Option<Self> {
                let hash = $crate::enum_str::str_hash_to_lower(s);
                $(
                    if $crate::enum_str::str_hash_to_lower( stringify!( $variant) ) == hash {
                        return Some( Self::$variant )
                    }
                )+
                None
            }
            fn to_str(self) -> &'static str {
                match self {
                    $( Self:: $variant => stringify!( $variant ), )*
                }
            }
            fn to_str_lower(self) -> &'static str {
                ::paste::paste! {
                    match self {
                        $( Self:: $variant => stringify!( [<$variant:lower>] ), )*
                    }
                }
            }
            fn to_str_upper(self) -> &'static str {
                ::paste::paste! {
                    match self {
                        $( Self:: $variant => stringify!( [<$variant:upper>] ), )*
                    }
                }
            }
        }
    };
}

pub trait EnumStr: Sized {
    /// list of variants as strings, (same, lower, upper)
    /// const STRS: &'static [(&'static str, &'static str, &'static str)];
    /// list of hashes of variants as strings, (same, lower, upper)
    /// const HASHES: &'static [(u32, u32, u32)];

    fn from_str(s: &str) -> Option<Self>;
    fn from_str_lower(s: &str) -> Option<Self>;
    fn from_str_upper(s: &str) -> Option<Self>;
    fn from_str_lower_or_upper(s: &str) -> Option<Self>;
    fn from_str_mixed(s: &str) -> Option<Self>;
    fn to_str(self) -> &'static str;
    fn to_str_lower(self) -> &'static str;
    fn to_str_upper(self) -> &'static str;
}

const FNV_OFFSET: u32 = 2166136261u32;
const FNV_PRIME: u32 = 16777619u32;

// TODO: use fxhasher
#[inline]
pub const fn str_hash(s: &str) -> u32 {
    let mut hash: u32 = FNV_OFFSET;
    let mut i = 0;
    while i < s.len() {
        let c = s.as_bytes()[i] as u32;
        hash = (hash ^ c).wrapping_mul(FNV_PRIME);
        i += 1;
    }
    hash
}
#[inline]
pub const fn str_hash_to_lower(s: &str) -> u32 {
    let mut hash: u32 = FNV_OFFSET;
    let mut i = 0;
    while i < s.len() {
        let c = s.as_bytes()[i];
        let lc = c.to_ascii_lowercase() as u32;
        hash = (hash ^ lc).wrapping_mul(FNV_PRIME);
        i += 1;
    }
    hash
}
#[inline]
pub const fn str_hash_to_upper(s: &str) -> u32 {
    let mut hash: u32 = FNV_OFFSET;
    let mut i = 0;
    while i < s.len() {
        let c = s.as_bytes()[i];
        let lc = c.to_ascii_uppercase() as u32;
        hash = (hash ^ lc).wrapping_mul(FNV_PRIME);
        i += 1;
    }
    hash
}

// pub fn simd_to_uppercase(v: u8x8) -> u8x8 {
//     let ascii_bit = u8x8::splat(0x80);
//     let upper_bit = u8x8::splat(0x20);
//     // mask where true lanes are ASCII bytes 0-127
//     let mask: mask8x8 = v.simd_lt(ascii_bit);
//     // Values where each byte is the input byte with the case bit set to 0 (uppercase)
//     let xored = v ^ upper_bit;
//     // Blend the original values and the xored values based on the mask
//     mask.select(xored, v)
// }

#[cfg(test)]
mod tests {
    use super::*;

    enum_str! {
        #[derive(Debug, PartialEq, Eq)]
        #[allow(non_camel_case_types)]
        enum TestEnum {
            Aa,
            BB,
            cc
        }
    }

    #[test]
    fn it_from_str() {
        assert_eq!(TestEnum::from_str("Aa"), Some(TestEnum::Aa));
        assert_eq!(TestEnum::from_str("BB"), Some(TestEnum::BB));
        assert_eq!(TestEnum::from_str("cc"), Some(TestEnum::cc));

        assert_eq!(TestEnum::from_str_lower("aa"), Some(TestEnum::Aa));
        assert_eq!(TestEnum::from_str_lower("bb"), Some(TestEnum::BB));
        assert_eq!(TestEnum::from_str_lower("cc"), Some(TestEnum::cc));

        assert_eq!(TestEnum::from_str_lower("Aa"), None);
        assert_eq!(TestEnum::from_str_lower("BB"), None);
        assert_eq!(TestEnum::from_str_lower("CC"), None);
        assert_eq!(TestEnum::from_str_lower("dd"), None);

        assert_eq!(TestEnum::from_str_upper("AA"), Some(TestEnum::Aa));
        assert_eq!(TestEnum::from_str_upper("BB"), Some(TestEnum::BB));
        assert_eq!(TestEnum::from_str_upper("CC"), Some(TestEnum::cc));

        assert_eq!(TestEnum::from_str_upper("Aa"), None);
        assert_eq!(TestEnum::from_str_upper("bb"), None);
        assert_eq!(TestEnum::from_str_upper("cC"), None);
        assert_eq!(TestEnum::from_str_upper("dd"), None);
    }

    #[test]
    fn it_to_str() {
        assert_eq!(TestEnum::Aa.to_str(), "Aa");
        assert_eq!(TestEnum::BB.to_str(), "BB");
        assert_eq!(TestEnum::cc.to_str(), "cc");
        assert_eq!(TestEnum::Aa.to_str_upper(), "AA");
        assert_eq!(TestEnum::BB.to_str_upper(), "BB");
        assert_eq!(TestEnum::cc.to_str_upper(), "CC");
        assert_eq!(TestEnum::Aa.to_str_lower(), "aa");
        assert_eq!(TestEnum::BB.to_str_lower(), "bb");
        assert_eq!(TestEnum::cc.to_str_lower(), "cc");
    }
}
