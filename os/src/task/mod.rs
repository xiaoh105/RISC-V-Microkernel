use lazy_static::lazy_static;
use crate::{blue_msg, green_msg};
use crate::config::MAX_APP_NUM;
use crate::loader::init_app_cx;
use crate::sync::up::UPSafeCell;
use crate::task::context::TaskContext;
use crate::task::switch::__switch;
use crate::task::task::{TaskControlBlock, TaskStatus};

pub mod context;
mod switch;
mod task;

pub struct TaskManager {
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>
}

pub struct TaskManagerInner {
    tasks: [TaskControlBlock; MAX_APP_NUM],
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
        let mut tasks = [
            TaskControlBlock {
                task_status: TaskStatus::UnInit,
                task_cx: TaskContext::zero_init()
            };
            MAX_APP_NUM
        ];
        for i in 0..num_app {
            tasks[i].task_cx = TaskContext::goto_restore(init_app_cx(i));
            tasks[i].task_status = TaskStatus::Ready;
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
        let mut current = (inner.current_task + 1) % MAX_APP_NUM;
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
            panic!("All applications completed!");
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
}