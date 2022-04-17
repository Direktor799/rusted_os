//! 系统设置

/// 2M内核堆
pub const KERNEL_HEAP_SIZE: usize = 0x20_0000;
/// 4K页大小
pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 12;
/// QEMU内存结束地址
pub const MEMORY_END_ADDR: usize = 0x8800_0000;
