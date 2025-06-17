mod fs;
mod process;

use fs::sys_write;
use process::sys_exit;
use crate::syscall::fs::sys_read;
use crate::syscall::process::{sys_exec, sys_fork, sys_get_time, sys_getpid, sys_shutdown, sys_waitpid, sys_yield};

const SYSCALL_OPEN: usize = 17;
const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_GETPID: usize = 172;
const SYSCALL_SHUTDOWN: usize = 201;
const SYSCALL_FORK: usize = 220;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_WAITPID: usize = 260;

pub fn syscall(id: usize, args: [usize; 3]) -> isize {
    match id {
        SYSCALL_READ => {
            sys_read(args[0], args[1] as *const u8, args[2])
        }
        SYSCALL_WRITE => {
            sys_write(args[0], args[1] as *const u8, args[2])
        }
        SYSCALL_EXIT => {
            sys_exit(args[0] as i32)
        }
        SYSCALL_YIELD => {
            sys_yield()
        }
        SYSCALL_GET_TIME => {
            sys_get_time()
        }
        SYSCALL_GETPID => {
            sys_getpid()
        }
        SYSCALL_SHUTDOWN => {
            sys_shutdown(args[0] as i32)
        }
        SYSCALL_FORK => {
            sys_fork()
        }
        SYSCALL_EXEC => {
            sys_exec(args[0] as *mut u8)
        }
        SYSCALL_WAITPID => {
            sys_waitpid(args[0] as isize, args[1] as *mut i32)
        }
        _ => {
            panic!("Unsupported syscall_id: {}", id);
        }
    }
}