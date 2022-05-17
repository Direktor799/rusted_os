pub mod inode;
pub mod rfs;
pub mod stdio;
use crate::memory::frame::user_buffer::UserBuffer;
pub trait File {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn read(&self, buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;
}

pub fn init() {
    rfs::init();
    println!("mod fs initialized!");
}

pub fn format() {
    rfs::format();
    println!("mod fs formated!");
}
