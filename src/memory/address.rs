use crate::config::{PAGE_SIZE, PAGE_SIZE_BITS};
use core::fmt::{self, Debug, Formatter};
use core::marker::Copy;

#[derive(Copy, Clone)]
pub struct PhysAddr(pub usize);

#[derive(Copy, Clone)]
pub struct VirtAddr(pub usize);

#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub struct PhysPageNum(pub usize);

#[derive(Copy, Clone)]
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

impl VirtAddr {
    pub fn page_num(&self) -> VirtPageNum {
        VirtPageNum(self.0 / PAGE_SIZE)
    }
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }
}

impl PhysAddr {
    pub fn page_num(&self) -> PhysPageNum {
        PhysPageNum(self.0 / PAGE_SIZE)
    }
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }
}

impl PhysPageNum {
    pub fn address(&self) -> PhysAddr {
        PhysAddr(self.0 << PAGE_SIZE_BITS)
    }
}
