use super::context::TaskContext;
use crate::os_fs::{Stdin,Stdout};
use crate::config::{KERNEL_STACK_SIZE, PAGE_SIZE, TRAMPOLINE, TRAP_CONTEXT};
use crate::interrupt::{context::Context, handler::interrupt_handler};
use crate::memory::frame::address::*;
use crate::memory::frame::{
    memory_set::MemorySet,
    memory_set::KERNEL_MEMORY_SET,
    page_table::{R, W},
};
use alloc::vec;
use alloc::vec::Vec;
use alloc::sync::Arc;
use crate::os_fs::{File};
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
    pub fd_table: Vec<Option<Arc<dyn File>>>,
}

impl TaskControlBlock {
    pub fn new(elf_data: &[u8], app_id: usize) -> Self {
        let (memory_set, user_sp, entry) = MemorySet::from_elf(elf_data);
        // guard page
        let kernel_stack_end = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
        let kernel_stack_start = kernel_stack_end - KERNEL_STACK_SIZE;
        unsafe {
            KERNEL_MEMORY_SET.insert_segment(
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
            unsafe { KERNEL_MEMORY_SET.satp_token() },
            kernel_stack_end - 1,
            interrupt_handler as usize,
        );
        Self {
            task_status: TaskStatus::Ready,
            task_cx: TaskContext::goto_trap_return(kernel_stack_end - 1),
            task_pos: TaskPos::Fcfs1,
            memory_set,
            trap_cx_ppn,
            fd_table: vec![
                        // 0 -> stdin
                        Some(Arc::new(Stdin)),
                        // 1 -> stdout
                        Some(Arc::new(Stdout)),
                        // 2 -> stderr
                        Some(Arc::new(Stdout)),
                    ],
        }
    }

    pub fn get_user_token(&self) -> usize {
        self.memory_set.satp_token()
    }

    pub fn get_trap_cx(&self) -> &'static mut Context {
        self.trap_cx_ppn.get_mut()
    }
    pub fn get_fd_table(&mut self) -> &mut Vec<Option<Arc<dyn File>>> {
        self.fd_table.as_mut()
    }

    pub fn alloc_fd(&mut self) -> usize {
        if let Some(fd) = (0..self.fd_table.len()).find(|fd| self.fd_table[*fd].is_none()) {
            fd
        } else {
            self.fd_table.push(None);
            self.fd_table.len() - 1
        }
    }
}
