use super::task::*;
use super::TaskContext;
use crate::config::{TASK_QUEUE_FCFS1_SLICE, TASK_QUEUE_FCFS2_SLICE, TASK_QUEUE_RR_SLICE};
use alloc::collections::VecDeque;

struct MultilevelFeedbackQueue {
    fcfs1_queue: VecDeque<TaskControlBlock>,
    fcfs2_queue: VecDeque<TaskControlBlock>,
    rr_queue: VecDeque<TaskControlBlock>,
}

impl MultilevelFeedbackQueue {
    pub fn new() -> Self {
        let tcb = TaskControlBlock {
            task_status: TaskStatus::UnInit,
            task_cx: TaskContext::zero_init(),
            task_pos: TaskPos::Fcfs1,
        };
        MultilevelFeedbackQueue {
            fcfs1_queue: VecDeque::new(),
            fcfs2_queue: VecDeque::new(),
            rr_queue: VecDeque::new(),
        }
    }
    pub fn requeue(&mut self, mut task: TaskControlBlock) -> bool {
        match task.task_pos {
            TaskPos::Fcfs1 => {
                task.task_pos = TaskPos::Fcfs2;
                self.fcfs2_queue.push_back(task);
                true
            }
            TaskPos::Fcfs2 => {
                task.task_pos = TaskPos::Rr;
                self.rr_queue.push_back(task);
                true
            }
            TaskPos::Rr => {
                task.task_pos = TaskPos::Rr;
                self.rr_queue.push_back(task);
                true
            }
        }
    }
    pub fn enqueue(&mut self, task: TaskControlBlock) {
        self.fcfs1_queue.push_back(task)
    }
    pub fn get_task(&mut self) -> Option<TaskControlBlock> {
        let task = self.fcfs1_queue.pop_front();
        if let Some(ref task) = task {
            return Option::from(*task);
        }
        let task = self.fcfs2_queue.pop_front();
        if let Some(ref task) = task {
            return Option::from(*task);
        }
        let task = self.rr_queue.pop_front();
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
    pub fn new() -> Self {
        SchdMaster {
            mlfq: MultilevelFeedbackQueue::new(),
        }
    }
    /// push_back current task control block into MLFQ and return the next task to be executed
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
