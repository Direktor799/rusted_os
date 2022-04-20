use super::context::TaskContext;
use crate::config::{KERNEL_STACK_SIZE, PAGE_SIZE, TRAMPOLINE, TRAP_CONTEXT};
use crate::interrupt::{interrupt_handler, Context};
use crate::memory::address::*;
use crate::memory::{MemorySet, KERNEL_MEMORY_SET, R, W};
use alloc::collections::VecDeque;

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskPos {
    Fcfs1,
    Fcfs2,
    Rr,
}

pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    pub task_pos: TaskPos,
    pub memory_set: MemorySet,
    pub trap_cx_ppn: PhysPageNum,
}

impl TaskControlBlock {
    pub fn new(elf_data: &[u8], app_id: usize) -> Self {
        let (memory_set, user_sp, entry) = MemorySet::from_elf(elf_data);
        // guard page
        let kernel_stack_end = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
        let kernel_stack_start = kernel_stack_end - KERNEL_STACK_SIZE;
        unsafe {
            KERNEL_MEMORY_SET.as_mut().unwrap().insert_segment(
                VirtAddr(kernel_stack_start).vpn(),
                VirtAddr(kernel_stack_end).vpn(),
                R | W,
                None,
            );
        }
        let trap_cx_ppn = memory_set
            .translate(VirtAddr(TRAP_CONTEXT).vpn())
            .expect("[kernel] Trap context not mapped!");
        let trap_cx = trap_cx_ppn.get_mut();
        *trap_cx = Context::app_init_context(
            entry,
            user_sp,
            unsafe { KERNEL_MEMORY_SET.as_ref().unwrap().satp_token() },
            kernel_stack_end - 1,
            interrupt_handler as usize,
        );
        Self {
            task_status: TaskStatus::Ready,
            task_cx: TaskContext::goto_trap_return(kernel_stack_end - 1),
            task_pos: TaskPos::Fcfs1,
            memory_set,
            trap_cx_ppn,
        }
    }

    pub fn get_user_token(&self) -> usize {
        self.memory_set.satp_token()
    }

    pub fn get_trap_cx(&self) -> &'static mut Context {
        self.trap_cx_ppn.get_mut()
    }
}
