use crate::memory::PageTable;
use crate::task::TASK_MANAGER;
const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let user_satp_token = unsafe { TASK_MANAGER.get_current_token() };
    let data_segments = PageTable::get_buffer_in_kernel(user_satp_token, buf, len);
    match fd {
        FD_STDOUT => {
            for data_segment in data_segments {
                let str = core::str::from_utf8(data_segment).unwrap();
                print!("{}", str);
            }
            len as isize
        }
        _ => {
            panic!("sys_write with fd not supported")
        }
    }
}
