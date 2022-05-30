//! 页表子模块

use super::address::*;
use super::frame_allocator::{frame_alloc, FrameTracker};
use alloc::vec;
use alloc::vec::Vec;
use core::fmt::Debug;

/// SV39页表项有效标志位
const V: u8 = 1 << 0;
/// SV39页表项可读标志位
pub const R: u8 = 1 << 1;
/// SV39页表项可写标志位
pub const W: u8 = 1 << 2;
/// SV39页表项可执行标志位
pub const X: u8 = 1 << 3;
/// SV39页表项用户标志位
pub const U: u8 = 1 << 4;
// /// SV39页表项全局标志位
// const G: u8 = 1 << 5;
// /// SV39页表项已使用标志位
// const A: u8 = 1 << 6;
// /// SV39页表项已修改标志位
// const D: u8 = 1 << 7;

/// SV39页表项标志位段
pub type PTEFlags = u8;

/// SV39页表项
#[derive(Copy, Clone)]
pub struct PageTableEntry(usize);

/// SV39页表
pub struct PageTable {
    root_ppn: PhysPageNum,
    frames: Vec<FrameTracker>,
}

impl PageTableEntry {
    /// 创建新页表项
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        Self((ppn.0 << 10) | flags as usize)
    }

    /// 创建空页表项
    pub fn empty() -> Self {
        Self(0)
    }

    /// 获取物理页号
    pub fn ppn(&self) -> PhysPageNum {
        PhysPageNum(self.0 >> 10)
    }

    /// 获取标志位段
    pub fn flags(&self) -> PTEFlags {
        self.0 as u8
    }

    /// 页表项是否有效
    pub fn valid(&self) -> bool {
        self.flags() & V != 0
    }
}

impl PageTable {
    /// 创建空页表
    pub fn new() -> Self {
        let frame = frame_alloc().unwrap();
        Self {
            root_ppn: frame.ppn(),
            frames: vec![frame],
        }
    }

    /// 根据token获取指定页表（不会获取页表页的所有权）
    pub fn from_token(token: usize) -> Self {
        Self {
            root_ppn: PhysPageNum(token & (1 << 44) - 1),
            frames: vec![],
        }
    }

    /// 添加指定虚拟页到物理页的映射
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.create_pte(vpn).unwrap();
        *pte = PageTableEntry::new(ppn, flags | V);
    }

    /// 删除指定虚拟页到物理页的映射
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self.find_pte(vpn).unwrap();
        *pte = PageTableEntry::empty();
    }

    /// 在页表中找到指定虚拟页的页表项
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

    /// 在页表中创建指定虚拟页的页表项
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
                let frame = frame_alloc().unwrap();
                *pte = PageTableEntry::new(frame.ppn(), V);
                self.frames.push(frame);
            }
            ppn = pte.ppn();
        }
        None
    }

    /// 查找虚拟页号对应的物理页号
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PhysPageNum> {
        Some(self.find_pte(vpn)?.ppn())
    }

    /// 获取页表token
    pub fn satp_token(&self) -> usize {
        8usize << 60 | self.root_ppn.0
    }
}

impl Debug for PageTable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut stack = vec![];
        let mut content = vec![];
        stack.push((0, self.root_ppn));
        while !stack.is_empty() {
            let (base, ppn) = stack.pop().unwrap();
            for (i, pte) in ppn.get_pte_array().iter().enumerate() {
                let vpn = (base << 9) + i;
                if pte.flags() == 1 {
                    stack.push((vpn, pte.ppn()));
                } else if pte.flags() != 0 {
                    if vpn != pte.ppn().0 {
                        content.push((VirtPageNum(vpn), pte.ppn(), pte.flags()));
                    }
                }
            }
        }
        f.debug_struct("PageTable")
            .field("root_ppn", &self.root_ppn)
            .field("frames", &self.frames)
            .field("non-identical", &content)
            .finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::memory::frame::frame_allocator::frame_alloc;
    test!(test_page_table, {
        let mut page_table = PageTable::new();
        let frame = frame_alloc().unwrap();
        page_table.map(VirtPageNum(0), frame.ppn(), R);
        let ppn = page_table.translate(VirtPageNum(0));
        test_assert!(ppn.is_some() && ppn.unwrap() == frame.ppn());
        page_table.unmap(VirtPageNum(0));
        let ppn = page_table.translate(VirtPageNum(0));
        test_assert!(ppn.is_none());
        Ok("passed")
    });
}
