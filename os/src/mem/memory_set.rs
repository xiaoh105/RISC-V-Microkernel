use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::arch::asm;
use bitflags::bitflags;
use lazy_static::lazy_static;
use alloc::sync::Arc;
use riscv::register::satp;
use riscv::register::satp::Satp;
use crate::blue_msg;
use crate::config::{MEMORY_END, SYSTEM_RESET_BASE_ADDR, TIMER_ADDR, TRAMPOLINE, TRAP_CONTEXT, UART_BASE_ADDR};
use crate::mem::address::{PageTableEntry, PhysAddr, PhysPageNum, StepByOne, VPNRange, VirtAddr, VirtPageNum, PAGE_SIZE};
use crate::mem::frame_allocator::{frame_alloc, FrameTracker};
use crate::mem::memory_set::MapType::{Identical, Framed};
use crate::mem::page_table::{PTEFlags, PageTable};
use crate::sync::up::UPSafeCell;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MapType {
    Identical, Framed
}

bitflags! {
    pub struct MapPermission: u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}

pub struct MapArea {
    vpn_range: VPNRange,
    data_frames: BTreeMap<VirtPageNum, FrameTracker>,
    map_type: MapType,
    map_permission: MapPermission
}

pub struct MemorySet {
    page_table: PageTable,
    areas: Vec<MapArea>
}

lazy_static! {
    pub static ref KERNEL_SPACE: Arc<UPSafeCell<MemorySet>> = unsafe { Arc::new(
        UPSafeCell::new(MemorySet::new_kernel())
    )};
}

impl MapArea {
    pub fn new(start_va: VirtAddr, end_va: VirtAddr, map_type: MapType, map_permission: MapPermission) -> Self {
        let start_vpn: VirtPageNum = start_va.floor();
        let end_vpn: VirtPageNum = end_va.ceil();
        Self {
            vpn_range: VPNRange::new(start_vpn, end_vpn),
            data_frames: BTreeMap::new(),
            map_type,
            map_permission
        }
    }
    pub fn map_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        let ppn: PhysPageNum;
        match self.map_type {
            Identical => {
                ppn = PhysPageNum(vpn.0);
            }
            Framed => {
                let frame = frame_alloc().unwrap();
                ppn = frame.ppn;
                self.data_frames.insert(vpn, frame);
            }
        }
        let pte_flags = PTEFlags::from_bits(self.map_permission.bits()).unwrap();
        page_table.map(vpn, ppn, pte_flags);
    }
    pub fn map(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.map_one(page_table, vpn);
        }
    }
    pub fn unmap_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        match self.map_type {
            Identical => {}
            Framed => {
                self.data_frames.remove(&vpn);
            }
        }
        page_table.unmap(vpn);
    }
    pub fn unmap(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.unmap_one(page_table, vpn);
        }
    }
    pub fn copy_data(&mut self, page_table: &PageTable, data: &[u8]) {
        assert_eq!(self.map_type, MapType::Framed);
        let mut start: usize = 0;
        let mut vpn = self.vpn_range.start();
        let len = data.len();
        loop {
            let src = &data[start..len.min(start + PAGE_SIZE)];
            let dst = &mut page_table.translate(vpn).unwrap().ppn().get_bytes_array()[..src.len()];
            dst.copy_from_slice(src);
            start += PAGE_SIZE;
            if start >= len {
                break;
            }
            vpn.step();
        }
    }
}

unsafe extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss_with_stack();
    fn ebss();
    fn ekernel();
    fn strampoline();
}

