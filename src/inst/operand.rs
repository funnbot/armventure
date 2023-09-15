use super::{
    create_fixup, label,
    util::{MaybeDisplay, OptionDisplay},
    Emitter, Error,
};
use crate::{enum_str::EnumStr, simpl, simpls};
use bit::{
    sint_in_range, sint_low_zeros, uint_in_range, uint_low_zeros, Align, BitCt, Int, IntN,
    IntOfBits,
};
use enum_variant_type::EnumVariantType;
use numb::int::U5;
use std::marker::PhantomData;
use subenum::subenum;

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
    Error,
}

#[derive(EnumVariantType, Debug, Clone)]
#[evt(module = "operands", derive(Debug, Clone))]
pub enum Ops {
    Gpr {
        reg: GprKind,
        size: GprSize,
    },
    Dpr {
        reg: u8,
        size: Size,
    },
    Imm(i64),
    Label(label::Key),
    Shift {
        kind: ShiftKind,
        amount: u8,
    },
    Extend {
        kind: ExtendKind,
        left_shift_amount: Option<u8>,
    },
    Cond(CondKind),
    Error,
}
pub use operands as op;

impl Ops {
    pub fn kind(&self) -> Kind {
        match self {
            Ops::Gpr { .. } => Kind::Gpr,
            Ops::Dpr { .. } => Kind::Dpr,
            Ops::Imm(..) => Kind::Imm,
            Ops::Label(..) => Kind::Label,
            Ops::Shift { .. } => Kind::Shift,
            Ops::Extend { .. } => Kind::Extend,
            Ops::Cond { .. } => Kind::Cond,
            Ops::Error => Kind::Error,
        }
    }
}

/// size in bytes
#[subenum(GprSize, VecSize, VecLanes)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
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

#[derive(Debug, Clone, Copy)]
pub enum GprKind {
    R(U5),
    SP,
    ZR,
}

crate::enum_str! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(u8)]
    pub enum ShiftKind {
        LSL = 0b00, // logical shift left
        LSR = 0b01, // logical shift right (zeros)
        ASR = 0b10, // arithmetic shift right (maintain sign)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ShiftAmount<const N: BitCt>(pub BitCt);

crate::enum_str! {
    #[derive(PartialEq, Eq, Clone, Copy, Debug)]
    #[repr(u8)]
    pub enum ExtendKind {
        /// zero extend to 1 byte
        UXTB = 0b000,
        /// zero extend to 2 bytes
        UXTH = 0b001,
        /// zero extend to 4 bytes, or LSL
        UXTW = 0b010,
        /// zero extend to 8 bytes
        UXTX = 0b011,
        /// sign extend to 1 byte
        SXTB = 0b100,
        /// sign extend to 2 bytes
        SXTH = 0b101,
        /// sign extend to 4 bytes
        SXTW = 0b110,
        /// sign extend to 8 bytes
        SXTX = 0b111,
    }
}

impl TryFrom<u8> for ExtendKind {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value <= 0b111 {
            Ok(unsafe { std::mem::transmute::<u8, ExtendKind>(value) })
        } else {
            Err(())
        }
    }
}

crate::enum_str! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(u8)]
    pub enum CondKind {
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

impl TryFrom<u8> for CondKind {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value <= 0b1111 {
            Ok(unsafe { std::mem::transmute::<u8, CondKind>(value) })
        } else {
            Err(())
        }
    }
}

//     // data processing register, aka scalar register
//     pub struct Dpr {
//         pub reg: Reg,
//         pub size: Size,
//     }

//     // simd vector register
//     pub struct Vr {
//         pub reg: Reg,
//         pub size: VecSize,
//         pub lanes: VecLanes,
//     }

//     // index scalar in vector register
//     pub struct IdxVr {
//         pub reg: Reg,
//         pub size: VecSize,
//         // index infers the lanes
//         pub index: u8,
//     }

pub trait Encoder<T> {
    type Int: IntOfBits<u32>;
    fn encode<E: Emitter>(v: &T, e: &mut E) -> Result<Self::Int, Error>;
}

trait EncoderOpt<T, EC: Encoder<T>>
where
    Self: Encoder<Option<T>, Int = EC::Int>,
{
    fn opt<E: Emitter>(v: &Option<T>, e: &mut E, def: EC::Int) -> Result<EC::Int, Error> {
        match v {
            Some(v) => EC::encode(v, e),
            None => Ok(def),
        }
    }
}
impl<T, EC: Encoder<T>> EncoderOpt<T, EC> for EC where Self: Encoder<Option<T>, Int = EC::Int> {}
// impl<T, EC: Encoder<T>> Encoder<Option<T>> for EC {
//     type Int = EC::Int;
//     fn encode<E: Emitter>(v: &Option<T>, e: &mut E) -> Result<Self::Int, Error> {
//         Self::opt(v, e, EC::Int::new(0))
//     }
// }

