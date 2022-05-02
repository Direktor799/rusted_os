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

use crate::drivers::BLOCK_DEVICE;
use crate::memory::frame::user_buffer::UserBuffer;
use crate::sync::mutex::Mutex;
use alloc::sync::Arc;
use block_cache::BlockCacheManager;
use block_cache::BLOCK_CACHE_MANAGER;
use efs::EasyFileSystem;
use vfs::InodeHandler;

pub static mut ROOT_INODE: Option<Arc<InodeHandler>> = None;

pub fn find_inode_by_full_path(full_path: &str) -> Option<Arc<InodeHandler>> {
    let root_inode = unsafe { ROOT_INODE.as_ref().unwrap().clone() };
    full_path[1..]
        .split('/')
        .fold(Some(root_inode), |node, name| node.unwrap().find(name))
}

pub trait File: Send + Sync {
    fn read(&self, buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;
}

pub fn init() {
    unsafe {
        BLOCK_CACHE_MANAGER = Some(Mutex::new(BlockCacheManager::new()));
        let efs = EasyFileSystem::open(BLOCK_DEVICE.as_ref().unwrap().clone());
        ROOT_INODE = Some(Arc::new(EasyFileSystem::root_inode(&efs)));
    }
    println!("mod fs initialized!");
}
