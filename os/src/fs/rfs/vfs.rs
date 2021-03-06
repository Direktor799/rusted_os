//! InodeHandler子模块
use super::{
    block_cache::{block_cache_sync_all, get_block_cache},
    block_dev::BlockDevice,
    layout::{Dirent, Inode, InodeType, DIRENT_SZ},
    rfs::RustedFileSystem,
};
use alloc::rc::Rc;
use core::cell::RefCell;

/// Inode句柄
pub struct InodeHandler {
    block_id: u32,
    block_offset: usize,
    fs: Rc<RefCell<RustedFileSystem>>,
    block_device: Rc<dyn BlockDevice>,
}

impl InodeHandler {
    /// 创建新的Inode句柄
    pub fn new(
        block_id: u32,
        block_offset: usize,
        fs: Rc<RefCell<RustedFileSystem>>,
        block_device: Rc<dyn BlockDevice>,
    ) -> Self {
        Self {
            block_id,
            block_offset,
            fs,
            block_device,
        }
    }

    /// 读取对应的Inode
    fn read_disk_inode<V>(&self, f: impl FnOnce(&Inode) -> V) -> V {
        get_block_cache(self.block_id as usize, Rc::clone(&self.block_device))
            .borrow()
            .read(self.block_offset, f)
    }

