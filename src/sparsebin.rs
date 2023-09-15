use bit::{BitCt, CreateMask};
use rustc_hash::FxHashMap as HashMap;
use std::{
    cmp::{max, min},
    io::BufRead,
    iter::FusedIterator,
    marker::PhantomData,
    ops::{Index, IndexMut, Range},
};

const PAGE_BITS: u32 = 12;
const PAGE_SIZE: usize = 2usize.pow(PAGE_BITS);

type PageData = [u8; PAGE_SIZE];

const ZERO_PAGE: Page = Page([0; PAGE_SIZE]);
/// must never mutate
unsafe fn zero_page_mut_ptr() -> *mut Page {
    &ZERO_PAGE as *const Page as *mut Page
}

// TODO: use MaybeUninit and a length field to avoid zeroing the page
#[repr(align(8))]
pub struct Page(PageData);

impl Index<usize> for Page {
    type Output = u8;
    fn index(&self, addr: usize) -> &Self::Output {
        unsafe { self.0.get_unchecked(extract_lo::<PAGE_BITS>(addr)) }
    }
}

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

struct SA {
    index: usize,
    offset: usize,
}
fn split_addr<T: Sized, const ALIGN: BitCt>(addr: usize) -> SA {
    if ALIGN > 0 {
        // comptime known
        debug_assert!(2usize.pow(ALIGN) == std::mem::align_of::<T>());
        debug_assert!(ALIGN <= 3, "ALIGN must be <= alignment of 8 bytes (3)");
        // `Page` is guaranteed to be aligned to 8 bytes
        debug_assert!(addr & usize::mask_lo_1s(ALIGN) == 0);
    }

    SA {
        index: addr >> PAGE_BITS,
        offset: extract_lo::<ALIGN>(addr),
    }
}

impl SparseBin {
    pub fn new() -> Self {
        Self {
            pages: HashMap::default(),
            active_page: unsafe { zero_page_mut_ptr() },
            active_idx: usize::MAX,
        }
    }

    fn set_active_page(&mut self, index: usize, page: &mut Page) {
        debug_assert!(index != usize::MAX);
        self.active_idx = index;
        self.active_page = page as *mut Page;
    }
    fn invalidate_active_page(&mut self) {
        self.active_idx = usize::MAX;
        self.active_page = unsafe { zero_page_mut_ptr() };
    }

    /// # Safety
    /// `self.active_idx != usize::MAX`
    unsafe fn active_page_mut(&mut self) -> &mut PageData {
        debug_assert!(self.active_idx != usize::MAX);
        &mut (*self.active_page).0
    }
    pub fn active_page(&self) -> &PageData {
        unsafe { &(*self.active_page).0 }
    }

    // TODO: move to PageIter
    fn page_or_zero(&self, index: usize) -> &PageData {
        match self.pages.get(&index) {
            Some(page) => &page.0,
            None => &ZERO_PAGE.0,
        }
    }

    // TODO: move to PageIter
    fn page_unaligned_first(&self, addr: usize) -> (usize, &[u8]) {
        let index = addr >> PAGE_BITS;
        let offset = extract_lo::<PAGE_BITS>(addr);
        if offset > 0 {
            let page = self.page_or_zero(index);
            // first index is the returned page, add one
            unsafe { (index + 1, page.get_unchecked(offset..)) }
        } else {
            // first index inclusive
            (index, &ZERO_PAGE.0[0..0])
        }
    }

    // TODO: move to PageIter
    fn page_unaligned_last(&self, addr: usize) -> (usize, &[u8]) {
        let index = addr >> PAGE_BITS;
        let offset = extract_lo::<PAGE_BITS>(addr);
        if offset > 0 {
            let page = self.page_or_zero(index);
            // last index exclusive, doesn't need to be modified
            unsafe { (index, page.get_unchecked(..offset)) }
        } else {
            // last index exclusive
            (index, &ZERO_PAGE.0[0..0])
        }
    }

    pub fn page_range_u8(&self, range: Range<usize>) -> (&[u8], PageRangeIter<'_>, &[u8]) {
        let Range { start, end } = range;
        assert!(start <= end);

        let (start_idx, first) = self.page_unaligned_first(start);
        let (end_idx, last) = self.page_unaligned_last(end);

        (first, PageRangeIter::new(self, start_idx, end_idx), last)
    }

    fn page_entry_mut(&mut self, index: usize) -> &mut PageData {
        if self.active_idx == index {
            unsafe { self.active_page_mut() }
        } else {
            let page = self.pages.entry(index).or_default();
            self.active_page = page as *mut Page;
            self.active_idx = index;
            unsafe { self.active_page_mut() }
        }
    }

    fn try_page_entry(&mut self, index: usize) -> Option<&PageData> {
        if self.active_idx == index {
            Some(self.active_page())
        } else {
            self.pages.get(&index).map(|p| &p.0)
        }
    }

    unsafe fn write<T: Sized, const ALIGN: u32>(&mut self, addr: usize, value: T) {
        let SA { index, offset } = split_addr::<T, ALIGN>(addr);
        let page = self.page_entry_mut(index);
        let ptr = page.as_mut_ptr().add(extract_lo::<PAGE_BITS>(addr));
        let ptr_t = ptr.cast::<T>();

        debug_assert!(ptr_t.is_aligned());
        ptr_t.write(value);
    }

    unsafe fn write_slice<T: Sized, const ALIGN: u32>(&mut self, addr: usize, slice: &[T]) {
        let slice_size = std::mem::size_of_val(slice);
        todo!()
    }

