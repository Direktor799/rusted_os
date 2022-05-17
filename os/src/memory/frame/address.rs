//! 地址，页号定义相关子模块

use super::page_table::PageTableEntry;
use crate::config::{PAGE_SIZE, PAGE_SIZE_BITS};
use core::fmt::{self, Debug, Formatter};
use core::marker::Copy;

/// 物理地址
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysAddr(pub usize);

/// 虚拟地址
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddr(pub usize);

/// 物理页号
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysPageNum(pub usize);

/// 虚拟页号
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
    /// 获取对应的物理页号
    pub fn ppn(&self) -> PhysPageNum {
        PhysPageNum(self.0 / PAGE_SIZE)
    }

    /// 获取该物理地址上的指定类型数据
    pub fn get_mut<T>(&self) -> &'static mut T {
        unsafe { (self.0 as *mut T).as_mut().unwrap() }
    }
}

impl VirtAddr {
    /// 获取对应的虚拟页号
    pub fn vpn(&self) -> VirtPageNum {
        VirtPageNum(self.0 / PAGE_SIZE)
    }

    /// 获取页内偏移
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
}

impl PhysPageNum {
    /// 获取物理页面对应地址
    pub fn addr(&self) -> PhysAddr {
        PhysAddr(self.0 << PAGE_SIZE_BITS)
    }

    /// 获取作为页表的物理页面内所有页表项
    pub fn get_pte_array(&self) -> &'static mut [PageTableEntry] {
        let pa = self.addr();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut PageTableEntry, PAGE_SIZE / 8) }
    }

    /// 获取物理页面内所有字节数据
    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let pa = self.addr();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut u8, PAGE_SIZE) }
    }

    /// 获取物理页面起始处指定类型数据
    pub fn get_mut<T>(&self) -> &'static mut T {
        self.addr().get_mut()
    }
}

impl VirtPageNum {
    /// 获取虚拟页面对应地址
    pub fn addr(&self) -> VirtAddr {
        VirtAddr(self.0 << PAGE_SIZE_BITS)
    }

    /// 获取虚拟页面对应三级页表索引
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

/// 虚拟页号区间迭代器
#[derive(Clone, Copy, Debug)]
pub struct VPNRange {
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
    /// 创建新的虚拟页号区间
    pub fn new(start_vpn: VirtPageNum, end_vpn: VirtPageNum) -> Self {
        Self {
            end_vpn,
            curr_vpn: start_vpn,
        }
    }
}

test!(test_phys_page_num, {
    let ppn = PhysPageNum(0x81000);
    test_assert!(ppn.addr().0 == 0x81000_000, "Addr doesn't match");
    let bytes = ppn.get_bytes_array();
    for byte in bytes {
        *byte = u8::MAX;
    }
    let first_usize = ppn.get_mut::<usize>();
    test_assert!(*first_usize == usize::MAX, "Read / write failed");
    Ok("passed")
});

test!(test_virt_page_num, {
    let vpn = VirtPageNum(0b111111111_101010101_000000000);
    test_assert!(
        vpn.addr().0 == 0b111111111_101010101_000000000_000000000000,
        "Addr doesn't match"
    );
    let indices = vpn.indices();
    test_assert!(
        indices[0] == 0b111111111 && indices[1] == 0b101010101 && indices[2] == 0b000000000,
        "Convertion from VPN to indices failed"
    );
    Ok("passed")
});

test!(test_vpn_range, {
    let start_vpn = VirtPageNum(0);
    let end_vpn = VirtPageNum(100);
    for (i, vpn) in VPNRange::new(start_vpn, end_vpn).enumerate() {
        test_assert!(vpn.0 == i, "Iteration failed")
    }
    Ok("passed")
});
