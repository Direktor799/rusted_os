use super::context::TaskContext;
use super::id::{pid_alloc, KernelStack, PidHandle};
use crate::config::TRAP_CONTEXT;
use crate::fs::stdio::{Stdin, Stdout};
use crate::fs::File;
use crate::interrupt::{context::Context, handler::interrupt_handler};
use crate::memory::frame::address::*;
use crate::memory::frame::user_buffer::put_user_value;
use crate::memory::frame::{memory_set::MemorySet, memory_set::KERNEL_MEMORY_SET};
use alloc::rc::{Rc, Weak};
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::mem::size_of;

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    // Running,
    Exited,
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskPos {
    Fcfs1,
    Fcfs2,
    Rr,
}

pub struct ProcessControlBlock {
    pub pid: PidHandle,
    pub kernel_stack: KernelStack,
    pub inner: RefCell<ProcessControlBlockInner>,
}

pub struct ProcessControlBlockInner {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    pub task_pos: TaskPos,
    pub memory_set: MemorySet,
    pub trap_cx_ppn: PhysPageNum,
    pub cwd: String,
    pub fd_table: Vec<Option<Rc<dyn File>>>,
    pub parent: Weak<ProcessControlBlock>,
    pub children: Vec<Rc<ProcessControlBlock>>,
    pub exit_code: i32,
}

impl ProcessControlBlock {
    pub fn new(elf_data: &[u8]) -> Self {
        let pid = pid_alloc();
        let kernel_stack = KernelStack::new(&pid);
        let kernel_stack_top = kernel_stack.get_top();
        let (memory_set, user_sp, entry) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr(TRAP_CONTEXT).vpn())
            .expect("[kernel] Trap context not mapped!");
        let trap_cx = trap_cx_ppn.addr().get_mut();
        *trap_cx = Context::app_init_context(
            entry,
            user_sp,
            unsafe { KERNEL_MEMORY_SET.satp_token() },
            kernel_stack_top,
            interrupt_handler as usize,
        );
        Self {
            pid,
            kernel_stack,
            inner: RefCell::new(ProcessControlBlockInner {
                task_status: TaskStatus::Ready,
                task_cx: TaskContext::goto_trap_return(kernel_stack_top),
                task_pos: TaskPos::Fcfs1,
                memory_set,
                trap_cx_ppn,
                cwd: String::from("/"),
                fd_table: vec![
                    // 0 -> stdin
                    Some(Rc::new(Stdin)),
                    // 1 -> stdout
                    Some(Rc::new(Stdout)),
                    // 2 -> stderr
                    Some(Rc::new(Stdout)),
                ],
                parent: Weak::new(),
                children: vec![],
                exit_code: 0,
            }),
        }
    }

    pub fn exec(&self, elf_data: &[u8], args: &[String]) {
        let (memory_set, mut user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr(TRAP_CONTEXT).vpn())
            .expect("[kernel] Trap context not mapped!");

        // push arguments on user stack
        user_sp -= (args.len() + 1) * size_of::<usize>();
        let argv_base = user_sp;
        let mut argv = vec![];
        for arg in args {
            user_sp -= (arg.len() + 1) * size_of::<usize>();
            argv.push(user_sp);
            let mut p = user_sp;
            for &c in arg.as_bytes() {
                put_user_value(memory_set.satp_token(), c, p as *mut u8);
                p += 1;
            }
            put_user_value(memory_set.satp_token(), 0, p as *mut u8);
        }
        argv.push(0);
        for (i, &arg_ptr) in argv.iter().enumerate() {
            put_user_value(
                memory_set.satp_token(),
                arg_ptr,
                (argv_base + i * size_of::<usize>()) as *mut u8,
            );
        }

        let mut inner = self.inner.borrow_mut();
        inner.memory_set = memory_set;
        inner.trap_cx_ppn = trap_cx_ppn;

        let mut trap_cx = Context::app_init_context(
            entry_point,
            user_sp,
            unsafe { KERNEL_MEMORY_SET.satp_token() },
            self.kernel_stack.get_top(),
            interrupt_handler as usize,
        );
        trap_cx.x[10] = args.len();
        trap_cx.x[11] = argv_base;
        *trap_cx_ppn.addr().get_mut() = trap_cx;
    }

    pub fn fork(self: Rc<Self>) -> Rc<Self> {
        let mut inner = self.inner.borrow_mut();
        let memory_set = inner.memory_set.clone();
        let pid_handle = pid_alloc();
        let kernel_stack = KernelStack::new(&pid_handle);
        let kernel_stack_top = kernel_stack.get_top();
        let trap_cx_ppn = memory_set
            .translate(VirtAddr(TRAP_CONTEXT).vpn())
            .expect("[kernel] Trap context not mapped!");
        let new_pcb = Rc::new(ProcessControlBlock {
            pid: pid_handle,
            kernel_stack,
            inner: RefCell::new(ProcessControlBlockInner {
                task_status: inner.task_status,
                task_cx: TaskContext::goto_trap_return(kernel_stack_top),
                task_pos: inner.task_pos,
                memory_set,
                trap_cx_ppn,
                cwd: inner.cwd.clone(),
                fd_table: inner.fd_table.clone(),
                parent: Rc::downgrade(&self),
                children: vec![],
                exit_code: 0,
            }),
        });
        inner.children.push(new_pcb.clone());
        let trap_cx = new_pcb.inner.borrow_mut().trap_cx();
        trap_cx.kernel_sp = kernel_stack_top;
        new_pcb
    }
}

impl ProcessControlBlockInner {
    pub fn token(&self) -> usize {
        self.memory_set.satp_token()
    }

    pub fn trap_cx(&self) -> &'static mut Context {
        self.trap_cx_ppn.addr().get_mut()
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
