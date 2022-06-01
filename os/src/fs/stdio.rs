use super::File;
use crate::fs::{CR, LF};
use crate::memory::frame::user_buffer::UserBuffer;
use crate::sbi::console_getchar;

pub struct Stdin;

pub struct Stdout;

impl File for Stdin {
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        false
    }
    fn read(&self, mut user_buf: UserBuffer) -> usize {
        assert_eq!(user_buf.len(), 1);
        // busy loop
        loop {
            let mut ch = console_getchar() as u8;
            if ch == 255 {
                break usize::MAX;
            } else {
                if ch == CR as u8 {
                    ch = LF as u8;
                }
                unsafe {
                    user_buf.0[0].as_mut_ptr().write_volatile(ch);
                }
                print!("{}", ch as char);
                break 1;
            }
        }
    }
    fn write(&self, _user_buf: UserBuffer) -> usize {
        panic!("Cannot write to stdin!");
    }
}

impl File for Stdout {
    fn readable(&self) -> bool {
        false
    }
    fn writable(&self) -> bool {
        true
    }
    fn read(&self, _user_buf: UserBuffer) -> usize {
        panic!("Cannot read from stdout!");
    }
    fn write(&self, user_buf: UserBuffer) -> usize {
        for buffer in user_buf.0.iter() {
            print!("{}", core::str::from_utf8(*buffer).unwrap());
        }
        user_buf.len()
    }
}
