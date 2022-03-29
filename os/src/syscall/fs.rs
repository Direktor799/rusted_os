use super::sys_call;

pub const SYS_CALL_WRITE: usize = 64;
pub const SYS_CALL_EXIT: usize = 93;

pub fn sys_write(fd: usize, buf: &[u8]) -> isize {
    sys_call(SYS_CALL_WRITE, [fd, buf.as_ptr() as usize, buf.len()])
}

pub fn sys_exit(state: i32) -> isize {
    sys_call(SYS_CALL_EXIT, [state as usize, 0, 0])
}
