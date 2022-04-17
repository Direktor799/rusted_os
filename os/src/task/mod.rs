mod context;
mod switch;
mod task;

use crate::config::{
    TASK_QUEUE_FCFS1_SLICE, TASK_QUEUE_FCFS2_SLICE, TASK_QUEUE_RR_SLICE, TASK_QUEUE_SIZE,
};
pub use context::TaskContext;
pub use switch::__switch;
pub use task::{TaskControlBlock, TaskStatus};

struct WaitingQueue<T> {
    queue: [T; TASK_QUEUE_SIZE],
    head: usize,
    tail: usize,
}

impl<T: Copy> WaitingQueue<T> {
    pub fn push(&mut self, item: T) -> usize {
        if self.size() == TASK_QUEUE_SIZE - 1 {
            return TASK_QUEUE_SIZE;
        }
        let tail = self.tail;
        self.tail = (self.tail + 1 + TASK_QUEUE_SIZE) % TASK_QUEUE_SIZE;
        self.queue[tail] = item;
        tail
    }
    pub fn pop(&mut self) -> Option<T> {
        if self.size() == 0 {
            return None;
        }
        let head = self.head;
        self.head = (self.head + 1 + TASK_QUEUE_SIZE) % TASK_QUEUE_SIZE;
        Option::from(self.queue[head])
    }
    pub fn size(&mut self) -> usize {
        (self.tail - self.head + TASK_QUEUE_SIZE) % TASK_QUEUE_SIZE
    }
    pub fn rewrite(&mut self, item: T, index: usize) {
        self.queue[index] = item;
    }
}

#[derive(Copy, Clone, PartialEq)]
enum TaskPos {
    Fcfs1,
    Fcfs2,
    Rr,
}

struct MultilevelFeedbackQueue {
    fcfs1_queue: WaitingQueue<TaskControlBlock>,
    fcfs2_queue: WaitingQueue<TaskControlBlock>,
    rr_queue: WaitingQueue<TaskControlBlock>,
    task_pos: TaskPos,
    task_now_index: usize,
    task_now_info: TaskControlBlock,
}

impl MultilevelFeedbackQueue {
    pub const fn new() -> Self {
        let tcb = TaskControlBlock {
            task_status: TaskStatus::UnInit,
            task_cx: TaskContext::zero_init(),
        };
        MultilevelFeedbackQueue {
            fcfs1_queue: WaitingQueue {
                queue: [tcb; TASK_QUEUE_SIZE],
                head: 0,
                tail: 0,
            },
            fcfs2_queue: WaitingQueue {
                queue: [tcb; TASK_QUEUE_SIZE],
                head: 0,
                tail: 0,
            },
            rr_queue: WaitingQueue {
                queue: [tcb; TASK_QUEUE_SIZE],
                head: 0,
                tail: 0,
            },
            task_pos: TaskPos::Fcfs1,
            task_now_index: 0,
            task_now_info: tcb,
        }
    }
    pub fn enqueue(&mut self, task: TaskControlBlock) -> bool {
        if self.fcfs1_queue.push(task) == TASK_QUEUE_SIZE {
            false
        } else {
            true
        }
    }
    pub fn dequeue(&mut self) {
        let dead_tcb = TaskControlBlock {
            task_status: TaskStatus::Exited,
            task_cx: TaskContext::zero_init(),
        };
        match self.task_pos {
            TaskPos::Fcfs1 => {
                self.fcfs1_queue.rewrite(dead_tcb, self.task_now_index);
            }
            TaskPos::Fcfs2 => {
                self.fcfs2_queue.rewrite(dead_tcb, self.task_now_index);
            }
            TaskPos::Rr => {
                self.rr_queue.rewrite(dead_tcb, self.task_now_index);
            }
        }
    }
    pub fn get_task(&mut self) -> Option<TaskControlBlock> {
        loop {
            let task = self.fcfs1_queue.pop();
            if let Some(ref task) = task {
                if task.task_status == TaskStatus::Exited {
                    continue;
                }
                self.task_pos = TaskPos::Fcfs2;
                self.task_now_index = self.fcfs2_queue.push(*task);
                return Option::from(*task);
            }
            let task = self.fcfs2_queue.pop();
            if let Some(ref task) = task {
                if task.task_status == TaskStatus::Exited {
                    continue;
                }
                self.task_pos = TaskPos::Fcfs1;
                self.task_now_index = self.rr_queue.push(*task);
                return Option::from(*task);
            }
            let task = self.rr_queue.pop();
            if let Some(ref task) = task {
                if task.task_status == TaskStatus::Exited {
                    continue;
                }
                self.task_pos = TaskPos::Rr;
                self.task_now_index = self.rr_queue.push(*task);
                return Option::from(*task);
            }
            break;
        }
        None
    }
}

static mut MLFQ: MultilevelFeedbackQueue = MultilevelFeedbackQueue::new();

pub fn set_active_task_status(stat: TaskStatus) {
    unsafe {
        MLFQ.task_now_info.task_status = stat;
    }
}

pub fn exit_current_and_run_next(exit_code: i32) {
    set_active_task_status(TaskStatus::Exited);
    // TODO set exit code in the task context
    do_scheduling();
}

pub fn do_scheduling() {
    let task_info;
    let mut prev_task_cb;
    unsafe {
        prev_task_cb = MLFQ.task_now_info;
        if prev_task_cb.task_status == TaskStatus::Exited {
            // if the task has completed, you should remove it from the MLFQ
            // CAUTION this method won't be working under multicore circumstance
            MLFQ.dequeue();
        }
        task_info = MLFQ.get_task(); // get a new task
    }
    if let Some(ref task_info) = task_info {
        let mut task_cb = *task_info;
        prev_task_cb.task_status = TaskStatus::Ready;
        task_cb.task_status = TaskStatus::Running;
        unsafe {
            // context switching here
            // FIXME a potential risk at the first time
            __switch(
                &mut prev_task_cb.task_cx as *mut TaskContext,
                &mut task_cb.task_cx as *mut TaskContext,
            );
        }
    }
}

#[inline(always)]
fn get_time_slice(pos: TaskPos) -> usize {
    match pos {
        TaskPos::Fcfs1 => {
            TASK_QUEUE_FCFS1_SLICE
        }
        TaskPos::Fcfs2 => {
            TASK_QUEUE_FCFS2_SLICE
        }
        TaskPos::Rr => {
            TASK_QUEUE_RR_SLICE
        }
    }
}

/// the callback function used in the supervisor time interrupt
/// to implement the basic task scheduling
pub fn schedule_callback() -> usize {
    let task_pos;
    unsafe {
        task_pos = MLFQ.task_pos;
    }
    do_scheduling();
    get_time_slice(task_pos)
}
