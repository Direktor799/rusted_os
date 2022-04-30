//! 用户程序加载模块

pub mod app_manager;
pub mod elf_decoder;

use app_manager::APP_MANAGER;

/// 初始化用户程序管理器
/// - [`app_manager::AppManager::init`]
pub fn init() {
    unsafe {
        APP_MANAGER.init();
    }
    println!("mod loader initialized!");
}
