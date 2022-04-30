use super::{get_block_cache, BlockDevice, BLOCK_SZ};
use alloc::sync::Arc;
//每个磁盘块上每64bit为一组,一共有64组,BitmapBlock.0为组号,BitmapBlock.1为组内偏移
//BitmapBlock大小为4096bit,每个bit又表示一个磁盘块,故一个Bitmap可以管理4096个磁盘块
type BitmapBlock = [u64; 64];

const BLOCK_BITS: usize = BLOCK_SZ * 8;

pub struct Bitmap {
    start_block_id: usize,
    blocks: usize,
}

//位于第几个bitmap,第几个组(64个块为一组),偏移是多少
/// Return (block_pos, bits64_pos, inner_pos)
fn decomposition(mut bit: usize) -> (usize, usize, usize) {
    let block_pos = bit / BLOCK_BITS;
    bit %= BLOCK_BITS;
    (block_pos, bit / 64, bit % 64)
}

impl Bitmap {
    pub fn new(start_block_id: usize, blocks: usize) -> Self {
        Self {
            start_block_id,
            blocks,
        }
    }

    pub fn alloc(&self, block_device: &Arc<dyn BlockDevice>) -> Option<usize> {
        //这个0..self.blocks是什么东西?
        for block_id in 0..self.blocks {
            let pos = get_block_cache(
                block_id + self.start_block_id as usize,
                Arc::clone(block_device),
            )
            .lock()
            .modify(0, |bitmap_block: &mut BitmapBlock| {
                if let Some((bits64_pos, inner_pos)) = bitmap_block
                    .iter()
                    .enumerate()
                    //**bits是什么东西?
                    .find(|(_, bits64)| **bits64 != u64::MAX)
                    // trailing_ones 从后往前数有多少个1(在未遇到0之前),如 7的trailing_ones为3(0111), 6为0(0110)
                    .map(|(bits64_pos, bits64)| (bits64_pos, bits64.trailing_ones() as usize))
                {
                    // modify cache
                    bitmap_block[bits64_pos] |= 1u64 << inner_pos;
                    Some(block_id * BLOCK_BITS + bits64_pos * 64 + inner_pos as usize)
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
    //dealloc一个没有被alloc的块会引起严重错误
    pub fn dealloc(&self, block_device: &Arc<dyn BlockDevice>, bit: usize) {
        //???
        let (block_pos, bits64_pos, inner_pos) = decomposition(bit);
        get_block_cache(block_pos + self.start_block_id, Arc::clone(block_device))
            .lock()
            .modify(0, |bitmap_block: &mut BitmapBlock| {
                assert!(bitmap_block[bits64_pos] & (1u64 << inner_pos) > 0);
                bitmap_block[bits64_pos] ^= 1u64 << inner_pos;
            });
    }

    pub fn maximum(&self) -> usize {
        self.blocks * BLOCK_BITS
    }
}
