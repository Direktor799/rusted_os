use crate::config::{KERNEL_STACK_SIZE, PAGE_SIZE, TRAMPOLINE};
use crate::memory::frame::{
    address::{VPNRange, VirtAddr},
    memory_set::KERNEL_MEMORY_SET,
    page_table::{R, W},
};
use crate::sync::uninit_cell::UninitCell;
use alloc::vec::Vec;

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
    unsafe { PidHandle(PID_ALLOCATOR.alloc()) }
}

impl Drop for PidHandle {
    fn drop(&mut self) {
        unsafe {
            PID_ALLOCATOR.dealloc(self.0);
        }
    }
}

pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}
/// Kernelstack for app
pub struct KernelStack {
    pid: usize,
}

impl KernelStack {
    /// Create a kernelstack from pid
    pub fn new(pid_handle: &PidHandle) -> Self {
        let pid = pid_handle.0;
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(pid);
        unsafe {
            KERNEL_MEMORY_SET.insert_segment(
                VPNRange::new(
                    VirtAddr(kernel_stack_bottom).vpn(),
                    VirtAddr(kernel_stack_top).vpn(),
                ),
                R | W,
                None,
            );
        }
        KernelStack { pid: pid_handle.0 }
    }

    /// Push a value on top of kernelstack
    pub fn push_on_top<T>(&self, value: T) -> *mut T
    where
        T: Sized,
    {
        let kernel_stack_top = self.get_top();
        let ptr_mut = (kernel_stack_top - core::mem::size_of::<T>()) as *mut T;
        unsafe {
            *ptr_mut = value;
        }
        ptr_mut
    }

    /// Get the value on the top of kernelstack
    pub fn get_top(&self) -> usize {
        let (_, kernel_stack_top) = kernel_stack_position(self.pid);
        kernel_stack_top
    }
}

impl Drop for KernelStack {
    fn drop(&mut self) {
        let (kernel_stack_bottom, _) = kernel_stack_position(self.pid);
        let kernel_stack_bottom_vpn = VirtAddr(kernel_stack_bottom).vpn();
        unsafe {
            KERNEL_MEMORY_SET.remove_segment(kernel_stack_bottom_vpn);
        }
    }
}

pub fn init() {
    unsafe {
        PID_ALLOCATOR = UninitCell::init(RecycleAllocator::new());
    }
}
