//! 内存管理模块
//!
//!

mod allocator;
mod heap;

/// 内存管理相关的子模块
///
/// - [`heap::init`]
pub fn init() {
    heap::init();
    println!("mod memory initialized");
}
