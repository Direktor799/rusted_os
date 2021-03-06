//! 中断上下文子模块

/// 中断切换上下文
#[repr(C)]
#[derive(Debug)]
pub struct Context {
    pub x: [usize; 32], // 32个通用寄存器
    pub sstatus: usize, // CSR寄存器
    pub sepc: usize,    // CSR寄存器
    pub kernel_satp: usize,
    pub kernel_sp: usize,
    pub interrupt_handler: usize,
}

impl Context {
    /// 设置上下文恢复后的sp地址
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }

    /// 初始化App首次运行的上下文
    pub fn app_init_context(
        entry: usize,
        sp: usize,
        kernel_satp: usize,
        kernel_sp: usize,
        interrupt_handler: usize,
    ) -> Self {
        let mut sstatus;
        unsafe {
            core::arch::asm!("csrr {}, sstatus", out(reg) sstatus);
        }
        // set SPP bit
        sstatus &= 1 << 8;
        let mut cx = Self {
            x: [0; 32],
            sstatus,
            sepc: entry,
            kernel_satp,
            kernel_sp,
            interrupt_handler,
        };
        cx.set_sp(sp);
        cx
    }
}
