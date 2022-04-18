use super::task::*;
use super::TaskContext;
use crate::config::{
    TASK_QUEUE_FCFS1_SLICE, TASK_QUEUE_FCFS2_SLICE, TASK_QUEUE_RR_SLICE, TASK_QUEUE_SIZE,
};

struct WaitingQueue<T> {
    queue: [T; TASK_QUEUE_SIZE],
    head: usize,
    tail: usize,
}

impl<T: Copy> WaitingQueue<T> {
    pub fn push(&mut self, item: T) -> bool {
        if self.size() == TASK_QUEUE_SIZE - 1 {
            return false;
        }
        let tail = self.tail;
        self.tail = (self.tail + 1 + TASK_QUEUE_SIZE) % TASK_QUEUE_SIZE;
        self.queue[tail] = item;
        true
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
        (self.tail + TASK_QUEUE_SIZE - self.head) % TASK_QUEUE_SIZE
    }
}

struct MultilevelFeedbackQueue {
    fcfs1_queue: WaitingQueue<TaskControlBlock>,
    fcfs2_queue: WaitingQueue<TaskControlBlock>,
    rr_queue: WaitingQueue<TaskControlBlock>,
}

impl MultilevelFeedbackQueue {
    pub const fn new() -> Self {
        let tcb = TaskControlBlock {
            task_status: TaskStatus::UnInit,
            task_cx: TaskContext::zero_init(),
            task_pos: TaskPos::Fcfs1,
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
        }
    }
    pub fn requeue(&mut self, mut task: TaskControlBlock) -> bool {
        match task.task_pos {
            TaskPos::Fcfs1 => {
                task.task_pos = TaskPos::Fcfs2;
                self.fcfs2_queue.push(task);
                true
            }
            TaskPos::Fcfs2 => {
                task.task_pos = TaskPos::Rr;
                self.rr_queue.push(task);
                true
            }
            TaskPos::Rr => {
                task.task_pos = TaskPos::Rr;
                self.rr_queue.push(task);
                true
            }
        }
    }
    pub fn enqueue(&mut self, task: TaskControlBlock) -> bool {
        self.fcfs1_queue.push(task)
    }
    pub fn get_task(&mut self) -> Option<TaskControlBlock> {
        let task = self.fcfs1_queue.pop();
        if let Some(ref task) = task {
            return Option::from(*task);
        }
        let task = self.fcfs2_queue.pop();
        if let Some(ref task) = task {
            return Option::from(*task);
        }
        let task = self.rr_queue.pop();
        if let Some(ref task) = task {
            return Option::from(*task);
        }
        None
    }
}

pub struct SchdMaster {
    mlfq: MultilevelFeedbackQueue,
}

impl SchdMaster {
    pub const fn new() -> Self {
        SchdMaster {
            mlfq: MultilevelFeedbackQueue::new(),
        }
    }
    /// push current task control block into MLFQ and return the next task to be executed
    /// 
    /// next task can be None
    pub fn get_next_and_requeue_current(
        &mut self,
        mut current_task_cb: TaskControlBlock,
    ) -> Option<TaskControlBlock> {
        let mut next_task_info = self.mlfq.get_task(); // get a new task
        if let Some(ref mut task_info) = next_task_info {
            let next_task_cb = *task_info;
            if current_task_cb.task_status != TaskStatus::Exited {
                current_task_cb.task_status = TaskStatus::Ready;
                self.mlfq.requeue(current_task_cb);
            }
            return Option::from(next_task_cb);
        }
        if current_task_cb.task_status == TaskStatus::Exited {
            None
        } else {
            Option::from(current_task_cb)
        }
    }

    pub fn add_new_task(&mut self, tcb: TaskControlBlock) {
        self.mlfq.enqueue(tcb);
    }
}

#[inline(always)]
pub fn get_time_slice(pos: TaskPos) -> usize {
    match pos {
        TaskPos::Fcfs1 => TASK_QUEUE_FCFS1_SLICE,
        TaskPos::Fcfs2 => TASK_QUEUE_FCFS2_SLICE,
        TaskPos::Rr => TASK_QUEUE_RR_SLICE,
    }
}

#[inline(always)]
pub fn get_default_time_slice() -> usize {
    TASK_QUEUE_FCFS1_SLICE
}
