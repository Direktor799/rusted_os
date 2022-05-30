//! 驱动程序子模块

use crate::memory::frame::address::*;
use crate::memory::frame::frame_allocator::*;
use crate::memory::frame::memory_set::KERNEL_MEMORY_SET;
use alloc::vec::Vec;
pub mod virtio_block;
use crate::fs::rfs::block_dev::BlockDevice;
use crate::tools::uninit_cell::UninitCell;
use alloc::rc::Rc;
use virtio_block::VirtIOBlock;

static mut QUEUE_FRAMES: Vec<FrameTracker> = Vec::new();

pub static mut BLOCK_DEVICE: UninitCell<Rc<dyn BlockDevice>> = UninitCell::uninit();

#[no_mangle]
pub extern "C" fn virtio_dma_alloc(pages: usize) -> PhysAddr {
    let mut ppn_base = PhysPageNum(0);
    for i in 0..pages {
        let frame = frame_alloc().unwrap();
        if i == 0 {
            ppn_base = frame.ppn();
        }
        assert_eq!(frame.ppn().0, ppn_base.0 + i);
        unsafe {
            QUEUE_FRAMES.push(frame);
        }
    }
    ppn_base.addr()
}

#[no_mangle]
pub extern "C" fn virtio_dma_dealloc(pa: PhysAddr, pages: usize) -> i32 {
    let start_ppn = pa.ppn();
    let end_ppn = PhysPageNum(pa.ppn().0 + pages);
    unsafe {
        QUEUE_FRAMES.retain(|frame| !(start_ppn..end_ppn).contains(&frame.ppn()));
    }
    0
}

#[no_mangle]
pub extern "C" fn virtio_phys_to_virt(paddr: PhysAddr) -> VirtAddr {
    VirtAddr(paddr.0)
}

#[no_mangle]
pub extern "C" fn virtio_virt_to_phys(vaddr: VirtAddr) -> PhysAddr {
    let vpn = vaddr.vpn();
    let ppn = unsafe { KERNEL_MEMORY_SET.translate(vpn).unwrap() };
    PhysAddr(ppn.addr().0 + vaddr.page_offset())
}

pub fn init() {
    unsafe {
        BLOCK_DEVICE = UninitCell::init(Rc::new(VirtIOBlock::new()));
    }
    println!("mod drivers initialized!");
}
