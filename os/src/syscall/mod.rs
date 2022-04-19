use fs::*;
use proc::*;

mod fs;
mod proc;

const SYS_CALL_WRITE: usize = 64;
const SYS_CALL_EXIT: usize = 93;
const SYS_CALL_YIELD: usize = 124;
const SYS_CALL_GET_TIME: usize = 169;

pub fn sys_call(which: usize, args: [usize; 3]) -> isize {
    match which {
        SYS_CALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYS_CALL_EXIT => sys_exit(args[0] as i32),
        SYS_CALL_YIELD => sys_yield(),
        SYS_CALL_GET_TIME => sys_get_time(),
        _ => {
            panic!("sys_call with unknown id: {}", which)
        }
    }
}
