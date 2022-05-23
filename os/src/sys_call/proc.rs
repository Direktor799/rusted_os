//! 进程相关系统调用子模块

use alloc::vec;

use crate::fs::rfs::{find_inode, get_full_path};
use crate::interrupt::timer::get_time_ms;
use crate::memory::frame::page_table::get_user_string_in_kernel;
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

// TODO: args and ret
pub fn sys_exec(path: *const u8) -> isize {
    let proc = get_current_process();
    let proc_inner = proc.inner.borrow();
    let path = get_user_string_in_kernel(proc_inner.token(), path);
    let path = get_full_path(&proc_inner.cwd, &path);
    drop(proc_inner);
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
    if let Some(app_inode) = find_inode(&path) {
        let size = app_inode.get_file_size() as usize;
        let mut app_data = vec![0u8; size];
        app_inode.read_at(0, &mut app_data);
        proc.exec(app_data.as_slice());
        // return argc because cx.x[10] will be covered with it later
        0
    } else {
        -1
    }
}
