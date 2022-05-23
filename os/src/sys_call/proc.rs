//! 进程相关系统调用子模块
use crate::interrupt::timer::get_time_ms;
use crate::task::{add_new_task, exit_current_and_run_next, suspend_current_and_run_next, get_current_process};

pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Application exit with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_get_time() -> isize {
    get_time_ms() as isize
}

pub fn sys_getpid() -> isize {
    let current = get_current_process().unwrap();
    current.pid.0 as isize
}

pub fn sys_fork() -> isize {
    let current = get_current_process().unwrap();
    let pcb = current.fork();
    let trap_cx = pcb.get_trap_cx();
    trap_cx.x[10] = 0;
    add_new_task(pcb.clone());
    println!("hello world!");
    pcb.pid.0 as isize
}

// pub fn sys_exec(path: *const u8, mut args: *const usize) -> isize {}