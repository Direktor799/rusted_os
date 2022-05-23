//! 文件相关系统调用子模块
use core::mem::size_of;
use core::ptr::slice_from_raw_parts;

use crate::fs::inode::{open_file, OpenFlags};
use crate::fs::rfs::layout::DIRENT_SZ;
use crate::fs::rfs::{find_inode, get_full_path, layout::InodeType};
use crate::fs::Stat;
use crate::memory::frame::page_table::{get_user_buffer_in_kernel, get_user_string_in_kernel};
use crate::task::TASK_MANAGER;

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    let user_satp_token = unsafe { TASK_MANAGER.get_current_token() };
    let user_buffer_path = get_user_string_in_kernel(user_satp_token, path);
    let task = unsafe { TASK_MANAGER.current_task.as_mut().unwrap() };
    let mut task_inner = task.inner.borrow_mut();
    let cwd = get_full_path(&task_inner.cwd, &user_buffer_path);
    if let Some(inode) = open_file(&cwd, OpenFlags(flags)) {
        let fd = task_inner.alloc_fd();
        task_inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    let task = unsafe { TASK_MANAGER.current_task.as_mut().unwrap() };
    let mut task_inner = task.inner.borrow_mut();
    if fd >= task_inner.fd_table.len() {
        return -1;
    }
    if task_inner.fd_table[fd].is_none() {
        return -1;
    }
    task_inner.fd_table[fd].take();
    0
}

