use crate::config::{MEMORY_END_ADDR, PAGE_SIZE};
use address::{PhysAddr, PhysPageNum};
use frame::FRAME_ALLOCATOR;

mod address;
mod frame;
mod memory_set;
mod page_table;
mod segment;

pub fn init() {
    extern "C" {
        fn kernel_end(); // 在linker script中定义的内存地址
    }
    let kernel_end_addr = kernel_end as usize;
    let frame_start_num = PhysPageNum((kernel_end_addr + PAGE_SIZE - 1) / PAGE_SIZE);
    unsafe {
        FRAME_ALLOCATOR
            .borrow_mut()
            .init(frame_start_num, PhysAddr(MEMORY_END_ADDR).ppn());
    }
}
