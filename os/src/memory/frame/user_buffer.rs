//! 用户地址空间的封装
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
