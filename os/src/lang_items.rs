use core::panic::PanicInfo;
use crate::drivers::misc::{system_reset, SystemResetOp};
use crate::red_msg;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        red_msg!("Panicked at {}: {} {}", location.file(), location.line(), info.message());
    } else {
        red_msg!("Panicked at {}", info.message());
    }
    unsafe {
        system_reset(SystemResetOp::ShutdownError);
    }
}