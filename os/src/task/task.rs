use crate::config::{kernel_stack_position, TRAP_CONTEXT};
use crate::mem::address::{PhysPageNum, VirtAddr};
use crate::mem::memory_set::{MapPermission, MemorySet, KERNEL_SPACE};
use crate::task::context::TaskContext;
use crate::trap::context::TrapContext;
use crate::trap::trap_handler;

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited
}

pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    pub memory_set: MemorySet,
    pub trap_cx_ppn: PhysPageNum,
    pub base_size: usize
}

impl TaskControlBlock {
    pub unsafe fn new(elf_data: &[u8], id: usize) -> Self {
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(id);
        KERNEL_SPACE.exclusive_access().insert_framed_data(
            kernel_stack_bottom.into(),
            kernel_stack_top.into(),
            MapPermission::R | MapPermission::W
        );
        let res = Self {
            task_status: TaskStatus::Ready,
            task_cx: TaskContext::goto_trap_return(kernel_stack_top),
            memory_set,
            trap_cx_ppn,
            base_size: user_sp.into()
        };
        let trap_cx = res.get_trap_cx();
        unsafe {
            *trap_cx = TrapContext::app_init_context(
                entry_point,
                user_sp,
                KERNEL_SPACE.exclusive_access().token(),
                kernel_stack_top,
                trap_handler as usize
            );
        }
        res
    }
    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }
}