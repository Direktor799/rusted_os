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
    let old_pid = current.pid.0;
    let pcb = current.fork();
    let current = get_current_process();
    if current.unwrap().pid.0 == old_pid {
        pcb.pid.0 as isize
    } else {
        add_new_task(pcb);
        0
    }
}

// pub fn sys_exec(path: *const u8, mut args: *const usize) -> isize {}