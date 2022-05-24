//! 用户地址空间的封装
use super::address::*;
use super::page_table::PageTable;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

/// 用户空间的地址在内核空间中的映射
pub struct UserBuffer(pub Vec<&'static mut [u8]>);

impl UserBuffer {
    /// 获取Buffer长度
    pub fn len(&self) -> usize {
        self.0.iter().map(|segment| segment.len()).sum()
    }
}

impl IntoIterator for UserBuffer {
    type Item = &'static mut u8;
    type IntoIter = Iter;
    fn into_iter(self) -> Self::IntoIter {
        Iter {
            segments: self.0,
            current_segment: 0,
            current_idx: 0,
        }
    }
}

pub struct Iter {
    segments: Vec<&'static mut [u8]>,
    current_segment: usize,
    current_idx: usize,
}

impl Iterator for Iter {
    type Item = &'static mut u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_segment < self.segments.len() {
            let byte = &mut self.segments[self.current_segment][self.current_idx] as *mut _;
            self.current_idx += 1;
            if self.current_idx == self.segments[self.current_segment].len() {
                self.current_idx = 0;
                self.current_segment += 1;
            }
            Some(unsafe { &mut *byte })
        } else {
            None
        }
    }
}

/// 通过指定token获取用户数据在内核中的映射
pub fn get_user_buffer(user_token: usize, ptr: *const u8, len: usize) -> UserBuffer {
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
pub fn get_user_string(user_token: usize, ptr: *const u8) -> String {
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
