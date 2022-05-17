use core::cell::RefCell;

use super::File;
use crate::fs::{find_inode_by_path, touch_by_path, InodeHandler};
use alloc::sync::Arc;
use alloc::vec::Vec;
use crate::memory::frame::user_buffer::UserBuffer;
pub struct OSInode {
    readable: bool,
    writable: bool,
    inner: RefCell<OSInodeInner>,
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
            inner: RefCell::new(OSInodeInner { offset: 0, inode }),
        }
    }
    pub fn read_all(&self) -> Vec<u8> {
        let mut inner = self.inner.borrow_mut();
        let mut buffer = [0u8; 512];
        let mut v: Vec<u8> = Vec::new();
        loop {
            let len = inner.inode.read_at(inner.offset, &mut buffer);
            if len == 0 {
                break;
            }
            inner.offset += len;
            v.extend_from_slice(&buffer[..len]);
        }
        v
    }
}
pub struct OpenFlags(pub u32);

const RDONLY: OpenFlags = OpenFlags(0);
const WRONLY: OpenFlags = OpenFlags(1 << 0);
const RDWR: OpenFlags = OpenFlags(1 << 1);
const CREATE: OpenFlags = OpenFlags(1 << 9);
const TRUNC: OpenFlags = OpenFlags(1 << 10);

impl OpenFlags {
    /// Do not check validity for simplicity
    /// Return (readable, writable)
    fn contains(&self, num: OpenFlags) -> bool {
        return self.0 & num.0 == self.0;
    }
    pub fn read_write(&self) -> (bool, bool) {
        // 判断是否OpenFlags为空
        if self.0 == 0 {
            (true, false)
        } else if self.contains(WRONLY) {
            (false, true)
        } else {
            (true, true)
        }
    }
}

pub fn open_file(name: &str, flags: OpenFlags) -> Option<Arc<OSInode>> {
    let (readable, writable) = flags.read_write();
    if flags.contains(CREATE) {
        if let Some(inode) = find_inode_by_path(name) {
            // clear size
            inode.clear();
            Some(Arc::new(OSInode::new(readable, writable, inode)))
        } else {
            // create file
            touch_by_path(name).map(|inode| Arc::new(OSInode::new(readable, writable, inode)))
        }
    } else {
        find_inode_by_path(name).map(|inode| {
            if flags.contains(TRUNC) {
                inode.clear();
            }
            Arc::new(OSInode::new(readable, writable, inode))
        })
    }
}

impl File for OSInode {
    fn readable(&self) -> bool {
        self.readable
    }
    fn writable(&self) -> bool {
        self.writable
    }
    fn read(&self, mut buf: UserBuffer) -> usize {
        let mut inner = self.inner.borrow_mut();
        let mut total_read_size = 0usize;
        for slice in buf.0.iter_mut() {
            let read_size = inner.inode.read_at(inner.offset, *slice);
            if read_size == 0 {
                break;
            }
            inner.offset += read_size;
            total_read_size += read_size;
        }
        total_read_size
    }
    fn write(&self, buf: UserBuffer) -> usize {
        let mut inner = self.inner.borrow_mut();
        let mut total_write_size = 0usize;
        for slice in  buf.0.iter() {
            let write_size = inner.inode.write_at(inner.offset, *slice);
            assert_eq!(write_size, slice.len());
            inner.offset += write_size;
            total_write_size += write_size;
        }
        total_write_size
    }
}
