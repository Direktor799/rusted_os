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

fn find_inode_by_full_path(full_path: &str) -> Option<Arc<InodeHandler>> {
    let root_inode = unsafe { ROOT_INODE.as_ref().unwrap().clone() };
    full_path[1..]
        .split('/')
        .fold(Some(root_inode), |node, name| node.unwrap().find(name))
}

pub fn delete_by_full_path(full_path: &str) {
    let (parent_path, target) = full_path.rsplit_once('/').expect("Invalid path");
    let current_inode = find_inode_by_full_path(full_path).expect("Invalid target");
    current_inode.clear();
    let parent_inode = find_inode_by_full_path(parent_path).expect("Invalid parent directory");
    parent_inode.delete(target);
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
        let root_inode = ROOT_INODE.as_ref().unwrap();
        root_inode.create_default_for_dir(0, 0);
    }
    println!("mod fs initialized!");
}
