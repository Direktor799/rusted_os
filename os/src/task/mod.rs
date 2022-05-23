mod context;
mod id;
pub mod schd;
mod switch;
mod task;

use crate::interrupt::context::Context;
use crate::interrupt::timer;
use crate::loader::app_manager::APP_MANAGER;
use crate::sync::uninit_cell::UninitCell;
use alloc::rc::Rc;
pub use context::TaskContext;
use schd::{get_time_slice, SchdMaster};
pub use switch::__switch;
pub use task::{ProcessControlBlock, TaskPos, TaskStatus};

pub struct TaskManager {
    pub current_task: Option<Rc<ProcessControlBlock>>,
    schd: SchdMaster,
}

impl TaskManager {
    fn init(&mut self) {
        unsafe {
            let app_num = APP_MANAGER.get_app_num();
            for i in 0..app_num {
                let tcb = Rc::new(ProcessControlBlock::new(APP_MANAGER.get_app_data(i)));
                if i == 0 {
                    self.current_task = Some(tcb);
                } else {
                    self.schd.add_new_task(tcb);
                }
            }
        }
    }

    fn switch_to_next_task(&mut self) {
        let current_task = self.current_task.take().expect("[kernel] No task");
        let mut current_task_inner = current_task.inner.borrow_mut();
        let current_task_cx = &mut current_task_inner.task_cx as *mut TaskContext;
        drop(current_task_inner);
        let next_task = self
            .schd
            .get_next_and_requeue_current(current_task)
            .expect("[kernel] All tasks have completed!");
        let mut next_task_inner = next_task.inner.borrow_mut();
        let next_task_cx = &mut next_task_inner.task_cx as *mut TaskContext;
        timer::set_next_timeout(get_time_slice(next_task_inner.task_pos));
        drop(next_task_inner);
        self.current_task = Some(next_task);
        unsafe {
            // println!("{:x?}", *current_task_cx);
            // println!("{:x?}", *next_task_cx);
            __switch(current_task_cx, next_task_cx);
        }
    }

    fn set_current_task_status(&mut self, stat: TaskStatus) {
        if let Some(current_task) = self.current_task.as_mut() {
            let mut inner = current_task.inner.borrow_mut();
            inner.task_status = stat;
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
    pub fn get_current_process(&self) -> Option<Rc<ProcessControlBlock>> {
        self.current_task.clone()
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

pub fn add_new_task(task: Rc<ProcessControlBlock>) {
    unsafe {
        TASK_MANAGER.schd.add_new_task(task);
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

pub fn get_current_process() -> Option<Rc<ProcessControlBlock>> {
    unsafe { TASK_MANAGER.get_current_process() }
}

pub fn init() {
    unsafe {
        id::init();
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
        let mut current_task_inner = current_task.inner.borrow_mut();
        let current_task_cx = &mut current_task_inner.task_cx as *mut TaskContext;
        drop(current_task_inner);
        let mut _unused = TaskContext::zero_init();
        __switch(&mut _unused as *mut TaskContext, current_task_cx);
        unreachable!();
    }
}
