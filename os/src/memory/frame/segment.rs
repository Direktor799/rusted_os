use super::address::{VPNRange, VirtPageNum};
use super::frame_allocator::{FrameTracker, FRAME_ALLOCATOR};
use crate::config::PAGE_SIZE;
use alloc::collections::BTreeMap;
use core::cmp::min;

pub type SegFlags = u8;

pub struct MemorySegment {
    vpn_range: VPNRange,
    pub data_frames: BTreeMap<VirtPageNum, FrameTracker>,
    flags: SegFlags,
}

impl MemorySegment {
    pub fn new(vpn_range: VPNRange, flags: SegFlags) -> Self {
        let mut data_frames = BTreeMap::new();
        for vpn in vpn_range {
            let frame = unsafe { FRAME_ALLOCATOR.alloc().unwrap() };
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
        for vpn in self.vpn_range {
            let src = &data[current_start..min(data.len(), current_start + PAGE_SIZE)];
            self.data_frames[&vpn].ppn().get_bytes_array()[..src.len()].copy_from_slice(src);
            current_start += PAGE_SIZE;
            if current_start >= data.len() {
                break;
            }
        }
    }
}
