use crate::config::{MEMORY_END_ADDR, PAGE_SIZE};
use address::{PhysAddr, PhysPageNum};
use frame::FRAME_ALLOCATOR;
use memory_set::MemorySet;

pub mod address;
mod frame;
pub mod memory_set;
mod page_table;
mod segment;

pub use page_table::{PageTable, R, W};
pub static mut KERNEL_MEMORY_SET: Option<MemorySet> = None;

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
    unsafe {
        KERNEL_MEMORY_SET = Some(memory_set::MemorySet::new_kernel());
        KERNEL_MEMORY_SET.as_ref().unwrap().activate();
    }
}
