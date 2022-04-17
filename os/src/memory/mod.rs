//! 内存管理模块
//!
//!

mod frame;
mod heap;

/// 内存管理相关的子模块
///
/// - [`heap::init`]
pub fn init() {
    heap::init();
    frame::init();
    println!("mod memory initialized!");
}
