use crate::loader::run_next_app;

const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            let str = core::str::from_utf8(slice).unwrap();
            print!("{}", str);
            len as isize
        }
        _ => {
            panic!("sys_write with fd not supported")
        }
    }
}

// pub fn sys_exit(state: i32) -> ! {
//     println!("[kernel] Application exit with code {}", state);
//     unsafe {
//         run_next_app();
//     }
// }
