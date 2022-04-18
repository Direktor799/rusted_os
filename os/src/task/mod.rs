mod context;
mod schd;
mod switch;
mod task;

use crate::sync::UPSafeCell;
pub use context::TaskContext;
use core::cell::RefCell;
use schd::{get_default_time_slice, get_time_slice, SchdMaster};
pub use switch::__switch;
pub use task::{TaskControlBlock, TaskStatus};

pub struct TaskManager(RefCell<TaskManagerInner>);

struct TaskManagerInner {
    current_task: Option<TaskControlBlock>,
    schd: SchdMaster,
}

impl TaskManager {
    fn run_next_and_return_slice(&self) -> usize {
        let mut inner = self.0.borrow_mut();
        let mut current_task = inner.current_task;
        if let Some(ref mut current_task) = current_task {
            let mut next_task = inner.schd.get_next_and_requeue_current(*current_task);
            inner.current_task = Option::from(next_task);
            unsafe {
                __switch(
                    &mut current_task.task_cx as *mut TaskContext,
                    &mut next_task.task_cx as *mut TaskContext,
                );
            }
            get_time_slice(next_task.task_pos)
        } else {
            get_default_time_slice()
        }
    }
    fn set_current_task_status(&self, stat: TaskStatus) {
        let inner = self.0.borrow_mut();
        let mut current_task = inner.current_task;
        if let Some(ref mut current_task) = current_task {
            current_task.task_status = stat;
        }
    }
}

pub static mut TASK_MANAGER: TaskManager = TaskManager(RefCell::new(TaskManagerInner {
    current_task: None,
    schd: SchdMaster::new(),
}));

pub fn set_current_task_status(stat: TaskStatus) {
    unsafe {
        TASK_MANAGER.set_current_task_status(stat);
    }
}

pub fn exit_current_and_run_next(exit_code: i32) {
    set_current_task_status(TaskStatus::Exited);
    // TODO set exit code in the task context
}

/// the callback function used in the supervisor time interrupt
/// to implement the basic task scheduling
pub fn schedule_callback() -> usize {
    unsafe { TASK_MANAGER.run_next_and_return_slice() }
}