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
use alloc::vec;
use alloc::vec::Vec;
pub use rfs::RustedFileSystem;
pub use vfs::InodeHandler;

pub static mut ROOT_INODE: UninitCell<Rc<InodeHandler>> = UninitCell::uninit();

pub fn find_inode_by_path(path: &str) -> Option<Rc<InodeHandler>> {
    let root_inode = unsafe { ROOT_INODE.clone() };
    path.split('/').fold(Some(root_inode), |node, name| {
        if name.len() > 0 {
            node.unwrap().find(name)
        } else {
            node
        }
    })
}
pub fn find_real_inode_by_path(path: &str) -> Option<Rc<InodeHandler>> {
    let root_inode = unsafe { ROOT_INODE.clone() };
    path.split('/').fold(Some(root_inode), |node, name| {
        if name.len() > 0 {
            let cur_inode = node.unwrap().find(name).unwrap();
            if cur_inode.is_link() {
                let file_size = cur_inode.get_file_size();
                let mut real_path = vec![0u8; file_size as usize];
                cur_inode.read_at(0, &mut real_path);
                find_inode_by_path(str::from_utf8(&real_path).unwrap())
            } else {
                Some(cur_inode)
            }
        } else {
            node
        }
    })
}
pub fn find_inode_path(path: &str) -> Vec<u8> {
    let cur_inode = find_inode_by_path(path);
    if cur_inode.as_ref().unwrap().is_link() {
        let inode_handler = cur_inode.as_ref().unwrap();
        let file_size = cur_inode.as_ref().unwrap().get_file_size();
        let mut real_path = alloc::vec![Default::default(); file_size as usize];
        inode_handler.read_at(0, &mut real_path);
        real_path
    } else {
        path.as_bytes().to_vec()
    }
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

pub fn touch_by_path(path: &str) -> Option<Rc<InodeHandler>> {
    let (parent_path, target) = path.rsplit_once('/').expect("Invalid path");
    let parent_inode = find_inode_by_path(parent_path).expect("Invalid parent directory");
    parent_inode.create(target, layout::InodeType::File)
}
pub fn create_link_by_path(path: &str, real_file_path: &str) {
    let (parent_path, target) = path.rsplit_once('/').expect("Invalid path");
    let parent_inode = find_inode_by_path(parent_path).expect("Invalid parent directory");
    let source_path = find_inode_path(real_file_path);
    parent_inode
        .create(target, layout::InodeType::SoftLink)
        .as_ref()
        .unwrap()
        .write_at(0, &source_path.as_slice());
}

pub fn check_valid_by_path(path: &str) -> bool {
    find_inode_by_path(path).is_some()
}

// pub trait File: Send + Sync {
//     fn read(&self, buf: UserBuffer) -> usize;
//     fn write(&self, buf: UserBuffer) -> usize;
// }

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
    }
}
