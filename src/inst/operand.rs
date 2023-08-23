use crate::bitwriter::{BitWriter, BitWriterU32};
use lasso::Spur;
use std::{marker::PhantomData, rc::Rc};
use subenum::subenum;

use super::{
    sint_in_range, sint_is_aligned, uint_in_range, uint_is_aligned, Emitter, Encode, Fixup, IntN,
    ParseError, Resolvable,
};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Kind {
    Gpr,
    Dpr,
    Imm,
    Label,
    AddrImm,
    AddrReg,
    AddrLabel,
    Shift,
    Extend,
    Cond,
}

pub enum Ops {
    Gpr {
        reg: Reg,
        size: GprSize,
    },
    Dpr {
        reg: Reg,
        size: Size,
    },
    Imm(i64),
    Label(Spur),
    Shift {
        kind: ShiftKind,
        amt: u8,
    },
    Extend {
        kind: ExtendKind,
        ls_amt: Option<u8>,
    },
    Cond(Cond),
}

#[const_trait]
pub trait Operand: Sized {
    const KIND: Kind;
    fn from_op(op: Ops) -> Result<Self, ParseError> {
        todo!()
    }
}

pub struct LabelValue(pub u64);

pub trait Zero: Sized {
    fn zero() -> Self;
}

#[rustfmt::skip]
#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Reg {
    R0 = 0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14, R15,
    R16, R17, R18, R19, R20, R21, R22, R23, R24, R25, R26, R27, R28, R29, R30, R31 = 31
}

/// reg no special
#[rustfmt::skip]
#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum RegNS {
    R0 = 0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14, R15,
    R16, R17, R18, R19, R20, R21, R22, R23, R24, R25, R26, R27, R28, R29, R30 = 30
}

/// size in bytes
#[subenum(GprSize, VecSize, VecLanes)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(u8)]
pub enum Size {
    /// byte, 8 bits, aka scalar `Bn`
    #[subenum(VecSize)]
    B1 = 1,
    /// word, 16 bits, aka scalar `Hn`
    #[subenum(VecSize)]
    B2 = 2,
    /// dword, 32 bits, aka scalar `Sn` or gpr `Wn`
    #[subenum(VecSize, GprSize)]
    B4 = 4,
    /// qword, 64 bits, aka scalar `Dn` or gpr `Xn`
    #[subenum(VecSize, GprSize, VecLanes)]
    B8 = 8,
    /// vector, 128 bits, aka scalar `Qn` or vector `Vn`
    #[subenum(VecLanes)]
    B16 = 16,
}

/// alignment `Bn` == 2^n
///
/// bits to shift right by and the number of trailing 0s
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub(super) enum AlignBits {
    B1 = 0,
    B2 = 1,
    B4 = 2,
    B8 = 3,
    B16 = 4,
    B32 = 5,
    B64 = 6,
    B128 = 7,
}

#[derive(Clone, Copy, Debug)]
pub enum ShiftKind {
    LSL = 0b00, // logical shift left
    LSR = 0b01, // logical shift right (zeros)
    ASR = 0b10, // arithmetic shift right (maintain sign)
}

#[derive(Clone, Copy, Debug)]
pub struct ShiftAmount<const BITS: u8>(pub u8);

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[allow(non_camel_case_types)]
pub enum ExtendKind {
    // zero extend to 1 byte
    UXTB = 0b000,
    // zero extend to 2 bytes
    UXTH = 0b001,
    // zero extend to 4 bytes
    UXTW_OR_LSL = 0b010,
    // zero extend to 8 bytes
    UXTX = 0b011,
    // sign extend to 1 byte
    SXTB = 0b100,
    // sign extend to 2 bytes
    SXTH = 0b101,
    // sign extend to 4 bytes
    SXTW = 0b110,
    // sign extend to 8 bytes
    SXTX = 0b111,
}

pub mod op {
    use super::*;

