use core::slice::from_raw_parts_mut;
use crate::mem::page_table::PTEFlags;

pub const PAGE_WIDTH: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_WIDTH;
pub const PA_WIDTH: usize = 56;
pub const PPN_WIDTH: usize = PA_WIDTH - PAGE_WIDTH;
pub const VA_WIDTH: usize = 39;
pub const VPN_WIDTH: usize = VA_WIDTH - PAGE_WIDTH;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(pub usize);
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(pub usize);
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysPageNum(pub usize);
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtPageNum(pub usize);
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PageTableEntry {
    pub bits: usize
}

impl PageTableEntry {
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        Self {
            bits: ppn.0 << 10 | (flags.bits() as usize)
        }
    }
    pub fn empty() -> Self {
        Self {
            bits: 0
        }
    }
    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> 10 & ((1usize << PPN_WIDTH) - 1)).into()
    }
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits((self.bits & ((1 << 8) - 1)) as u8).unwrap()
    }
    pub fn is_valid(&self) -> bool {
        self.flags().contains(PTEFlags::V)
    }
    pub fn readable(&self) -> bool {
        self.flags().contains(PTEFlags::R)
    }
    pub fn writable(&self) -> bool {
        self.flags().contains(PTEFlags::W)
    }
    pub fn executable(&self) -> bool {
        self.flags().contains(PTEFlags::X)
    }
}

impl VirtAddr {
    pub fn page_offset(&self) -> usize { self.0 & (PAGE_SIZE - 1) }
    pub fn floor(&self) -> VirtPageNum { VirtPageNum(self.0 >> PAGE_WIDTH) }
    pub fn ceil(&self) -> VirtPageNum { VirtPageNum((self.0 + PAGE_SIZE - 1) >> PAGE_WIDTH) }
}

impl VirtPageNum {
    pub fn indexes(&self) -> [usize; 3] {
        let mut vpn = self.0;
        let mut res = [0usize; 3];
        for i in (0..3).rev() {
            res[i] = vpn & 511;
            vpn >>= 9;
        }
        res
    }
}

impl PhysAddr {
    pub fn page_offset(&self) -> usize { self.0 & (PAGE_SIZE - 1) }
    pub fn floor(&self) -> PhysPageNum { PhysPageNum(self.0 >> PAGE_WIDTH) }
    pub fn ceil(&self) -> PhysPageNum { PhysPageNum((self.0 + PAGE_SIZE - 1) >> PAGE_WIDTH) }
}

impl PhysPageNum {
    pub fn get_pte_array(&self) -> &'static mut [PageTableEntry] {
        let addr: PhysAddr = (*self).into();
        unsafe {
            from_raw_parts_mut(addr.0 as *mut PageTableEntry, 512)
        }
    }
    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let addr: PhysAddr = (*self).into();
        unsafe {
            from_raw_parts_mut(addr.0 as *mut u8, PAGE_SIZE)
        }
    }
    pub fn get_mut<T>(&self) -> &'static mut T {
        let addr: PhysAddr = (*self).into();
        unsafe {
            (addr.0 as *mut T).as_mut().unwrap()
        }
    }
}

impl From<usize> for VirtAddr {
    fn from(val: usize) -> Self {
        Self(val & ((1 << VA_WIDTH) - 1))
    }
}

impl From<VirtPageNum> for VirtAddr {
    fn from(val: VirtPageNum) -> Self { Self(val.0 << PAGE_WIDTH) }
}

impl From<VirtAddr> for VirtPageNum {
    fn from(val: VirtAddr) -> Self { 
        assert_eq!(val.page_offset(), 0); 
        val.floor()
    }
}

impl From<VirtAddr> for usize {
    fn from(val: VirtAddr) -> Self { val.0 }
}

impl From<usize> for PhysAddr {
    fn from(val: usize) -> Self {
        Self(val & ((1 << PA_WIDTH) - 1))
    }
}

impl From<PhysPageNum> for PhysAddr {
    fn from(val: PhysPageNum) -> Self { Self(val.0 << PAGE_WIDTH) }
}

impl From<usize> for PhysPageNum {
    fn from(val: usize) -> Self {
        Self(val & ((1 << PPN_WIDTH) - 1))
    }
}

impl From<PhysAddr> for PhysPageNum {
    fn from(val: PhysAddr) -> Self {
        assert_eq!(val.page_offset(), 0);
        val.floor()
    }
}

impl From<PhysAddr> for usize {
    fn from(val: PhysAddr) -> Self { val.0 }
}

impl From<PhysPageNum> for usize {
    fn from(val: PhysPageNum) -> Self { val.0 }
}

pub trait StepByOne {
    fn step(&mut self);
}

impl StepByOne for VirtPageNum {
    fn step(&mut self) {
        self.0 += 1;
    }
}

#[derive(Copy, Clone)]
pub struct SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd
{
    l: T,
    r: T
}

impl<T> SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd
{
    pub fn new(start: T, end: T) -> Self {
        assert!(start <= end);
        Self {
            l: start,
            r: end
        }
    }
    pub fn start(&self) -> T { self.l }
    pub fn end(&self) -> T { self.r }
}

impl<T> IntoIterator for SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd
{
    type Item = T;
    type IntoIter = SimpleRangeIterator<T>;
    fn into_iter(self) -> Self::IntoIter {
        SimpleRangeIterator::new(self.l, self.r)
    }
}

pub struct SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd
{
    cur: T,
    end: T
}

impl<T> SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd
{
    pub fn new(start: T, end: T) -> Self {
        Self {
            cur: start,
            end
        }
    }
}

impl<T> Iterator for SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.cur == self.end {
            None
        } else {
            let t = self.cur;
            self.cur.step();
            Some(t)
        }
    }
}

pub type VPNRange = SimpleRange<VirtPageNum>;