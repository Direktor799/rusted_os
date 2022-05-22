mod context;
pub mod schd;
mod switch;
mod task;

use crate::interrupt::context::Context;
use crate::interrupt::timer;
use crate::loader::app_manager::APP_MANAGER;
use crate::sync::uninit_cell::UninitCell;
pub use context::TaskContext;
use schd::{get_time_slice, SchdMaster};
pub use switch::__switch;
pub use task::{ProcessControlBlock, TaskPos, TaskStatus};

pub struct TaskManager {
    pub current_task: Option<ProcessControlBlock>,
    schd: SchdMaster,
}

impl TaskManager {
    fn init(&mut self) {
        unsafe {
            let app_num = APP_MANAGER.get_app_num();
            for i in 0..app_num {
                let tcb = ProcessControlBlock::new(APP_MANAGER.get_app_data(i), i);
                if i == 0 {
                    self.current_task = Some(tcb);
                } else {
                    self.schd.add_new_task(tcb);
                }
            }
        }
    }

    fn switch_to_next_task(&mut self) {
        let mut current_task = self.current_task.take().expect("[kernel] No current task");
        let current_task_cx = &mut current_task.task_cx as *mut TaskContext;
        let mut next_task = self
            .schd
            .get_next_and_requeue_current(current_task)
            .expect("[kernel] All tasks have completed!");
        let next_task_cx = &mut next_task.task_cx as *mut TaskContext;
        timer::set_next_timeout(get_time_slice(next_task.task_pos));
        self.current_task = Some(next_task);
        drop(self);
        unsafe {
            // println!(
            //     "switching to 0x{:x}",
            //     (*((*next_task_cx).sp as *const Context)).sepc
            // );
            __switch(current_task_cx, next_task_cx);
        }
        unreachable!();
    }

    fn set_current_task_status(&mut self, stat: TaskStatus) {
        if let Some(current_task) = self.current_task.as_mut() {
            (*current_task).task_status = stat;
        }
    }

    pub fn get_current_token(&self) -> usize {
        let current = self.current_task.as_ref().unwrap();
        current.get_user_token()
    }
    pub fn get_current_trap_cx(&self) -> &mut Context {
        let current = self.current_task.as_ref().unwrap();
        current.get_trap_cx()
    }
    // pub fn current_fd_table(&self) -> &mut Vec<Option<Rc<dyn File>>> {
    //     let inner = self.0.as_ref().unwrap().borrow();
    //     let current = inner.current_task.as_ref().unwrap();
    //     current.get_fd_table()
    // }
    // pub fn get_current_task(&mut self) -> &'static mut ProcessControlBlock {

    //     let mut inner = self.0.as_ref().unwrap().borrow_mut();
    //     inner.current_task.as_mut().unwrap()
    //     // self.0.as_ref().unwrap().borrow_mut().current_task.as_mut().unwrap()
    // }
}

pub static mut TASK_MANAGER: UninitCell<TaskManager> = UninitCell::uninit();

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
        TASK_MANAGER = UninitCell::init(TaskManager {
            current_task: None,
            schd: SchdMaster::new(),
        });
        TASK_MANAGER.init();
        println!("mod task initialized!");
    }
}

pub fn run() {
    unsafe {
        let current_task = TASK_MANAGER.current_task.as_mut().unwrap();
        let current_task_cx = &mut current_task.task_cx as *mut TaskContext;
        let mut _unused = TaskContext::zero_init();
        // println!(
        //     "first time switching to 0x{:x}",
        //     (*((*current_task_cx).sp as *const Context)).sepc
        // );
        __switch(&mut _unused as *mut TaskContext, current_task_cx);
        unreachable!();
    }
}