    // general purpose register
    pub struct Gpr {
        pub reg: RegNS,
        pub size: GprSize,
    }

    pub struct GprOrSp {
        pub reg: Reg,
        pub size: GprSize,
    }

    pub struct GprOrZr {
        pub reg: Reg,
        pub size: GprSize,
    }

    // data processing register, aka scalar register
    pub struct Dpr {
        pub reg: Reg,
        pub size: Size,
    }

    // simd vector register
    pub struct Vr {
        pub reg: Reg,
        pub size: VecSize,
        pub lanes: VecLanes,
    }

    // index scalar in vector register
    pub struct IdxVr {
        pub reg: Reg,
        pub size: VecSize,
        // index infers the lanes
        pub index: u8,
    }

    // Label will get resolved at emit time, during parsing it will be a Spur to an interned string
    // tried to resolve when emitting/encoding, but in order to get the byte index of the label, it needs to
    // already be resolved, so there needs to be a fixup step after emitting.
    // or the addresses could be calculated and then the emitting step could take place.

    // Fixup step afterwards would just apply labels with already calculated addresses
    // fixup needs the byte address and the length to mask
    // it also would be parsing an i64 label address, so it needs to have error handling of ParseError
    // If the address gets applied in two parts, then it will need to make two fixup pushes.
    // The key of the fixup needs to be stored during emitting, probably in the instruction operand, and it needs to access global state to create that key.

    pub struct LabelSImm<const BITS: u8, const LS: u8 = 0>(pub Option<Spur>);
    pub struct LabelUImm<const BITS: u8, const LS: u8 = 0>(pub Option<Spur>);

    pub struct Label<T>(pub Option<Spur>, pub std::marker::PhantomData<T>);

    pub struct Shift<const BITS: u8> {
        pub kind: ShiftKind,
        pub amount: ShiftAmount<BITS>,
    }

    pub struct ShiftConst<const KIND: u8, const AMT: u8>(pub bool);

    pub struct Extend {
        pub kind: ExtendKind,
        pub lsl_amount: ShiftAmount<3>,
    }

    #[derive(Debug)]
    pub struct SImm<const BITS: u8, const ALIGN_BITS: u8 = 0>(pub i32);
    #[derive(Debug)]
    pub struct UImm<const BITS: u8, const ALIGN_BITS: u8 = 0>(pub u32);

    #[rustfmt::skip]
    #[repr(u8)]
    pub enum Cond {
        EQ = 0b0000, // Z == 1
        NE = 0b0001, // !EQ
        CS = 0b0010, // C == 1
        CC = 0b0011, // !CC
        MI = 0b0100, // N == 1
        PL = 0b0101, // !MI
        VS = 0b0110, // V == 1
        VC = 0b0111, // !VC
        HI = 0b1000, // C == 1 && Z == 0
        LS = 0b1001, // !HI
        GE = 0b1010, // N == V
        LT = 0b1011, // !GE
        GT = 0b1100, // Z == 0 && N == V
        LE = 0b1101, // !GT
        AL = 0b1110, // Any
        NV = 0b1111, // Any (unused)
    }
}
pub use op::*;

pub trait Encoder<T> {
    // TODO: return Result<IntN, ParseError>
    fn encode<E: Emitter>(v: &T, e: &mut E) -> IntN;
}

pub mod enc {
    pub struct Op;
    pub struct OpShiftKind;
    pub struct OpShiftAmount;
    pub struct OpExtendKind;
    pub struct OpKind;
}

impl Encoder<Gpr> for enc::Op {
    fn encode<E: Emitter>(v: &Gpr, e: &mut E) -> IntN {
        IntN(v.reg as u32, 5)
    }
}
impl<const BITS: u8> Encoder<Shift<BITS>> for enc::OpKind {
    fn encode<E: Emitter>(v: &Shift<BITS>, e: &mut E) -> IntN {
        IntN(v.kind as u32, 2)
    }
}

