use core::arch::asm;

const SYS_CALL_GETCWD: usize = 17;
const SYS_CALL_MKDIR: usize = 34;
const SYS_CALL_UNLINK: usize = 35;
const SYS_CALL_SYMLINK: usize = 36;
const SYS_CALL_CHDIR: usize = 49;
const SYS_CALL_OPEN: usize = 56;
const SYS_CALL_CLOSE: usize = 57;
const SYS_CALL_LSEEK: usize = 62;
const SYS_CALL_READ: usize = 63;
const SYS_CALL_WRITE: usize = 64;
const SYS_CALL_READLINK: usize = 78;
const SYS_CALL_FSTAT: usize = 80;
const SYS_CALL_EXIT: usize = 93;
const SYS_CALL_YIELD: usize = 124;
const SYS_CALL_GETTIME: usize = 169;

fn sys_call(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x17") id
        );
    }
    ret
}

pub fn sys_read(fd: usize, buffer: &mut [u8]) -> isize {
    sys_call(
        SYS_CALL_READ,
        [fd, buffer.as_mut_ptr() as usize, buffer.len()],
    )
}

pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    sys_call(SYS_CALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}

pub fn sys_exit(exit_code: i32) -> isize {
    sys_call(SYS_CALL_EXIT, [exit_code as usize, 0, 0])
}

pub fn sys_yield() -> isize {
    sys_call(SYS_CALL_YIELD, [0, 0, 0])
}

pub fn sys_gettime() -> isize {
    sys_call(SYS_CALL_GETTIME, [0, 0, 0])
}

pub fn sys_getcwd(buf: &mut [u8]) -> isize {
    sys_call(SYS_CALL_GETCWD, [buf.as_ptr() as usize, buf.len(), 0])
}

pub fn sys_chdir(path: *const u8) -> isize {
    sys_call(SYS_CALL_CHDIR, [path as usize, 0, 0])
}

pub fn sys_mkdir(path: *const u8) -> isize {
    sys_call(SYS_CALL_MKDIR, [path as usize, 0, 0])
}

// 返回值为-1表示open失败，否则返回句柄(>=0)
pub fn sys_open(path: *const u8, flags: u32) -> isize {
    sys_call(SYS_CALL_OPEN, [path as usize, flags as usize, 0])
}
// 返回值为-1表示close失败，为0表示执行成功
pub fn sys_close(fd: usize) -> isize {
    sys_call(SYS_CALL_CLOSE, [fd as usize, 0, 0])
}

pub fn sys_symlink(target: *const u8, link_path: *const u8) -> isize {
    sys_call(SYS_CALL_SYMLINK, [target as usize, link_path as usize, 0])
}

pub fn sys_lseek(fd: usize, offset: isize, whence: u32) -> isize {
    sys_call(SYS_CALL_LSEEK, [fd, offset as usize, whence as usize])
}

pub fn sys_readlink(path: *const u8, buf: &mut [u8]) -> isize {
    sys_call(
        SYS_CALL_READLINK,
        [path as usize, buf.as_ptr() as usize, buf.len()],
    )
}

pub fn sys_unlink(path: *const u8, flags: u32) -> isize {
    sys_call(SYS_CALL_UNLINK, [path as usize, flags as usize, 0])
}

pub fn sys_fstat(fd: usize, stat: *mut u8) -> isize {
    sys_call(SYS_CALL_FSTAT, [fd as usize, stat as usize, 0])
}
