use alloc::vec::Vec;
use lazy_static::lazy_static;
use crate::config::kernel_stack_position;
use crate::mem::address::VirtAddr;
use crate::mem::memory_set::{MapPermission, KERNEL_SPACE};
use crate::sync::up::UPSafeCell;

pub struct PidHandle(pub usize);

struct PidAllocator {
    cur: usize,
    recycled: Vec<usize>
}

pub struct KernalStack {
    pid: usize
}

lazy_static! {
    static ref PID_ALLOCATOR : UPSafeCell<PidAllocator> = unsafe {
        UPSafeCell::new(PidAllocator::new())
    };
}

impl PidAllocator {
    pub fn new() -> Self {
        Self {
            cur: 0,
            recycled: Vec::new()
        }
    }
    pub fn alloc(&mut self) -> PidHandle {
        if let Some(pid) = self.recycled.pop() {
            PidHandle(pid)
        } else {
            self.cur += 1;
            PidHandle(self.cur)
        }
    }
    pub fn dealloc(&mut self, pid: usize) {
        assert!(pid <= self.cur);
        assert!(self.recycled.iter().find(|ppid| **ppid == pid).is_none(),
            "pid {} was not deallocated", pid);
        self.recycled.push(pid);
    }
}

pub fn pid_alloc() -> PidHandle {
    PID_ALLOCATOR.exclusive_access().alloc()
}

impl Drop for PidHandle {
    fn drop(&mut self) {
        PID_ALLOCATOR.exclusive_access().dealloc(self.0);
    }
}

impl KernalStack {
    pub fn new(pid_handle: &PidHandle) -> Self {
        let pid = pid_handle.0;
        let (kstack_bottom, kstack_top) = kernel_stack_position(pid);
        KERNEL_SPACE.exclusive_access().insert_framed_data(
            kstack_bottom.into(),
            kstack_top.into(),
            MapPermission::R |  MapPermission::W
        );
        KernalStack {
            pid
        }
    }
    
    pub fn get_top(&self) -> usize {
        let (_, kernel_stack_top) = kernel_stack_position(self.pid);
        kernel_stack_top
    }
    
    pub fn push_on_top<T>(&self, value: T) -> *mut T
    where T: Sized {
        let kstack_top = self.get_top();
        let ptr_mut = (kstack_top - core::mem::size_of::<T>()) as *mut T;
        unsafe { *ptr_mut = value; }
        ptr_mut
    }
}

impl Drop for KernalStack {
    fn drop(&mut self) {
        let (kstack_bottom, _) = kernel_stack_position(self.pid);
        let kstack_bottom_va: VirtAddr = kstack_bottom.into();
        KERNEL_SPACE.exclusive_access().remove_area_with_start_vpn(kstack_bottom_va.into());
    }
}