use crate::config::MMIO;
use crate::fs::block_dev::BlockDevice;
use core::ops::Deref;
use virtio_drivers::{VirtIOBlk, VirtIOHeader};
pub struct VirtIOBlock(VirtIOBlk<'static>);

impl VirtIOBlock {
    pub fn new() -> Self {
        Self(VirtIOBlk::new(unsafe { &mut *(MMIO[0].0 as *mut VirtIOHeader) }).unwrap())
    }
}

impl Deref for VirtIOBlock {
    type Target = VirtIOBlk<'static>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl BlockDevice for VirtIOBlock {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        self.read_block(block_id, buf);
    }
    fn write_block(&self, block_id: usize, buf: &[u8]) {
        self.write_block(block_id, buf);
    }
}

use crate::memory::frame::address::*;
use crate::memory::frame::frame::*;
use alloc::vec::Vec;
use lazy_static::*;
use spin::Mutex;

lazy_static! {
    static ref QUEUE_FRAMES: Mutex<Vec<FrameTracker>> = Mutex::new(Vec::new());
}

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
