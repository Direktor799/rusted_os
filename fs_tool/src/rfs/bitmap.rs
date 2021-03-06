//! Bitmap操作子模块。

use super::{block_cache::get_block_cache, block_dev::BlockDevice, BLOCK_SZ};
use std::rc::Rc;

/// 方便分组读写的BitmapBlock定义
type BitmapBlock = [u64; 64];

/// 一个块中的bit数
const BLOCK_BITS: usize = BLOCK_SZ * 8;

/// 块设备中Bitmap的内存抽象
pub struct Bitmap {
    start_block_id: usize,
    blocks: usize,
}

impl Bitmap {
    /// 创建Bitmap
    pub fn new(start_block_id: usize, blocks: usize) -> Self {
        Self {
            start_block_id,
            blocks,
        }
    }

    /// 分配一个空闲块
    pub fn alloc(&self, block_device: &Rc<dyn BlockDevice>) -> Option<usize> {
        // 遍历每一个BitmapBlock
        for block_id in 0..self.blocks {
            let pos = get_block_cache(block_id + self.start_block_id, Rc::clone(block_device))
                .borrow_mut()
                .modify(0, |bitmap_block: &mut BitmapBlock| {
                    if let Some((group_pos, bit_pos)) = bitmap_block
                        .iter()
                        .enumerate()
                        // 以64位为一组查找有空闲位的组
                        .find(|(_, bits_group)| **bits_group != u64::MAX)
                        .map(|(group_pos, bits_group)| {
                            (group_pos, bits_group.trailing_ones() as usize)
                        })
                    {
                        // 标为已分配
                        bitmap_block[group_pos] |= 1u64 << bit_pos;
                        // 返回块号
                        Some(block_id * BLOCK_BITS + group_pos * 64 + bit_pos)
                    } else {
                        None
                    }
                });
            if pos.is_some() {
                return pos;
            }
        }
        None
    }

    /// 最多可管理的资源数量
    pub fn maximum(&self) -> usize {
        self.blocks * BLOCK_BITS
    }
}