pub mod enc {
    use super::{BitCt, PhantomData};

    pub struct Gpr;
    pub struct GprOrSp;
    pub struct GprOrZr;
    pub struct SImm<const BITS: BitCt>;
    pub struct UImm<const BITS: BitCt>;
    pub struct ShiftKind;
    pub struct ShiftAmount<const BITS: BitCt>;
    pub struct ShiftConst<const KIND: u8, const AMT: u8>;
    pub struct ExtendKind;
    pub struct ExtendLShift;
    pub struct Sf;
    /// signed immediate that is right shifted by `RS` and then stored in `BITS` bits
    pub struct SImmAlign<const BITS: BitCt, const RS: BitCt>;
    pub struct UImmAlign<const BITS: BitCt, const RS: BitCt>;
    pub struct Label<EC>(pub PhantomData<EC>);
    pub struct Cond;
}

fn fixup_label_fn<E: Emitter, EC: Encoder<Option<op::Imm>>>(
    label_addr: u64,
    e: &mut E,
) -> Result<IntN, Error> {
    let current_instr_addr = e.pc() as i64;
    let offset = label_addr as i64 - current_instr_addr;
    let value = EC::encode(&Some(op::Imm(offset)), e);
    let n = <EC::Int as IntOfBits<u32>>::N as u8;
    value.map(|v| IntN(v.get(), n))
}

impl Encoder<op::Gpr> for enc::Gpr {
    type Int = Int<5>;
    fn encode<E: Emitter>(v: &op::Gpr, _: &mut E) -> Result<Self::Int, Error> {
        match v.reg {
            GprKind::R(idx) => {
                let v: u8 = idx.into();
                assert!(v <= 30);
                Ok(Int(v as u32))
            }
            GprKind::ZR | GprKind::SP => Err(Error::InvalidGpr),
        }
    }
}
impl Encoder<op::Gpr> for enc::GprOrSp {
    type Int = Int<5>;
    fn encode<E: Emitter>(v: &op::Gpr, _: &mut E) -> Result<Self::Int, Error> {
        match v.reg {
            GprKind::R(idx) => {
                let v: u8 = idx.into();
                assert!(v <= 30);
                Ok(Int(v as u32))
            }
            GprKind::SP => Ok(Int(31)),
            GprKind::ZR => Err(Error::InvalidGpr),
        }
    }
}
impl Encoder<op::Gpr> for enc::GprOrZr {
    type Int = Int<5>;
    fn encode<E: Emitter>(v: &op::Gpr, _: &mut E) -> Result<Self::Int, Error> {
        match v.reg {
            GprKind::R(idx) => {
                let v: u8 = idx.into();
                assert!(v <= 30);
                Ok(Int(v as u32))
            }
            GprKind::ZR => Ok(Int(31)),
            GprKind::SP => Err(Error::InvalidGpr),
        }
    }
}
impl Encoder<op::Gpr> for enc::Sf {
    type Int = Int<1>;
    fn encode<E: Emitter>(v: &op::Gpr, _: &mut E) -> Result<Self::Int, Error> {
        Ok(Int((v.size == GprSize::B8) as u32))
    }
}
impl Encoder<op::Shift> for enc::ShiftKind {
    type Int = Int<2>;
    fn encode<E: Emitter>(v: &op::Shift, _: &mut E) -> Result<Self::Int, Error> {
        Ok(Int(v.kind as u32))
    }
}
impl<const BITS: BitCt> Encoder<op::Shift> for enc::ShiftAmount<BITS> {
    type Int = Int<BITS>;
    fn encode<E: Emitter>(v: &op::Shift, _: &mut E) -> Result<Self::Int, Error> {
        debug_assert!(BITS <= 32);
        Ok(Int(v.amount as u32))
    }
}
impl<EC: Encoder<Option<op::Imm>>> Encoder<op::Label> for enc::Label<EC> {
    type Int = EC::Int;
    fn encode<E: Emitter>(v: &op::Label, e: &mut E) -> Result<Self::Int, Error> {
        match e.resolve_label(v.0) {
            Some(label_addr) => fixup_label_fn::<E, EC>(label_addr, e).map(|v| Self::Int::new(v.0)),
            None => {
                let fixup = create_fixup(e, v.0, fixup_label_fn::<E, EC>);
                e.push_label_fixup(fixup);
                EC::encode(&None, e)
            }
        }
    }
}
impl<const BITS: BitCt> Encoder<i64> for enc::SImm<BITS> {
    type Int = Int<BITS>;
    fn encode<E: Emitter>(v: &i64, _: &mut E) -> Result<Self::Int, Error> {
        if sint_in_range::<BITS>(*v) {
            Ok(Int(*v as u32))
        } else {
            Err(Error::OutOfRange)
        }
    }
}
impl<const BITS: BitCt> Encoder<op::Imm> for enc::SImm<BITS> {
    type Int = Int<BITS>;
    fn encode<E: Emitter>(v: &op::Imm, e: &mut E) -> Result<Self::Int, Error> {
        Self::encode(&v.0, e)
    }
}
impl<const RS: BitCt, const BITS: BitCt> Encoder<op::Imm> for enc::SImmAlign<BITS, RS> {
    type Int = Int<BITS>;
    fn encode<E: Emitter>(v: &op::Imm, e: &mut E) -> Result<Self::Int, Error> {
        if sint_low_zeros::<RS>(v.0) {
            let shifted = v.0 >> RS;
            enc::SImm::<BITS>::encode(&shifted, e)
        } else {
            Err(Error::NotAligned)
        }
    }
}

