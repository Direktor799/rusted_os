mod bitmap;
pub mod block_cache;
pub mod block_dev;
pub mod layout;
pub mod rfs;
mod vfs;

/// 磁盘块大小
pub const BLOCK_SZ: usize = 512;
/// 数据块
type DataBlock = [u8; BLOCK_SZ];

pub use rfs::RustedFileSystem;
pub use vfs::InodeHandler;
