use rustc_hash::FxHashMap as HashMap;

const PAGE_BITS: u32 = 12;
const PAGE_SIZE: usize = 2usize.pow(PAGE_BITS);

#[repr(align(8))]
struct Page([u8; PAGE_SIZE]);

/// int with ALIGN number of low bits set to 0
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Aligned<const ALIGN: usize>(usize);

impl<const ALIGN: usize> Aligned<ALIGN> {
    pub const fn new(value: usize) -> Option<Self> {
        if value & (ALIGN - 1) == 0 {
            Some(Self(value))
        } else {
            None
        }
    }

    pub const unsafe fn new_unchecked(value: usize) -> Self {
        debug_assert!(value & (ALIGN - 1) == 0);
        Self(value)
    }
}

pub fn foo() {
    let _ = Aligned::<8>(0);
}

impl Default for Page {
    #[inline]
    fn default() -> Self {
        Self([0; PAGE_SIZE])
    }
}

pub struct SparseBin {
    pages: HashMap<usize, Page>,
    active_page: *mut Page,
    active_idx: usize,
}

impl SparseBin {
    pub fn new() -> Self {
        Self {
            pages: HashMap::default(),
            active_page: std::ptr::null_mut(),
            active_idx: usize::MAX,
        }
    }

    fn page_mut(&mut self, index: usize) -> &mut Page {
        if !self.active_page.is_null() && self.active_idx == index {
            unsafe { &mut *self.active_page }
        } else {
            let page = self.pages.entry(index).or_default();
            self.active_page = page as *mut Page;
            self.active_idx = index;
            page
        }
    }

    fn page(&self, index: usize) -> Option<&Page> {
        if !self.active_page.is_null() && self.active_idx == index {
            unsafe { Some(&*self.active_page) }
        } else {
            self.pages.get(&index)
        }
    }

    unsafe fn write<T: Sized, const ALIGN: u32>(&mut self, addr: usize, value: T) {
        debug_assert!(2usize.pow(ALIGN) == std::mem::align_of::<T>()); // comptime known
        debug_assert!(ALIGN <= 3, "ALIGN must be <= alignment of 8 bytes (3)"); // comptime known

        // `Page` is guaranteed to be aligned to 8 bytes
        debug_assert!(addr & mask_lo::<ALIGN>() == 0);

        // `PAGE_ALIGN` bits of `addr` are used as index to `Page` and guaranteed to be in range
        let page = self.page_mut(addr >> PAGE_BITS);
        let ptr = page.0.as_mut_ptr().add(extract_lo::<PAGE_BITS>(addr));
        let ptr_t = ptr.cast::<T>();

        debug_assert!(ptr_t.is_aligned());
        ptr_t.write(value);
    }

    unsafe fn write_slice<T: Sized, const ALIGN: u32>(&mut self, addr: usize, slice: &[T]) {
        let slice_size = std::mem::size_of_val(slice);
        todo!()
    }

    unsafe fn get<T: Sized + Copy, const ALIGN: u32>(&self, addr: usize) -> Option<T> {
        debug_assert!(2usize.pow(ALIGN) == std::mem::align_of::<T>()); // comptime known
        debug_assert!(ALIGN <= 3, "ALIGN must be <= alignment of 8 bytes (3)"); // comptime known

        // `Page` is guaranteed to be aligned to 8 bytes
        debug_assert!(addr & mask_lo::<ALIGN>() == 0);

        // `PAGE_ALIGN` bits of `addr` are used as index to `Page` and guaranteed to be in range
        let page = self.page(addr >> PAGE_BITS)?;
        let ptr = page.0.as_ptr().add(extract_lo::<PAGE_BITS>(addr));
        let ptr_t = ptr.cast::<T>();

        debug_assert!(ptr_t.is_aligned());
        Some(ptr_t.read())
    }