impl<const BITS: BitCt> Encoder<u64> for enc::UImm<BITS> {
    type Int = Int<BITS>;
    fn encode<E: Emitter>(v: &u64, _: &mut E) -> Result<Self::Int, Error> {
        if uint_in_range::<BITS>(*v) {
            Ok(Int(*v as u32))
        } else {
            Err(Error::OutOfRange)
        }
    }
}
impl<const BITS: BitCt> Encoder<op::Imm> for enc::UImm<BITS> {
    type Int = Int<BITS>;
    fn encode<E: Emitter>(v: &op::Imm, e: &mut E) -> Result<Self::Int, Error> {
        let uval = v.0 as u64;
        Self::encode(&uval, e)
    }
}
impl<const RS: BitCt, const BITS: BitCt> Encoder<op::Imm> for enc::UImmAlign<BITS, RS> {
    type Int = Int<BITS>;
    fn encode<E: Emitter>(v: &op::Imm, e: &mut E) -> Result<Self::Int, Error> {
        let uval = v.0 as u64;
        if uint_low_zeros::<RS>(uval) {
            let shifted = uval >> RS;
            enc::UImm::<BITS>::encode(&shifted, e)
        } else {
            Err(Error::NotAligned)
        }
    }
}

impl<const KIND: u8, const AMT: u8> Encoder<Option<op::Shift>> for enc::ShiftConst<KIND, AMT> {
    type Int = Int<1>;
    fn encode<E: Emitter>(v: &Option<op::Shift>, _: &mut E) -> Result<Self::Int, Error> {
        match v {
            Some(v) => {
                if v.kind as u8 == KIND && v.amount == AMT {
                    Ok(Int(1))
                } else {
                    Err(Error::MismatchedConstShift)
                }
            }
            None => Ok(Int(0)),
        }
    }
}
impl Encoder<op::Extend> for enc::ExtendKind {
    type Int = Int<3>;
    fn encode<E: Emitter>(v: &op::Extend, _: &mut E) -> Result<Self::Int, Error> {
        Ok(Int(v.kind as u32))
    }
}
impl Encoder<op::Extend> for enc::ExtendLShift {
    type Int = Int<3>;
    fn encode<E: Emitter>(v: &op::Extend, _: &mut E) -> Result<Self::Int, Error> {
        Ok(Int(v.left_shift_amount.unwrap_or(0) as u32))
    }
}
impl Encoder<op::Cond> for enc::Cond {
    type Int = Int<4>;
    fn encode<E: Emitter>(v: &op::Cond, _: &mut E) -> Result<Self::Int, Error> {
        Ok(Int(v.0 as u32))
    }
}

