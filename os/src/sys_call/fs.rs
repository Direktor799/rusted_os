//! 文件相关系统调用子模块
use crate::memory::frame::page_table::get_user_buffer_in_kernel;
use crate::sbi::console_getchar;
use crate::task::TASK_MANAGER;
use alloc::string::String;

const FD_STDIN: usize = 0;
const FD_STDOUT: usize = 1;

pub fn sys_read(fd: usize, buf: *mut u8, len: usize) -> isize {
    let user_satp_token = unsafe { TASK_MANAGER.get_current_token() };
    let mut user_buffer = get_user_buffer_in_kernel(user_satp_token, buf, len);
    match fd {
        FD_STDIN => {
            loop {
                let ch = console_getchar() as u8;
                if ch != 255 {
                    unsafe {
                        user_buffer.0[0].as_mut_ptr().write_volatile(ch);
                    }
                    break;
                }
            }
            1
        }
        _ => {
            panic!("sys_read with fd not supported")
        }
    }
}

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let user_satp_token = unsafe { TASK_MANAGER.get_current_token() };
    let user_buffer = get_user_buffer_in_kernel(user_satp_token, buf, len);
    match fd {
        FD_STDOUT => {
            let str = user_buffer
                .into_iter()
                .map(|&mut byte| byte as char)
                .collect::<String>();
            print!("{}", str);
            len as isize
        }
        _ => {
            panic!("sys_write with fd not supported")
        }
    }
}
