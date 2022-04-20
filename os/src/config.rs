//! 系统设置

/// 2M内核堆
pub const KERNEL_HEAP_SIZE: usize = 0x20_0000;
/// 4K页大小
pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 12;
/// QEMU内存结束地址
pub const MEMORY_END_ADDR: PhysAddr = PhysAddr(0x8800_0000);

/// 第一级队列时间片
pub const TASK_QUEUE_FCFS1_SLICE: usize = 4000;
/// 第二级队列时间片
pub const TASK_QUEUE_FCFS2_SLICE: usize = 8000;
/// 第三级队列时间片
pub const TASK_QUEUE_RR_SLICE: usize = 12000;

/// QEMU时钟频率
pub const CLOCK_FREQ: usize = 10000000;
