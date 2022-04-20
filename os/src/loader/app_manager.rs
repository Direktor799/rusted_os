//! loader subsystem
use crate::config::{
    APP_BASE_ADDRESS, APP_SIZE_LIMIT, KERNEL_STACK_SIZE, MAX_APP_NUM, USER_STACK_SIZE,
};
use crate::interrupt::Context;
use core::arch::asm;
use core::cell::RefCell;
use core::ops::Deref;

pub struct AppManager {
    app_num: usize,
    current_app: usize,
    app_start: [usize; MAX_APP_NUM + 1],
}

impl AppManager {
    pub const fn new() -> Self {
        Self {
            app_num: 0,
            current_app: 0,
            app_start: [0; MAX_APP_NUM + 1],
        }
    }

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
            asm!("fence.i");
            for app_id in 0..self.app_num {
                let app_addr = self.get_app_addr(app_id);
                // println!("loading app_{} to 0x{:x}", app_id, app_addr);
                core::slice::from_raw_parts_mut(app_addr as *mut u8, APP_SIZE_LIMIT).fill(0);
                let app_src = core::slice::from_raw_parts(
                    self.app_start[app_id] as *const u8,
                    self.app_start[app_id + 1] - self.app_start[app_id],
                );
                let app_dst = core::slice::from_raw_parts_mut(app_addr as *mut u8, app_src.len());
                app_dst.copy_from_slice(app_src);
            }
        }
    }

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

    pub fn get_app_addr(&self, app_id: usize) -> usize {
        APP_BASE_ADDRESS + app_id * APP_SIZE_LIMIT
    }

    pub fn get_app_data(&self, app_id: usize) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self.app_start[app_id] as *const u8,
                self.app_start[app_id + 1] - self.app_start[app_id],
            )
        }
    }

    pub fn get_app_num(&self) -> usize {
        self.app_num
    }
}

pub struct OutsideAppManager(RefCell<AppManager>);

impl OutsideAppManager {
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

pub static mut APP_MANAGER: OutsideAppManager = OutsideAppManager::new();