fn fixup_label_fn<E: Emitter, T: Operand + Zero>(label_addr: u64, e: &mut E) -> IntN
where
    enc::Op: Encoder<T>,
{
    let current_instr_addr = e.pc() as i64;
    let offset = label_addr as i64 - current_instr_addr;
    let op = Ops::Imm(offset);
    let result = T::from_op(op).unwrap();
    <enc::Op as Encoder<T>>::encode(&result, e)
}

impl<T> Encoder<Label<T>> for enc::Op
where
    T: Operand + Zero,
    enc::Op: Encoder<T>,
{
    fn encode<E: Emitter>(v: &Label<T>, e: &mut E) -> IntN {
        match v.0 {
            Some(spur) => match e.resolve_label(spur) {
                Some(label_addr) => fixup_label_fn::<E, T>(label_addr, e),
                None => {
                    let fixup = super::create_fixup(e, spur, fixup_label_fn::<E, T>);
                    e.push_label_fixup(fixup);
                    <enc::Op as Encoder<T>>::encode(&Zero::zero(), e)
                }
            },
            None => <enc::Op as Encoder<T>>::encode(&Zero::zero(), e),
        }
    }
}
impl<const BITS: u8, const ALIGN: u8> Encoder<SImm<BITS, ALIGN>> for enc::Op {
    fn encode<E: Emitter>(v: &SImm<BITS, ALIGN>, e: &mut E) -> IntN {
        IntN(v.0 as u32, BITS)
    }
}

impl const Operand for Gpr {
    const KIND: Kind = Kind::Gpr;
}
impl Operand for GprOrSp {
    const KIND: Kind = Kind::Gpr;
}
impl Operand for GprOrZr {
    const KIND: Kind = Kind::Gpr;
}
impl Operand for Dpr {
    const KIND: Kind = Kind::Dpr;
}
impl<const BITS: u8, const ALIGN: u8> Operand for UImm<BITS, ALIGN> {
    const KIND: Kind = Kind::Imm;
    fn from_op(op: Ops) -> Result<Self, ParseError> {
        match op {
            Ops::Imm(imm) => {
                let value = imm as u64;
                if !uint_is_aligned::<ALIGN>(value) {
                    return Err(ParseError::NotAligned);
                }
                let aligned = value >> ALIGN;
                if !uint_in_range::<BITS>(aligned) {
                    Err(ParseError::OutOfRange)
                } else {
                    Ok(Self(aligned as u32))
                }
            }
            _ => Err(ParseError::Unexpected),
        }
    }
}
impl<const BITS: u8, const ALIGN: u8> Operand for SImm<BITS, ALIGN> {
    const KIND: Kind = Kind::Imm;
    fn from_op(op: Ops) -> Result<Self, ParseError> {
        match op {
            Ops::Imm(imm) => {
                if !sint_is_aligned::<ALIGN>(imm) {
                    return Err(ParseError::NotAligned);
                }
                let aligned = imm >> ALIGN;
                if !sint_in_range::<BITS>(aligned) {
                    Err(ParseError::OutOfRange)
                } else {
                    Ok(Self(aligned as i32))
                }
            }
            _ => Err(ParseError::Unexpected),
        }
    }
}
impl<const BITS: u8> Operand for Shift<BITS> {
    const KIND: Kind = Kind::Shift;
}
impl<const KIND: u8, const AMT: u8> Operand for ShiftConst<KIND, AMT> {
    const KIND: Kind = Kind::Shift;
}
impl Operand for Extend {
    const KIND: Kind = Kind::Extend;
}
impl<const BITS: u8, const ALIGN: u8> Operand for LabelUImm<BITS, ALIGN> {
    const KIND: Kind = Kind::Label;
}
impl<const BITS: u8, const ALIGN: u8> Operand for LabelSImm<BITS, ALIGN> {
    const KIND: Kind = Kind::Label;
    fn from_op(op: Ops) -> Result<Self, ParseError> {
        match op {
            Ops::Label(label) => Ok(Self(Some(label))),
            _ => Err(ParseError::Unexpected),
        }
    }
}
impl<T> Operand for Label<T> {
    const KIND: Kind = Kind::Label;
    fn from_op(op: Ops) -> Result<Self, ParseError> {
        match op {
            Ops::Label(label) => Ok(Self(Some(label), PhantomData)),
            _ => Err(ParseError::Unexpected),
        }
    }
}
impl Operand for Cond {
    const KIND: Kind = Kind::Cond;
}

