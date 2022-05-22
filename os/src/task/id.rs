use crate::sync::uninit_cell::UninitCell;
use alloc::{
    vec::Vec
};

pub struct RecycleAllocator {
    current: usize,
    recycled: Vec<usize>,
}

impl RecycleAllocator {
    pub fn new() -> Self {
        RecycleAllocator {
            current: 0,
            recycled: Vec::new(),
        }
    }
    pub fn alloc(&mut self) -> usize {
        if let Some(id) = self.recycled.pop() {
            id
        } else {
            self.current += 1;
            self.current - 1
        }
    }
    pub fn dealloc(&mut self, id: usize) {
        assert!(id < self.current);
        assert!(
            !self.recycled.iter().any(|i| *i == id),
            "id {} has been deallocated!",
            id
        );
        self.recycled.push(id);
    }
}

pub static mut PID_ALLOCATOR: UninitCell<RecycleAllocator> = UninitCell::uninit();

pub struct PidHandle(pub usize);

pub fn pid_alloc() -> PidHandle {
    unsafe {
        PidHandle(PID_ALLOCATOR.alloc())
    }
}

impl Drop for PidHandle {
    fn drop(&mut self) {
        unsafe {
            PID_ALLOCATOR.dealloc(self.0);
        }
    }
}

pub fn init() {
    UninitCell::init(RecycleAllocator::new());
}