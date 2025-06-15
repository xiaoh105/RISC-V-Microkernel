use alloc::vec;
use alloc::vec::Vec;
use bitflags::bitflags;
use crate::mem::address::{PageTableEntry, PhysPageNum, VirtPageNum, PPN_WIDTH};
use crate::mem::frame_allocator::{frame_alloc, FrameTracker};

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
        assert!(!pte.is_valid(), "Trying to map vpn {} twice.", vpn.0);
        *pte = PageTableEntry::new(ppn, flags);
    }
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self.find_pte(vpn).unwrap();
        assert!(pte.is_valid(), "Trying to unmap non-existent vpn {}.",  vpn.0);
        *pte = PageTableEntry::empty();
    }
    pub fn translate(&self, v: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(v).map(|pte| { pte.clone() })
    }
    pub fn token(&self) -> usize {
        8usize << 60 | self.root_ppn.0
    }
}