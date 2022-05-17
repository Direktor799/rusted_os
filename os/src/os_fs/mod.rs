mod stdio;
mod inode;
use crate::memory::frame::user_buffer::UserBuffer;
pub trait File {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn read(&self, buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;
}
pub use inode::{open_file, OSInode, OpenFlags};
pub use stdio::{Stdin, Stdout};