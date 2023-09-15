use bit::{BitCt, CreateMask};
use std::marker::PhantomData;

/// byte count of alignment to number of zero low bits
#[repr(u32)]
pub enum AlignBitCt {
    B1 = 0,
    B2 = 1,
    B4 = 2,
    B8 = 3,
    B16 = 4,
}

pub struct Addr<const ALIGN: BitCt = 0>(usize);

impl<const ALIGN: BitCt> Addr<ALIGN> {
    pub fn new(addr: usize) -> Self {
        assert!(addr & usize::mask_lo_1s(ALIGN) == 0);
        Self(addr)
    }
    pub fn try_new(addr: usize) -> Option<Self> {
        todo!()
    }
    pub fn new_unchecked(addr: usize) -> Self {
        debug_assert!(addr & usize::mask_lo_1s(ALIGN) == 0);
        Self(addr)
    }
}

pub macro Addr {
    ($byte_count:ident) => {
        $crate::addr::Addr<
            $crate::addr::AlignBitCt:: $byte_count
                as $crate::bit::BitCt>
    }
}
