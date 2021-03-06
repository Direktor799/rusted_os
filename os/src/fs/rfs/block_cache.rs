//! 块缓存管理子模块
use super::{block_dev::BlockDevice, BLOCK_SZ};
use crate::tools::uninit_cell::UninitCell;
use alloc::rc::Rc;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;

/// 内存中的块缓存
pub struct BlockCache {
    cache: Vec<u8>,
    modified: bool,
    device: Rc<dyn BlockDevice>,
    block_id: usize,
}

impl BlockCache {
    /// 从磁盘块读到缓存中
    pub fn new(block_id: usize, device: Rc<dyn BlockDevice>) -> Self {
        let mut cache = vec![0u8; BLOCK_SZ];
        device.read_block(block_id, &mut cache);
        Self {
            cache,
            modified: false,
            device,
            block_id,
        }
    }

    /// 块内偏移对应到内存中的地址
    fn addr_of_offset(&self, offset: usize) -> usize {
        &self.cache[offset] as *const _ as usize
    }

    /// 获取只读的缓存引用
    fn get_ref<T>(&self, offset: usize) -> &T
    where
        T: Sized,
    {
        let type_size = core::mem::size_of::<T>();
        assert!(offset + type_size <= BLOCK_SZ);
        let addr = self.addr_of_offset(offset);
        unsafe { &*(addr as *const T) }
    }

    /// 获取可变的的缓存引用
    fn get_mut<T>(&mut self, offset: usize) -> &mut T
    where
        T: Sized,
    {
        let type_size = core::mem::size_of::<T>();
        assert!(offset + type_size <= BLOCK_SZ);
        self.modified = true;
        let addr = self.addr_of_offset(offset);
        unsafe { &mut *(addr as *mut T) }
    }

    /// 对缓存引用的只读操作
    pub fn read<T, V>(&self, offset: usize, f: impl FnOnce(&T) -> V) -> V {
        f(self.get_ref(offset))
    }

    /// 对缓存引用的可变操作
    pub fn modify<T, V>(&mut self, offset: usize, f: impl FnOnce(&mut T) -> V) -> V {
        f(self.get_mut(offset))
    }

    /// 将对缓存的操作写回磁盘
    pub fn sync(&mut self) {
        if self.modified {
            self.modified = false;
            self.device.write_block(self.block_id, &self.cache);
        }
    }
}
/// 给BlockCache添加Drop Trait, 在每个BlockCache被替换出去时将块中内容写入物理磁盘
impl Drop for BlockCache {
    fn drop(&mut self) {
        self.sync()
    }
}

/// 磁盘块缓冲区数量
const BLOCK_CACHE_SIZE: usize = 16;

/// 块缓存管理器
pub struct BlockCacheManager {
    queue: Vec<(usize, Rc<RefCell<BlockCache>>)>,
}

impl BlockCacheManager {
    pub fn new() -> Self {
        Self { queue: Vec::new() }
    }
    pub fn get_block_cache(
        &mut self,
        block_id: usize,
        block_device: Rc<dyn BlockDevice>,
    ) -> Rc<RefCell<BlockCache>> {
        if let Some((_, cache)) = self.queue.iter().find(|(id, _)| *id == block_id) {
            cache.clone()
        } else {
            if self.queue.len() == BLOCK_CACHE_SIZE {
                if let Some((idx, _)) = self
                    .queue
                    .iter()
                    .enumerate()
                    .find(|(_, (_, cache))| Rc::strong_count(cache) == 1)
                {
                    self.queue.swap_remove(idx);
                } else {
                    panic!("Run out of BlockCache!");
                }
            }
            let block_cache = Rc::new(RefCell::new(BlockCache::new(
                block_id,
                block_device.clone(),
            )));
            self.queue.push((block_id, Rc::clone(&block_cache)));
            block_cache
        }
    }
}

/// 全局块缓存管理器
pub static mut BLOCK_CACHE_MANAGER: UninitCell<BlockCacheManager> = UninitCell::uninit();

/// 获取块缓存
pub fn get_block_cache(block_id: usize, device: Rc<dyn BlockDevice>) -> Rc<RefCell<BlockCache>> {
    unsafe { BLOCK_CACHE_MANAGER.get_block_cache(block_id, device) }
}

/// 同步所有块缓存
pub fn block_cache_sync_all() {
    unsafe {
        for (_, cache) in BLOCK_CACHE_MANAGER.queue.iter() {
            cache.borrow_mut().sync();
        }
    }
}

pub fn init() {
    unsafe {
        BLOCK_CACHE_MANAGER = UninitCell::init(BlockCacheManager::new());
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use crate::drivers::BLOCK_DEVICE;
    use alloc::string::String;
    test!(test_block_cache, {
        let cur_block;
        unsafe {
            cur_block = BLOCK_CACHE_MANAGER.get_block_cache(10, BLOCK_DEVICE.clone());
        }
        cur_block.borrow_mut().modify(0, |test: &mut [u8; 8]| {
            test[0] = b'1';
            test[1] = b'2';
            test[2] = b'3';
            test[3] = b'4';
        });
        for i in 0..4 {
            let s = cur_block.borrow().read(i, |test: &[u8; 4]| {
                String::from_utf8(test.to_vec()).unwrap()
            });
            test_assert!(s[..4 - i] == "1234"[i..], "Read or Write Failed");
        }
        Ok("passed")
    });
}
