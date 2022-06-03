use crate::config::{KERNEL_STACK_SIZE, PAGE_SIZE, TRAMPOLINE};
use crate::memory::frame::{
    address::VirtAddr,
    memory_set::KERNEL_MEMORY_SET,
    page_table::{R, W},
};
use crate::tools::uninit_cell::UninitCell;
use alloc::vec::Vec;

/// 循环分配器
pub struct RecycleAllocator {
    current: usize,
    recycled: Vec<usize>,
}

impl RecycleAllocator {
    /// 创建新循环分配器
    pub fn new() -> Self {
        RecycleAllocator {
            current: 0,
            recycled: Vec::new(),
        }
    }
    /// 分配
    pub fn alloc(&mut self) -> usize {
        if let Some(id) = self.recycled.pop() {
            id
        } else {
            self.current += 1;
            self.current - 1
        }
    }
    /// 释放
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

/// 全局PID分配器
pub static mut PID_ALLOCATOR: UninitCell<RecycleAllocator> = UninitCell::uninit();

/// PID句柄
pub struct PidHandle(pub usize);

/// 分配PID句柄
pub fn pid_alloc() -> PidHandle {
    unsafe { PidHandle(PID_ALLOCATOR.alloc()) }
}

/// PID句柄Drop实现
impl Drop for PidHandle {
    fn drop(&mut self) {
        unsafe {
            PID_ALLOCATOR.dealloc(self.0);
        }
    }
}

/// 获取目前内核栈位置
pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}
/// app 内核栈
pub struct KernelStack {
    pid: usize,
}

impl KernelStack {
    /// 由 PID 分配栈空间
    pub fn new(pid_handle: &PidHandle) -> Self {
        let pid = pid_handle.0;
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(pid);
        unsafe {
            KERNEL_MEMORY_SET.insert_segment(
                VirtAddr(kernel_stack_bottom).vpn()..VirtAddr(kernel_stack_top).vpn(),
                R | W,
                None,
            );
        }
        KernelStack { pid: pid_handle.0 }
    }

    /// 获取栈顶
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

/// 资源分配模块初始化
pub fn init() {
    unsafe {
        PID_ALLOCATOR = UninitCell::init(RecycleAllocator::new());
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test!(test_pid_allocator, {
        let pid1 = pid_alloc();
        let pid2 = pid_alloc();
        let pid3 = pid_alloc();
        let pid4 = pid_alloc();

        let mut i = 0;
        while i <= 1000000 {
            let pid = pid_alloc();

            assert!(pid.0 != pid1.0);
            assert!(pid.0 != pid2.0);
            assert!(pid.0 != pid3.0);
            assert!(pid.0 != pid4.0);

            drop(pid);
            i += 1;
        }

        Ok("passed")
    });
}
