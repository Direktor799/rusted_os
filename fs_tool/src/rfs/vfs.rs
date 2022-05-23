use super::{
    block_cache::{block_cache_sync_all, get_block_cache},
    block_dev::BlockDevice,
    layout::{Dirent, Inode, InodeType, DIRENT_SZ},
    rfs::RustedFileSystem,
};
use std::cell::RefCell;
use std::rc::Rc;

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
            block_id: block_id,
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

    pub fn get_inode_id(&self) -> u32 {
        self.fs
            .borrow()
            .get_disk_inode_id(self.block_id, self.block_offset)
    }

    fn find_inode_id(&self, name: &str, disk_inode: &Inode) -> Option<u32> {
        // assert it is a directory
        assert!(disk_inode.is_dir());
        let file_count = (disk_inode.size as usize) / DIRENT_SZ;
        let mut dirent = Dirent::empty();
        for i in 0..file_count {
            assert_eq!(
                disk_inode.read_at(DIRENT_SZ * i, dirent.as_bytes_mut(), &self.block_device,),
                DIRENT_SZ,
            );
            if dirent.name() == name {
                return Some(dirent.inode_number() as u32);
            }
        }
        None
    }

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

    pub fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        let mut fs = self.fs.borrow_mut();
        let size = self.modify_disk_inode(|disk_inode| {
            self.increase_size((offset + buf.len()) as u32, disk_inode, &mut fs);
            disk_inode.write_at(offset, buf, &self.block_device)
        });
        block_cache_sync_all();
        size
    }
}
