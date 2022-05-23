//! 页式内存分配器子模块
use super::address::{PhysAddr, PhysPageNum};
use crate::config::{MEMORY_END_ADDR, PAGE_SIZE};
use crate::tools::uninit_cell::UninitCell;
use alloc::vec;
use alloc::vec::Vec;

/// 物理页帧（方便通过RAII自动管理内存）
#[derive(Debug)]
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
pub static mut FRAME_ALLOCATOR: UninitCell<FrameAllocator> = UninitCell::uninit();

/// 栈式物理页面分配器
pub struct FrameAllocator {
    curr_ppn: PhysPageNum,
    end_ppn: PhysPageNum,
    recycled: Vec<PhysPageNum>,
}

impl FrameAllocator {
    /// 根据传入的物理页面区间创建新的分配器
    pub const fn new(start_ppn: PhysPageNum, end_ppn: PhysPageNum) -> Self {
        FrameAllocator {
            curr_ppn: start_ppn,
            end_ppn,
            recycled: vec![],
        }
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

pub fn init() {
    extern "C" {
        fn kernel_end(); // 在linker script中定义的内存地址
    }
    let kernel_end_addr = kernel_end as usize;
    let frame_start_num = PhysPageNum((kernel_end_addr + PAGE_SIZE - 1) / PAGE_SIZE);
    unsafe {
        FRAME_ALLOCATOR = UninitCell::init(FrameAllocator::new(
            frame_start_num,
            PhysAddr(MEMORY_END_ADDR).ppn(),
        ));
    }
}

test!(test_frame_allocator, {
    unsafe {
        let start_ppn = FRAME_ALLOCATOR.curr_ppn;
        let f1 = FRAME_ALLOCATOR.alloc().expect("No space");
        test_assert!(
            f1.ppn() == PhysPageNum(start_ppn.0),
            "Wrong frame allocated"
        );
        {
            let f2 = FRAME_ALLOCATOR.alloc().expect("No space");
            test_assert!(
                f2.ppn() == PhysPageNum(start_ppn.0 + 1),
                "Wrong frame allocated"
            );
            test_assert!(
                FRAME_ALLOCATOR.curr_ppn == PhysPageNum(start_ppn.0 + 2)
                    && FRAME_ALLOCATOR.recycled.is_empty(),
                "Alloc error"
            );
        }
        test_assert!(
            FRAME_ALLOCATOR.curr_ppn == PhysPageNum(start_ppn.0 + 2)
                && FRAME_ALLOCATOR.recycled.len() == 1,
            "Dealloc error"
        );
        let f2 = FRAME_ALLOCATOR.alloc().expect("No space");
        test_assert!(
            f2.ppn() == PhysPageNum(start_ppn.0 + 1),
            "Wrong frame allocated"
        );
        assert!(
            FRAME_ALLOCATOR.curr_ppn == PhysPageNum(start_ppn.0 + 2)
                && FRAME_ALLOCATOR.recycled.is_empty(),
            "Alloc error"
        );
    }
    Ok("passed")
});
