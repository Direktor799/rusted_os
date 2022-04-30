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
