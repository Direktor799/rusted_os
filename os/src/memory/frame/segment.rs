use super::address::VirtPageNum;
use super::frame_allocator::{frame_alloc, FrameTracker};
use crate::config::PAGE_SIZE;
use alloc::collections::BTreeMap;
use core::cmp::min;
use core::ops::Range;

pub type SegFlags = u8;

#[derive(Debug)]
pub struct MemorySegment {
    pub vpn_range: Range<VirtPageNum>,
    pub data_frames: BTreeMap<VirtPageNum, FrameTracker>,
    pub flags: SegFlags,
}

impl MemorySegment {
    pub fn new(vpn_range: Range<VirtPageNum>, flags: SegFlags) -> Self {
        let mut data_frames = BTreeMap::new();
        for vpn in vpn_range.clone() {
            let frame = frame_alloc().unwrap();
            data_frames.insert(vpn, frame);
        }
        Self {
            vpn_range,
            data_frames,
            flags,
        }
    }

    pub fn copy_data(&self, data: &[u8]) {
        let mut current_start = 0;
        for vpn in self.vpn_range.clone() {
            let src = &data[current_start..min(data.len(), current_start + PAGE_SIZE)];
            self.data_frames[&vpn].ppn().get_bytes_array()[..src.len()].copy_from_slice(src);
            current_start += PAGE_SIZE;
            if current_start >= data.len() {
                break;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::memory::frame::page_table::*;
    test!(test_memory_segment, {
        let seg = MemorySegment::new(VirtPageNum(0)..VirtPageNum(1), R | W);
        let mut data = [0u8; PAGE_SIZE];
        for byte in data.iter_mut().step_by(2) {
            *byte = u8::MAX;
        }
        seg.copy_data(&data);
        let mut should_be = u8::MAX;
        for byte in seg.data_frames[&VirtPageNum(0)].ppn().get_bytes_array() {
            test_assert!(*byte == should_be);
            should_be = !should_be;
        }
        Ok("passed")
    });
}
