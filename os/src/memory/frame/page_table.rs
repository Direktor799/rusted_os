//! 页表

use super::address::{PhysPageNum, VirtPageNum};
use super::frame::{FrameTracker, FRAME_ALLOCATOR};
use alloc::vec;
use alloc::vec::Vec;

const V: u8 = 1 << 0;
pub const R: u8 = 1 << 1;
pub const W: u8 = 1 << 2;
pub const X: u8 = 1 << 3;
pub const U: u8 = 1 << 4;
const G: u8 = 1 << 5;
const A: u8 = 1 << 6;
const D: u8 = 1 << 7;

pub type PTEFlags = u8;

#[derive(Copy, Clone)]
pub struct PageTableEntry(usize);

pub struct PageTable {
    root_ppn: PhysPageNum,
    frames: Vec<FrameTracker>,
}

impl PageTableEntry {
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        Self((ppn.0 << 10) | flags as usize)
    }

    pub fn empty() -> Self {
        Self(0)
    }

    pub fn ppn(&self) -> PhysPageNum {
        PhysPageNum(self.0 >> 10)
    }

    pub fn flags(&self) -> PTEFlags {
        self.0 as u8
    }

    pub fn valid(&self) -> bool {
        self.flags() & V != 0
    }
}

impl PageTable {
    pub fn new() -> Self {
        unsafe {
            let frame = FRAME_ALLOCATOR.borrow_mut().alloc().unwrap();
            Self {
                root_ppn: frame.ppn(),
                frames: vec![frame],
            }
        }
    }

    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.create_pte(vpn).unwrap();
        *pte = PageTableEntry::new(ppn, flags | V);
    }

    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self.find_pte(vpn).unwrap();
        *pte = PageTableEntry::empty();
    }

    fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let indices = vpn.indices();
        let mut ppn = self.root_ppn;
        for i in 0..3 {
            let pte_array = ppn.get_pte_array();
            let pte = &mut pte_array[indices[i]];
            if !pte.valid() {
                return None;
            }
            if i == 2 {
                return Some(pte);
            }
            ppn = pte.ppn();
        }
        None
    }

    fn create_pte(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let indices = vpn.indices();
        let mut ppn = self.root_ppn;
        for i in 0..3 {
            let pte_array = ppn.get_pte_array();
            let pte = &mut pte_array[indices[i]];
            if i == 2 {
                return Some(pte);
            }
            if !pte.valid() {
                unsafe {
                    let frame = FRAME_ALLOCATOR.borrow_mut().alloc().unwrap();
                    *pte = PageTableEntry::new(frame.ppn(), V);
                    self.frames.push(frame);
                }
            }
            ppn = pte.ppn();
        }
        None
    }

    pub fn translate(&self, vpn: VirtPageNum) -> Option<PhysPageNum> {
        Some(self.find_pte(vpn)?.ppn())
    }

    pub fn satp_token(&self) -> usize {
        8usize << 60 | self.root_ppn.0
    }
}
