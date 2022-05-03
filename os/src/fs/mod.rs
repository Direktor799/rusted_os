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

fn find_inode_by_path(path: &str) -> Option<Arc<InodeHandler>> {
    let root_inode = unsafe { ROOT_INODE.as_ref().unwrap().clone() };
    path.split('/').fold(Some(root_inode), |node, name| {
        if name.len() > 0 {
            node.unwrap().find(name)
        } else {
            node
        }
    })
}

pub fn ls_by_path(path: &str) {
    let inode = find_inode_by_path(path).expect("Invaild target");
    inode
        .ls()
        .into_iter()
        .skip(2)
        .for_each(|str| println!("{}", str));
}

pub fn delete_by_path(path: &str) {
    let (parent_path, target) = path.rsplit_once('/').expect("Invalid path");
    let current_inode = find_inode_by_path(path).expect("Invalid target");
    current_inode.clear();
    let parent_inode = find_inode_by_path(parent_path).expect("Invalid parent directory");
    parent_inode.delete(target);
}

pub fn mkdir_by_path(path: &str) {
    let (parent_path, target) = path.rsplit_once('/').expect("Invalid path");
    let parent_inode = find_inode_by_path(parent_path).expect("Invalid parent directory");
    if let Some(child_inode) = parent_inode.create(target, layout::InodeType::Directory) {
        child_inode.set_default_dirent(parent_inode.get_inode_id());
    } else {
        println!("cannot create directory '{}': File exists", target);
    };
}

pub fn touch_by_path(path: &str) {
    let (parent_path, target) = path.rsplit_once('/').expect("Invalid path");
    let parent_inode = find_inode_by_path(parent_path).expect("Invalid parent directory");
    parent_inode.create(target, layout::InodeType::File);
}

pub fn check_valid_by_path(path: &str) -> bool {
    find_inode_by_path(path).is_some()
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

pub fn format() {
    unsafe {
        BLOCK_CACHE_MANAGER = Some(Mutex::new(BlockCacheManager::new()));
        let efs = EasyFileSystem::format(BLOCK_DEVICE.as_ref().unwrap().clone(), 4096, 1);
        ROOT_INODE = Some(Arc::new(EasyFileSystem::root_inode(&efs)));
        let root_inode = ROOT_INODE.as_ref().unwrap();
        root_inode.set_default_dirent(root_inode.get_inode_id());
    }
    println!("mod fs formated!");
}
