//! 磁盘块管理子模块

use super::{
    bitmap::Bitmap,
    block_cache::{block_cache_sync_all, get_block_cache},
    block_dev::BlockDevice,
    layout::{Inode, InodeType, SuperBlock},
    vfs::InodeHandler,
};
use super::{DataBlock, BLOCK_SZ};
use alloc::sync::Arc;
use core::mem::size_of;
use spin::Mutex;

/// 块内Inode数量
const INODES_PER_BLOCK: u32 = (BLOCK_SZ / size_of::<Inode>()) as u32;

/// efs文件系统
pub struct EasyFileSystem {
    pub block_device: Arc<dyn BlockDevice>,
    pub inode_bitmap: Bitmap,
    pub data_bitmap: Bitmap,
    inode_start_block: u32,
    data_start_block: u32,
}

impl EasyFileSystem {
    /// 根据参数在设备上创建新的文件系统
    pub fn format(
        block_device: Arc<dyn BlockDevice>,
        total_blocks: u32,
        inode_bitmap_blocks: u32,
    ) -> Arc<Mutex<Self>> {
        // 计算磁盘布局
        let inode_bitmap = Bitmap::new(1, inode_bitmap_blocks as usize);
        let inode_blocks =
            ((inode_bitmap.maximum() * size_of::<Inode>() + BLOCK_SZ - 1) / BLOCK_SZ) as u32;
        let inode_total_blocks = inode_bitmap_blocks + inode_blocks;
        let data_total_blocks = total_blocks - 1 - inode_total_blocks;
        // 4097块为一组（Bitmap和其管理的4096块）
        let data_bitmap_blocks = (data_total_blocks + 4097 - 1) / 4097;
        let data_blocks = data_total_blocks - data_bitmap_blocks;
        let data_bitmap = Bitmap::new(
            (1 + inode_bitmap_blocks + inode_blocks) as usize,
            data_bitmap_blocks as usize,
        );
        let mut efs = Self {
            block_device: Arc::clone(&block_device),
            inode_bitmap,
            data_bitmap,
            inode_start_block: 1 + inode_bitmap_blocks,
            data_start_block: 1 + inode_total_blocks + data_bitmap_blocks,
        };
        // 清空数据
        for i in 0..total_blocks {
            get_block_cache(i as usize, Arc::clone(&block_device))
                .lock()
                .modify(0, |data_block: &mut DataBlock| {
                    for byte in data_block.iter_mut() {
                        *byte = 0;
                    }
                });
        }
        // 初始化超级块
        get_block_cache(0, Arc::clone(&block_device)).lock().modify(
            0,
            |super_block: &mut SuperBlock| {
                super_block.init(
                    total_blocks,
                    inode_bitmap_blocks,
                    inode_blocks,
                    data_bitmap_blocks,
                    data_blocks,
                );
            },
        );
        // 初始化根Inode
        let root_inode = efs.alloc_inode();
        let (root_inode_block_id, root_inode_offset) = efs.get_disk_inode_pos(root_inode);
        get_block_cache(root_inode_block_id as usize, Arc::clone(&block_device))
            .lock()
            .modify(root_inode_offset, |disk_inode: &mut Inode| {
                disk_inode.init(InodeType::Directory);
            });
        // 立刻写回
        block_cache_sync_all();
        Arc::new(Mutex::new(efs))
    }

    /// 打开设备上的文件系统
    pub fn open(block_device: Arc<dyn BlockDevice>) -> Arc<Mutex<Self>> {
        // 根据超级块信息初始化文件系统
        get_block_cache(0, Arc::clone(&block_device))
            .lock()
            .read(0, |super_block: &SuperBlock| {
                assert!(super_block.is_valid(), "Error loading EFS!");
                let inode_total_blocks = super_block.inode_bitmap_blocks + super_block.inode_blocks;
                let efs = Self {
                    block_device,
                    inode_bitmap: Bitmap::new(1, super_block.inode_bitmap_blocks as usize),
                    data_bitmap: Bitmap::new(
                        (1 + inode_total_blocks) as usize,
                        super_block.data_bitmap_blocks as usize,
                    ),
                    inode_start_block: 1 + super_block.inode_bitmap_blocks,
                    data_start_block: 1 + inode_total_blocks + super_block.data_bitmap_blocks,
                };
                Arc::new(Mutex::new(efs))
            })
    }

    pub fn root_inode(efs: &Arc<Mutex<Self>>) -> InodeHandler {
        let block_device = Arc::clone(&efs.lock().block_device);
        // acquire efs lock temporarily
        let (block_id, block_offset) = efs.lock().get_disk_inode_pos(0);
        // release efs lock
        InodeHandler::new(block_id, block_offset, Arc::clone(efs), block_device)
    }

    /// 获取Inode在磁盘上的块号和偏移
    pub fn get_disk_inode_pos(&self, inode_id: u32) -> (u32, usize) {
        let block_id = self.inode_start_block + inode_id / INODES_PER_BLOCK;
        (
            block_id,
            (inode_id % INODES_PER_BLOCK) as usize * size_of::<Inode>(),
        )
    }

    /// 获取数据块对应的磁盘块号
    pub fn get_data_block_id(&self, data_block_id: u32) -> u32 {
        self.data_start_block + data_block_id
    }

    /// 分配Inode
    pub fn alloc_inode(&mut self) -> u32 {
        self.inode_bitmap.alloc(&self.block_device).unwrap() as u32
    }

    /// TODO:回收Inode
    pub fn dealloc_inode(&mut self, inode_id: u32) {
        let (block_id, block_offset) = self.get_disk_inode_pos(inode_id);
        let recycled_blocks = get_block_cache(block_id as usize, Arc::clone(&self.block_device))
            .lock()
            .modify(block_offset, |inode: &mut Inode| {
                inode.clear_size(&self.block_device)
            });
        for block in recycled_blocks {}
        self.inode_bitmap
            .dealloc(&self.block_device, inode_id as usize);
    }

    /// 分配数据块
    pub fn alloc_data(&mut self) -> u32 {
        self.data_bitmap.alloc(&self.block_device).unwrap() as u32 + self.data_start_block
    }

    /// 回收数据块
    pub fn dealloc_data(&mut self, block_id: u32) {
        get_block_cache(block_id as usize, Arc::clone(&self.block_device))
            .lock()
            .modify(0, |data_block: &mut DataBlock| {
                data_block.iter_mut().for_each(|p| {
                    *p = 0;
                })
            });
        self.data_bitmap.dealloc(
            &self.block_device,
            (block_id - self.data_start_block) as usize,
        )
    }
}
