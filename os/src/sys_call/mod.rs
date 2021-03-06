//! 系统调用模块
mod fs;
mod proc;

use fs::*;
use proc::*;

const SYS_CALL_GETCWD: usize = 17;
const SYS_CALL_DUP2: usize = 24;
const SYS_CALL_MKDIR: usize = 34;
const SYS_CALL_UNLINK: usize = 35;
const SYS_CALL_CHDIR: usize = 49;
const SYS_CALL_OPEN: usize = 56;
const SYS_CALL_CLOSE: usize = 57;
const SYS_CALL_PIPE: usize = 59;
const SYS_CALL_LSEEK: usize = 62;
const SYS_CALL_READ: usize = 63;
const SYS_CALL_WRITE: usize = 64;
const SYS_CALL_FSTAT: usize = 80;
const SYS_CALL_EXIT: usize = 93;
const SYS_CALL_YIELD: usize = 124;
const SYS_CALL_KILL: usize = 129;
const SYS_CALL_GETTIME: usize = 169;
const SYS_CALL_GETPID: usize = 172;
const SYS_CALL_FORK: usize = 220;
const SYS_CALL_EXEC: usize = 221;
const SYS_CALL_WAITPID: usize = 260;

pub fn sys_call(which: usize, args: [usize; 3]) -> isize {
    match which {
        SYS_CALL_GETCWD => sys_getcwd(args[0] as *const u8, args[1]),
        SYS_CALL_DUP2 => sys_dup2(args[0], args[1]),
        SYS_CALL_MKDIR => sys_mkdir(args[0] as *const u8),
        SYS_CALL_UNLINK => sys_unlink(args[0] as *const u8, args[1] as u32),
        SYS_CALL_CHDIR => sys_chdir(args[0] as *const u8),
        SYS_CALL_OPEN => sys_open(args[0] as *const u8, args[1] as u32),
        SYS_CALL_CLOSE => sys_close(args[0]),
        SYS_CALL_PIPE => sys_pipe(args[0] as *mut usize),
        SYS_CALL_LSEEK => sys_lseek(args[0], args[1] as isize, args[2] as u32),
        SYS_CALL_READ => sys_read(args[0], args[1] as *mut u8, args[2]),
        SYS_CALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYS_CALL_FSTAT => sys_fstat(args[0], args[1] as *mut u8),
        SYS_CALL_EXIT => sys_exit(args[0] as i32),
        SYS_CALL_YIELD => sys_yield(),
        SYS_CALL_KILL => sys_kill(args[0]),
        SYS_CALL_GETTIME => sys_gettime(),
        SYS_CALL_GETPID => sys_getpid(),
        SYS_CALL_FORK => sys_fork(),
        SYS_CALL_EXEC => sys_exec(args[0] as *const u8, args[1] as *const *const u8),
        SYS_CALL_WAITPID => sys_waitpid(args[0] as isize, args[1] as *mut u8),
        _ => panic!("sys_call with unknown id: {}", which),
    }
}
