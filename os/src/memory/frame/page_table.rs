//! 页表子模块

use super::address::*;
use super::frame_allocator::{FrameTracker, FRAME_ALLOCATOR};
use super::user_buffer::UserBuffer;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

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
/// SV39页表项全局标志位
const G: u8 = 1 << 5;
/// SV39页表项已使用标志位
const A: u8 = 1 << 6;
/// SV39页表项已修改标志位
const D: u8 = 1 << 7;

/// SV39页表项标志位段
pub type PTEFlags = u8;

/// SV39页表项
#[derive(Copy, Clone)]
pub struct PageTableEntry(usize);

/// SV39页表
#[derive(Debug)]
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
        unsafe {
            let frame = FRAME_ALLOCATOR.alloc().unwrap();
            Self {
                root_ppn: frame.ppn(),
                frames: vec![frame],
            }
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
                unsafe {
                    let frame = FRAME_ALLOCATOR.alloc().unwrap();
                    *pte = PageTableEntry::new(frame.ppn(), V);
                    self.frames.push(frame);
                }
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

/// 通过指定token获取用户数据在内核中的映射
pub fn get_user_buffer_in_kernel(user_token: usize, ptr: *const u8, len: usize) -> UserBuffer {
    let mut data_segments = vec![];
    let user_page_table = PageTable::from_token(user_token);
    let mut current_start = ptr as usize;
    let end = current_start + len;
    while current_start < end {
        let start_va = VirtAddr(current_start);
        let ppn = user_page_table
            .translate(start_va.vpn())
            .expect("[kernel] User space address not mapped!");
        let end_va = core::cmp::min(VirtAddr(end), VirtPageNum(start_va.vpn().0 + 1).addr());
        data_segments
            .push(&mut ppn.get_bytes_array()[start_va.page_offset()..end_va.page_offset()]);
        current_start = end_va.0;
    }
    UserBuffer(data_segments)
}

/// 通过指定token获取用户字符串
pub fn get_user_string_in_kernel(user_token: usize, ptr: *const u8) -> String {
    let user_page_table = PageTable::from_token(user_token);
    let mut string = String::new();
    let mut va = VirtAddr(ptr as usize);
    loop {
        let ppn = user_page_table
            .translate(va.vpn())
            .expect("[kernel] User space address not mapped!");
        let ch = *(PhysAddr(ppn.addr().0 + va.page_offset()).get_mut::<u8>());
        if ch == 0 {
            break;
        }
        string.push(ch as char);
        va = VirtAddr(va.0 + 1);
    }
    string
}
