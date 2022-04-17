//! 中断模块
//!
//!

pub mod context;
mod handler;
mod timer;

/// 初始化中断相关的子模块
///
/// - [`handler::init`]
/// - [`timer::init`]
pub fn init() {
    handler::init();
    // timer::init();
    println!("mod interrupt initialized!");
}