pub fn sys_read(fd: usize, buf: *mut u8, len: usize) -> isize {
    let user_satp_token = unsafe { TASK_MANAGER.get_current_token() };
    let task = unsafe { TASK_MANAGER.current_task.as_mut().unwrap() };
    let mut task_inner = task.inner.borrow_mut();
    let fd_table = &mut task_inner.fd_table;
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
    let task = unsafe { TASK_MANAGER.current_task.as_mut().unwrap() };
    let mut task_inner = task.inner.borrow_mut();
    let fd_table = &mut task_inner.fd_table;
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
    let mut task_inner = task.inner.borrow_mut();
    let cwd = get_full_path(&task_inner.cwd, &target_path);
    if let Some(inode) = find_inode(&cwd) {
        if inode.is_dir() {
            task_inner.cwd = cwd;
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

pub fn sys_getcwd(buf: *const u8, len: usize) -> isize {
    let user_satp_token = unsafe { TASK_MANAGER.get_current_token() };
    let mut user_buffer = get_user_buffer_in_kernel(user_satp_token, buf, len);
    let task = unsafe { TASK_MANAGER.current_task.as_mut().unwrap() };
    let task_inner = task.inner.borrow();
    let cwd = task_inner.cwd.as_bytes();
    if cwd.len() > len {
        return -1;
    }
    let mut cur_offset = 0;
    for slice in user_buffer.0.iter_mut() {
        let len = slice.len().min(cwd.len() - cur_offset);
        slice[..len].copy_from_slice(&cwd[cur_offset..cur_offset + len]);
        cur_offset += len;
    }
    cwd.len() as isize
}

pub fn sys_mkdir(path: *const u8) -> isize {
    let user_satp_token = unsafe { TASK_MANAGER.get_current_token() };
    let target_path = get_user_string_in_kernel(user_satp_token, path);
    let task = unsafe { TASK_MANAGER.current_task.as_mut().unwrap() };
    let task_inner = task.inner.borrow();
    let new_path = get_full_path(&task_inner.cwd, &target_path);
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
    // 获取userbuffer
    let user_satp_token = unsafe { TASK_MANAGER.get_current_token() };
    let target_path = get_user_string_in_kernel(user_satp_token, target);
    let linkfile_path = get_user_string_in_kernel(user_satp_token, link_path);
    // 处理相对路径
    let task = unsafe { TASK_MANAGER.current_task.as_mut().unwrap() };
    let mut task_inner = task.inner.borrow_mut();

    let new_target_path = get_full_path(&task_inner.cwd, &target_path);
    let new_linkfile_path = get_full_path(&task_inner.cwd, &linkfile_path);

    let (parent_path, link_target) = new_linkfile_path.rsplit_once('/').unwrap();

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

const SEEK_SET: u32 = 0;
const SEEK_CUR: u32 = 1;
const SEEK_END: u32 = 2;

pub fn sys_lseek(fd: usize, offset: isize, whence: u32) -> isize {
    let task = unsafe { TASK_MANAGER.current_task.as_mut().unwrap() };
    let mut task_inner = task.inner.borrow_mut();

    if fd >= task_inner.fd_table.len() {
        return -1;
    }
    if task_inner.fd_table[fd].is_none() {
        return -1;
    }
    let file = task_inner.fd_table[fd].as_ref().unwrap();
    let cur_offset = file.get_offset() as isize;
    let file_size = file.get_file_size() as isize;
    let new_offset = match whence {
        SEEK_SET => offset,
        SEEK_CUR => cur_offset + offset,
        SEEK_END => file_size + offset,
        _ => return -2,
    };
    if new_offset < 0 {
        -1
    } else {
        file.set_offset(new_offset as usize);
        new_offset
    }
}

pub fn sys_readlink(path: *const u8, buf: *const u8, len: usize) -> isize {
    let user_satp_token = unsafe { TASK_MANAGER.get_current_token() };
    let target_path = get_user_string_in_kernel(user_satp_token, path);
    let mut user_buffer = get_user_buffer_in_kernel(user_satp_token, buf, len);
    let task = unsafe { TASK_MANAGER.current_task.as_mut().unwrap() };
    let mut task_inner = task.inner.borrow_mut();

    let new_path = get_full_path(&task_inner.cwd, &target_path);
    if let Some(inode) = find_inode(&new_path) {
        if !inode.is_link() {
            // not link file
            return -3;
        }
        if inode.get_file_size() as usize > len {
            // not big enough
            return -2;
        }
        let mut cur_offset = 0;
        for slice in user_buffer.0.iter_mut() {
            let len = slice.len().min(inode.get_file_size() as usize - cur_offset);
            inode.read_at(cur_offset, &mut slice[cur_offset..cur_offset + len]);
            cur_offset += len;
        }
        inode.get_file_size() as isize
    } else {
        // no such file
        -1
    }
}

const AT_REMOVEDIR: u32 = 1;

pub fn sys_unlink(path: *const u8, flags: u32) -> isize {
    let user_satp_token = unsafe { TASK_MANAGER.get_current_token() };
    let target_path = get_user_string_in_kernel(user_satp_token, path);
    let task = unsafe { TASK_MANAGER.current_task.as_mut().unwrap() };
    let mut task_inner = task.inner.borrow_mut();

    let new_path = get_full_path(&task_inner.cwd, &target_path);
    let (parent_path, target) = new_path.rsplit_once('/').unwrap();
    if let Some(inode) = find_inode(&new_path) {
        if flags & AT_REMOVEDIR == 0 && !inode.is_dir() {
            inode.clear();
            find_inode(parent_path).unwrap().delete(target);
            0
        } else if flags & AT_REMOVEDIR == 1 && inode.is_dir() {
            if inode.get_file_size() as usize == DIRENT_SZ * 2 {
                inode.clear();
                find_inode(parent_path).unwrap().delete(target);
                0
            } else {
                // not empty
                -3
            }
        } else {
            // type not matched
            -2
        }
    } else {
        // no such file
        -1
    }
}

pub fn sys_fstat(fd: usize, stat: *mut u8) -> isize {
    let user_satp_token = unsafe { TASK_MANAGER.get_current_token() };
    let user_buffer = get_user_buffer_in_kernel(user_satp_token, stat, size_of::<Stat>());
    let task = unsafe { TASK_MANAGER.current_task.as_mut().unwrap() };
    let mut task_inner = task.inner.borrow_mut();

    if fd >= task_inner.fd_table.len() {
        return -1;
    }
    if task_inner.fd_table[fd].is_none() {
        return -1;
    }
    let file = task_inner.fd_table[fd].as_ref().unwrap();
    let tmp_stat = Stat::from(file);
    let stat_buf = slice_from_raw_parts(&tmp_stat as *const _ as *const u8, size_of::<Stat>());
    for (i, byte) in user_buffer.into_iter().enumerate() {
        unsafe {
            *byte = (*stat_buf)[i];
        }
    }
    0
}
