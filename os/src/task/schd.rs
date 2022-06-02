use super::task::*;
use crate::config::{TASK_QUEUE_FCFS1_SLICE_MS, TASK_QUEUE_FCFS2_SLICE_MS, TASK_QUEUE_RR_SLICE_MS};
use alloc::collections::VecDeque;
use alloc::rc::Rc;

/// 多级反馈队列
struct MultilevelFeedbackQueue {
    fcfs1_queue: VecDeque<Rc<ProcessControlBlock>>,
    fcfs2_queue: VecDeque<Rc<ProcessControlBlock>>,
    rr_queue: VecDeque<Rc<ProcessControlBlock>>,
}

impl MultilevelFeedbackQueue {
    /// 创建多级反馈队列
    pub fn new() -> Self {
        MultilevelFeedbackQueue {
            fcfs1_queue: VecDeque::new(),
            fcfs2_queue: VecDeque::new(),
            rr_queue: VecDeque::new(),
        }
    }
    /// 旧任务再次入队
    pub fn requeue(&mut self, task: Rc<ProcessControlBlock>) -> bool {
        let mut inner = task.inner.borrow_mut();
        match inner.task_pos {
            TaskPos::Fcfs1 => {
                inner.task_pos = TaskPos::Fcfs2;
                drop(inner);
                self.fcfs2_queue.push_back(task);
                true
            }
            TaskPos::Fcfs2 => {
                inner.task_pos = TaskPos::Rr;
                drop(inner);
                self.rr_queue.push_back(task);
                true
            }
            TaskPos::Rr => {
                inner.task_pos = TaskPos::Rr;
                drop(inner);
                self.rr_queue.push_back(task);
                true
            }
        }
    }
    /// 新任务入队
    pub fn enqueue(&mut self, task: Rc<ProcessControlBlock>) {
        self.fcfs1_queue.push_back(task)
    }
    /// 按照调度算法取出下一个任务
    pub fn get_task(&mut self) -> Option<Rc<ProcessControlBlock>> {
        let task = self.fcfs1_queue.pop_front();
        if task.is_some() {
            return task;
        }
        let task = self.fcfs2_queue.pop_front();
        if task.is_some() {
            return task;
        }
        self.rr_queue.pop_front()
    }

    /// 返回迭代器
    pub fn iter(&self) -> impl Iterator<Item = &Rc<ProcessControlBlock>> {
        self.fcfs1_queue
            .iter()
            .chain(self.fcfs2_queue.iter())
            .chain(self.rr_queue.iter())
    }
}

/// 调度器
pub struct SchdMaster {
    mlfq: MultilevelFeedbackQueue,
}

impl SchdMaster {
    /// 创建新的调度器实例
    pub fn new() -> Self {
        SchdMaster {
            mlfq: MultilevelFeedbackQueue::new(),
        }
    }

    /// 当前任务再次入队
    pub fn requeue_current(&mut self, current_task_cb: Rc<ProcessControlBlock>) {
        self.mlfq.requeue(current_task_cb);
    }

    /// 按调度算法取出下一个任务
    pub fn get_next(&mut self) -> Option<Rc<ProcessControlBlock>> {
        self.mlfq.get_task()
    }

    /// 新任务入队
    pub fn add_new_task(&mut self, tcb: Rc<ProcessControlBlock>) {
        self.mlfq.enqueue(tcb);
    }

    /// 返回迭代器
    pub fn tasks(&self) -> impl Iterator<Item = &Rc<ProcessControlBlock>> {
        self.mlfq.iter()
    }
}

/// 获取任务类型对应的时间片
#[inline(always)]
pub fn get_time_slice(pos: TaskPos) -> usize {
    match pos {
        TaskPos::Fcfs1 => TASK_QUEUE_FCFS1_SLICE_MS,
        TaskPos::Fcfs2 => TASK_QUEUE_FCFS2_SLICE_MS,
        TaskPos::Rr => TASK_QUEUE_RR_SLICE_MS,
    }
}

/// 获取默认时间片
#[inline(always)]
pub fn get_default_time_slice() -> usize {
    TASK_QUEUE_FCFS1_SLICE_MS
}
