use super::{BlockDevice, BLOCK_SZ};
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use spin::Mutex;

/// 内存中的块缓存
pub struct BlockCache {
    cache: [u8; BLOCK_SZ],
    modified: bool,
    device: Arc<dyn BlockDevice>,
    block_id: usize,
}

impl BlockCache {
    /// 从磁盘块读到缓存中
    pub fn new(block_id: usize, device: Arc<dyn BlockDevice>) -> Self {
        let mut cache = [0u8; BLOCK_SZ];
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
    pub fn get_ref<T>(&self, offset: usize) -> &T
    where
        T: Sized,
    {
        let type_size = core::mem::size_of::<T>();
        assert!(offset + type_size <= BLOCK_SZ);
        let addr = self.addr_of_offset(offset);
        unsafe { &*(addr as *const T) }
    }

    /// 获取可变的的缓存引用
    pub fn get_mut<T>(&mut self, offset: usize) -> &mut T
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

impl Drop for BlockCache {
    fn drop(&mut self) {
        self.sync()
    }
}

/// 磁盘块缓冲区数量
const BLOCK_CACHE_SIZE: usize = 16;

/// 块缓存管理器
pub struct BlockCacheManager {
    queue: VecDeque<(usize, Arc<Mutex<BlockCache>>)>,
}

impl BlockCacheManager {
    /// 新建块缓存管理器
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    /// 加载对应块到缓存
    pub fn get_block_cache(
        &mut self,
        block_id: usize,
        device: Arc<dyn BlockDevice>,
    ) -> Arc<Mutex<BlockCache>> {
        if let Some(pair) = self.queue.iter().find(|pair| pair.0 == block_id) {
            Arc::clone(&pair.1)
        } else {
            // 需要替换
            if self.queue.len() == BLOCK_CACHE_SIZE {
                // 删除当前未被使用的块
                if let Some((idx, _)) = self
                    .queue
                    .iter()
                    .enumerate()
                    .find(|(_, pair)| Arc::strong_count(&pair.1) == 1)
                {
                    self.queue.remove(idx);
                } else {
                    panic!("Run out of BlockCache!");
                }
            }
            // 加载新的缓存
            let block_cache = Arc::new(Mutex::new(BlockCache::new(block_id, Arc::clone(&device))));
            self.queue.push_back((block_id, Arc::clone(&block_cache)));
            block_cache
        }
    }
}

/// 全局块缓存管理器
pub static mut BLOCK_CACHE_MANAGER: Option<Mutex<BlockCacheManager>> = None;

/// 获取块缓存
pub fn get_block_cache(block_id: usize, device: Arc<dyn BlockDevice>) -> Arc<Mutex<BlockCache>> {
    unsafe {
        BLOCK_CACHE_MANAGER
            .as_ref()
            .unwrap()
            .lock()
            .get_block_cache(block_id, device)
    }
}

/// 同步所有块缓存
pub fn block_cache_sync_all() {
    let manager = unsafe { BLOCK_CACHE_MANAGER.as_ref().unwrap().lock() };
    for (_, cache) in manager.queue.iter() {
        cache.lock().sync();
    }
}
