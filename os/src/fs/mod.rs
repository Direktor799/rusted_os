mod bitmap;
mod block_cache;
pub mod block_dev;
pub mod efs;
pub mod layout;
mod vfs;
use core::str;
/// 磁盘块大小
pub const BLOCK_SZ: usize = 512;
/// 数据块
type DataBlock = [u8; BLOCK_SZ];

use crate::drivers::BLOCK_DEVICE;
use crate::memory::frame::user_buffer::UserBuffer;
use crate::sync::mutex::Mutex;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec::Vec;
use block_cache::BlockCacheManager;
use block_cache::BLOCK_CACHE_MANAGER;
use efs::EasyFileSystem;
use vfs::InodeHandler;

pub static mut ROOT_INODE: Option<Arc<InodeHandler>> = None;

pub fn find_inode_by_path(path: &str) -> Option<Arc<InodeHandler>> {
    let root_inode = unsafe { ROOT_INODE.as_ref().unwrap().clone() };
    path.split('/').fold(Some(root_inode), |node, name| {
        if name.len() > 0 {
            let cur_inode = node.unwrap().find(name);
            if cur_inode.as_ref().unwrap().is_link() {
                let inode_handler = cur_inode.as_ref().unwrap();
                let file_size = cur_inode.as_ref().unwrap().get_file_size();
                let mut real_path = alloc::vec![Default::default(); file_size as usize];
                inode_handler.read_at(0, &mut real_path);
                find_inode_by_path(str::from_utf8(&real_path).unwrap())
            } else {
                cur_inode
            }
        } else {
            node
        }
    })
}
// pub fn find_inode(path: &str) -> Option<Arc<InodeHandler>> {
//     let cur_inode = find_inode_by_path(path);
//     if cur_inode.as_ref().unwrap().is_link() {
//         let inode_handler = cur_inode.as_ref().unwrap();
//         let file_size = cur_inode.as_ref().unwrap().get_file_size();
//         let mut real_path = alloc::vec![Default::default(); file_size as usize];
//         inode_handler.read_at(0, &mut real_path);
//         find_inode_by_path(str::from_utf8(&real_path).unwrap())
//     } else {
//         cur_inode
//     }
// }
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

pub fn touch_by_path(path: &str) {
    let (parent_path, target) = path.rsplit_once('/').expect("Invalid path");
    let parent_inode = find_inode_by_path(parent_path).expect("Invalid parent directory");
    parent_inode.create(target, layout::InodeType::File);
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
unit_test!(test_file_link, {
    // 创建链接 c->b->a
    format();
    touch_by_path("/a");
    create_link_by_path("/b", "/a");
    create_link_by_path("/c", "/b");

    let inode_a_ = find_inode_by_path("/a");
    let inode_a = inode_a_.as_ref().unwrap();
    
    inode_a.write_at(0, "write_from_a".as_bytes());

    let mut read_buffer = [0u8; 127];
    let mut read_str = alloc::string::String::new();
    let inode_b_ = find_inode_by_path("/b");
    let inode_c_ = find_inode_by_path("/c");
    let inode_b = inode_b_.as_ref().unwrap();
    let inode_c = inode_c_.as_ref().unwrap();
    inode_b.read_at(0, &mut read_buffer);
    read_str = core::str::from_utf8(&read_buffer[..127])
        .unwrap()
        .to_string();
    utest_assert!(read_str == "a_write_from_0", "file link is bad");

    inode_b.write_at(0, "b_write_from_0".as_bytes());
    inode_c.write_at(0, "c_write_from_0".as_bytes());
    inode_b.write_at(3, "a_write_from_3".as_bytes());
    inode_a.read_at(0, &mut read_buffer);
    read_str = core::str::from_utf8(&read_buffer[..127])
        .unwrap()
        .to_string();
    utest_assert!(read_str == "c_wa_write_from_3", "file link is bad");
    Ok("file link is ok")
});

unit_test!(test_dir_link, {
    format();
    mkdir_by_path("/test");
    touch_by_path("/test/a");
    create_link_by_path("/link", "/test");
    let inode_a = find_inode_by_path("/test/a");
    inode_a
        .as_ref()
        .unwrap()
        .write_at(0, "test link string".as_bytes());

    let mut read_buffer = [0u8; 127];
    let mut read_str = alloc::string::String::new();
    let inode_b = find_inode_by_path("/link/a");
    inode_b.as_ref().unwrap().read_at(0, &mut read_buffer);
    read_str = core::str::from_utf8(&read_buffer[..127])
        .unwrap()
        .to_string();
    utest_assert!(read_str == "test link string", "dir link is bad");
    Ok("dir link is ok")
});
