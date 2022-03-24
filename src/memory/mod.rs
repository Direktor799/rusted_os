//! 内存管理模块
//!
//!

mod address;
mod buddy_system;
pub mod frame;
mod heap;

pub use address::PhysAddr;

/// 内存管理相关的子模块
///
/// - [`heap::init`]
pub fn init() {
    heap::init();
    frame::init();
}
