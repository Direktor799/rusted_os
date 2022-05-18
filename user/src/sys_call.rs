use core::arch::asm;

const SYS_CALL_GET_CWD: usize = 17;
const SYS_CALL_CHDIR: usize = 49;
const SYS_CALL_READ: usize = 63;
const SYS_CALL_WRITE: usize = 64;
const SYS_CALL_EXIT: usize = 93;
const SYS_CALL_YIELD: usize = 124;
const SYS_CALL_GET_TIME: usize = 169;
const SYS_CALL_MKDIR: usize = 34;

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

pub fn sys_get_time() -> isize {
    sys_call(SYS_CALL_GET_TIME, [0, 0, 0])
}

pub fn sys_get_cwd(buf: &mut [u8]) -> isize {
    sys_call(SYS_CALL_GET_CWD, [buf.as_ptr() as usize, buf.len(), 0])
}

pub fn sys_chdir(path: *const u8) -> isize {
    sys_call(SYS_CALL_CHDIR, [path as usize, 0, 0])
}

pub fn sys_mkdir(path: *const u8) -> isize {
    sys_call(SYS_CALL_MKDIR, [path as usize, 0, 0])
}