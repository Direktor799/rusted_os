mod context;
pub mod schd;
mod switch;
mod task;

use crate::interrupt::context::Context;
use crate::interrupt::timer;
use crate::loader::app_manager::APP_MANAGER;
pub use context::TaskContext;
use core::cell::RefCell;
use schd::{get_time_slice, SchdMaster};
pub use switch::__switch;
pub use task::{TaskControlBlock, TaskPos, TaskStatus};

pub struct TaskManager(Option<RefCell<TaskManagerInner>>);

struct TaskManagerInner {
    current_task: Option<TaskControlBlock>,
    schd: SchdMaster,
}

impl TaskManager {
    fn init(&mut self) {
        unsafe {
            let app_num = APP_MANAGER.get_app_num();
            for i in 0..app_num {
                let tcb = TaskControlBlock::new(APP_MANAGER.get_app_data(i), i);
                if i == 0 {
                    self.0.as_ref().unwrap().borrow_mut().current_task = Some(tcb);
                } else {
                    self.0.as_ref().unwrap().borrow_mut().schd.add_new_task(tcb);
                }
            }
        }
    }

    fn switch_to_next_task(&self) {
        let mut inner = self.0.as_ref().unwrap().borrow_mut();
        let mut current_task = inner.current_task.take().expect("[kernel] No current task");
        let current_task_cx = &mut current_task.task_cx as *mut TaskContext;
        let mut next_task = inner
            .schd
            .get_next_and_requeue_current(current_task)
            .expect("[kernel] All tasks have completed!");
        let next_task_cx = &mut next_task.task_cx as *mut TaskContext;
        timer::set_next_timeout(get_time_slice(next_task.task_pos));
        inner.current_task = Some(next_task);
        drop(inner);
        unsafe {
            // println!(
            //     "switching to 0x{:x}",
            //     (*((*next_task_cx).sp as *const Context)).sepc
            // );
            __switch(current_task_cx, next_task_cx);
        }
        unreachable!();
    }

    fn set_current_task_status(&self, stat: TaskStatus) {
        let mut inner = self.0.as_ref().unwrap().borrow_mut();
        if let Some(current_task) = inner.current_task.as_mut() {
            (*current_task).task_status = stat;
        }
    }

    pub fn get_current_token(&self) -> usize {
        let inner = self.0.as_ref().unwrap().borrow();
        let current = inner.current_task.as_ref().unwrap();
        current.get_user_token()
    }
    pub fn get_current_trap_cx(&self) -> &mut Context {
        let inner = self.0.as_ref().unwrap().borrow();
        let current = inner.current_task.as_ref().unwrap();
        current.get_trap_cx()
    }
}

pub static mut TASK_MANAGER: TaskManager = TaskManager(None);

fn set_current_task_status(stat: TaskStatus) {
    unsafe {
        TASK_MANAGER.set_current_task_status(stat);
    }
}

pub fn exit_current_and_run_next() {
    set_current_task_status(TaskStatus::Exited);
    unsafe {
        TASK_MANAGER.switch_to_next_task();
    }
}

pub fn suspend_current_and_run_next() {
    unsafe {
        TASK_MANAGER.switch_to_next_task();
    }
}

/// the callback function used in the supervisor time interrupt
/// to implement the basic task scheduling
pub fn schedule_callback() {
    unsafe {
        TASK_MANAGER.switch_to_next_task();
    }
}

pub fn init() {
    unsafe {
        TASK_MANAGER = TaskManager(Some(RefCell::new(TaskManagerInner {
            current_task: None,
            schd: SchdMaster::new(),
        })));
        TASK_MANAGER.init();
        println!("mod task initialized!");
    }
}

pub fn run() {
    unsafe {
        let mut task_manager = TASK_MANAGER.0.as_mut().unwrap().borrow_mut();
        let current_task = task_manager.current_task.as_mut().unwrap();
        let current_task_cx = &mut current_task.task_cx as *mut TaskContext;
        drop(task_manager);
        let mut _unused = TaskContext::zero_init();
        // println!(
        //     "first time switching to 0x{:x}",
        //     (*((*current_task_cx).sp as *const Context)).sepc
        // );
        __switch(&mut _unused as *mut TaskContext, current_task_cx);
        unreachable!();
    }
}
