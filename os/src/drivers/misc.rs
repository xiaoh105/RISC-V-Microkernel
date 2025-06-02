use core::ptr::NonNull;
use riscv::register::mhartid;
use volatile::VolatilePtr;
use crate::config::{SYSTEM_RESET_BASE_ADDR, TIMER_ADDR};

pub enum SystemResetOp {
    ShutdownNormal,
    ShutdownError,
    Reset
}

pub unsafe fn system_reset(op: SystemResetOp) -> ! {
    unsafe {
        let ptr = VolatilePtr::new(NonNull::new_unchecked(SYSTEM_RESET_BASE_ADDR as *mut u32));
        match op {
            SystemResetOp::ShutdownNormal => {
                ptr.write(0x5555);
            }
            SystemResetOp::ShutdownError => {
                ptr.write(0x3333);
            }
            SystemResetOp::Reset => {
                ptr.write(0x7777);
            }
        }
    }
    unreachable!()
}

pub unsafe fn get_time() -> usize {
    let ptr = unsafe {
        VolatilePtr::new(NonNull::new_unchecked(TIMER_ADDR as *mut usize))
    };
    ptr.read()
}

/// This function is allowed to be called ONLY UNDER M-Mode.
pub fn hart_id() -> usize {
    mhartid::read()
}

/// This function is allowed to be called ONLY UNDER M-Mode
pub unsafe fn set_mtime_cmp(addr: usize, time: usize) {
    let ptr = unsafe {
        VolatilePtr::new(NonNull::new_unchecked(addr as *mut usize))
    };
    ptr.write(time);
}