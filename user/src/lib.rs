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
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::mem::size_of;
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
pub extern "C" fn _start(argc: usize, argv: usize) -> ! {
    unsafe {
        HEAP_ALLOCATOR = UninitCell::init(HeapAllocator(RefCell::new(
            BuddySystemAllocator::<32>::new(HEAP_SPACE.as_ptr() as usize, USER_HEAP_SIZE),
        )));
    }
    let mut v: Vec<&'static str> = Vec::new();
    for i in 0..argc {
        let str_start =
            unsafe { ((argv + i * core::mem::size_of::<usize>()) as *const usize).read_volatile() };
        let len = (0usize..)
            .find(|i| unsafe { ((str_start + *i) as *const u8).read_volatile() == 0 })
            .unwrap();
        v.push(
            core::str::from_utf8(unsafe {
                core::slice::from_raw_parts(str_start as *const u8, len)
            })
            .unwrap(),
        );
    }
    exit(main(&v))
}

#[linkage = "weak"]
#[no_mangle]
fn main(_args: &[&str]) -> i32 {
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

pub fn gettime() -> isize {
    sys_gettime()
}

pub fn getcwd(s: &mut String) -> isize {
    let mut buffer = vec![0u8; 128];
    let len = sys_getcwd(&mut buffer);
    *s = str::from_utf8(&buffer[0..len as usize])
        .unwrap()
        .to_string();
    len
}

pub fn chdir(path: &str) -> isize {
    let path = String::from(path) + "\0";
    sys_chdir(path.as_ptr())
}

pub fn mkdir(path: &str) -> isize {
    let path = String::from(path) + "\0";
    sys_mkdir(path.as_ptr())
}
pub fn open(path: &str, flags: u32) -> isize {
    let path = String::from(path) + "\0";
    sys_open(path.as_ptr(), flags)
}

pub fn close(fd: usize) -> isize {
    sys_close(fd)
}

pub fn pipe(pipe_fd: &mut [usize]) -> isize{
    sys_pipe(pipe_fd)
}

pub fn symlink(target_path: &str, link_path: &str) -> isize {
    let target_path = String::from(target_path) + "\0";
    let link_path = String::from(link_path) + "\0";
    sys_symlink(target_path.as_ptr(), link_path.as_ptr())
}

pub const SEEK_SET: u32 = 0;
pub const SEEK_CUR: u32 = 1;
pub const SEEK_END: u32 = 2;

pub fn lseek(fd: usize, offset: isize, whence: u32) -> isize {
    sys_lseek(fd, offset, whence)
}

pub fn readlink(path: &str, s: &mut String) -> isize {
    let path = String::from(path) + "\0";
    let mut buffer = vec![0u8; 128];
    let len = sys_readlink(path.as_ptr(), &mut buffer);
    if len < 0 {
        return len;
    }
    *s = str::from_utf8(&buffer[0..len as usize])
        .unwrap()
        .to_string();
    len
}

pub const AT_REMOVEDIR: u32 = 1;

pub fn unlink(path: &str, flags: u32) -> isize {
    let path = String::from(path) + "\0";
    sys_unlink(path.as_ptr(), flags)
}

pub const CHR: usize = 0;
pub const REG: usize = 1;
pub const DIR: usize = 2;
pub const LNK: usize = 3;

pub struct Stat {
    pub ino: u32,
    pub mode: u32,
    pub off: u32,
    pub size: u32,
}

impl Stat {
    pub fn new() -> Self {
        Self {
            ino: 0,
            mode: 0,
            off: 0,
            size: 0,
        }
    }
}

pub fn fstat(fd: usize, stat: &mut Stat) -> isize {
    sys_fstat(fd, stat as *mut _ as *mut _)
}

pub const NAME_LENGTH_LIMIT: usize = 27;
#[repr(C)]
pub struct Dirent {
    pub name: [u8; NAME_LENGTH_LIMIT + 1],
    pub inode_number: u32,
}

pub const DIRENT_SZ: usize = size_of::<Dirent>();

pub fn fork() -> isize {
    sys_fork()
}

pub fn exec(path: &str, args: &[&str]) -> isize {
    let path = String::from(path) + "\0";
    let args = args
        .iter()
        .map(|&arg| String::from(arg) + "\0")
        .collect::<Vec<_>>();
    let mut arg_ptrs = args.iter().map(|arg| arg.as_ptr()).collect::<Vec<_>>();
    arg_ptrs.push(0 as *const _);
    sys_exec(path.as_ptr(), arg_ptrs.as_ptr())
}

pub fn wait(exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(-1, exit_code as *mut _ as *mut _) {
            -2 => {
                sys_yield();
            }
            pid => return pid,
        }
    }
}

pub fn waitpid(pid: usize, exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(pid as isize, exit_code as *mut _ as *mut _) {
            -2 => {
                sys_yield();
            }
            pid => return pid,
        }
    }
}

pub fn getpid() -> isize {
    sys_getpid()
}