impl MemorySet {
    pub fn new_bare() -> Self {
        Self {
            page_table: PageTable::new(),
            areas: Vec::new()
        }
    }
    pub fn new_kernel() -> Self {
        let mut memory_set = Self::new_bare();
        memory_set.map_trampoline();
        blue_msg!("[kernel] .text [{:#x}, {:#x})", stext as usize, etext as usize);
        blue_msg!("[kernel] .rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
        blue_msg!("[kernel] .data [{:#x}, {:#x})", sdata as usize, edata as usize);
        blue_msg!("[kernel] .bss [{:#x}, {:#x})", sbss_with_stack as usize, ebss as usize);
        memory_set.push(MapArea::new(
            (stext as usize).into(),
            (etext as usize).into(),
            Identical,
            MapPermission::R | MapPermission::X)
        , None);
        memory_set.push(MapArea::new(
            (srodata as usize).into(),
            (erodata as usize).into(),
            Identical,
            MapPermission::R)
        , None);
        memory_set.push(MapArea::new(
            (sdata as usize).into(),
            (edata as usize).into(),
            Identical,
            MapPermission::R | MapPermission::W)
        , None);
        memory_set.push(MapArea::new(
            (sbss_with_stack as usize).into(),
            (ebss as usize).into(),
            Identical,
            MapPermission::R | MapPermission::W)
        , None);
        memory_set.push(MapArea::new(
            (ekernel as usize).into(),
            MEMORY_END.into(),
            Identical,
            MapPermission::R | MapPermission::W)
        , None);
        memory_set.push(MapArea::new(
            UART_BASE_ADDR.into(),
            (UART_BASE_ADDR + PAGE_SIZE - 1).into(),
            Identical,
            MapPermission::R | MapPermission::W
        ), None);
        memory_set.push(MapArea::new(
            TIMER_ADDR.into(),
            (TIMER_ADDR + PAGE_SIZE - 1).into(),
            Identical,
            MapPermission::R | MapPermission::W
        ), None);
        memory_set.push(MapArea::new(
            SYSTEM_RESET_BASE_ADDR.into(),
            (SYSTEM_RESET_BASE_ADDR + PAGE_SIZE - 1).into(),
            Identical,
            MapPermission::R | MapPermission::W
        ), None);
        memory_set
    }
    pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize) {
        let mut memory_set = Self::new_bare();
        memory_set.map_trampoline();
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        assert_eq!(elf.header.pt1.magic, [0x7f, 0x45, 0x4c, 0x46], "Invalid ELF file");
        let ph_count = elf.header.pt2.ph_count();
        let mut max_end_vpn = VirtPageNum(0);
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() != xmas_elf::program::Type::Load {
                continue;
            }
            let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
            let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();
            let mut permission = MapPermission::U;
            let ph_flags = ph.flags();
            if ph_flags.is_read() { permission |= MapPermission::R; }
            if ph_flags.is_write() { permission |= MapPermission::W; }
            if ph_flags.is_execute() { permission |= MapPermission::X; }
            let map_area = MapArea::new(start_va, end_va, Framed, permission);
            max_end_vpn = map_area.vpn_range.end();
            memory_set.push(
                map_area,
                Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize]),
            );
        }
        let max_end_va: VirtAddr = max_end_vpn.into();
        let user_stack_bottom: usize = usize::from(max_end_va) + PAGE_SIZE;
        let user_stack_top: usize = user_stack_bottom + PAGE_SIZE;
        memory_set.push(MapArea::new(
            user_stack_bottom.into(),
            user_stack_top.into(),
            Framed,
            MapPermission::R | MapPermission::W | MapPermission::U
        ), None);
        memory_set.push(MapArea::new(
            TRAP_CONTEXT.into(),
            TRAMPOLINE.into(),
            Framed,
            MapPermission::R | MapPermission::W
        ), None);
        (memory_set, user_stack_top, elf.header.pt2.entry_point() as usize)
    }
    fn push(&mut self, mut map_area: MapArea, data: Option<&[u8]>) {
        map_area.map(&mut self.page_table);
        if let Some(data) = data {
            map_area.copy_data(&self.page_table, data);
        }
        self.areas.push(map_area);
    }
    pub fn insert_framed_data(
        &mut self,
        start_va: VirtAddr, end_va: VirtAddr, permission: MapPermission
    ) {
        self.push(MapArea::new(start_va, end_va, Framed, permission), None);
    }
    pub fn translate(&self, va: VirtPageNum) -> Option<PageTableEntry> {
        self.page_table.translate(va)
    }
    pub fn token(&self) -> usize {
        self.page_table.token()
    }
    pub fn activate(&self) {
        let satp_val = Satp::from_bits(self.page_table.token());
        unsafe {
            satp::write(satp_val);
            asm!("sfence.vma");
        }
    }
    pub fn map_trampoline(&mut self) {
        self.page_table.map(
            VirtAddr::from(TRAMPOLINE).into(),
            PhysAddr::from(strampoline as usize).into(),
            PTEFlags::R | PTEFlags::X
        );
    }
}

pub fn remap_test() {
    let kernel_space = KERNEL_SPACE.exclusive_access();
    let mid_text: VirtAddr = ((stext as usize + etext as usize) / 2).into();
    let mid_rodata: VirtAddr = ((srodata as usize + erodata as usize) / 2).into();
    let mid_data: VirtAddr = ((sdata as usize + edata as usize) / 2).into();
    assert_eq!(
        kernel_space.page_table.translate(mid_text.floor()).unwrap().writable(),
        false
    );
    assert_eq!(
        kernel_space.page_table.translate(mid_rodata.floor()).unwrap().writable(),
        false,
    );
    assert_eq!(
        kernel_space.page_table.translate(mid_data.floor()).unwrap().executable(),
        false,
    );
}
