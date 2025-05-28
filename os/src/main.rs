#![no_std]
#![no_main]

mod lang_items;
mod drivers;
mod config;
mod io;

use core::arch::global_asm;
use crate::drivers::uart::UartPort;

global_asm!(include_str!("asm/entry.asm"));

unsafe fn clear_bss() {
    unsafe extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { 
        (a as *mut u8).write_volatile(0) 
    });
}

#[unsafe(no_mangle)]
pub unsafe fn rust_main() -> ! {
    init_uart();
    unsafe {
        clear_bss();
    }
    println!("Hello, world!");
    panic!("Shutdown machine!");
}

pub fn init_uart() {
    UartPort.init();
    println!(r#" ______     ______     ______     ______     ______ "#);
    println!(r#"|  __  \   /      \   /  __  \   |  __  \   |  ____|"#);
    println!(r#"| |__) |  |  .-----  |  |  |  |  | |__) |   | |____ "#);
    println!(r#"|  _  /   |  |       |  |  |  |  |  _  /    |  ____|"#);
    println!(r#"| | \ \   |  `----.  |  `--'  |  | | \ \    | |____ "#);
    println!(r#"|_|  \_\  \______/    \______/   |_|  \_\   |______|"#);
    green_msg!("[RustSBI] UART initialized.");
}