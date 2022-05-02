use super::{
    block_cache::{block_cache_sync_all, get_block_cache},
    block_dev::BlockDevice,
    efs::EasyFileSystem,
    layout::{Dirent, Inode, InodeType, DIRENT_SZ},
};
use crate::sync::mutex::{Mutex, MutexGuard};
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

/// Inode句柄
pub struct InodeHandler {
    block_id: u32,
    block_offset: usize,
    fs: Arc<Mutex<EasyFileSystem>>,
    block_device: Arc<dyn BlockDevice>,
}

impl InodeHandler {
    /// 创建新的Inode句柄
    pub fn new(
        block_id: u32,
        block_offset: usize,
        fs: Arc<Mutex<EasyFileSystem>>,
        block_device: Arc<dyn BlockDevice>,
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
        get_block_cache(self.block_id as usize, Arc::clone(&self.block_device))
            .lock()
            .read(self.block_offset, f)
    }

    /// 修改对应的Inode
    fn modify_disk_inode<V>(&self, f: impl FnOnce(&mut Inode) -> V) -> V {
        get_block_cache(self.block_id as usize, Arc::clone(&self.block_device))
            .lock()
            .modify(self.block_offset, f)
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

    pub fn find(&self, name: &str) -> Option<Arc<InodeHandler>> {
        let fs = self.fs.lock();
        self.read_disk_inode(|disk_inode| {
            self.find_inode_id(name, disk_inode).map(|inode_id| {
                let (block_id, block_offset) = fs.get_disk_inode_pos(inode_id);
                Arc::new(Self::new(
                    block_id,
                    block_offset,
                    self.fs.clone(),
                    self.block_device.clone(),
                ))
            })
        })
    }

    fn increase_size(
        &self,
        new_size: u32,
        disk_inode: &mut Inode,
        fs: &mut MutexGuard<EasyFileSystem>,
    ) {
        if new_size < disk_inode.size {
            return;
        }
        let blocks_needed = disk_inode.blocks_needed(new_size);
        let mut v: Vec<u32> = Vec::new();
        for _ in 0..blocks_needed {
            v.push(fs.alloc_data());
        }
        disk_inode.increase_size(new_size, v, &self.block_device);
    }
    fn decrease_size(
        &self,
        new_size: u32,
        disk_inode: &mut Inode,
        fs: &mut MutexGuard<EasyFileSystem>,
    ) {
        if new_size >= disk_inode.size {
            return;
        }
        let blocks_unneeded = disk_inode.blocks_unneeded(new_size);
        let mut v: Vec<u32> = Vec::new();
        let total_blocks = disk_inode.data_blocks();
        for num in 0..blocks_unneeded {
            //回收数据块
            let discard_block_id = disk_inode.get_block_id(total_blocks - num, &fs.block_device);
            fs.dealloc_data(discard_block_id);
            v.push(discard_block_id);
        }
        disk_inode.decrease_size(new_size, v, &self.block_device);
    }

    pub fn create(&self, name: &str, filetype: InodeType) -> Option<Arc<InodeHandler>> {
        let mut fs = self.fs.lock();
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
        get_block_cache(new_inode_block_id as usize, Arc::clone(&self.block_device))
            .lock()
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
        Some(Arc::new(Self::new(
            block_id,
            block_offset,
            self.fs.clone(),
            self.block_device.clone(),
        )))
    }
    pub fn delete(&self, name: &str) {
        let mut fs = self.fs.lock();
        let op = |dir_inode: &Inode| {
            // assert it is a directory
            assert!(dir_inode.is_dir());
            // has the file been created?
            let find_result = self.find_inode_id(name, dir_inode);
            if find_result.is_some() {
                let (block_id, block_offset) = fs.get_disk_inode_pos(find_result.unwrap());
                //释放文件的Inode
                let discard_inode_handle = Self::new(
                    block_id,
                    block_offset,
                    self.fs.clone(),
                    self.block_device.clone(),
                );
                discard_inode_handle.clear();
            }
            find_result
        };

        //找到该文件条目的id
        let discard_block_id = self.read_disk_inode(op).unwrap() as usize;

        self.modify_disk_inode(|dir_inode| {
            // delete file in the dirent
            let file_count = (dir_inode.size as usize) / DIRENT_SZ;
            // write dirent
            let mut dirent = Dirent::new("", 0);
            dir_inode.read_at(
                dir_inode.size as usize - DIRENT_SZ,
                dirent.as_bytes_mut(),
                &self.block_device,
            );
            dir_inode.write_at(
                discard_block_id * DIRENT_SZ,
                dirent.as_bytes(),
                &self.block_device,
            );
            // decrease size
            let new_size = (file_count - 1) * DIRENT_SZ;
            self.decrease_size(new_size as u32, dir_inode, &mut fs);
        });
    }

    pub fn ls(&self) -> Vec<String> {
        let _fs = self.fs.lock();
        self.read_disk_inode(|disk_inode| {
            let file_count = (disk_inode.size as usize) / DIRENT_SZ;
            let mut v: Vec<String> = Vec::new();
            for i in 0..file_count {
                let mut dirent = Dirent::empty();
                assert_eq!(
                    disk_inode.read_at(i * DIRENT_SZ, dirent.as_bytes_mut(), &self.block_device,),
                    DIRENT_SZ,
                );
                v.push(String::from(dirent.name()));
            }
            v
        })
    }

    pub fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        let _fs = self.fs.lock();
        self.read_disk_inode(|disk_inode| disk_inode.read_at(offset, buf, &self.block_device))
    }

    pub fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        let mut fs = self.fs.lock();
        let size = self.modify_disk_inode(|disk_inode| {
            self.increase_size((offset + buf.len()) as u32, disk_inode, &mut fs);
            disk_inode.write_at(offset, buf, &self.block_device)
        });
        block_cache_sync_all();
        size
    }

    /// 清空所有数据并回收块
    pub fn clear(&self) {
        let mut fs = self.fs.lock();
        let inode_id = fs.get_disk_inode_id(self.block_id as u32, self.block_offset);
        self.modify_disk_inode(|disk_inode| {
            let data_blocks_dealloc = disk_inode.clear_size(&self.block_device);
            fs.dealloc_inode(inode_id);
            for data_block in data_blocks_dealloc.into_iter() {
                fs.dealloc_data(data_block);
            }
        });
        block_cache_sync_all();
    }
}
