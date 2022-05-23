//! 进程相关系统调用子模块
use alloc::vec::Vec;

use crate::interrupt::timer::get_time_ms;
use crate::memory::frame::page_table::get_user_string_in_kernel;
use crate::task::{
    add_new_task, exit_current_and_run_next, get_current_process, suspend_current_and_run_next,
};

pub fn sys_exit(exit_code: i32) -> ! {
    let current = get_current_process().unwrap();
    println!(
        "[kernel] Process {} exit with code {}",
        current.pid.0, exit_code
    );
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
    let new = current.fork();
    let trap_cx = new.get_trap_cx();
    trap_cx.x[10] = 0;
    add_new_task(new.clone());
    new.pid.0 as isize
}

pub fn sys_exec(path: *const u8, mut args: *const usize) -> isize {
    let current = get_current_process().unwrap();
    let token = current.get_user_token();
    let path = get_user_string_in_kernel(token, path);
    // let mut args_vec: Vec<String> = Vec::new();
    // loop {
    //     let arg_str_ptr = *translated_ref(token, args);
    //     if arg_str_ptr == 0 {
    //         break;
    //     }
    //     args_vec.push(get_user_string_in_kernel(token, arg_str_ptr as *const u8));
    //     unsafe {
    //         args = args.add(1);
    //     }
    // }
    // if let Some(app_inode) = open_file(path.as_str(), OpenFlags::RDONLY) {
    //     let all_data = app_inode.read_all();
    //     let process = get_current_process().unwrap();
    //     let argc = args_vec.len();
    //     process.exec(all_data.as_slice(), args_vec);
    //     // return argc because cx.x[10] will be covered with it later
    //     argc as isize
    // } else {
    //     -1
    // }
    -1
}
