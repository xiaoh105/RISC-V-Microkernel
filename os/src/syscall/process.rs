use crate::batch::run_next_app;
use crate::println;

pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Process exited with code {}", exit_code);
    unsafe { run_next_app(); }
}