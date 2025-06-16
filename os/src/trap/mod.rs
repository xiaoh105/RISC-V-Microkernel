use core::arch::{asm, global_asm};
use riscv::interrupt::{Exception, Trap, Interrupt};
use riscv::register::mtvec::TrapMode;
use riscv::register::{scause, sip, stval, stvec};
use riscv::register::stvec::Stvec;
use crate::config::{TRAMPOLINE, TRAP_CONTEXT};
use crate::{println, red_msg};
use crate::syscall::syscall;
use crate::task::{current_trap_cx, current_user_token, exit_current_and_run_next, suspend_current_and_run_next};
use crate::trap::context::TrapContext;

pub mod context;

global_asm!(include_str!("asm/trap.asm"));

#[unsafe(no_mangle)]
fn trap_from_kernel() -> ! {
    panic!("Trap from kernel")
}

pub fn set_kernel_trap_entry() {
    unsafe {
        let mut vec: Stvec = Stvec::from_bits(0);
        vec.set_address(trap_from_kernel as usize);
        vec.set_trap_mode(TrapMode::Direct);
        stvec::write(vec);
    }
}

unsafe fn set_user_trap_entry() {
    unsafe {
        let mut vec: Stvec = Stvec::from_bits(0);
        vec.set_address(TRAMPOLINE as usize);
        vec.set_trap_mode(TrapMode::Direct);
        stvec::write(vec);
    }
}

#[unsafe(no_mangle)]
pub unsafe fn trap_return() -> ! {
    unsafe { set_user_trap_entry(); }
    let trap_cx_ptr = TRAP_CONTEXT;
    let user_satp = current_user_token();
    unsafe extern "C" {
        fn __alltraps();
        fn __restore();
    }
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
    unsafe {
        asm!(
            "fence.i",
            "jr {restore_va}",
            restore_va = in(reg) restore_va,
            in("a0") trap_cx_ptr,
            in("a1") user_satp,
            options(noreturn)
        )
    }
}

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
pub unsafe fn trap_handler(cx: &mut TrapContext) -> ! {
    set_kernel_trap_entry();
    let cx = current_trap_cx();
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
    unsafe { trap_return() }
}