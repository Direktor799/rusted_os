//! loader subsystem
use crate::interrupt::context::Context;
use core::arch::asm;
use core::cell::RefCell;
use core::ops::Deref;

const USER_STACK_SIZE: usize = 4096 * 2;
const KERNEL_STACK_SIZE: usize = 4096 * 2;
const MAX_APP_NUM: usize = 16;
const APP_BASE_ADDRESS: usize = 0x80400000;
const APP_SIZE_LIMIT: usize = 0x20000;

#[repr(align(4096))]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

#[repr(align(4096))]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

static KERNEL_STACK: KernelStack = KernelStack {
    data: [0; KERNEL_STACK_SIZE],
};
static USER_STACK: UserStack = UserStack {
    data: [0; USER_STACK_SIZE],
};

impl KernelStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
    pub fn push_context(&self, cx: Context) -> &'static mut Context {
        let cx_ptr = (self.get_sp() - core::mem::size_of::<Context>()) as *mut Context;
        unsafe {
            *cx_ptr = cx;
        }
        unsafe { cx_ptr.as_mut().unwrap() }
    }
}

impl UserStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

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

/// init loader subsystem
pub fn init() {
    unsafe {
        APP_MANAGER.borrow_mut().init();
        APP_MANAGER.borrow_mut().print_app_info();
    }
    println!("mod loader initialized!");
}

pub fn run_next_app() -> ! {
    unsafe {
        let mut app_manager = APP_MANAGER.borrow_mut();
        if app_manager.current_app >= app_manager.app_num {
            panic!("All done!");
        }
        let addr = app_manager.get_app_addr(app_manager.current_app);
        println!(
            "[kernel] Running app_{} at {:x}",
            app_manager.current_app, addr
        );
        let context = Context::app_init_context(addr, USER_STACK.get_sp());
        app_manager.current_app += 1;
        drop(app_manager);
        extern "C" {
            fn __restore(cx_addr: usize);
        }
        __restore(KERNEL_STACK.push_context(context) as *const _ as usize);
        unreachable!();
    }
}