//! 用户程序管理子模块
use crate::config::MAX_APP_NUM;

/// 用户程序管理器
pub struct AppManager {
    app_num: usize,
    app_start: [usize; MAX_APP_NUM + 1],
}

impl AppManager {
    ///创建空管理器
    pub const fn new() -> Self {
        Self {
            app_num: 0,
            app_start: [0; MAX_APP_NUM + 1],
        }
    }

    /// 根据汇编中的symbol初始化用户程序信息
    pub fn init(&mut self) {
        extern "C" {
            fn _app_num();
        }
        unsafe {
            let app_num_ptr = _app_num as usize as *const usize;
            let app_num = app_num_ptr.read_volatile();
            let mut app_start: [usize; MAX_APP_NUM + 1] = [0; MAX_APP_NUM + 1];
            let app_start_raw: &[usize] =
                core::slice::from_raw_parts(app_num_ptr.add(1), app_num + 1);
            app_start[..=app_num].copy_from_slice(app_start_raw);
            self.app_num = app_num;
            self.app_start = app_start;
        }
    }

    /// 获取用户程序数据
    pub fn get_app_data(&self, app_id: usize) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self.app_start[app_id] as *const u8,
                self.app_start[app_id + 1] - self.app_start[app_id],
            )
        }
    }

    /// 获取用户程序数量
    pub fn get_app_num(&self) -> usize {
        self.app_num
    }
}

/// 全局用户程序管理器实例
pub static mut APP_MANAGER: AppManager = AppManager::new();
