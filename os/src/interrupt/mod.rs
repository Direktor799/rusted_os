//! 中断处理模块

pub mod context;
pub mod handler;
pub mod timer;

/// 初始化中断相关的子模块
/// - [`handler::init`]
/// - [`timer::init`]
pub fn init() {
    handler::init();
    timer::init();
    println!("mod interrupt initialized!");
}
