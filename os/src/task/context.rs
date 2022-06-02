//! Implementation of [`TaskContext`]

use crate::interrupt::handler::interrupt_return;

/// Task Context
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct TaskContext {
    /// return address ( e.g. __restore ) of __switch ASM function
    ra: usize,
    /// kernel stack pointer of app
    pub sp: usize,
    /// callee saved registers:  s 0..11
    x: [usize; 12],
}

impl TaskContext {
    /// 全0初始化
    pub const fn zero_init() -> Self {
        Self {
            ra: 0,
            sp: 0,
            x: [0; 12],
        }
    }

    pub fn goto_trap_return(kernel_sp: usize) -> Self {
        Self {
            ra: interrupt_return as usize,
            x: [0; 12],
            sp: kernel_sp,
        }
    }
}
