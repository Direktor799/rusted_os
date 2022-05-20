//! 文件相关系统调用子模块
use crate::fs::inode::{open_file, OpenFlags};
use crate::fs::rfs::{find_inode, get_full_path, layout::InodeType};
use crate::memory::frame::page_table::{get_user_buffer_in_kernel, get_user_string_in_kernel};
use crate::task::TASK_MANAGER;

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    let user_satp_token = unsafe { TASK_MANAGER.get_current_token() };
    let user_buffer_path = get_user_string_in_kernel(user_satp_token, path);
    let task = unsafe { TASK_MANAGER.current_task.as_mut().unwrap() };
    let cwd = get_full_path(&task.cwd, &user_buffer_path);
    if let Some(inode) = open_file(&cwd, OpenFlags(flags)) {
        let fd = task.alloc_fd();
        task.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    let task = unsafe { TASK_MANAGER.current_task.as_mut().unwrap() };
    if fd >= task.fd_table.len() {
        return -1;
    }
    if task.fd_table[fd].is_none() {
        return -1;
    }
    task.fd_table[fd].take();
    0
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
    let target_path = get_user_string_in_kernel(user_satp_token, path);
    let task = unsafe { TASK_MANAGER.current_task.as_mut().unwrap() };
    let cwd = get_full_path(&task.cwd, &target_path);
    if let Some(inode) = find_inode(&cwd) {
        if inode.is_dir() {
            task.cwd = cwd;
            0
        } else {
            // not dir
            -2
        }
    } else {
        // no such file
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

pub fn sys_mkdir(path: *const u8) -> isize {
    let user_satp_token = unsafe { TASK_MANAGER.get_current_token() };
    let target_path = get_user_string_in_kernel(user_satp_token, path);
    let task = unsafe { TASK_MANAGER.current_task.as_mut().unwrap() };
    let new_path = get_full_path(&task.cwd, &target_path);
    let (parent_path, target) = new_path.rsplit_once('/').unwrap();
    if let Some(parent_inode) = find_inode(parent_path) {
        if let Some(cur_inode) = parent_inode.create(target, InodeType::Directory) {
            cur_inode.set_default_dirent(parent_inode.get_inode_id());
            0
        } else {
            // file exists
            -2
        }
    } else {
        // no such file
        -1
    }
}

// target为源文件, link_path为link文件路径
pub fn sys_symlink(target: *const u8, link_path: *const u8) -> isize {
    //获取userbuffer
    let user_satp_token = unsafe { TASK_MANAGER.get_current_token() };
    let target_path = get_user_string_in_kernel(user_satp_token, target);
    let linkfile_path = get_user_string_in_kernel(user_satp_token, link_path);
    //处理相对路径
    let task = unsafe { TASK_MANAGER.current_task.as_mut().unwrap() };

    let new_target_path = get_full_path(&task.cwd, &target_path);
    let new_linkfile_path = get_full_path(&task.cwd, &linkfile_path);

    let (parent_path, link_target) = new_linkfile_path.rsplit_once('/').unwrap();
    println!("{} {} {} \n", new_linkfile_path, parent_path, link_target);

    if let Some(parent_inode) = find_inode(parent_path) {
        if let Some(cur_inode) = parent_inode.create(link_target, InodeType::SoftLink) {
            cur_inode.write_at(0, new_target_path.as_bytes());
            0
        } else {
            // file exists
            -2
        }
    } else {
        // no such file
        -1
    }
}

pub fn sys_lseek(fd: usize, offset: isize, origin: i32) -> isize {
    let task = unsafe { TASK_MANAGER.current_task.as_mut().unwrap() };
    if fd >= task.fd_table.len() {
        return -1;
    }
    if task.fd_table[fd].is_none() {
        return -1;
    }
    // let file = task.fd_table[fd].as_ref().unwrap();
    //这里怎么获取这两个变量呢
    let mut file_offset: isize = 0;
    let file_size: isize = 0;

    let mut new_offset: isize;
    match origin {
        0 => new_offset = offset,
        1 => new_offset = file_offset + offset,
        2 => new_offset = file_size + offset,
        _ => panic!(),
    }
    if new_offset < 0 {
        -1
    }
    else {
        file_offset = new_offset;
        new_offset
    }
}
