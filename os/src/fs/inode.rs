use super::rfs::layout::InodeType;
use super::rfs::{find_inode, InodeHandler};
use super::{File, DIR, LNK, REG};
use crate::memory::frame::user_buffer::UserBuffer;
use alloc::rc::Rc;
use core::cell::RefCell;

pub struct OSInode {
    readable: bool,
    writable: bool,
    inner: RefCell<OSInodeInner>,
}
pub struct OSInodeInner {
    offset: usize,
    inode: Rc<InodeHandler>,
}

impl OSInode {
    pub fn new(readable: bool, writable: bool, inode: Rc<InodeHandler>) -> Self {
        Self {
            readable,
            writable,
            inner: RefCell::new(OSInodeInner { offset: 0, inode }),
        }
    }
}

pub struct OpenFlags(pub u32);

// pub const RDONLY: OpenFlags = OpenFlags(0);
const WRONLY: OpenFlags = OpenFlags(1 << 0);
// const RDWR: OpenFlags = OpenFlags(1 << 1);
const CREATE: OpenFlags = OpenFlags(1 << 9);
const TRUNC: OpenFlags = OpenFlags(1 << 10);

impl OpenFlags {
    /// Do not check validity for simplicity
    /// Return (readable, writable)
    fn contains(&self, num: OpenFlags) -> bool {
        return self.0 | num.0 == self.0;
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

pub fn open_file(path: &str, flags: OpenFlags) -> Option<Rc<OSInode>> {
    // TODO: app mode
    let (readable, writable) = flags.read_write();
    if flags.contains(CREATE) {
        if let Some(inode) = find_inode(path) {
            // clear size
            inode.clear();
            Some(Rc::new(OSInode::new(readable, writable, inode)))
        } else {
            let (parent_path, target) = path.rsplit_once('/')?;
            let parent_inode = find_inode(parent_path)?;
            parent_inode
                .create(target, InodeType::File)
                .map(|inode| Rc::new(OSInode::new(readable, writable, inode)))
        }
    } else {
        find_inode(path).map(|inode| {
            if flags.contains(TRUNC) {
                inode.clear();
            }
            Rc::new(OSInode::new(readable, writable, inode))
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

    fn read(&self, buf: UserBuffer) -> usize {
        let mut inner = self.inner.borrow_mut();
        let mut total_read_size = 0usize;
        for slice in buf.0.into_iter() {
            let read_size = inner.inode.read_at(inner.offset, slice);
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
        for slice in buf.0.iter() {
            let write_size = inner.inode.write_at(inner.offset, *slice);
            assert_eq!(write_size, slice.len());
            inner.offset += write_size;
            total_write_size += write_size;
        }
        total_write_size
    }

    fn get_offset(&self) -> usize {
        self.inner.borrow().offset
    }

    fn set_offset(&self, offset: usize) {
        self.inner.borrow_mut().offset = offset;
    }

    fn get_file_size(&self) -> usize {
        self.inner.borrow().inode.get_file_size() as usize
    }

    fn get_inode_id(&self) -> usize {
        self.inner.borrow().inode.get_inode_id() as usize
    }

    fn get_mode(&self) -> usize {
        let inode = &self.inner.borrow().inode;
        if inode.is_file() {
            REG
        } else if inode.is_dir() {
            DIR
        } else {
            LNK
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    test!(test_file_attr, {
        let test_file = open_file("/test_file_attr", CREATE).unwrap();
        test_assert!(test_file.readable, "Test file attr failed");
        test_assert!(test_file.writable, "Test file attr failed");
        test_assert!(test_file.get_mode() == REG, "Test file attr failed");
        let test_file = open_file("/test_file_attr", WRONLY).unwrap();
        test_assert!(!test_file.readable, "Test file attr failed");
        test_assert!(test_file.writable, "Test file attr failed");
        test_assert!(test_file.get_mode() == REG, "Test file attr failed");
        Ok("passed")
    });
}
