//! 文件相关系统调用子模块
use core::mem::size_of;
use core::ptr::{read, slice_from_raw_parts};

use crate::fs::inode::{open_file, OpenFlags};
use crate::fs::pipe::make_pipe;
use crate::fs::rfs::layout::DIRENT_SZ;
use crate::fs::rfs::{find_inode, get_full_path, layout::InodeType};
use crate::fs::Stat;
use crate::memory::frame::user_buffer::{get_user_buffer, get_user_string, put_user_value};
use crate::task::get_current_process;

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    let proc = get_current_process();
    let mut proc_inner = proc.inner.borrow_mut();
    let path = get_user_string(proc_inner.token(), path);
    let path = get_full_path(&proc_inner.cwd, &path);

    if let Some(inode) = open_file(&path, OpenFlags(flags)) {
        let fd = proc_inner.alloc_fd();
        proc_inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    let proc = get_current_process();
    let mut proc_inner = proc.inner.borrow_mut();
    let fd_table = &mut proc_inner.fd_table;

    if fd >= fd_table.len() || fd_table[fd].is_none() {
        return -1;
    }
    fd_table[fd].take();
    0
}

pub fn sys_read(fd: usize, buf: *mut u8, len: usize) -> isize {
    let proc = get_current_process();
    let mut proc_inner = proc.inner.borrow_mut();
    let user_buffer = get_user_buffer(proc_inner.token(), buf, len);
    let fd_table = &mut proc_inner.fd_table;

    if fd >= fd_table.len() {
        return -1;
    }
    if let Some(file) = &fd_table[fd] {
        if !file.readable() {
            return -1;
        }
        file.read(user_buffer) as isize
    } else {
        -1
    }
}

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let proc = get_current_process();
    let mut proc_inner = proc.inner.borrow_mut();
    let user_buffer = get_user_buffer(proc_inner.token(), buf, len);
    let fd_table = &mut proc_inner.fd_table;

    if fd >= fd_table.len() {
        return -1;
    }
    if let Some(file) = &fd_table[fd] {
        if !file.writable() {
            return -1;
        }
        file.write(user_buffer) as isize
    } else {
        -1
    }
}

pub fn sys_chdir(path: *const u8) -> isize {
    let proc = get_current_process();
    let mut proc_inner = proc.inner.borrow_mut();
    let path = get_user_string(proc_inner.token(), path);
    let path = get_full_path(&proc_inner.cwd, &path);

    if let Some(inode) = find_inode(&path) {
        if inode.is_dir() {
            proc_inner.cwd = path;
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
    let proc = get_current_process();
    let proc_inner = proc.inner.borrow_mut();
    let user_buffer = get_user_buffer(proc_inner.token(), buf, len);
    let cwd = proc_inner.cwd.as_bytes();

    if cwd.len() > len {
        return -1;
    }
    let mut cur_offset = 0;
    for slice in user_buffer.0.into_iter() {
        let len = slice.len().min(cwd.len() - cur_offset);
        slice[..len].copy_from_slice(&cwd[cur_offset..cur_offset + len]);
        cur_offset += len;
    }
    cwd.len() as isize
}

pub fn sys_mkdir(path: *const u8) -> isize {
    let proc = get_current_process();
    let proc_inner = proc.inner.borrow_mut();
    let path = get_user_string(proc_inner.token(), path);
    let path = get_full_path(&proc_inner.cwd, &path);

    let (parent_path, target) = path.rsplit_once('/').unwrap();
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

pub fn sys_pipe(pipe: *mut usize) -> isize {
    let proc = get_current_process();
    let mut proc_inner = proc.inner.borrow_mut();
    let (pipe_read, pipe_write) = make_pipe();
    let read_fd = proc_inner.alloc_fd();
    proc_inner.fd_table[read_fd] = Some(pipe_read);
    let write_fd = proc_inner.alloc_fd();
    proc_inner.fd_table[write_fd] = Some(pipe_write);

    put_user_value(proc_inner.token(), read_fd, pipe as *mut u8);
    put_user_value(proc_inner.token(), write_fd, unsafe { pipe.add(1) }
        as *mut u8);
    0
}

// target为源文件, link_path为link文件路径
pub fn sys_symlink(target_path: *const u8, link_path: *const u8) -> isize {
    let proc = get_current_process();
    let proc_inner = proc.inner.borrow_mut();
    let target_path = get_user_string(proc_inner.token(), target_path);
    let target_path = get_full_path(&proc_inner.cwd, &target_path);
    let link_path = get_user_string(proc_inner.token(), link_path);
    let link_path = get_full_path(&proc_inner.cwd, &link_path);

    let (parent_path, target) = link_path.rsplit_once('/').unwrap();
    if let Some(parent_inode) = find_inode(parent_path) {
        if let Some(cur_inode) = parent_inode.create(target, InodeType::SoftLink) {
            cur_inode.write_at(0, target_path.as_bytes());
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
    let proc = get_current_process();
    let mut proc_inner = proc.inner.borrow_mut();
    let fd_table = &mut proc_inner.fd_table;

    if fd >= fd_table.len() || fd_table[fd].is_none() {
        return -1;
    }
    let file = fd_table[fd].clone().unwrap();
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
    let proc = get_current_process();
    let proc_inner = proc.inner.borrow_mut();
    let path = get_user_string(proc_inner.token(), path);
    let path = get_full_path(&proc_inner.cwd, &path);
    let user_buffer = get_user_buffer(proc_inner.token(), buf, len);

    if let Some(inode) = find_inode(&path) {
        if !inode.is_link() {
            // not link file
            return -3;
        }
        if inode.get_file_size() as usize > len {
            // not big enough
            return -2;
        }
        let mut cur_offset = 0;
        for slice in user_buffer.0.into_iter() {
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
    let proc = get_current_process();
    let proc_inner = proc.inner.borrow_mut();
    let path = get_user_string(proc_inner.token(), path);
    let path = get_full_path(&proc_inner.cwd, &path);

    let (parent_path, target) = path.rsplit_once('/').unwrap();
    if let Some(inode) = find_inode(&path) {
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
    let proc = get_current_process();
    let mut proc_inner = proc.inner.borrow_mut();
    let user_buffer = get_user_buffer(proc_inner.token(), stat, size_of::<Stat>());
    let fd_table = &mut proc_inner.fd_table;

    if fd >= fd_table.len() || fd_table[fd].is_none() {
        return -1;
    }
    let file = fd_table[fd].clone().unwrap();
    let tmp_stat = Stat::from(file);
    let stat_buf = slice_from_raw_parts(&tmp_stat as *const _ as *const u8, size_of::<Stat>());
    for (i, byte) in user_buffer.into_iter().enumerate() {
        unsafe {
            *byte = (*stat_buf)[i];
        }
    }
    0
}
