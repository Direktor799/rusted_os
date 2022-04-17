use super::address::{VPNRange, VirtPageNum};
use super::frame::{FrameTracker, FRAME_ALLOCATOR};
use super::page_table::{R, U, W, X};
use alloc::collections::BTreeMap;

enum MapType {
    Identical,
    Framed,
}

pub type SegFlags = u8;

pub struct MemorySegment {
    vpn_range: VPNRange,
    data_frames: BTreeMap<VirtPageNum, FrameTracker>,
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

    pub fn map_iter(&self) -> alloc::collections::btree_map::Iter<VirtPageNum, FrameTracker> {
        self.data_frames.iter()
    }
}
