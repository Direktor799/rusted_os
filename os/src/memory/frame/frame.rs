//! 页式内存分配器子模块
use super::address::PhysPageNum;
use alloc::vec;
use alloc::vec::Vec;

/// 物理页帧（方便通过RAII自动管理内存）
pub struct FrameTracker(PhysPageNum);

impl FrameTracker {
    /// 根据物理页号创建物理页帧
    pub fn new(ppn: PhysPageNum) -> Self {
        let bytes_array = ppn.get_bytes_array();
        for byte in bytes_array {
            *byte = 0;
        }
        Self(ppn)
    }

    /// 获取物理页帧对应物理页号
    pub fn ppn(&self) -> PhysPageNum {
        self.0
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        unsafe {
            FRAME_ALLOCATOR.dealloc(self.0);
        }
    }
}

/// 全局物理页分配器实例
pub static mut FRAME_ALLOCATOR: FrameAllocator = FrameAllocator::new();

/// 栈式物理页面分配器
pub struct FrameAllocator {
    curr_ppn: PhysPageNum,
    end_ppn: PhysPageNum,
    recycled: Vec<PhysPageNum>,
}

impl FrameAllocator {
    /// 创建新的分配器
    pub const fn new() -> Self {
        FrameAllocator {
            curr_ppn: PhysPageNum(0),
            end_ppn: PhysPageNum(0),
            recycled: vec![],
        }
    }

    /// 根据传入的物理页面区间初始化分配器
    pub fn init(&mut self, start_ppn: PhysPageNum, end_ppn: PhysPageNum) {
        self.curr_ppn = start_ppn;
        self.end_ppn = end_ppn;
    }

    /// 分配物理页面
    pub fn alloc(&mut self) -> Option<FrameTracker> {
        if let Some(ppn) = self.recycled.pop() {
            // 优先使用已回收的页面
            Some(FrameTracker::new(ppn))
        } else if self.curr_ppn < self.end_ppn {
            // 其次使用未分配的页面
            let ppn = self.curr_ppn;
            self.curr_ppn.0 += 1;
            Some(FrameTracker::new(ppn))
        } else {
            None // 否则分配失败
        }
    }

    /// 回收物理页面
    pub fn dealloc(&mut self, ppn: PhysPageNum) {
        if ppn >= self.curr_ppn || self.recycled.iter().any(|&v| v == ppn) {
            panic!("{:?} is not allocated!", ppn);
        }
        self.recycled.push(ppn);
    }
}
