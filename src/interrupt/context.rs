use riscv::register::sstatus::Sstatus;

#[repr(C)]
#[derive(Debug)]
/// 中断切换上下文
pub struct Context {
    pub x: [usize; 32],   // 32个通用寄存器
    pub sstatus: Sstatus, // CSR寄存器
    pub sepc: usize,      // CSR寄存器
}
