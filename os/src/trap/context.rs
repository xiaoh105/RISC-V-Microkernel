use riscv::register::sstatus;
use riscv::register::sstatus::{Sstatus, SPP};

#[repr(C)]
pub struct TrapContext {
    pub reg: [usize; 32],
    pub sstatus: Sstatus,
    pub sepc: usize
}

impl TrapContext {
    pub fn set_sp(&mut self, sp: usize) {
        self.reg[2] = sp;
    }
    pub fn app_init_context(entry: usize, sp: usize) -> Self {
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        let mut cx = Self {
            reg: [0; 32],
            sstatus,
            sepc: entry
        };
        cx.set_sp(sp);
        cx
    }
}