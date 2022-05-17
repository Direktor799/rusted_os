//! VirtIO块设备驱动
use crate::config::MMIO;
use crate::fs::rfs::block_dev::BlockDevice;
use core::cell::RefCell;
use core::marker::Sync;
use virtio_drivers::{VirtIOBlk, VirtIOHeader};

pub struct VirtIOBlock(RefCell<VirtIOBlk<'static>>);

impl VirtIOBlock {
    pub fn new() -> Self {
        Self(RefCell::new(
            VirtIOBlk::new(unsafe { &mut *(MMIO[0].0 as *mut VirtIOHeader) }).unwrap(),
        ))
    }
}

unsafe impl Sync for VirtIOBlock {}

impl BlockDevice for VirtIOBlock {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        self.0
            .borrow_mut()
            .read_block(block_id, buf)
            .expect("read error");
    }
    fn write_block(&self, block_id: usize, buf: &[u8]) {
        self.0
            .borrow_mut()
            .write_block(block_id, buf)
            .expect("write error");
    }
}