impl Encode for Gpr {
    fn encode(&self) -> IntN {
        IntN(self.reg as u32, 5)
    }
}
impl Encode for GprOrSp {
    fn encode(&self) -> IntN {
        IntN(self.reg as u32, 5)
    }
}
impl Encode for GprOrZr {
    fn encode(&self) -> IntN {
        IntN(self.reg as u32, 5)
    }
}
impl<const BITS: u8, const ALIGN: u8> Encode for SImm<BITS, ALIGN> {
    fn encode(&self) -> IntN {
        IntN(self.0 as u32, BITS)
    }
}
impl<const BITS: u8, const ALIGN: u8> Encode for UImm<BITS, ALIGN> {
    fn encode(&self) -> IntN {
        IntN(self.0, BITS)
    }
}
impl<const KIND: u8, const AMT: u8> Encode for ShiftConst<KIND, AMT> {
    fn encode(&self) -> IntN {
        IntN(self.0 as u32, 1)
    }
}

impl<const BITS: u8, const ALIGN: u8> Zero for SImm<BITS, ALIGN> {
    fn zero() -> Self {
        Self(0)
    }
}
impl<const BITS: u8, const ALIGN: u8> Zero for UImm<BITS, ALIGN> {
    fn zero() -> Self {
        Self(0)
    }
}
impl<const BITS: u8> Zero for Shift<BITS> {
    fn zero() -> Self {
        Self {
            kind: ShiftKind::LSL,
            amount: ShiftAmount(0),
        }
    }
}
impl<const KIND: u8, const AMT: u8> Zero for ShiftConst<KIND, AMT> {
    fn zero() -> Self {
        Self(false)
    }
}
impl Zero for Extend {
    fn zero() -> Self {
        Self {
            kind: ExtendKind::UXTB,
            lsl_amount: ShiftAmount(0),
        }
    }
}
impl<const BITS: u8, const ALIGN: u8> Zero for LabelSImm<BITS, ALIGN> {
    fn zero() -> Self {
        Self(None)
    }
}
impl<const BITS: u8, const ALIGN: u8> Zero for LabelUImm<BITS, ALIGN> {
    fn zero() -> Self {
        Self(None)
    }
}

impl Size {
    pub fn bits(self) -> u8 {
        (self as u8) * 8u8
    }
    pub fn get(self) -> u8 {
        self as u8
    }
}

impl<const BITS: u8, const ALIGN: u8> Resolvable for LabelSImm<BITS, ALIGN> {
    type Key = Spur;
    type Value = u64;
    type Result = SImm<BITS, ALIGN>;
    fn value_to_op(value: Self::Value) -> self::Ops {
        Ops::Imm(value as i64)
    }
}

impl<T: Operand + Zero> Resolvable for Label<T> {
    type Key = Spur;
    type Value = u64;
    type Result = T;
    fn value_to_op(value: Self::Value) -> self::Ops {
        Ops::Imm(value as i64)
    }
}

impl IdxVr {
    pub fn new(reg: Reg, size: VecSize, index: u8) -> Option<Self> {
        if !Self::index_in_range(index, size) {
            return None;
        }
        Some(Self { reg, size, index })
    }
    pub const fn index_in_range(index: u8, size: VecSize) -> bool {
        index < ((Size::B16 as u8) / (size as u8))
    }
}
