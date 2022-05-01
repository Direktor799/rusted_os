use super::{
    block_dev::BlockDevice,
    efs::EasyFileSystem,
    layout::{Dirent, Inode, InodeType, DIRENT_SZ},
    vfs::InodeHandler,
};
use crate::sync::mutex::{Mutex, MutexGuard};
use alloc::string::String;
use alloc::vec::Vec;
use alloc::sync::Arc;

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
    //     let mut inner = self.inner.exclusive_access();
    //     let mut buffer = [0u8; 512];
    //     let mut v: Vec<u8> = Vec::new();
    //     loop {
    //         let len = inner.inode.read_at(inner.offset, &mut buffer);
    //         if len == 0 {
    //             break;
    //         }
    //         inner.offset += len;
    //         v.extend_from_slice(&buffer[..len]);
    //     }
    //     v
    // }
}
pub static mut ROOT_INODE: Option<Arc<InodeHandler>> = None;

pub fn get_tree_node(path: &str) -> Vec<&str> {
    path.split('/').collect()
}

pub fn get_path(tree_node: &Vec<&str>) -> String {
    tree_node.iter().fold(String::new(), |mut name1, name2| {
        name1.push_str("/");
        name1.push_str(&name2);
        name1
    })
}

unit_test!(test_node_to_str, {
    let ss = "/root/1/2/123";
    utest_assert!(
        get_tree_node(&ss) == alloc::vec!["","root", "1", "2", "123"],
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
