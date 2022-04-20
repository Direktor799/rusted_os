use super::address::{VPNRange, VirtPageNum};
use super::frame::{FrameTracker, FRAME_ALLOCATOR};
use crate::config::PAGE_SIZE;
use alloc::collections::BTreeMap;
use core::cmp::min;

enum MapType {
    Identical,
    Framed,
}

pub type SegFlags = u8;

pub struct MemorySegment {
    vpn_range: VPNRange,
    pub data_frames: BTreeMap<VirtPageNum, FrameTracker>,
    map_type: MapType,
    flags: SegFlags,
}

impl MemorySegment {
    pub fn new(vpn_range: VPNRange, flags: SegFlags) -> Self {
        let mut data_frames = BTreeMap::new();
        for vpn in vpn_range {
            let frame = unsafe { FRAME_ALLOCATOR.borrow_mut().alloc().unwrap() };
            data_frames.insert(vpn, frame);
        }
        Self {
            vpn_range,
            data_frames,
            map_type: MapType::Framed,
            flags,
        }
    }

    pub fn copy_data(&self, vpn_range: VPNRange, data: &[u8]) {
        let mut current_start = 0;
        for vpn in vpn_range {
            let src = &data[current_start..min(data.len(), current_start + PAGE_SIZE)];
            self.data_frames[&vpn].ppn().get_bytes_array()[..src.len()].copy_from_slice(src);
            current_start += PAGE_SIZE;
            if current_start >= data.len() {
                break;
            }
        }
    }
}
