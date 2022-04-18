mod context;
mod schd;
mod switch;
mod task;

use crate::interrupt::Context;
use crate::loader::APP_MANAGER;
use crate::timer;
pub use context::TaskContext;
use core::cell::RefCell;
use schd::{get_default_time_slice, get_time_slice, SchdMaster};
pub use switch::__switch;
pub use task::{TaskControlBlock, TaskPos, TaskStatus};

pub struct TaskManager(RefCell<TaskManagerInner>);

struct TaskManagerInner {
    current_task: Option<TaskControlBlock>,
    schd: SchdMaster,
}

impl TaskManager {
    fn init(&mut self) {
        unsafe {
            let app_num = APP_MANAGER.borrow_mut().get_app_num();
            for i in 0..app_num {
                let tcb = TaskControlBlock::new(
                    APP_MANAGER.borrow_mut().init_app_context(i) as *mut Context as usize
                );
                if i == 0 {
                    self.0.borrow_mut().current_task = Some(tcb);
                } else {
                    self.0.borrow_mut().schd.add_new_task(tcb);
                }
            }
        }
    }

    fn switch_to_next_task(&self) {
        let mut inner = self.0.borrow_mut();
        let current_task = inner.current_task;
        if let None = current_task {
            timer::set_next_timeout(get_default_time_slice())
        }
        let mut current_task = current_task.unwrap();
        let next_task = inner.schd.get_next_and_requeue_current(current_task);
        inner.current_task = Option::from(next_task);
        if let None = next_task {
            panic!("all tasks have completed!");
        }
        let mut next_task = next_task.unwrap();
        drop(inner);
        timer::set_next_timeout(get_time_slice(next_task.task_pos));
        unsafe {
            println!(
                "switching to 0x{:x}",
                (*(next_task.task_cx.sp as *const Context)).sepc
            );
            __switch(
                &mut current_task.task_cx as *mut TaskContext,
                &mut next_task.task_cx as *mut TaskContext,
            );
        }
        unreachable!();
    }
    fn set_current_task_status(&self, stat: TaskStatus) {
        let mut inner = self.0.borrow_mut();
        if let Some(current_task) = inner.current_task.as_mut() {
            (*current_task).task_status = stat;
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
    schedule_callback();
    // TODO set exit code in the task context
}

/// the callback function used in the supervisor time interrupt
/// to implement the basic task scheduling
pub fn schedule_callback() {
    unsafe { TASK_MANAGER.switch_to_next_task(); }
}

pub fn init() {
    unsafe {
        TASK_MANAGER.init();
        let task_manager = TASK_MANAGER.0.borrow_mut();
        let current_task = task_manager.current_task.as_ref().unwrap().clone();
        drop(task_manager);
        let mut _unused = TaskContext::zero_init();
        println!(
            "first time switching to 0x{:x}",
            (*(current_task.task_cx.sp as *const Context)).sepc
        );
        __switch(&mut _unused as *mut TaskContext, &current_task.task_cx);
        unreachable!();
    }
}
