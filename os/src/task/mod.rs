use lazy_static::lazy_static;
use alloc::vec::Vec;
use crate::{blue_msg, green_msg};
use crate::drivers::misc::{system_reset, SystemResetOp};
use crate::loader::get_app_data;
use crate::sync::up::UPSafeCell;
use crate::task::context::TaskContext;
use crate::task::switch::__switch;
use crate::task::task::{TaskControlBlock, TaskStatus};
use crate::trap::context::TrapContext;

pub mod context;
mod switch;
mod task;

pub struct TaskManager {
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>
}

pub struct TaskManagerInner {
    tasks: Vec<TaskControlBlock>,
    current_task: usize
}

pub fn get_app_num() -> usize {
    unsafe extern "C" {
        safe fn _num_app();
    }
    unsafe {
        (_num_app as usize as *const usize).read_volatile()
    }
}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_app_num();
        let mut tasks: Vec<TaskControlBlock> = Vec::new();
        for i in 0..num_app {
            unsafe {
                tasks.push(TaskControlBlock::new(get_app_data(i),i));
            }
        }
        green_msg!("[kernel] All task context initialized");
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0
                })
            }
        }
    };
}

pub fn suspend_current_and_run_next() {
    TASK_MANAGER.mark_current_task_suspended();
    run_next_task();
}

pub fn exit_current_and_run_next() {
    TASK_MANAGER.mark_current_task_exited();
    run_next_task();
}

fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

pub fn run_first_task() -> ! {
    TASK_MANAGER.run_first_task();
}

impl TaskManager {
    pub fn mark_current_task_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
    }
    
    pub fn mark_current_task_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
    }
    
    pub fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let mut current = (inner.current_task + 1) % get_app_num();
        for _ in 0..self.num_app {
            if inner.tasks[current].task_status == TaskStatus::Ready {
                return Some(current);
            }
            current = (current + 1) % self.num_app;
        }
        None
    }
    
    pub fn run_next_task(&self) {
        if let Some(id) =  self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[id].task_status = TaskStatus::Running;
            inner.current_task = id;
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[id].task_cx as *const TaskContext;
            drop(inner);
            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }
        } else {
            green_msg!("All applications completed!");
            unsafe { system_reset(SystemResetOp::ShutdownNormal); }
        }
    }
    
    pub fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.task_status = TaskStatus::Running;
        let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
        drop(inner);
        blue_msg!("[kernel] Starting the first app.");
        let mut _unused = TaskContext::zero_init();
        unsafe {
            __switch(&mut _unused as *mut TaskContext, next_task_cx_ptr);
        }
        unreachable!()
    }

    fn get_current_token(&self) -> usize {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].get_user_token()
    }
    
    fn get_current_trap_cx(&self) -> &mut TrapContext {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].get_trap_cx()
    }
}

pub fn current_user_token() -> usize {
    TASK_MANAGER.get_current_token()
}

pub fn current_trap_cx() -> &'static mut TrapContext {
    TASK_MANAGER.get_current_trap_cx()
}