impl Encoder<Option<op::Shift>> for enc::ShiftKind {
    type Int = Int<2>;
    fn encode<E: Emitter>(v: &Option<op::Shift>, e: &mut E) -> Result<Self::Int, Error> {
        Self::opt(v, e, Int(0))
    }
}
impl<const BITS: BitCt> Encoder<Option<op::Shift>> for enc::ShiftAmount<BITS> {
    type Int = Int<BITS>;
    fn encode<E: Emitter>(v: &Option<op::Shift>, e: &mut E) -> Result<Self::Int, Error> {
        Self::opt(v, e, Int(0))
    }
}
impl<const BITS: BitCt> Encoder<Option<op::Imm>> for enc::UImm<BITS> {
    type Int = Int<BITS>;
    fn encode<E: Emitter>(v: &Option<op::Imm>, e: &mut E) -> Result<Self::Int, Error> {
        Self::opt(v, e, Int(0))
    }
}
impl<const RS: BitCt, const BITS: BitCt> Encoder<Option<op::Imm>> for enc::SImmAlign<BITS, RS> {
    type Int = Int<BITS>;
    fn encode<E: Emitter>(v: &Option<op::Imm>, e: &mut E) -> Result<Self::Int, Error> {
        Self::opt(v, e, Int(0))
    }
}
impl Encoder<Option<op::Extend>> for enc::ExtendKind {
    type Int = Int<3>;
    fn encode<E: Emitter>(v: &Option<op::Extend>, e: &mut E) -> Result<Self::Int, Error> {
        Self::opt(v, e, Int(0))
    }
}
impl Encoder<Option<op::Extend>> for enc::ExtendLShift {
    type Int = Int<3>;
    fn encode<E: Emitter>(v: &Option<op::Extend>, e: &mut E) -> Result<Self::Int, Error> {
        Self::opt(v, e, Int(0))
    }
}

impl enc::ExtendKind {
    pub fn valid_width(ext: &Option<op::Extend>, gpr: &op::Gpr) -> Result<(), Error> {
        let Some(ext) = ext else {
            return Ok(());
        };
        if (ext.kind == ExtendKind::SXTX || ext.kind == ExtendKind::UXTX) && gpr.size != GprSize::B8
        {
            Err(Error::InvalidExtendWidth)
        } else {
            Ok(())
        }
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

// impl IdxVr {
//     pub fn new(reg: Reg, size: VecSize, index: u8) -> Option<Self> {
//         if !Self::index_in_range(index, size) {
//             return None;
//         }
//         Some(Self { reg, size, index })
//     }
//     pub const fn index_in_range(index: u8, size: VecSize) -> bool {
//         index < ((Size::B16 as u8) / (size as u8))
//     }
// }

simpls! {
    { Default |op::Shift| op::Shift{ kind: ShiftKind::LSL, amount: 0 } }
    { Default |op::Extend| op::Extend{ kind: ExtendKind::UXTB, left_shift_amount: None } }
    { Default |op::Cond| op::Cond(CondKind::AL) }
    { Default |op::Imm| op::Imm(0) }
    { Default |op::Gpr| op::Gpr{ reg: GprKind::ZR, size: GprSize::B8 } }
    { Default |op::Dpr| op::Dpr{ reg: 0, size: Size::B8 } }

    // Display shouldn't be used for the complex formatting of instructions, they are too context dependent
    { Display |self: ExtendKind| "{}", self.to_str() }
    { Display |self: ShiftKind| "{}", self.to_str() }
    { Display [const N: BitCt] |self: ShiftAmount<N>| "{:01$}", self.0, N as usize }
    { Display |self: op::Gpr, f| match self.reg {
        GprKind::R(i) => write!(f, "{}{}", if self.size == GprSize::B8 { 'X' } else { 'W' }, u8::from(i)),
        GprKind::SP => write!(f, "{}", if self.size == GprSize::B8 { "SP" } else { "WSP" }),
        GprKind::ZR => write!(f, "{}", if self.size == GprSize::B8 { "ZR" } else { "WZR" }),
    } }
    { Display |self: op::Shift| "{} #{}", self.kind, self.amount }
    // This is wrong, UXTW is LSL when size is W, and UXTX is LSL when size is X
    // needs reference to GprSize of Rm field
    { Display |self: op::Extend, f| match self.left_shift_amount {
        Some(lsa) => match self.kind {
            ExtendKind::UXTW if lsa > 0 => write!(f, "LSL #{}", lsa),
            _ => write!(f, "{} #{}", self.kind, lsa),
        }
        None => write!(f, "{}", self.kind)
    } }
    { Display |self: op::Cond| "{}", self.0.to_str() }
    { Display |self: op::Imm| "#{}", self.0 }
    { Display |self: op::Label| "<label:{:?}>", self.0 }

    // Maybe display is unnecessary, use Debug printing instead
    // and Display will only be used for operands that aren't context dependent,
    // instruction decoding in the disassembler will be far more complicated and require more context
    // that is where the rules like MaybeDisplay can be applied
    { MaybeDisplay |self: op::Extend|
        if self.kind == ExtendKind::UXTW && self.left_shift_amount.is_none() {
            DisplayNone
        } else {
            DisplaySome(self)
        }
    }
    { MaybeDisplay |self: op::Shift|
        if self.kind == ShiftKind::LSL && self.amount == 0 {
            DisplayNone
        } else {
            DisplaySome(self)
        }
    }
}
