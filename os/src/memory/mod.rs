//! 内存管理模块

pub mod frame;
pub mod heap;

/// 初始化内存管理相关的子模块
/// - [`heap::heap_allocator::init`]
/// - [`frame::frame_allocator::init`]
/// - [`frame::memory_set::init()`]
pub fn init() {
    heap::heap_allocator::init();
    frame::frame_allocator::init();
    frame::memory_set::init();
    println!("mod memory initialized!");
}
