use alloc::sync::Arc;
use crate::drivers::misc::{system_reset, SystemResetOp};
use crate::loader::get_app_data_by_name;
use crate::mem::page_table::{translated_refmut, translated_str};
use crate::println;
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next};
use crate::task::manager::add_task;
use crate::task::processor::{current_task, current_user_token};
use crate::timer::get_time_ms;

pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Process exited with code {}", exit_code);
    exit_current_and_run_next(exit_code);
    unreachable!();
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_get_time() -> isize {
    get_time_ms() as isize
}

pub fn sys_fork() -> isize {
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
    trap_cx.reg[10] = 0;
    add_task(new_task);
    new_pid as isize
}

pub fn sys_exec(path: *const u8) -> isize {
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(data) = get_app_data_by_name(&path) {
        let task = current_task().unwrap();
        task.exec(data);
        0
    } else {
        -1
    }
}

pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if inner.children
        .iter()
        .find(|p| pid == -1 || pid as usize == p.getpid())
        .is_none() {
        return -1;
    }
    let pair = inner.children
        .iter()
        .enumerate()
        .find(|(_, p)| {
            p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        });
    if let Some((index, _)) = pair {
        let child = inner.children.remove(index);
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        let exit_code = child.inner_exclusive_access().exit_code;
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
}

pub fn sys_getpid() -> isize {
    current_task().unwrap().pid.0 as isize
}

pub fn sys_shutdown(code: i32) -> isize {
    if code == 0 {
        unsafe { system_reset(SystemResetOp::ShutdownNormal); }
    } else {
        unsafe { system_reset(SystemResetOp::ShutdownError) }
    }
}