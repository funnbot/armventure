use bit::{BitIndex, BitCt};
use numb;
use numb::int;
use crate::{
    bitstack::{push_bits_u32, BitStackU32},
    simpl,
    sparsebin::{Aligned, SparseBin},
};
use rustc_hash::FxHashMap as HashMap;
use std::fmt;

use super::{operand::Kind, util::Param};

pub struct VmState {
    pc: usize,
    mem: SparseBin,
}

const fn check_cmp<const LEN: u32, I: numb::UInt>(_: &I) {
    assert!(LEN == I::BITS);
}

macro_rules! _cmp {
    ( _, $value:tt ) => {
        true
    };
    ($var_name:ident, $value:tt) => {{
        const PAT: BitPat<u32> = bit_pat_str_32($pat);
        check_cmp::<{ PAT.len }, _>(&$value);
        debug_assert!($value.map(|v| !PAT.test(v as u32)));
        true
    }};
    ( ! $pat:literal, $value:tt ) => {{
        const PAT: BitPat<u32> = bit_pat_str_32($pat);
        check_cmp::<{ PAT.len }, _>(&$value);
        $value.map(|v| !PAT.test(v as u32))
    }};
    ( $pat:literal, $value:tt ) => {{
        const PAT: BitPat<u32> = bit_pat_str_32($pat);
        check_cmp::<{ PAT.len }, _>(&$value);
        $value.map(|v| PAT.test(v as u32))
    }};
}
macro_rules! _cmp_decode_one {
    ((!$pat:literal) $value:tt ) => {{
        const PAT: BitPat<u32> = bit_pat_str_32($pat);
        check_cmp::<{ PAT.len }, _>(&$value);
        $value.map(|v| !PAT.test(v as u32))
    }};
    (($pat:literal) $value:tt ) => {{
        const PAT: BitPat<u32> = bit_pat_str_32($pat);
        check_cmp::<{ PAT.len }, _>(&$value);
        $value.map(|v| PAT.test(v as u32))
    }};
}
macro_rules! bit_index {
    ([$br_high:literal] $value:ident) => {
        unsafe { numb::int::U1::new_unchecked($value.bi($br_high, $br_high)) }
    };
    ([$br_high:literal : $br_low:literal] $value:ident) => {
        unsafe {
            numb::UIntOf::<{ $br_high - $br_low + 1 }>::new_unchecked($value.bi($br_high, $br_low))
        }
    };
}
macro_rules! def_bit_range {
    ([$br_high:literal]) => {
        ($br_high as u32, $br_high as u32)
    };
    ([$br_high:literal : $br_low:literal]) => {
        ($br_high as u32, $br_low as u32)
    };
}
// macro_rules! _def_decode_param_len {
//     ($br_high:literal) => {
//         1
//     };
//     ($br_high:literal : $br_low:literal) => {
//         $br_high - $br_low + 1
//     };
// }
macro_rules! assign_bit_decode {
    ( ( $( $name:ident $opts:tt ),* ) $value:ident) => {
        $( let $name = bit_index!($opts $value); )*
    }
}
macro_rules! assign_bit_encode {
    ( ( $( $name:ident $opts:tt ),* ) ) => {
        $( const $name = def_bit_range!($opts); )*
    }
}
macro_rules! _cmp_decode {
    ( ( $( $name:ident $opts:tt ),* ) ) => {
        $( _cmp_decode_one!($opts $name) )&&*
    }
}

// macro_rules! match_cmp {
//     ($tuple:ident, ( $( $cmp_op:tt $( $cmp_pat:literal )? ),* ) ) => {
//         $( _cmp!( $cmp_op $( $cmp_pat )?, ($tuple.${index()}) ) )&&*
//     };
// }
// macro_rules! match_def {
//     ($value:ident, ( $( $br_high:tt $( : $br_low:literal )? ),* ) ) => {
//         ( $( _def_decode_param!($value, $br_high $(: $br_low )? ) , )* )
//     };
// }
// macro_rules! _def_decoder {
//     ($value:ident, $out_value:ident, $out_lens:ident, ( $( $br_high:tt $( : $br_low:literal )? ),* ) ) => {
//         const $out_lens: ( $( ${ignore(br_high)} u32 , )* ) =
//             ( $( _def_decode_param_len!($br_high $(: $br_low )? ) , )* );
//         let $out_value = ( $( _def_decode_param!($value, $br_high $( : $br_low )? ) , )* );
//     };
// }

macro_rules! _unroll_class_body {
    ( $target:ident $def_outer_decode:tt {
         $( $cmp_decode:tt => $instr_name:ident $( $instr_variant:ident )? ( $( $args:tt )* ) )*
    } ) => {
        _def_decode!($target, $def_outer_decode);
        $( _matcher! { Instr $target $cmp_decode $instr_name ( $( $args )* ) $( $instr_variant )? } )*
        ::std::unreachable!("unmatched class");
    };
}

macro_rules! unroll_group_body {
    ( {
        $( $kind:ident $desc:literal if $cmp_bits:tt => let $assign_bits:tt $body:tt )* 
    } ) => {

    };
}

