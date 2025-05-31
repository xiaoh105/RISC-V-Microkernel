use crate::println;
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next};

pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Process exited with code {}", exit_code);
    exit_current_and_run_next();
    unreachable!();
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}