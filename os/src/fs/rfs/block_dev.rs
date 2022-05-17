//! 块设备接口定义

use core::any::Any;

/// 为任何块设备实现这个trait，使其可以被fs使用
pub trait BlockDevice: Send + Sync + Any {
    fn read_block(&self, block_id: usize, buf: &mut [u8]);
    fn write_block(&self, block_id: usize, buf: &[u8]);
}
