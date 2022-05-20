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
use crate::sync::uninit_cell::UninitCell;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec;
pub use rfs::RustedFileSystem;
pub use vfs::InodeHandler;

pub static mut ROOT_INODE: UninitCell<Rc<InodeHandler>> = UninitCell::uninit();

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

pub fn init() {
    block_cache::init();
    unsafe {
        let rfs = RustedFileSystem::open(BLOCK_DEVICE.clone());
        ROOT_INODE = UninitCell::init(Rc::new(RustedFileSystem::root_inode(&rfs)));
    }
}

pub fn format() {
    block_cache::init();
    unsafe {
        let rfs = RustedFileSystem::format(BLOCK_DEVICE.clone(), 4096, 1);
        ROOT_INODE = UninitCell::init(Rc::new(RustedFileSystem::root_inode(&rfs)));
        ROOT_INODE.set_default_dirent(ROOT_INODE.get_inode_id());
        block_cache::block_cache_sync_all();
    }
}
