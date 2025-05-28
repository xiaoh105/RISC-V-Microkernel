use core::panic::PanicInfo;
use crate::drivers::misc::{system_reset, SystemResetOp};
use crate::println;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!("Panicked at {}: {} {}", location.file(), location.line(), info.message());
    } else {
        println!("Panicked at {}", info.message());
    }
    unsafe {
        system_reset(SystemResetOp::ShutdownError);
    }
}