//! 用户程序管理子模块
use crate::config::{KERNEL_STACK_SIZE, MAX_APP_NUM, USER_STACK_SIZE};
use crate::interrupt::context::Context;
use core::arch::asm;
use core::cell::RefCell;
use core::ops::Deref;

/// 用户程序管理器
pub struct AppManager {
    app_num: usize,
    current_app: usize,
    app_start: [usize; MAX_APP_NUM + 1],
}

impl AppManager {
    ///创建空管理器
    pub const fn new() -> Self {
        Self {
            app_num: 0,
            current_app: 0,
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

    /// 输出用户程序信息
    pub fn print_app_info(&self) {
        println!("[kernel] app_num = {}", self.app_num);
        for i in 0..self.app_num {
            println!(
                "[kernel] app_{} [{:#x}, {:#x})",
                i,
                self.app_start[i],
                self.app_start[i + 1]
            );
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

/// 单线程的用户程序管理器
pub struct OutsideAppManager(RefCell<AppManager>);

impl OutsideAppManager {
    /// 创建新的单线程用户程序管理器
    pub const fn new() -> Self {
        Self(RefCell::new(AppManager::new()))
    }
}

impl Deref for OutsideAppManager {
    type Target = RefCell<AppManager>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// 全局用户程序管理器实例
pub static mut APP_MANAGER: OutsideAppManager = OutsideAppManager::new();
