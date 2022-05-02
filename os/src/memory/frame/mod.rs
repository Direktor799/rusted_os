//! 页式内存管理子模块
use crate::config::{MEMORY_END_ADDR, PAGE_SIZE};
use address::{PhysAddr, PhysPageNum};
use frame::FRAME_ALLOCATOR;
use memory_set::{MemorySet, KERNEL_MEMORY_SET};

pub mod address;
pub mod frame;
pub mod memory_set;
pub mod page_table;
pub mod segment;
pub mod user_buffer;

/// 初始化页式内存分配器和内核地址空间
/// - [`frame::FrameAllocator::init`]
/// - [`memory_set::MemorySet::new_kernel`]
/// - [`memory_set::MemorySet::activate`]
pub fn init() {
    extern "C" {
        fn kernel_end(); // 在linker script中定义的内存地址
    }
    let kernel_end_addr = kernel_end as usize;
    let frame_start_num = PhysPageNum((kernel_end_addr + PAGE_SIZE - 1) / PAGE_SIZE);
    unsafe {
        FRAME_ALLOCATOR.init(frame_start_num, PhysAddr(MEMORY_END_ADDR).ppn());
    }
    unsafe {
        KERNEL_MEMORY_SET = Some(MemorySet::new_kernel());
        KERNEL_MEMORY_SET.as_ref().unwrap().activate();
    }
}
