#![no_std]
#![no_main]

mod lang_items;
mod drivers;
mod config;
mod io;
mod sync;
mod batch;
mod trap;
mod syscall;

use core::arch::{asm, global_asm};
use riscv::register::{mepc, mstatus, pmpaddr0, pmpcfg0, satp};
use riscv::register::mstatus::MPP;
use riscv::register::satp::Satp;
use crate::drivers::uart::UartPort;

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
    batch::init();
    green_msg!("[kernel] BatchOS initialized.");
    unsafe { batch::run_next_app(); }
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