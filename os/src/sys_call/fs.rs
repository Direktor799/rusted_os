//! 文件相关系统调用子模块
use crate::memory::frame::page_table::{get_user_buffer_in_kernel, get_user_string_in_kernel};
use crate::os_fs::{open_file, OpenFlags};
use crate::task::TASK_MANAGER;
const FD_STDIN: usize = 0;
const FD_STDOUT: usize = 1;
// pub fn sys_open(path: *const u8, flags: u32) -> isize {
//     let user_satp_token = unsafe { TASK_MANAGER.get_current_token() };
//     let user_buffer_path = get_user_string_in_kernel(user_satp_token, path);
//     let task = unsafe { TASK_MANAGER.get_current_task() };
//     let mut task_inner = task.inner.borrow_mut();

//     if let Some(inode) = open_file(user_buffer_path.as_str(), OpenFlags(flags)) {
//         let fd = task_inner.alloc_fd();
//         task_inner.fd_table[fd] = Some(inode);
//         fd as isize
//     } else {
//         -1
//     }
// }
pub fn sys_read(fd: usize, buf: *mut u8, len: usize) -> isize {
    let user_satp_token = unsafe { TASK_MANAGER.get_current_token() };
    let mut inner = unsafe { TASK_MANAGER.0.as_ref().unwrap().borrow_mut() };
    let fd_table = &mut inner.current_task.as_mut().unwrap().fd_table;
    if fd >= fd_table.len() {
        return -1;
    }
    if let Some(file) = &fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        drop(inner);
        let user_buffer = get_user_buffer_in_kernel(user_satp_token, buf, len);
        file.read(user_buffer) as isize
    } else {
        -1
    }
}

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let user_satp_token = unsafe { TASK_MANAGER.get_current_token() };
    let mut inner = unsafe { TASK_MANAGER.0.as_ref().unwrap().borrow_mut() };
    let fd_table = &mut inner.current_task.as_mut().unwrap().fd_table;
    if fd >= fd_table.len() {
        return -1;
    }
    if let Some(file) = &fd_table[fd] {
        let file = file.clone();
        if !file.writable() {
            return -1;
        }
        drop(inner);
        let user_buffer = get_user_buffer_in_kernel(user_satp_token, buf, len);
        file.write(user_buffer) as isize
    } else {
        -1
    }
}
