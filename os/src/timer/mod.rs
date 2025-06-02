use core::arch::global_asm;
use riscv::register::{mie, mscratch, mstatus, mtvec};
use riscv::register::mtvec::{Mtvec, TrapMode};
use crate::config::*;
use crate::drivers::misc::{get_time, hart_id, set_mtime_cmp};

const MICRO_PER_SEC: usize = 1_000;

global_asm!(include_str!("asm/timer_trap.asm"));

pub fn get_time_ms() -> usize {
    unsafe { get_time() / (TIMER_FREQ / MICRO_PER_SEC) }
}

#[unsafe(link_section = ".bss.stack")]
#[unsafe(no_mangle)]
pub static mut SCRATCH: [[usize; 5]; CPUS] = [[0; 5]; CPUS];

pub unsafe fn init_timer() {
    let hart_id = hart_id();
    unsafe {
        set_mtime_cmp(TIMER_CMP_BASE_ADDR + (hart_id << 3), get_time() + SCHED_PERIOD);
    }

    let scratch = unsafe { &mut SCRATCH[0] };
    scratch[3] = TIMER_CMP_BASE_ADDR + (hart_id << 3);
    scratch[4] = SCHED_PERIOD;
    unsafe {
        mscratch::write(scratch as *const usize as usize);
    }

    unsafe extern "C" {
        safe fn __timer_traps();
    }
    let mut vec = Mtvec::from_bits(0);
    vec.set_trap_mode(TrapMode::Direct);
    vec.set_address(__timer_traps as usize);
    unsafe {
        mtvec::write(vec);
        mstatus::set_mie();
        mie::set_mtimer();
    }
}