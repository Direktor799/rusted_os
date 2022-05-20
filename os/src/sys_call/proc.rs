//! 进程相关系统调用子模块
use crate::interrupt::timer::get_time_ms;
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next};

pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Application exit with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_gettime() -> isize {
    get_time_ms() as isize
}
