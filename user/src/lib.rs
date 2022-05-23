#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

extern crate alloc;

#[macro_use]
pub mod console;
mod heap;
mod panic;
mod sys_call;
mod uninit_cell;

use crate::heap::heap_allocator::*;
use crate::uninit_cell::UninitCell;
use alloc::alloc::Layout;
use alloc::string::String;
use alloc::string::ToString;
use core::cell::RefCell;
use core::str;
use sys_call::*;

const USER_HEAP_SIZE: usize = 4096;

static mut HEAP_SPACE: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];
/// 全局堆内存分配器

#[global_allocator]
pub static mut HEAP_ALLOCATOR: UninitCell<HeapAllocator> = UninitCell::uninit();

/// 全局堆内存分配失败处理
#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("Allocation error: {:?}", layout);
}

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    unsafe {
        HEAP_ALLOCATOR = UninitCell::init(HeapAllocator(RefCell::new(
            BuddySystemAllocator::<32>::new(HEAP_SPACE.as_ptr() as usize, USER_HEAP_SIZE),
        )));
    }
    exit(main());
}

#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Cannot find main!");
}

pub const RDONLY: u32 = 0;
pub const WRONLY: u32 = 1 << 0;
pub const RDWR: u32 = 1 << 1;
pub const CREATE: u32 = 1 << 9;
pub const TRUNC: u32 = 1 << 10;

pub fn read(fd: usize, buf: &mut [u8]) -> isize {
    sys_read(fd, buf)
}

pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}

pub fn exit(exit_code: i32) -> ! {
    sys_exit(exit_code);
    unreachable!();
}

pub fn r#yield() -> isize {
    sys_yield()
}

pub fn get_time() -> isize {
    sys_get_time()
}

pub fn getcwd(s: &mut String) -> isize {
    let mut buffer = [0u8; 128];
    let ret = sys_get_cwd(&mut buffer);
    let len = buffer.iter().position(|&v| v == 0).unwrap_or(buffer.len());
    *s = str::from_utf8(&buffer[0..len]).unwrap().to_string();
    ret
}

pub fn chdir(path: &str) -> isize {
    let mut zero_ended = String::from(path);
    zero_ended.push(0 as char);
    sys_chdir(zero_ended.as_ptr())
}

pub fn mkdir(path: &str) -> isize {
    let mut zero_ended = String::from(path);
    zero_ended.push(0 as char);
    sys_mkdir(zero_ended.as_ptr())
}
pub fn open(path: &str, flags: u32) -> isize {
    let mut zero_ended = String::from(path);
    zero_ended.push(0 as char);
    sys_open(zero_ended.as_ptr(), flags)
}

pub fn close(fd: usize) -> isize {
    sys_close(fd)
}

pub fn fork() -> isize {
    sys_fork()
}
