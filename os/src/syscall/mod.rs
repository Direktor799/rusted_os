use fs::*;
use proc::*;

mod fs;
mod proc;

const SYS_CALL_WRITE: usize = 64;
const SYS_CALL_EXIT: usize = 93;

pub fn sys_call(which: usize, args: [usize; 3]) -> isize {
    match which {
        SYS_CALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYS_CALL_EXIT => sys_exit(args[0] as i32),
        _ => {
            panic!("sys_call with unknown id: {}", which)
        }
    }
}