    /// get random access
    /// won't change the active page
    unsafe fn get_ra<T: Sized + Copy, const ALIGN: u32>(&self, addr: usize) -> T {
        todo!()
    }
    /// get sequential
    /// can change the active page
    unsafe fn get_seq<T: Sized + Copy, const ALIGN: u32>(&mut self, addr: usize) -> T {
        todo!()
    }

    /// get if page exists 
    unsafe fn try_get<T: Sized + Copy, const ALIGN: u32>(&mut self, addr: usize) -> Option<T> {
        todo!()
    }

    /// get sequential
    unsafe fn get<T: Sized + Copy, const ALIGN: u32>(&mut self, addr: usize) -> T {
        let SA { index, offset } = split_addr::<T, ALIGN>(addr);
        let page = self.page_entry_mut(addr);
        let ptr = page.as_ptr().add(extract_lo::<PAGE_BITS>(addr));
        let ptr_t = ptr.cast::<T>();

        debug_assert!(ptr_t.is_aligned());
        ptr_t.read()
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

    pub fn get_u8(&mut self, addr: usize) -> u8 {
        unsafe { self.get::<u8, 0>(addr) }
    }

    pub fn get_u32(&mut self, addr: Aligned<4>) -> u32 {
        unsafe { self.get::<u32, 2>(addr.0) }
    }

    pub fn get_u64(&mut self, addr: Aligned<8>) -> u64 {
        unsafe { self.get::<u64, 3>(addr.0) }
    }
}

pub struct PageRangeIter<'a> {
    pages: &'a HashMap<usize, Page>,
    idx: usize,
    end_idx: usize,
}

impl<'a> PageRangeIter<'a> {
    fn new(bin: &'a SparseBin, start_idx: usize, end_idx: usize) -> Self {
        debug_assert!(start_idx <= end_idx);
        Self {
            pages: &bin.pages,
            idx: start_idx,
            end_idx,
        }
    }
}
impl<'a> Iterator for PageRangeIter<'a> {
    type Item = &'a [u8; PAGE_SIZE];
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.end_idx {
            None
        } else {
            let page = match self.pages.get(&self.idx) {
                Some(page) => &page.0,
                None => &ZERO_PAGE.0,
            };
            self.idx += 1;
            Some(page)
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}
impl ExactSizeIterator for PageRangeIter<'_> {
    fn len(&self) -> usize {
        self.end_idx - self.idx
    }
}
impl FusedIterator for PageRangeIter<'_> {}

/// `value == ((value >> N)) << N) | extract_lo::<N>(value))`
fn extract_lo<const N: u32>(value: usize) -> usize {
    value & usize::mask_lo_1s(N)
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
        assert_eq!(bin.get_u8(0), 0x12);
        assert_eq!(bin.get_u8(1), 0x34);
        assert_eq!(bin.get_u8(2), 0x56);
        assert_eq!(bin.get_u8(3), 0x78);
    }

    #[test]
    fn it_write_sparse_u8() {
        let mut bin = SparseBin::new();
        bin.write_u8(0, 0x23);
        bin.write_u8(0x100000000, 0x4);
        bin.write_u8(0x100000001, 0x84);
        bin.write_u8(1, 0x11);
        assert_eq!(bin.get_u8(0), 0x23);
        assert_eq!(bin.get_u8(0x100000000), 0x4);
        assert_eq!(bin.get_u8(0x100000001), 0x84);
        assert_eq!(bin.get_u8(1), 0x11);
    }

    #[test]
    fn it_write_sparse_u64() {
        let mut bin = SparseBin::new();
        bin.write_u64(Aligned::new(0).unwrap(), 0x1234567890abcdef);
        bin.write_u64(Aligned::new(8).unwrap(), 0xD800);
        bin.write_u64(Aligned::new(0x100000000).unwrap(), 0xabd18238);
        bin.write_u64(Aligned::new(0x100000010).unwrap(), 0x111111190abcdef);
        assert_eq!(bin.get_u64(Aligned::new(0).unwrap()), 0x1234567890abcdef);
        assert_eq!(bin.get_u64(Aligned::new(0x100000000).unwrap()), 0xabd18238);
        assert_eq!(bin.get_u64(Aligned::new(8).unwrap()), 0xD800);
        assert_eq!(
            bin.get_u64(Aligned::new(0x100000010).unwrap()),
            0x111111190abcdef
        );
    }

    #[test]
    fn page_range_iter() {
        let bin = SparseBin::new();
        let (first, mut iter, last) = bin.page_range_u8(0..0);
        assert_eq!(first.len(), 0);
        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next(), None);
        assert_eq!(last.len(), 0);
    }
    #[test]
    fn page_range_iter_len() {
        let bin = SparseBin::new();
        let start = 10 << PAGE_BITS;
        let end = 20 << PAGE_BITS;
        let (first, iter, last) = bin.page_range_u8(start..end);
        assert_eq!(first.len(), 0);
        assert_eq!(iter.len(), 10);
        assert_eq!(last.len(), 0);
    }
    #[test]
    fn page_range_iter_next() {
        let bin = SparseBin::new();
        let start = 1 << PAGE_BITS;
        let end = 2 << PAGE_BITS;
        let (first, mut iter, last) = bin.page_range_u8(start..end);
        assert_eq!(first.len(), 0);
        assert_eq!(iter.len(), 1);
        assert!(iter.next().is_some());
        assert!(iter.next().is_none());
        assert_eq!(last.len(), 0);
    }
}
