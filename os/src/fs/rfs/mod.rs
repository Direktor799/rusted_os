//! 内核态文件系统模块
mod bitmap;
mod block_cache;
pub mod block_dev;
pub mod layout;
pub mod rfs;
mod vfs;
use core::str;
/// 磁盘块大小
pub const BLOCK_SZ: usize = 512;
/// 数据块
type DataBlock = [u8; BLOCK_SZ];

use crate::drivers::BLOCK_DEVICE;
use crate::tools::uninit_cell::UninitCell;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec;
pub use rfs::RustedFileSystem;
pub use vfs::InodeHandler;
/// 根目录节点
pub static mut ROOT_INODE: UninitCell<Rc<InodeHandler>> = UninitCell::uninit();
/// 由路径找到文件的inodehandler
pub fn find_inode(path: &str) -> Option<Rc<InodeHandler>> {
    let root_inode = unsafe { ROOT_INODE.clone() };
    path.split('/').fold(Some(root_inode), |res, name| {
        if let Some(node) = res {
            if !name.is_empty() {
                node.find(name)
            } else {
                Some(node)
            }
        } else {
            None
        }
    })
}
/// 做相对路径和绝对路径之间的转换
pub fn get_full_path(cwd: &String, path: &String) -> String {
    let mut v = vec![];
    let new_path = if path.as_str().chars().next().unwrap() == '/' {
        // absolute
        String::from(path)
    } else {
        // relative
        String::from(cwd) + "/" + path
    };
    new_path.split('/').for_each(|name| {
        if name.is_empty() || name == "." || (name == ".." && v.is_empty()) {
            // do nothing
        } else if name == ".." {
            v.pop();
        } else {
            v.push(name);
        }
    });
    String::from("/") + &v.join("/")
}
/// 初始化文件系统,创建root目录
pub fn init() {
    block_cache::init();
    unsafe {
        if let Some(rfs) = RustedFileSystem::open(BLOCK_DEVICE.clone()) {
            ROOT_INODE = UninitCell::init(Rc::new(RustedFileSystem::root_inode(&rfs)));
        } else {
            println!("[kernel] RFS corrupted, formatting");
            let rfs = RustedFileSystem::format(BLOCK_DEVICE.clone(), 4096, 1);
            ROOT_INODE = UninitCell::init(Rc::new(RustedFileSystem::root_inode(&rfs)));
            ROOT_INODE.set_default_dirent(ROOT_INODE.get_inode_id());
            block_cache::block_cache_sync_all();
        }
    }
}
