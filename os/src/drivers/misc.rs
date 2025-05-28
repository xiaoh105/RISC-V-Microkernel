use core::ptr::NonNull;
use volatile::VolatilePtr;
use crate::config::SYSTEM_RESET_BASE_ADDR;

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