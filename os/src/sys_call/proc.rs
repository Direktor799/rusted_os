//! 进程相关系统调用子模块

use core::mem::size_of;
use core::ptr::slice_from_raw_parts;

use alloc::rc::Rc;
use alloc::vec;

use crate::fs::rfs::{find_inode, get_full_path};
use crate::interrupt::timer::get_time_ms;
use crate::memory::frame::page_table::{get_user_buffer_in_kernel, get_user_string_in_kernel};
use crate::task::{
    add_new_task, exit_current_and_run_next, get_current_process, suspend_current_and_run_next,
    TaskStatus,
};

pub fn sys_exit(exit_code: i32) -> ! {
    exit_current_and_run_next(exit_code);
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

// TODO: args
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

pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut u8) -> isize {
    let process = get_current_process();
    let mut inner = process.inner.borrow_mut();
    let user_buffer = get_user_buffer_in_kernel(inner.token(), exit_code_ptr, size_of::<i32>());

    if !inner
        .children
        .iter()
        .any(|child| pid == -1 || pid as usize == child.pid.0)
    {
        // child not found
        return -1;
    }

    let pair = inner.children.iter().enumerate().find(|(_, child)| {
        child.inner.borrow().task_status == TaskStatus::Exited
            && (pid == -1 || pid as usize == child.pid.0)
    });

    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        assert_eq!(Rc::strong_count(&child), 1);
        let child_pid = child.pid.0;
        let exit_code = child.inner.borrow().exit_code;
        let exit_code_buf =
            slice_from_raw_parts(&exit_code as *const _ as *const u8, size_of::<i32>());
        for (i, byte) in user_buffer.into_iter().enumerate() {
            unsafe {
                *byte = (*exit_code_buf)[i];
            }
        }
        child_pid as isize
    } else {
        // child running
        -2
    }
}
