//! 文件相关系统调用子模块
use crate::memory::frame::page_table::{get_user_buffer_in_kernel, get_user_string_in_kernel};
use crate::os_fs::{open_file, OpenFlags};
use crate::task::{TASK_MANAGER};
const FD_STDIN: usize = 0;
const FD_STDOUT: usize = 1;
pub fn sys_open(path: *const u8, flags: u32) -> isize {
    let user_satp_token = unsafe { TASK_MANAGER.get_current_token() };
    let user_buffer_path = get_user_string_in_kernel(user_satp_token, path);
    let task = unsafe { TASK_MANAGER.get_current_task() };

    if let Some(inode) = open_file(user_buffer_path.as_str(), OpenFlags(flags)) {
        let fd = task.alloc_fd();
        task.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}
pub fn sys_read(fd: usize, buf: *mut u8, len: usize) -> isize {
    let user_satp_token = unsafe { TASK_MANAGER.get_current_token() };
    let task = unsafe { TASK_MANAGER.get_current_task() };
    if fd >= task.fd_table.len() {
        return -1;
    }
    if let Some(file) = &task.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        drop(task);
        let user_buffer = get_user_buffer_in_kernel(user_satp_token, buf, len);
        file.read(user_buffer) as isize
    } else {
        -1
    }
}

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let user_satp_token = unsafe { TASK_MANAGER.get_current_token() };
    let task = unsafe { TASK_MANAGER.get_current_task() };
    if fd >= task.fd_table.len() {
        return -1;
    }
    if let Some(file) = &task.fd_table[fd] {
        let file = file.clone();
        if !file.writable() {
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        drop(task);
        let user_buffer = get_user_buffer_in_kernel(user_satp_token, buf, len);
        file.write(user_buffer) as isize
    } else {
        -1
    }
}

