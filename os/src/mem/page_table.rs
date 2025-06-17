use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use bitflags::bitflags;
use crate::mem::address::{PageTableEntry, PhysAddr, PhysPageNum, StepByOne, VirtAddr, VirtPageNum, PPN_WIDTH};
use crate::mem::frame_allocator::{frame_alloc, FrameTracker};
use crate::println;

bitflags! {
    pub struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

pub struct PageTable {
    root_ppn: PhysPageNum,
    frames: Vec<FrameTracker>
}

impl PageTable {
    pub fn print(&self) {
        let table1 = self.root_ppn.get_pte_array();
        for i in 0..512 {
            let entry1 = table1[i];
            if entry1.is_valid() {
                println!("Level 1 page: virt = {:#x}, phys = {:#x}", i, entry1.ppn().0);
                let table2 = entry1.ppn().get_pte_array();
                for j in 0..512 {
                    let entry2 = table2[j];
                    if entry2.is_valid() {
                        println!("Level 2 page: virt = {:#x}, phys = {:#x}", i * 512 + j, entry2.ppn().0);
                        let table3 = entry2.ppn().get_pte_array();
                        for k in 0..512 {
                            let entry3 = table3[k];
                            if entry3.is_valid() {
                                println!("Level 3 page:  virt = {:#x}, phys = {:#x}, flag = {}", i * 512 * 512 + j * 512 + k, entry3.ppn().0, entry3.flags().bits());
                            }
                        }
                    }
                }
            }
        }
    }
    pub fn new() -> Self {
        let frame = frame_alloc().unwrap();
        Self {
            root_ppn: frame.ppn,
            frames: vec![FrameTracker::from(frame)]
        }
    }
    pub fn from_token(satp: usize) -> Self {
        Self {
            root_ppn: (satp & ((1 << PPN_WIDTH) - 1)).into(),
            frames: Vec::new()
        }
    }
    fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idx = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut res: Option<&mut PageTableEntry> = None;
        for i in 0..3 {
            let entry = &mut ppn.get_pte_array()[idx[i]];
            if i == 2 {
                res = Some(entry);
                break;
            }
            if !entry.is_valid() {
                let frame = frame_alloc().unwrap();
                *entry = PageTableEntry::new(frame.ppn, PTEFlags::V);
                self.frames.push(frame);
            }
            ppn = entry.ppn();
        }
        res
    }
    fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idx = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut res: Option<&mut PageTableEntry> = None;
        for i in 0..3 {
            let entry = &mut ppn.get_pte_array()[idx[i]];
            if i == 2 {
                res = Some(entry);
                break;
            }
            if !entry.is_valid() {
                return None;
            }
            ppn = entry.ppn();
        }
        res
    }
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.find_pte_create(vpn).unwrap();
        assert!(!pte.is_valid(), "Trying to map vpn {:#x} twice.", vpn.0);
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self.find_pte(vpn).unwrap();
        assert!(pte.is_valid(), "Trying to unmap non-existent vpn {}.",  vpn.0);
        *pte = PageTableEntry::empty();
    }
    pub fn translate(&self, v: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(v).map(|pte| { pte.clone() })
    }
    pub fn translate_va(&self, va: VirtAddr) -> Option<PhysAddr> {
        self.find_pte(va.clone().floor()).map(|pte| {
            let aligned_pa: PhysAddr = pte.ppn().into();
            let offset = va.page_offset();
            let aligned_pa_usize: usize = aligned_pa.into();
            (aligned_pa_usize | offset).into()
        })
    }
    pub fn token(&self) -> usize {
        8usize << 60 | self.root_ppn.0
    }
}

pub fn translated_byte_buffer(token: usize, ptr: *const u8, len: usize) -> Vec<&'static mut [u8]> {
    let page_table = PageTable::from_token(token);
    let mut start = ptr as usize;
    let end = start + len;
    let mut v = Vec::new();
    while start < end {
        let start_va = VirtAddr::from(start);
        let mut vpn = start_va.floor();
        let ppn = page_table.translate(vpn).unwrap().ppn();
        vpn.step();
        let mut end_va: VirtAddr = vpn.into();
        end_va = end_va.min(VirtAddr::from(end));
        assert!(start_va.floor() == end_va.floor());
        if end_va.page_offset() == 0 {
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..]);
        } else {
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..end_va.page_offset()]);
        }
        start = end_va.into();
    }
    v
}

pub fn translated_str(token: usize, ptr: *const u8) -> String {
    let page_table = PageTable::from_token(token);
    let mut string = String::new();
    let mut va = ptr as usize;
    loop {
        let ch: u8 = *(page_table.translate_va(va.into()).unwrap().get_mut());
        if ch == 0 {
            break;
        } else {
            string.push(ch as char);
            va += 1;
        }
    }
    string
}

pub fn translated_refmut<T>(token: usize, ptr: *mut T) -> &'static mut T {
    let page_table = PageTable::from_token(token);
    let va = ptr as usize;
    page_table.translate_va(va.into()).unwrap().get_mut()
}