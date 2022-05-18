//! 文件相关系统调用子模块
use crate::fs::inode::{open_file, OpenFlags};
use crate::fs::rfs::{extend_path, find_inode,layout::InodeType::Directory};
use crate::memory::frame::page_table::{get_user_buffer_in_kernel, get_user_string_in_kernel};
use crate::task::TASK_MANAGER;
use alloc::string::String;
pub fn sys_open(path: *const u8, flags: u32) -> isize {
    let user_satp_token = unsafe { TASK_MANAGER.get_current_token() };
    let user_buffer_path = get_user_string_in_kernel(user_satp_token, path);
    let task = unsafe { TASK_MANAGER.current_task.as_mut().unwrap() };
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
    let fd_table = unsafe { &mut TASK_MANAGER.current_task.as_mut().unwrap().fd_table };
    if fd >= fd_table.len() {
        return -1;
    }
    if let Some(file) = &fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        let user_buffer = get_user_buffer_in_kernel(user_satp_token, buf, len);
        file.read(user_buffer) as isize
    } else {
        -1
    }
}

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let user_satp_token = unsafe { TASK_MANAGER.get_current_token() };
    let fd_table = unsafe { &mut TASK_MANAGER.current_task.as_mut().unwrap().fd_table };
    if fd >= fd_table.len() {
        return -1;
    }
    if let Some(file) = &fd_table[fd] {
        let file = file.clone();
        if !file.writable() {
            return -1;
        }
        let user_buffer = get_user_buffer_in_kernel(user_satp_token, buf, len);
        file.write(user_buffer) as isize
    } else {
        -1
    }
}

pub fn sys_chdir(path: *const u8) -> isize {
    let user_satp_token = unsafe { TASK_MANAGER.get_current_token() };
    let task = unsafe { TASK_MANAGER.current_task.as_mut().unwrap() };
    let target_path = get_user_string_in_kernel(user_satp_token, path);
    let folded_cwd = String::from(&task.cwd) + "/" + &target_path;
    let cwd = extend_path(folded_cwd);
    if let Some(_) = find_inode(&cwd) {
        task.cwd = cwd;
        0
    } else {
        -1
    }
}

pub fn sys_get_cwd(buf: *const u8, len: usize) -> isize {
    let user_satp_token = unsafe { TASK_MANAGER.get_current_token() };
    let mut user_buffer = get_user_buffer_in_kernel(user_satp_token, buf, len);
    let cwd = unsafe { TASK_MANAGER.current_task.as_ref().unwrap().cwd.as_bytes() };
    if cwd.len() > len {
        return -1;
    }
    let mut cur_offset = 0;
    for slice in user_buffer.0.iter_mut() {
        let len = slice.len().min(cwd.len() - cur_offset);
        slice[..len].copy_from_slice(&cwd[cur_offset..cur_offset + len]);
        cur_offset += len;
    }
    0
}

pub fn sys_mkdir(path: *const u8) -> isize{
    let user_satp_token = unsafe { TASK_MANAGER.get_current_token() };
    let user_buffer_path = get_user_string_in_kernel(user_satp_token, path);
    let (parent_path, target) = user_buffer_path.rsplit_once('/').unwrap();
    let parent_inode = find_inode(parent_path);
    let cur_inode = parent_inode.create(target, Directory);
    cur_inode.set_default_dirent(parent_inode.get_inode_id());
    0
}

pub fn sys_rmdir(path: *const u8) -> isize{
    let user_satp_token = unsafe { TASK_MANAGER.get_current_token() };
    let user_buffer_path = get_user_string_in_kernel(user_satp_token, path);
    if remove_dir(user_buffer_path.as_str()).is_some(){
        0
    }
    else {
        -1
    }
}