use crate::config::TRAP_CONTEXT;
use crate::mem::address::{PhysPageNum, VirtAddr};
use crate::mem::memory_set::{MemorySet, KERNEL_SPACE};
use crate::sync::up::UPSafeCell;
use crate::task::context::TaskContext;
use crate::task::pid::{pid_alloc, KernalStack, PidHandle};
use crate::trap::context::TrapContext;
use crate::trap::trap_handler;
use alloc::sync::Arc;
use alloc::sync::Weak;
use alloc::vec::Vec;
use core::cell::RefMut;

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Zombie,
}

pub struct ProcessControlBlock {
    pub pid: PidHandle,
    pub kernal_stack: KernalStack,
    inner: UPSafeCell<ProcessControlBlockInner>,
}

pub struct ProcessControlBlockInner {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    pub memory_set: MemorySet,
    pub trap_cx_ppn: PhysPageNum,
    pub base_size: usize,
    pub parent: Option<Weak<ProcessControlBlock>>,
    pub children: Vec<Arc<ProcessControlBlock>>,
    pub exit_code: i32
}

impl ProcessControlBlockInner {
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }
    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }
    pub fn get_status(&self) -> TaskStatus {
        self.task_status
    }
    pub fn is_zombie(&self) -> bool {
        self.task_status == TaskStatus::Zombie
    }
}

impl ProcessControlBlock {
    pub fn new(elf_data: &[u8]) -> Self {
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        let pid_handle = pid_alloc();
        let kernel_stack = KernalStack::new(&pid_handle);
        let kstack_top = kernel_stack.get_top();
        let ret = Self {
            pid: pid_handle,
            kernal_stack: kernel_stack,
            inner: unsafe {
                UPSafeCell::new(ProcessControlBlockInner {
                    trap_cx_ppn,
                    base_size: user_sp,
                    task_cx: TaskContext::goto_trap_return(kstack_top),
                    task_status: TaskStatus::Ready,
                    memory_set,
                    parent: None,
                    children: Vec::new(),
                    exit_code: 0
                })
            }
        };
        let trap_cx = ret.inner_exclusive_access().get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            kstack_top,
            trap_handler as usize
        );
        ret
    }
    pub fn inner_exclusive_access(&self) -> RefMut<'_, ProcessControlBlockInner> {
        self.inner.exclusive_access()
    }
    pub fn getpid(&self) -> usize { self.pid.0 }
    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        let mut parent_inner = self.inner_exclusive_access();
        let memory_set = MemorySet::from_existed_user(&parent_inner.memory_set);
        let trap_cx_ppn = memory_set.translate(VirtAddr::from(TRAP_CONTEXT).into()).unwrap().ppn();
        let pid_handle = pid_alloc();
        let kernel_stack = KernalStack::new(&pid_handle);
        let kstack_top = kernel_stack.get_top();
        let ret = Arc::new(Self {
            pid: pid_handle,
            kernal_stack: kernel_stack,
            inner: unsafe {
                UPSafeCell::new(ProcessControlBlockInner {
                    trap_cx_ppn,
                    base_size: parent_inner.base_size,
                    task_cx: TaskContext::goto_trap_return(kstack_top),
                    task_status: TaskStatus::Ready,
                    memory_set,
                    parent: Some(Arc::downgrade(self)),
                    children: Vec::new(),
                    exit_code: 0
                })
            }
        });
        parent_inner.children.push(ret.clone());
        let trap_cx = ret.inner_exclusive_access().get_trap_cx();
        trap_cx.kernel_sp = kstack_top;
        ret
    }
    pub fn exec(&self, elf_data: &[u8]) {
        let (memory_set,  user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set.translate(VirtAddr::from(TRAP_CONTEXT).into()).unwrap().ppn();
        let mut inner = self.inner_exclusive_access();
        inner.memory_set = memory_set;
        inner.trap_cx_ppn = trap_cx_ppn;
        let trap_cx = inner.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            self.kernal_stack.get_top(),
            trap_handler as usize
        )
    }
}