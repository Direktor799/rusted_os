// block_dev::BlockDevice,
// efs::EasyFileSystem,
// layout::{Dirent, Inode, InodeType, DIRENT_SZ},
// use crate::sync::mutex::{Mutex, MutexGuard};
// use alloc::string::String;
// use alloc::vec::Vec;

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