macro_rules! unroll_class_body {
    ( {
        Args: $args:tt 
        $( $kind:ident $desc:literal if $cmp_bits:tt => let $assign_bits:tt $body:tt )* 
    } ) => {

    };
}

// macro_rules! _matcher {
//     ( Group $target:ident $outer_assign_bits:tt $cmp_bits:tt $assign_bits:tt $body:tt ) => {
//         if _cmp_decode!($cmp_decode) {
//             _unroll_group_body! { $target $def_decode $body }
//         }
//     };
//     ( Class $target:ident $outer_assign_bits:tt $cmp_bits:tt $assign_bits:tt $body:tt ) => {
//         if _cmp_decode!($cmp_decode) {
//             _unroll_class_body! { $target $def_decode $body }
//         }
//     };
//     ( Instr $target:ident $outer_assign_bits:tt $cmp_bits:tt $instr_name:ident ) => {
//         if _cmp_decode!($cmp_decode) {
//             return $crate::inst::Mnemonic::$instr_name;
//         }
//     };
// }

macro_rules! encode_body {
    ( Group $cmp_bits:tt $assign_bits:tt $body:tt ) => {
        unroll_group_encode! { $assign_bits $body }
    };
    ( Class $cmp_bits:tt $assign_bits:tt $body:tt ) => {
        
    };
    ( Instr $cmp_bits:tt $instr_name:ident ) => {
        
    };
}

macro_rules! unroll_group_encode {
    ( $outer_assign_bits:tt {
        $( $kind:ident $desc:literal if $cmp_bits:tt => let $assign_bits:tt $body:tt )* 
    } ) => {
        {
            assign_bit_encode!($outer_assign_bits);
            $( encode_body! { $kind $cmp_bits $assign_bits $body } )*
        }
    };
}

macro_rules! unroll_class_encode {
    ( {
        Args: $args:tt 
        $( $kind:ident $desc:literal if $cmp_bits:tt => let $assign_bits:tt $body:tt )* 
    } ) => {

    };
}

macro_rules! mnem_body {
    ( Group $cmp_bits:tt $assign_bits:tt $body:tt ) => {
        unroll_group_mnem! { $assign_bits $body }
    };
    ( Class $cmp_bits:tt $assign_bits:tt $body:tt ) => {
        
    };
    ( Instr $cmp_bits:tt $instr_name:ident ) => {
        
    };
}

macro_rules! unroll_group_mnem {
    ( $outer_assign_bits:tt {
        $( $kind:ident $desc:literal if $cmp_bits:tt => let $assign_bits:tt $body:tt )* 
    } ) => {
        {
            assign_bit_mnem!($outer_assign_bits);
            $( mnem_body! { $kind $cmp_bits $assign_bits $body } )*
        }
    };
}

macro_rules! unroll_class_mnem {
    ( {
        Args: $args:tt 
        $( $kind:ident $desc:literal if $cmp_bits:tt => let $assign_bits:tt $body:tt )* 
    } ) => {

    };
}

macro_rules! _unroll_group_body {
    ( $target:ident $def_outer_decode:tt {
        $( $kind:ident $cmp_decode:tt => $def_decode:tt $body:tt )*
    } ) => {
        {
            _def_decode!($target, $def_outer_decode);
            $( _matcher! { $kind $target $cmp_decode $def_decode $body } )*
            ::std::unreachable!("unmatched group");
        }
    }
}

macro_rules! def_matcher {
    { $fn_name:ident $def_top_decode:tt $body:tt } => {
        pub fn $fn_name(target: u32) -> $crate::inst::Mnemonic {
            _unroll_group_body! { target $def_top_decode $body }
        }
    }
}

macro_rules! define_instr_set {
    {
        mnemonics: $mnem_body:tt
        encodings: let $def_top_decode:tt $encodings_body:tt
    } => {
        fn decode() {
            
        }
        fn encode() {
            unroll_group_encode! { $def_top_decode $encodings_body }
        }
    }
}

define_instr_set! {
    mnemonics: {

    }
    encodings: let() {

    }
}

// when parsing, you have the Mnemonic, and the list of parsed args, the args can be parsed without context, like knowing the mnemonic.
// then, match the parsed arg kinds to the list of a list of allowed arg args for that mnemonic, selecting the variant used for that menonic from the list that matched
// there are some cases where the list of allowed args, due to optional args, leads to multiple matched variants, one needs to be prefered, for ADD ShiftedRegister and ADD ExtendedRegister, ExtendedRegister allows for special (SP, ZR) registers in some operands, which allows for differentiating, however, I don't know if thats the case for every instruction,

// def_matcher! {
//     decode_instr (sf(31), main_group(28:25)) {
//         Class (sf("0"), main_group("0000")) => (op0(30:29), op1(24:16)) {
//             (op0("00"), op1("000000000")) => ADD Immediate ()
//         }
//         Group (main_group("100x")) => (op0(25:23)) {
//             Class (op0("010")) => (opc(30), s(29), sh(22), imm12(12:10), Rn(9:5), Rd(4:0)) {
//                 (opc("0"), s("0")) => ADD Immediate ()
//             }
//         }
//     }
// }

