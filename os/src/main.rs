#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
extern crate alloc;

mod lang_items;
mod drivers;
mod config;
mod io;
mod sync;
mod trap;
mod syscall;
mod loader;
mod task;
mod timer;
mod utils;
mod mem;

use core::arch::{asm, global_asm};
use riscv::register::{mepc, mstatus, pmpaddr0, pmpcfg0, satp, sie, sstatus};
use riscv::register::mstatus::MPP;
use riscv::register::satp::Satp;
use crate::drivers::uart::UartPort;
use crate::loader::load_apps;
use crate::mem::frame_allocator::{frame_allocator_test, init_frame_allocator};
use crate::mem::heap_allocator::{heap_test, init_heap};
use crate::mem::memory_set::KERNEL_SPACE;
use crate::task::run_first_task;
use crate::timer::init_timer;

global_asm!(include_str!("asm/entry.asm"));
global_asm!(include_str!("asm/link_app.asm"));

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
pub unsafe fn sbi_entry() -> ! {
    unsafe {
        mstatus::set_mpp(MPP::Supervisor);
        mepc::write(rust_main as usize);
        satp::write(Satp::from_bits(0));
        pmpaddr0::write(0x3fffffffffffffusize);
        pmpcfg0::write(0xf);
        init_timer();
        asm!(
            "csrw mideleg, {mideleg}",
            "csrw medeleg, {medeleg}",
            medeleg = in(reg) !0,
            mideleg = in(reg) !0,
        );
        asm!("mret", options(noreturn))
    }
}

#[unsafe(no_mangle)]
pub unsafe fn rust_main() -> ! {
    init_uart();
    unsafe {
        clear_bss();
    }
    unsafe { trap::init_trap(); }
    green_msg!("[kernel] Trap info correctly set.");
    unsafe { init_heap(); }
    green_msg!("[kernel] Kernel heap initialized.");
    heap_test();
    green_msg!("[kernel] Heap test passed!.");
    unsafe { init_frame_allocator(); }
    green_msg!("[kernel] Frame allocator initialized.");
    frame_allocator_test();
    green_msg!("[kernel] Frame allocator test passed!");
    KERNEL_SPACE.exclusive_access().activate();
    load_apps();
    green_msg!("[kernel] All apps loaded.");
    unsafe {
        sstatus::set_sie();
        sie::set_stimer();
        sie::set_sext();
        sie::set_ssoft();
    }
    run_first_task();
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