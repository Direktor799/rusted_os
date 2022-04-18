use super::context::TaskContext;

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

#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    pub task_pos: TaskPos,
}

impl TaskControlBlock {
    pub fn new(kernel_sp: usize) -> Self {
        Self {
            task_status: TaskStatus::Ready,
            task_cx: TaskContext::goto_restore(kernel_sp),
            task_pos: TaskPos::Fcfs1,
        }
    }
}