    /// 修改对应的Inode
    fn modify_disk_inode<V>(&self, f: impl FnOnce(&mut Inode) -> V) -> V {
        get_block_cache(self.block_id as usize, Rc::clone(&self.block_device))
            .borrow_mut()
            .modify(self.block_offset, f)
    }
    /// 获取自身inode_id
    pub fn get_inode_id(&self) -> u32 {
        self.fs
            .borrow()
            .get_disk_inode_id(self.block_id, self.block_offset)
    }
    /// 根据当前目录下的文件名找到inode_id
    fn find_inode_id(&self, name: &str, disk_inode: &Inode) -> Option<u32> {
        // assert it is a directory
        assert!(disk_inode.is_dir());
        let file_count = (disk_inode.size as usize) / DIRENT_SZ;
        let mut dirent = Dirent::empty();
        for i in 0..file_count {
            disk_inode.read_at(DIRENT_SZ * i, dirent.as_bytes_mut(), &self.block_device);
            if dirent.name() == name {
                return Some(dirent.inode_number() as u32);
            }
        }
        None
    }
    /// 根据当前目录下的文件名找到inodehandler
    pub fn find(&self, name: &str) -> Option<Rc<InodeHandler>> {
        let fs = self.fs.borrow();
        self.read_disk_inode(|disk_inode| {
            let inode_id = self.find_inode_id(name, disk_inode)?;
            let (block_id, block_offset) = fs.get_disk_inode_pos(inode_id);
            Some(Rc::new(Self::new(
                block_id,
                block_offset,
                self.fs.clone(),
                self.block_device.clone(),
            )))
        })
    }
    /// 扩充当前文件的大小
    fn increase_size(&self, new_size: u32, disk_inode: &mut Inode, fs: &mut RustedFileSystem) {
        if new_size < disk_inode.size {
            return;
        }
        let v = (0..disk_inode.blocks_needed(new_size))
            .into_iter()
            .map(|_| fs.alloc_data())
            .collect();
        disk_inode.increase_size(new_size, v, &self.block_device);
    }
    /// 回收当前文件的部分空间
    fn decrease_size(&self, new_size: u32, disk_inode: &mut Inode, fs: &mut RustedFileSystem) {
        if new_size >= disk_inode.size {
            return;
        }
        disk_inode
            .decrease_size(new_size, &self.block_device)
            .into_iter()
            .for_each(|block_id| fs.dealloc_data(block_id));
    }
    /// 在当前目录下创建文件
    pub fn create(&self, name: &str, filetype: InodeType) -> Option<Rc<InodeHandler>> {
        let mut fs = self.fs.borrow_mut();
        let op = |dir_inode: &Inode| {
            // assert it is a directory
            assert!(dir_inode.is_dir());
            // has the file been created?
            self.find_inode_id(name, dir_inode)
        };
        if self.read_disk_inode(op).is_some() {
            return None;
        }
        // create a new file
        // alloc a inode with an indirect block
        let new_inode_id = fs.alloc_inode();
        // initialize inode
        let (new_inode_block_id, new_inode_block_offset) = fs.get_disk_inode_pos(new_inode_id);
        get_block_cache(new_inode_block_id as usize, Rc::clone(&self.block_device))
            .borrow_mut()
            .modify(new_inode_block_offset, |new_inode: &mut Inode| {
                new_inode.init(filetype);
            });
        self.modify_disk_inode(|dir_inode| {
            // append file in the dirent
            let file_count = (dir_inode.size as usize) / DIRENT_SZ;
            let new_size = (file_count + 1) * DIRENT_SZ;
            // increase size
            self.increase_size(new_size as u32, dir_inode, &mut fs);
            // write dirent
            let dirent = Dirent::new(name, new_inode_id);
            dir_inode.write_at(
                file_count * DIRENT_SZ,
                dirent.as_bytes(),
                &self.block_device,
            );
        });

        let (block_id, block_offset) = fs.get_disk_inode_pos(new_inode_id);
        block_cache_sync_all();
        // return inode
        if filetype != InodeType::Directory {
            return Some(Rc::new(Self::new(
                block_id,
                block_offset,
                self.fs.clone(),
                self.block_device.clone(),
            )));
        } else {
            let new_inode_handler = Self::new(
                block_id,
                block_offset,
                self.fs.clone(),
                self.block_device.clone(),
            );
            let mut dirent_self = Dirent::new("", 0);
            self.read_disk_inode(|disk_inode| {
                disk_inode.read_at(0, dirent_self.as_bytes_mut(), &self.block_device)
            });
            // new_inode_handler.create_default_for_dir(dirent_self.inode_number(), new_inode_id);
            Some(Rc::new(new_inode_handler))
        }
    }
    /// 给当前文件设置默认目录项(.和..)
    pub fn set_default_dirent(&self, parent_inode_id: u32) {
        let mut fs = self.fs.borrow_mut();
        self.modify_disk_inode(|cur_dir_inode| {
            // increase size
            self.increase_size(2 * DIRENT_SZ as u32, cur_dir_inode, &mut fs);
            // write . dirent
            let dirent_self =
                Dirent::new(".", fs.get_disk_inode_id(self.block_id, self.block_offset));
            cur_dir_inode.write_at(0, dirent_self.as_bytes(), &self.block_device);

            // write .. dirent
            let dirent_parent = Dirent::new("..", parent_inode_id);
            cur_dir_inode.write_at(DIRENT_SZ, dirent_parent.as_bytes(), &self.block_device);
        });
    }
    /// 根据当前目录下的文件名删除指定文件
    pub fn delete(&self, name: &str) {
        let mut fs = self.fs.borrow_mut();
        self.modify_disk_inode(|dir_inode| {
            assert!(dir_inode.is_dir());
            self.find_inode_id(name, dir_inode).expect("No target");
            // 读取最后一个目录项
            let mut last_dirent = Dirent::new("", 0);
            dir_inode.read_at(
                dir_inode.size as usize - DIRENT_SZ,
                last_dirent.as_bytes_mut(),
                &self.block_device,
            );
            // 查找到当前目录项,并用最后一个目录项的内容替换当前目录项
            let file_count = (dir_inode.size as usize) / DIRENT_SZ;
            for i in 0..file_count {
                let mut dirent = Dirent::empty();
                dir_inode.read_at(i * DIRENT_SZ, dirent.as_bytes_mut(), &self.block_device);
                if dirent.name() == name {
                    fs.dealloc_inode(dirent.inode_number());
                    dir_inode.write_at(i * DIRENT_SZ, last_dirent.as_bytes(), &self.block_device);
                    let new_size = (file_count - 1) * DIRENT_SZ;
                    self.decrease_size(new_size as u32, dir_inode, &mut fs);
                    return;
                }
            }
        });
    }
    /// 判断当前文件是否为目录
    pub fn is_dir(&self) -> bool {
        self.read_disk_inode(|disk_inode| disk_inode.is_dir())
    }
    /// 判断当前文件是否为普通文件
    pub fn is_file(&self) -> bool {
        self.read_disk_inode(|disk_inode| disk_inode.is_file())
    }
    /// 从指定偏移处读文件内容
    pub fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        let _fs = self.fs.borrow();
        self.read_disk_inode(|disk_inode| disk_inode.read_at(offset, buf, &self.block_device))
    }
    /// 向指定偏移处写入文件内容
    pub fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        let mut fs = self.fs.borrow_mut();
        let size = self.modify_disk_inode(|disk_inode| {
            self.increase_size((offset + buf.len()) as u32, disk_inode, &mut fs);
            disk_inode.write_at(offset, buf, &self.block_device)
        });
        block_cache_sync_all();
        size
    }
    /// 获取当前文件的大小
    pub fn get_file_size(&self) -> u32 {
        self.read_disk_inode(|disk_inode| disk_inode.size)
    }

    /// 清空所有数据并回收块
    pub fn clear(&self) {
        let mut fs = self.fs.borrow_mut();
        self.modify_disk_inode(|disk_inode| {
            let data_blocks_dealloc = disk_inode.decrease_size(0, &self.block_device);
            for data_block in data_blocks_dealloc.into_iter() {
                fs.dealloc_data(data_block);
            }
        });
        block_cache_sync_all();
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use crate::fs::rfs::String;
    use crate::fs::rfs::ROOT_INODE;
    test!(test_inodehandler_create, {
        unsafe {
            let test_handler = ROOT_INODE.create("test", InodeType::File).unwrap();
            let find_result = ROOT_INODE.find("test").unwrap();
            test_assert!(
                find_result.block_id == test_handler.block_id
                    && find_result.block_offset == test_handler.block_offset,
                "Block id is wrong"
            );
        }
        Ok("passed")
    });

    test!(test_inodehandler_delete, {
        unsafe {
            let old_size = ROOT_INODE.get_file_size();
            ROOT_INODE.delete("test");
            let new_size = ROOT_INODE.get_file_size();
            let find_result = ROOT_INODE.find("test");
            test_assert!(
                find_result.is_none() && old_size - new_size == 32,
                "Delete Failed"
            );
        }
        Ok("passed")
    });

    test!(test_inodehandler_file_type, {
        unsafe {
            let test_handler = ROOT_INODE.create("test", InodeType::File).unwrap();
            test_assert!(test_handler.is_file(), "Test_reg is not file");
            test_assert!(!test_handler.is_dir(), "Test_reg is dir");
            test_assert!(ROOT_INODE.is_dir(), "ROOT_INODE is not dir");
            test_assert!(!ROOT_INODE.is_file(), "ROOT_INODE is file");
        }
        Ok("passed")
    });

    test!(test_inodehandler_read_write, {
        unsafe {
            let test_handler = ROOT_INODE.create("test1", InodeType::File).unwrap();
            test_handler.write_at(0, "1234".as_bytes());
            let mut buf = [0u8; 2];
            test_handler.read_at(1, buf.as_mut());
            test_assert!(
                "23" == String::from_utf8(buf.to_vec()).unwrap(),
                "Read Error"
            );
            Ok("passed")
        }
    });
}
