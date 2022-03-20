use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::NonNull;

pub struct Dummy {
    start_addr: usize,
    end_addr: usize,
}

impl Dummy {
    pub const fn new() -> Self {
        Self {
            start_addr: 0,
            end_addr: 0,
        }
    }

    pub fn init(&mut self, start_addr: usize, size: usize) {
        self.start_addr = start_addr;
        self.end_addr = start_addr + size;
    }
}

unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        println!("{:?} {:?}", self.start_addr, self.end_addr);
        NonNull::new(self.start_addr as *mut u8).unwrap().as_ptr()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("Dealloc? Never!");
    }
}
