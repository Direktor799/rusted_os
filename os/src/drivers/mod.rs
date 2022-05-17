//! 驱动程序子模块

use crate::memory::frame::address::*;
use crate::memory::frame::frame_allocator::*;
use crate::sync::mutex::Mutex;
use alloc::vec::Vec;
pub mod virtio_block;
use crate::fs::block_dev::BlockDevice;
use alloc::sync::Arc;
use virtio_block::VirtIOBlock;

static QUEUE_FRAMES: Mutex<Vec<FrameTracker>> = Mutex::new(Vec::new());

pub static mut BLOCK_DEVICE: Option<Arc<dyn BlockDevice>> = None;

#[no_mangle]
pub extern "C" fn virtio_dma_alloc(pages: usize) -> PhysAddr {
    let mut ppn_base = PhysPageNum(0);
    for i in 0..pages {
        let frame = unsafe { FRAME_ALLOCATOR.alloc().unwrap() };
        if i == 0 {
            ppn_base = frame.ppn();
        }
        assert_eq!(frame.ppn().0, ppn_base.0 + i);
        QUEUE_FRAMES.lock().push(frame);
    }
    ppn_base.addr()
}

#[no_mangle]
pub extern "C" fn virtio_dma_dealloc(pa: PhysAddr, pages: usize) -> i32 {
    let mut ppn_base: PhysPageNum = pa.ppn();
    for _ in 0..pages {
        unsafe { FRAME_ALLOCATOR.dealloc(ppn_base) };
        ppn_base = PhysPageNum(ppn_base.0 + 1);
    }
    0
}

#[no_mangle]
pub extern "C" fn virtio_phys_to_virt(paddr: PhysAddr) -> VirtAddr {
    VirtAddr(paddr.0)
}

#[no_mangle]
pub extern "C" fn virtio_virt_to_phys(vaddr: VirtAddr) -> PhysAddr {
    PhysAddr(vaddr.0)
}

pub fn init() {
    unsafe {
        BLOCK_DEVICE = Some(Arc::new(VirtIOBlock::new()));
    }
    println!("mod drivers initialized!");
}
