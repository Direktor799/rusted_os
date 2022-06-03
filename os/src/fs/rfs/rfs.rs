//! 磁盘块管理子模块

use super::{
    bitmap::Bitmap,
    block_cache::{block_cache_sync_all, get_block_cache},
    block_dev::BlockDevice,
    layout::{Inode, InodeType, SuperBlock},
    vfs::InodeHandler,
};
use super::{DataBlock, BLOCK_SZ};
use alloc::rc::Rc;
use core::cell::RefCell;
use core::mem::size_of;

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

    /// 打开设备上的文件系统
    pub fn open(block_device: Rc<dyn BlockDevice>) -> Option<Rc<RefCell<Self>>> {
        // 根据超级块信息初始化文件系统
        get_block_cache(0, Rc::clone(&block_device))
            .borrow()
            .read(0, |super_block: &SuperBlock| {
                if !super_block.is_valid() {
                    return None;
                }
                let inode_total_blocks = super_block.inode_bitmap_blocks + super_block.inode_blocks;
                let rfs = Self {
                    block_device,
                    inode_bitmap: Bitmap::new(1, super_block.inode_bitmap_blocks as usize),
                    data_bitmap: Bitmap::new(
                        (1 + inode_total_blocks) as usize,
                        super_block.data_bitmap_blocks as usize,
                    ),
                    inode_start_block: 1 + super_block.inode_bitmap_blocks,
                    data_start_block: 1 + inode_total_blocks + super_block.data_bitmap_blocks,
                };
                Some(Rc::new(RefCell::new(rfs)))
            })
    }
    /// 获取根目录的引用
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

    /// 回收Inode
    pub fn dealloc_inode(&mut self, inode_id: u32) {
        self.inode_bitmap
            .dealloc(&self.block_device, inode_id as usize)
    }

    /// 分配数据块
    pub fn alloc_data(&mut self) -> u32 {
        self.data_bitmap.alloc(&self.block_device).unwrap() as u32 + self.data_start_block
    }

    /// 回收数据块
    pub fn dealloc_data(&mut self, block_id: u32) {
        get_block_cache(block_id as usize, Rc::clone(&self.block_device))
            .borrow_mut()
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
#[cfg(test)]
mod test {
    use super::*;
    use crate::drivers::BLOCK_DEVICE;
    test!(test_get_inode_id, {
        unsafe {
            if let Some(rfs) = RustedFileSystem::open(BLOCK_DEVICE.clone()) {
                test_assert!(
                    rfs.as_ref().borrow().get_disk_inode_id(2, 500) == 3,
                    "get_disk_inode_id_failed"
                );
                test_assert!(
                    rfs.as_ref().borrow().get_disk_inode_id(3, 0) == 4,
                    "get_disk_inode_id_failed"
                );
                test_assert!(
                    rfs.as_ref().borrow().get_disk_inode_id(3, 200) == 5,
                    "get_disk_inode_id_failed"
                );
                test_assert!(
                    rfs.as_ref().borrow().get_disk_inode_id(4, 100) == 8,
                    "get_disk_inode_id_failed"
                );
            }
        }
        Ok("passed")
    });

    test!(test_get_inode_pos, {
        unsafe {
            if let Some(rfs) = RustedFileSystem::open(BLOCK_DEVICE.clone()) {
                let (disk_id, offset) = rfs.as_ref().borrow().get_disk_inode_pos(3);
                test_assert!(disk_id == 2 && offset == 384, "get_disk_inode_id_failed");
                let (disk_id, offset) = rfs.as_ref().borrow().get_disk_inode_pos(4);
                test_assert!(disk_id == 3 && offset == 0, "get_disk_inode_id_failed");
                let (disk_id, offset) = rfs.as_ref().borrow().get_disk_inode_pos(5);
                test_assert!(disk_id == 3 && offset == 128, "get_disk_inode_id_failed");
                let (disk_id, offset) = rfs.as_ref().borrow().get_disk_inode_pos(8);
                test_assert!(disk_id == 4 && offset == 0, "get_disk_inode_id_failed");
            }
        }
        Ok("passed")
    });

    test!(test_alloc, {
        unsafe {
            if let Some(rfs) = RustedFileSystem::open(BLOCK_DEVICE.clone()) {
                let mut using_rfs = rfs.as_ref().borrow_mut();
                let cur_inode = using_rfs.alloc_inode();
                using_rfs.dealloc_inode(cur_inode);
                test_assert!(using_rfs.alloc_inode() == cur_inode, "test failed");
                let cur_data = using_rfs.alloc_data();
                using_rfs.dealloc_data(cur_data);
                test_assert!(using_rfs.alloc_data() == cur_data, "test failed");
            }
        }
        Ok("passed")
    });
}
