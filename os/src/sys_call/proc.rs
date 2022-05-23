//! 进程相关系统调用子模块
use crate::interrupt::timer::get_time_ms;
use crate::task::{
    add_new_task, exit_current_and_run_next, get_current_process, suspend_current_and_run_next,
};

pub fn sys_exit(exit_code: i32) -> ! {
    let proc = get_current_process();
    println!(
        "[kernel] Process {} exit with code {}",
        proc.pid.0, exit_code
    );
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

pub fn sys_getpid() -> isize {
    let proc = get_current_process();
    proc.pid.0 as isize
}

pub fn sys_fork() -> isize {
    let proc = get_current_process();
    let new_proc = proc.fork();
    let trap_cx = new_proc.inner.borrow().trap_cx();
    trap_cx.x[10] = 0;
    add_new_task(new_proc.clone());
    new_proc.pid.0 as isize
}

// pub fn sys_exec(path: *const u8, mut args: *const usize) -> isize {}
