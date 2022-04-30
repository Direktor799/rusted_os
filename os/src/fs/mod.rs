// pub trait File: Send + Sync {
//     fn readable(&self) -> bool;
//     fn writable(&self) -> bool;
//     fn read(&self, buf: UserBuffer) -> usize;
//     fn write(&self, buf: UserBuffer) -> usize;
// }

mod bitmap;
mod block_cache;
pub mod block_dev;
mod efs;
mod layout;
mod vfs;

pub const BLOCK_SZ: usize = 512;
use bitmap::Bitmap;
pub use block_cache::BlockCache;
use block_cache::{block_cache_sync_all, get_block_cache};
pub use block_dev::BlockDevice;
pub use efs::EasyFileSystem;
use layout::*;
pub use vfs::Inode;
