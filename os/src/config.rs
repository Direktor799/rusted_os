//! 系统设置

/// 8M内核堆
pub const KERNEL_HEAP_SIZE: usize = 0x80_0000;

/// 4K页大小
pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 12;

/// QEMU内存结束地址
pub const MEMORY_END_ADDR: usize = 0x8800_0000;

/// 第一级队列时间片
pub const TASK_QUEUE_FCFS1_SLICE_MS: usize = 10;

/// 第二级队列时间片
pub const TASK_QUEUE_FCFS2_SLICE_MS: usize = 20;

/// 第三级队列时间片
pub const TASK_QUEUE_RR_SLICE_MS: usize = 50;

/// QEMU时钟频率
pub const CLOCK_FREQ: usize = 10000000;

/// 8K用户栈空间
pub const USER_STACK_SIZE: usize = 4096 * 2;

/// 8K内核栈空间
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;

/// 跳板地址
pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;

/// 用户空间上下文地址
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;

/// VirtIO总线地址区间
pub const MMIO: &[(usize, usize)] = &[(0x1000_1000, 0x1000)];
