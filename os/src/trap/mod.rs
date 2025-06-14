use core::arch::global_asm;
use riscv::interrupt::{Exception, Trap, Interrupt};
use riscv::register::mtvec::TrapMode;
use riscv::register::{scause, sip, stval, stvec};
use riscv::register::stvec::Stvec;
use crate::red_msg;
use crate::syscall::syscall;
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next};
use crate::trap::context::TrapContext;

pub mod context;

global_asm!(include_str!("asm/trap.asm"));

pub unsafe fn init_trap() {
    unsafe extern "C" {
        fn __alltraps();
    }
    unsafe {
        let mut stvec_target: Stvec = Stvec::from_bits(0);
        stvec_target.set_trap_mode(TrapMode::Direct);
        stvec_target.set_address(__alltraps as usize);
        stvec::write(stvec_target);
    }
}

#[unsafe(no_mangle)]
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause().try_into::<Interrupt, Exception>().unwrap() {
        Trap::Interrupt(Interrupt::SupervisorSoft) => {
            unsafe { sip::clear_ssoft(); }
            suspend_current_and_run_next();
        }
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            cx.reg[10] = syscall(cx.reg[17], [cx.reg[10], cx.reg[11], cx.reg[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) => {
            red_msg!("[kernel] Page fault in application. Kernel killed it.");
            exit_current_and_run_next();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            red_msg!("[kernel] Illegal instruction in application. Kernel killed it.");
            exit_current_and_run_next();
        }
        _ => {
            panic!("Unsupported trap {:?}, trap {:#x}!", scause.cause(), stval);
        }
    }
    cx
}