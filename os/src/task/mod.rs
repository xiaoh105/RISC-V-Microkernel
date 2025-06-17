use alloc::sync::Arc;
use lazy_static::lazy_static;
use crate::loader::get_app_data_by_name;
use crate::task::context::TaskContext;
use crate::task::manager::add_task;
use crate::task::processor::{current_task, schedule};
use crate::task::task::{ProcessControlBlock, TaskStatus};

pub mod context;
mod switch;
pub mod task;
pub mod pid;
pub mod manager;
pub mod processor;


lazy_static! {
    pub static ref INITPROC: Arc<ProcessControlBlock> = Arc::new(
        ProcessControlBlock::new(get_app_data_by_name("initproc").unwrap())
    );
}

pub fn add_initproc() {
    add_task(INITPROC.clone());
}

pub fn suspend_current_and_run_next() {
    let task = current_task().unwrap();
    let mut task_inner =  task.inner_exclusive_access();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    task_inner.task_status = TaskStatus::Ready;
    drop(task_inner);
    add_task(task);
    schedule(task_cx_ptr);
}

pub fn exit_current_and_run_next(exit_code: i32) {
    let task = current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();
    task_inner.task_status = TaskStatus::Zombie;
    task_inner.exit_code = exit_code;

    {
        let mut initproc_inner = INITPROC.inner_exclusive_access();
        for child in task_inner.children.iter() {
            child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child.clone());
        }
    }

    task_inner.children.clear();
    task_inner.memory_set.recycled_data_pages();
    drop(task_inner);
    drop(task);
    let mut _unused = TaskContext::zero_init();
    schedule(&mut _unused as *mut TaskContext);
}