    pub fn write_slice_u8(&mut self, addr: usize, slice: &[u8]) {
        let mut addr = addr;
        for &value in slice {
            self.write_u8(addr, value);
            addr += 1;
        }
    }

    pub fn write_u8(&mut self, addr: usize, value: u8) {
        unsafe { self.write::<u8, 0>(addr, value) };
    }

    pub fn write_u32(&mut self, addr: Aligned<4>, value: u32) {
        unsafe { self.write::<u32, 2>(addr.0, value) };
    }

    pub fn write_u64(&mut self, addr: Aligned<8>, value: u64) {
        unsafe { self.write::<u64, 3>(addr.0, value) };
    }

    pub fn get_u8(&self, addr: usize) -> Option<u8> {
        unsafe { self.get::<u8, 0>(addr) }
    }

    pub fn get_u32(&self, addr: Aligned<4>) -> Option<u32> {
        unsafe { self.get::<u32, 2>(addr.0) }
    }

    pub fn get_u64(&self, addr: Aligned<8>) -> Option<u64> {
        unsafe { self.get::<u64, 3>(addr.0) }
    }
}

/// a mask with low N bits set to 1 and the rest 0
const fn mask_lo<const N: u32>() -> usize {
    (1 << N) - 1
}
/// a mask with low N bits set to 0 and the rest 1
const fn not_mask_lo<const N: u32>() -> usize {
    !mask_lo::<N>()
}
/// a mask with high N bits set to 1 and the rest 0
const fn mask_hi<const N: u32>() -> usize {
    usize::MAX >> N
}

/// `value == ((value >> N)) << N) | extract_lo::<N>(value))`
const fn extract_lo<const N: u32>(value: usize) -> usize {
    value & mask_lo::<N>()
}

const fn log2(n: u64) -> u32 {
    assert!(n.is_power_of_two());
    (u64::BITS - 1) - n.leading_zeros()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_write_u8() {
        let mut bin = SparseBin::new();
        bin.write_u8(0, 0x12);
        bin.write_u8(1, 0x34);
        bin.write_u8(2, 0x56);
        bin.write_u8(3, 0x78);
        assert_eq!(bin.get_u8(0), Some(0x12));
        assert_eq!(bin.get_u8(1), Some(0x34));
        assert_eq!(bin.get_u8(2), Some(0x56));
        assert_eq!(bin.get_u8(3), Some(0x78));
    }

    #[test]
    fn it_write_sparse_u8() {
        let mut bin = SparseBin::new();
        bin.write_u8(0, 0x23);
        bin.write_u8(0x100000000, 0x4);
        bin.write_u8(0x100000001, 0x84);
        bin.write_u8(1, 0x11);
        assert_eq!(bin.get_u8(0), Some(0x23));
        assert_eq!(bin.get_u8(0x100000000), Some(0x4));
        assert_eq!(bin.get_u8(0x100000001), Some(0x84));
        assert_eq!(bin.get_u8(1), Some(0x11));
    }

    #[test]
    fn it_write_sparse_u64() {
        let mut bin = SparseBin::new();
        bin.write_u64(Aligned::new(0).unwrap(), 0x1234567890abcdef);
        bin.write_u64(Aligned::new(8).unwrap(), 0xD800);
        bin.write_u64(Aligned::new(0x100000000).unwrap(), 0xabd18238);
        bin.write_u64(Aligned::new(0x100000010).unwrap(), 0x111111190abcdef);
        assert_eq!(
            bin.get_u64(Aligned::new(0).unwrap()),
            Some(0x1234567890abcdef)
        );
        assert_eq!(
            bin.get_u64(Aligned::new(0x100000000).unwrap()),
            Some(0xabd18238)
        );
        assert_eq!(bin.get_u64(Aligned::new(8).unwrap()), Some(0xD800));
        assert_eq!(
            bin.get_u64(Aligned::new(0x100000010).unwrap()),
            Some(0x111111190abcdef)
        );
    }
}
