//! 系统设置

use crate::memory::PhysAddr;

/// 8M内核堆
pub const KERNEL_HEAP_SIZE: usize = 0x80_0000;
/// 4K页大小
pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 12;
/// QEMU内存结束地址
pub const MEMORY_END_ADDR: PhysAddr = PhysAddr(0x8800_0000);
