use riscv::register::sstatus::{self, Sstatus, SPP};

/// 中断切换上下文
#[repr(C)]
#[derive(Debug)]
pub struct Context {
    pub x: [usize; 32],   // 32个通用寄存器
    pub sstatus: Sstatus, // CSR寄存器
    pub sepc: usize,      // CSR寄存器
}

impl Context {
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }
    pub fn app_init_context(entry: usize, sp: usize) -> Self {
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        let mut cx = Self {
            x: [0; 32],
            sstatus,
            sepc: entry,
        };
        cx.set_sp(sp);
        cx
    }
}
