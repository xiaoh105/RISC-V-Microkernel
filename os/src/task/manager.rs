use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::lazy_static;
use crate::sync::up::UPSafeCell;
use crate::task::task::ProcessControlBlock;

pub struct TaskManager {
    ready_queue: VecDeque<Arc<ProcessControlBlock>>
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new()
        }
    }
    pub fn add(&mut self, process: Arc<ProcessControlBlock>) {
        self.ready_queue.push_back(process);
    }
    pub fn fetch(&mut self) -> Option<Arc<ProcessControlBlock>> {
        self.ready_queue.pop_front()
    }
}

lazy_static! {
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> = unsafe {
        UPSafeCell::new(TaskManager::new())
    };
}

pub fn add_task(process: Arc<ProcessControlBlock>) {
    TASK_MANAGER.exclusive_access().add(process);
}

pub fn fetch_task() -> Option<Arc<ProcessControlBlock>> {
    TASK_MANAGER.exclusive_access().fetch()
}