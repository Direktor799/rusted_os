use super::page_table::PageTableEntry;
use crate::config::{PAGE_SIZE, PAGE_SIZE_BITS};
use core::fmt::{self, Debug, Formatter};
use core::marker::Copy;

#[derive(Copy, Clone)]
pub struct PhysAddr(pub usize);

#[derive(Copy, Clone)]
pub struct VirtAddr(pub usize);

#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub struct PhysPageNum(pub usize);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtPageNum(pub usize);

impl Debug for PhysAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PhysAddr: {:#x}", self.0))
    }
}

impl Debug for VirtAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VirtAddr: {:#x}", self.0))
    }
}

impl Debug for PhysPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PhysPageNum: {:#x}", self.0))
    }
}

impl Debug for VirtPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VirtPageNum: {:#x}", self.0))
    }
}

impl PhysAddr {
    pub fn ppn(&self) -> PhysPageNum {
        PhysPageNum(self.0 / PAGE_SIZE)
    }
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }
}

impl VirtAddr {
    pub fn vpn(&self) -> VirtPageNum {
        VirtPageNum(self.0 / PAGE_SIZE)
    }
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }
}

impl PhysPageNum {
    pub fn addr(&self) -> PhysAddr {
        PhysAddr(self.0 << PAGE_SIZE_BITS)
    }

    pub fn get_pte_array(&self) -> &'static mut [PageTableEntry] {
        let pa = self.addr();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut PageTableEntry, PAGE_SIZE / 8) }
    }

    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let pa = self.addr();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut u8, PAGE_SIZE / 8) }
    }
}

impl VirtPageNum {
    pub fn indices(&self) -> [usize; 3] {
        let mut vpn = self.0;
        let mut indices = [0; 3];
        for i in (0..3).rev() {
            indices[i] = vpn & (PAGE_SIZE / 8 - 1);
            vpn >>= PAGE_SIZE_BITS - 3;
        }
        indices
    }
}

#[derive(Clone, Copy)]
pub struct VPNRange {
    start_vpn: VirtPageNum,
    end_vpn: VirtPageNum,
    curr_vpn: VirtPageNum,
}

impl Iterator for VPNRange {
    type Item = VirtPageNum;
    fn next(&mut self) -> Option<Self::Item> {
        let mut res = Option::None;
        if self.curr_vpn < self.end_vpn {
            res = Some(self.curr_vpn);
            self.curr_vpn.0 += 1;
        }
        res
    }
}

impl VPNRange {
    pub fn new(start_vpn: VirtPageNum, end_vpn: VirtPageNum) -> Self {
        Self {
            start_vpn,
            end_vpn,
            curr_vpn: start_vpn,
        }
    }
}