macro_rules! any_syntax {
    ($($tt:tt)*) => {};
}

any_syntax! {
    encodings: let(sf[31], main_group[28:25]) {
        Group DataProc_Immediate:
        if(main_group("100x")) => let(op0[25:23]) {
            Class Add_Subtract:
            if(op0("010")) => let(opc[30], S[29]) {
                Args: (Rd: GprOrSp([4:0]), Rd: GprOrSp([9:5]), imm: UImm([21:10], Align4), sh: ConstShift([22], LSL, 12))
                Encoding: (sf opc S "100010" sh imm Rn Rd)
                Mnems: [ADD(0, 0, 0), ADDS(0, 0, 1), SUB(0, 1, 0), SUBS(0, 1, 1)]
                if(sf("0"), opc("0"), S("0")) => |sf: bool, opc: bool, |
            }
        }
    }
    operations: |a: Args, e: Executor| {
        ADD_Immediate(sf: u8, opc: u8, tagged: u8): {
            
        }
    }
}

// pub fn decode(mut state: VmState) {
//     let instr = state.mem.get_u32(Aligned::new(state.pc).unwrap());
//     //decode_instr(instr);
// }

trait Executor {
    fn read_u32(&mut self) -> u32;
}

#[derive(Default, Clone, Copy)]
pub struct BitPat<Int> {
    mask: Int,
    set_bits: Int,
    len: u32,
}

simpl! {Debug |self: BitPat<u32>| "(mask: 0b{:b}, bits: 0b{:b})", self.mask, self.set_bits }

// impl<I: numb::UInt> BitPat<I> {
//     #[inline]
//     pub const fn test(self, value: I) -> bool {
//         let uval = value.as_usize();
//         let ones = self.ones.as_usize();
//         let zeroes = self.zeroes.as_usize();
//         (uval & ones == ones) && (!uval & zeroes == zeroes)
//     }
// }

impl BitPat<u32> {
    /// check if all set bits in `ones` are set in value
    /// and all set bits in `zeroes` are not set in value
    #[inline]
    pub const fn test(self, value: u32) -> bool {
        value & self.mask == self.set_bits
    }
}

pub const fn bit_pat_str_len(str: &str) -> u32 {
    let s = str.as_bytes();
    let mut len: u32 = 0;
    let mut i: usize = 0;
    while i < s.len() {
        if s[i] != b'_' {
            len += 1;
        }
        i += 1;
    }
    len
}

pub const fn bit_pat_str<const LEN: BitCt>(str: &str) -> (u64, u64) {
    let s = str.as_bytes();
    let mut mask: u64 = 0;
    let mut value: u64 = 0;
    let mut len: BitCt = 0;
    let mut i: usize = 0;
    while i < s.len() {
        if s[i] != b'_' {
            mask <<= 1;
            value <<= 1;
            len += 1;
        }
        match s[i] {
            b'0' => {
                mask |= 1;
            }
            b'1' => {
                mask |= 1;
                value |= 1;
            }
            b'x' | b'_' => (),
            _ => panic!("invalid bit pattern character"),
        }
        i += 1;
    }
    if len == 0 {
        panic!("bit pattern zero length");
    }
    if mask == 0 {
        panic!("bit pattern matches anything");
    }
    if LEN != len {
        panic!("bit pattern length mismatch");
    }
    (mask, value)
}

macro_rules! bitpat {
    ($pat_str:literal) => {
        {
            const LEN: u32 = bit_pat_str_len($pat_str);
            type Int = ::numb::UIntOf::<LEN>;
            type PrimInt = ::numb::PrimUIntOf::<LEN>;
            const PAT: (u64, u64) = bit_pat_str::<LEN>($pat_str);

            ::numb::BitPat::<Int> {
                mask: unsafe { Int::new_unchecked(PAT.0 as PrimInt) },
                value: unsafe { Int::new_unchecked(PAT.1 as PrimInt) },
            }
        }
    }
}

pub const fn bit_pat_str_32(s: &str) -> BitPat<u32> {
    bit_pat_bstr_32(s.as_bytes())
}

pub const fn bit_pat_bstr_32(s: &[u8]) -> BitPat<u32> {
    assert!(s.len() <= 32);
    let mut mask: u32 = 0;
    let mut set_bits: u32 = 0;
    let mut len: u32 = 0;
    let mut i: usize = 0;
    while i < s.len() {
        if s[i] != b'_' {
            mask <<= 1;
            set_bits <<= 1;
            len += 1;
        }
        match s[i] {
            b'0' => {
                mask |= 1;
            }
            b'1' => {
                mask |= 1;
                set_bits |= 1;
            }
            b'x' | b'_' => (),
            _ => panic!("invalid bit pattern character"),
        }
        i += 1;
    }
    if len == 0 {
        panic!("bit pattern zero length");
    }
    if mask == 0 {
        panic!("bit pattern matches anything");
    }
    BitPat {
        mask,
        set_bits,
        len,
    }
}
