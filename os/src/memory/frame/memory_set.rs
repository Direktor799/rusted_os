use super::address::{VPNRange, VirtPageNum};
use super::page_table::PageTable;
use super::segment::{MemorySegment, SegFlags};
use alloc::{vec, vec::Vec};

struct MemorySet {
    page_table: PageTable,
    segments: Vec<MemorySegment>,
}

impl MemorySet {
    pub fn new() -> Self {
        Self {
            page_table: PageTable::new(),
            segments: vec![],
        }
    }

    pub fn insert_segment(
        &mut self,
        start_vpn: VirtPageNum,
        end_vpn: VirtPageNum,
        seg_flags: SegFlags,
    ) {
        let vpn_range = VPNRange::new(start_vpn, end_vpn);
        let segment = MemorySegment::new(vpn_range, seg_flags);
        for (&vpn, frame) in segment.map_iter() {
            self.page_table.map(vpn, frame.ppn(), seg_flags);
        }
        self.segments.push(segment);
    }
}
