//! 内存管理模块
//!
//!

mod frame;
mod heap;

pub use frame::address;
pub use frame::memory_set::MemorySet;
pub use frame::KERNEL_MEMORY_SET;
pub use frame::{R, W};

/// 内存管理相关的子模块
///
/// - [`heap::init`]
pub fn init() {
    heap::init();
    frame::init();
    println!("mod memory initialized!");
}
