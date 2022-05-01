use super::{
    block_dev::BlockDevice,
    efs::EasyFileSystem,
    layout::{Dirent, Inode, InodeType, DIRENT_SZ},
    vfs::InodeHandler,
};
use crate::sync::mutex::{Mutex, MutexGuard};
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

pub struct OSInode {
    readable: bool,
    writable: bool,
    inner: Mutex<OSInodeInner>,
}

pub struct OSInodeInner {
    offset: usize,
    inode: Arc<InodeHandler>,
}

impl OSInode {
    pub fn new(readable: bool, writable: bool, inode: Arc<InodeHandler>) -> Self {
        Self {
            readable,
            writable,
            inner: unsafe { Mutex::new(OSInodeInner { offset: 0, inode }) },
        }
    }
    // pub fn read_all(&self) -> Vec<u8> {
    //     let mut buffer = [0u8; 512];
    //     let mut v: Vec<u8> = Vec::new();
    //     loop {
    //         let len = self.inner.lock().inode.read_at(self.inner.lock().offset, &mut buffer);
    //         if len == 0 {
    //             break;
    //         }
    //         self.inner.lock().offset += len;
    //         v.extend_from_slice(&buffer[..len]);
    //     }
    //     v
    // }
}
pub static mut ROOT_INODE: Option<Arc<InodeHandler>> = None;

pub fn get_tree_node(full_path: &str) -> Vec<&str> {
    full_path[1..].split('/').collect()
    // 由于路径由'/'开始，spilt的结果的第1个元素为空字符串,这是我们不需要的
}

pub fn get_path(tree_node: &Vec<&str>) -> String {
    tree_node.iter().fold(String::new(), |mut name1, name2| {
        name1.push_str("/");
        name1.push_str(&name2);
        name1
    })
}
pub fn find_inode_by_full_path(full_path: &str) -> Option<Arc<InodeHandler>> {
    let nodes_name = get_tree_node(&full_path);
    let root_inode = unsafe { ROOT_INODE.as_ref().unwrap().clone() };
    nodes_name
        .into_iter()
        .fold(Some(root_inode), |node, name| node.unwrap().find(name))
}
unit_test!(test_node_to_str, {
    let ss = "/root/1/2/123";
    utest_assert!(
        get_tree_node(&ss) == alloc::vec!["", "root", "1", "2", "123"],
        "Cannot get appropriate tree node from path"
    );
    Ok("passed!")
});

unit_test!(test_get_path, {
    let ss = alloc::vec!["root", "1", "2", "123"];
    utest_assert!(
        get_path(&ss) == "/root/1/2/123",
        "Cannot get appropriate path from vector"
    );
    Ok("passed!")
});
// pub fn list_apps() {
//     println!("/**** APPS ****");
//     unsafe{
//         for app in ROOT_INODE.ls() {
//             println!("{}", app);
//         }
//     }
//     println!("**************/");
// }

// pub enum OpenFlags {
//     RDONLY = 0,
//     WRONLY = 1 << 0,
//     RDWR = 1 << 1,
//     CREATE = 1 << 9,
//     TRUNC = 1 << 10
// }

// impl OpenFlags {
//     /// Do not check validity for simplicity
//     /// Return (readable, writable)
//     pub fn read_write(&self) -> (bool, bool) {
//         if self.is_empty() {
//             (true, false)
//         } else if self.contains(Self::WRONLY) {
//             (false, true)
//         } else {
//             (true, true)
//         }
//     }
// }

// pub fn open_file(name: &str, flags: OpenFlags) -> Option<Arc<OSInode>> {
//     let (readable, writable) = flags.read_write();
//     if flags.contains(OpenFlags::CREATE) {
//         if let Some(inode) = ROOT_INODE.find(name) {
//             // clear size
//             inode.clear();
//             Some(Arc::new(OSInode::new(readable, writable, inode)))
//         } else {
//             // create file
//             ROOT_INODE
//                 .create(name)
//                 .map(|inode| Arc::new(OSInode::new(readable, writable, inode)))
//         }
//     } else {
//         ROOT_INODE.find(name).map(|inode| {
//             if flags.contains(OpenFlags::TRUNC) {
//                 inode.clear();
//             }
//             Arc::new(OSInode::new(readable, writable, inode))
//         })
//     }
// }

// impl File for OSInode {
//     fn readable(&self) -> bool {
//         self.readable
//     }
//     fn writable(&self) -> bool {
//         self.writable
//     }
//     fn read(&self, mut buf: UserBuffer) -> usize {
//         let mut inner = self.inner.exclusive_access();
//         let mut total_read_size = 0usize;
//         for slice in buf.buffers.iter_mut() {
//             let read_size = inner.inode.read_at(inner.offset, *slice);
//             if read_size == 0 {
//                 break;
//             }
//             inner.offset += read_size;
//             total_read_size += read_size;
//         }
//         total_read_size
//     }
//     fn write(&self, buf: UserBuffer) -> usize {
//         let mut inner = self.inner.exclusive_access();
//         let mut total_write_size = 0usize;
//         for slice in buf.buffers.iter() {
//             let write_size = inner.inode.write_at(inner.offset, *slice);
//             assert_eq!(write_size, slice.len());
//             inner.offset += write_size;
//             total_write_size += write_size;
//         }
//         total_write_size
//     }
// }
