//! 文件系统模块
pub mod inode;
pub mod pipe;
pub mod rfs;
pub mod stdio;
use alloc::rc::Rc;

use crate::memory::frame::user_buffer::UserBuffer;

const CHR: usize = 0;
const REG: usize = 1;
const DIR: usize = 2;
const LNK: usize = 3;

const EOT: char = '\x04';
const LF: char = '\x0a';
const CR: char = '\x0d';

pub struct Stat {
    pub ino: u32,
    pub mode: u32,
    pub off: u32,
    pub size: u32,
}

impl From<Rc<dyn File>> for Stat {
    fn from(file: Rc<dyn File>) -> Self {
        Self {
            ino: file.get_inode_id() as u32,
            mode: file.get_mode() as u32,
            off: file.get_offset() as u32,
            size: file.get_file_size() as u32,
        }
    }
}

pub trait File {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn read(&self, buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;
    fn get_offset(&self) -> usize {
        0
    }
    fn set_offset(&self, _offset: usize) {}
    fn get_file_size(&self) -> usize {
        0
    }
    fn get_inode_id(&self) -> usize {
        0
    }
    fn get_mode(&self) -> usize {
        CHR
    }
}

pub fn init() {
    rfs::init();
    println!("mod fs initialized!");
}
