//! 磁盘块管理子模块

use super::{
    bitmap::Bitmap,
    block_cache::{block_cache_sync_all, get_block_cache},
    block_dev::BlockDevice,
    layout::{Inode, InodeType, SuperBlock},
    vfs::InodeHandler,
};
use super::{DataBlock, BLOCK_SZ};
use std::cell::RefCell;
use std::mem::size_of;
use std::rc::Rc;

/// 块内Inode数量
const INODES_PER_BLOCK: u32 = (BLOCK_SZ / size_of::<Inode>()) as u32;

/// rfs文件系统
pub struct RustedFileSystem {
    pub block_device: Rc<dyn BlockDevice>,
    pub inode_bitmap: Bitmap,
    pub data_bitmap: Bitmap,
    inode_start_block: u32,
    data_start_block: u32,
}

impl RustedFileSystem {
    /// 根据参数在设备上创建新的文件系统
    pub fn format(
        block_device: Rc<dyn BlockDevice>,
        total_blocks: u32,
        inode_bitmap_blocks: u32,
    ) -> Rc<RefCell<Self>> {
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
        let mut rfs = Self {
            block_device: Rc::clone(&block_device),
            inode_bitmap,
            data_bitmap,
            inode_start_block: 1 + inode_bitmap_blocks,
            data_start_block: 1 + inode_total_blocks + data_bitmap_blocks,
        };
        // 清空数据
        for i in 0..total_blocks {
            get_block_cache(i as usize, Rc::clone(&block_device))
                .borrow_mut()
                .modify(0, |data_block: &mut DataBlock| {
                    for byte in data_block.iter_mut() {
                        *byte = 0;
                    }
                });
        }
        // 初始化超级块
        get_block_cache(0, Rc::clone(&block_device))
            .borrow_mut()
            .modify(0, |super_block: &mut SuperBlock| {
                super_block.init(
                    total_blocks,
                    inode_bitmap_blocks,
                    inode_blocks,
                    data_bitmap_blocks,
                    data_blocks,
                );
            });
        // 初始化根Inode
        let root_inode = rfs.alloc_inode();
        let (root_inode_block_id, root_inode_offset) = rfs.get_disk_inode_pos(root_inode);
        get_block_cache(root_inode_block_id as usize, Rc::clone(&block_device))
            .borrow_mut()
            .modify(root_inode_offset, |disk_inode: &mut Inode| {
                disk_inode.init(InodeType::Directory);
            });
        // 立刻写回
        block_cache_sync_all();
        Rc::new(RefCell::new(rfs))
    }

    pub fn root_inode(rfs: &Rc<RefCell<Self>>) -> InodeHandler {
        let block_device = Rc::clone(&rfs.borrow().block_device);
        // acquire rfs lock temporarily
        let (block_id, block_offset) = rfs.borrow().get_disk_inode_pos(0);
        // release rfs lock
        InodeHandler::new(block_id, block_offset, Rc::clone(rfs), block_device)
    }

    /// 根据Inode编号获取在磁盘上的块号和偏移
    pub fn get_disk_inode_pos(&self, inode_id: u32) -> (u32, usize) {
        let block_id = self.inode_start_block + inode_id / INODES_PER_BLOCK;
        (
            block_id,
            (inode_id % INODES_PER_BLOCK) as usize * size_of::<Inode>(),
        )
    }

    /// 根据在磁盘上的块号和偏移获取Inode编号
    pub fn get_disk_inode_id(&self, block_id: u32, block_offset: usize) -> u32 {
        (block_id - self.inode_start_block) * INODES_PER_BLOCK
            + block_offset as u32 / size_of::<Inode>() as u32
    }

    /// 分配Inode
    pub fn alloc_inode(&mut self) -> u32 {
        self.inode_bitmap.alloc(&self.block_device).unwrap() as u32
    }

    /// 分配数据块
    pub fn alloc_data(&mut self) -> u32 {
        self.data_bitmap.alloc(&self.block_device).unwrap() as u32 + self.data_start_block
    }
}
