use core::arch::{asm, global_asm};
use riscv::interrupt::{Exception, Trap, Interrupt};
use riscv::register::mtvec::TrapMode;
use riscv::register::{scause, sip, stval, stvec};
use riscv::register::stvec::Stvec;
use crate::config::{TRAMPOLINE, TRAP_CONTEXT};
use crate::drivers::misc::{system_reset, SystemResetOp};
use crate::drivers::uart::handle_irq;
use crate::{println, red_msg};
use crate::syscall::syscall;
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next};
use crate::task::processor::{current_trap_cx, current_user_token};
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
    let mut cx = current_trap_cx();
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause().try_into::<Interrupt, Exception>().unwrap() {
        Trap::Interrupt(Interrupt::SupervisorSoft) => {
            unsafe { sip::clear_ssoft(); }
            suspend_current_and_run_next();
        }
        Trap::Interrupt(Interrupt::SupervisorExternal) => {
            let sip = sip::read().bits();
            unsafe {
                asm! {"csrw sip, {sip}", sip = in(reg) sip ^ (1 << 9)};
            }
            handle_irq();
        }
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            let res = syscall(cx.reg[17], [cx.reg[10], cx.reg[11], cx.reg[12]]) as usize;
            cx = current_trap_cx();
            cx.reg[10] = res;
        }
        Trap::Exception(Exception::StoreFault) | 
        Trap::Exception(Exception::StorePageFault) |
        Trap::Exception(Exception::InstructionFault) |
        Trap::Exception(Exception::InstructionPageFault) |
        Trap::Exception(Exception::LoadFault) |
        Trap::Exception(Exception::LoadPageFault) => {
            red_msg!("[kernel] {:?} in application, bad addr = {:#x}, bad instruction = {:#x}, core dumped.",
                scause.cause(),
                stval,
                current_trap_cx().sepc,
            );
            exit_current_and_run_next(-2);
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            red_msg!("[kernel] Illegal instruction in application. Kernel killed it.");
            exit_current_and_run_next(-3);
        }
        _ => {
            panic!("Unsupported trap {:?}, trap {:#x}!", scause.cause(), stval);
        }
    }
    unsafe { trap_return() }
}