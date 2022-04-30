//! 内存管理模块

pub mod frame;
pub mod heap;

/// 初始化内存管理相关的子模块
/// - [`heap::init`]
/// - [`frame::init`]
pub fn init() {
    heap::init();
    frame::init();
    println!("mod memory initialized!");
}
