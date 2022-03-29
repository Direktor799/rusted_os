//! 内存管理模块
//!
//!

mod address;
mod buddy_system;
mod frame;
mod heap;
mod page_table;

pub use address::PhysAddr;

/// 内存管理相关的子模块
///
/// - [`heap::init`]
pub fn init() {
    heap::init();
    frame::init();
}
