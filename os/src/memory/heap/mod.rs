//! 堆内存管理子模块

use crate::config::KERNEL_HEAP_SIZE;
use allocator::HEAP_ALLOCATOR;

mod allocator;
mod linked_list;

/// 内核堆
static mut KERNEL_HEAP: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

/// 初始化堆内存分配器
/// - [`allocator::BuddySystemAllocator::init`]
pub fn init() {
    unsafe {
        HEAP_ALLOCATOR
            .borrow_mut()
            .init(KERNEL_HEAP.as_ptr() as usize, KERNEL_HEAP_SIZE);
    }
}
