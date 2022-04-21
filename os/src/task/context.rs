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
    s: [usize; 12],
}

impl TaskContext {
    /// init task context
    pub const fn zero_init() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }

    /// set task context {__restore ASM funciton, kernel stack, s_0..12 }
    pub fn goto_restore(kstack_ptr: usize) -> Self {
        extern "C" {
            fn __restore();
        }
        Self {
            ra: __restore as usize,
            sp: kstack_ptr,
            s: [0; 12],
        }
    }

    pub fn goto_trap_return(kernel_sp: usize) -> Self {
        Self {
            ra: interrupt_return as usize,
            s: [0; 12],
            sp: kernel_sp,
        }
    }
}
