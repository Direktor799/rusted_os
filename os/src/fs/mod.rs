// pub trait File: Send + Sync {
//     fn readable(&self) -> bool;
//     fn writable(&self) -> bool;
//     fn read(&self, buf: UserBuffer) -> usize;
//     fn write(&self, buf: UserBuffer) -> usize;
// }

mod bitmap;
mod block_cache;
pub mod block_dev;
pub mod efs;
pub mod layout;
mod vfs;

/// 磁盘块大小
pub const BLOCK_SZ: usize = 512;

/// 数据块
type DataBlock = [u8; BLOCK_SZ];

use block_cache::BlockCacheManager;
use block_cache::BLOCK_CACHE_MANAGER;
use spin::Mutex;

pub fn init() {
    unsafe {
        BLOCK_CACHE_MANAGER = Some(Mutex::new(BlockCacheManager::new()));
    }
    println!("mod fs initialized!");
}
