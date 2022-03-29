use super::address::{PhysAddr, PhysPageNum};
use crate::config::{MEMORY_END_ADDR, PAGE_SIZE};
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::ops::Deref;

pub struct FrameTracker(PhysPageNum);

impl FrameTracker {
    pub fn new(ppn: PhysPageNum) -> Self {
        let bytes_array = ppn.get_bytes_array();
        for byte in bytes_array {
            *byte = 0;
        }
        Self(ppn)
    }

    pub fn addr(&self) -> PhysAddr {
        self.0.addr()
    }

    pub fn ppn(&self) -> PhysPageNum {
        self.0
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        unsafe {
            FRAME_ALLOCATOR.borrow_mut().dealloc(self.0);
        }
    }
}

pub static mut FRAME_ALLOCATOR: OutsideFrameAllocator = OutsideFrameAllocator::new();

pub struct OutsideFrameAllocator(RefCell<FrameAllocator>);

impl OutsideFrameAllocator {
    pub const fn new() -> Self {
        OutsideFrameAllocator(RefCell::new(FrameAllocator::new()))
    }
}

impl Deref for OutsideFrameAllocator {
    type Target = RefCell<FrameAllocator>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct FrameAllocator {
    curr_ppn: PhysPageNum,
    end_ppn: PhysPageNum,
    recycled: Vec<PhysPageNum>,
}

impl FrameAllocator {
    pub const fn new() -> Self {
        FrameAllocator {
            curr_ppn: PhysPageNum(0),
            end_ppn: PhysPageNum(0),
            recycled: vec![],
        }
    }

    pub fn init(&mut self, start_ppn: PhysPageNum, end_ppn: PhysPageNum) {
        self.curr_ppn = start_ppn;
        self.end_ppn = end_ppn;
    }

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
        FRAME_ALLOCATOR
            .borrow_mut()
            .init(frame_start_num, MEMORY_END_ADDR.ppn());
    }
}
