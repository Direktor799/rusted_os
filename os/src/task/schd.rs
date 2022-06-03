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

#[cfg(test)]
mod test {
    use alloc::vec;
    use crate::fs::rfs::find_inode;
    use super::*;

    test!(test_mlfq, {
        let app_inode = find_inode("/bin/daemon").expect("[kernel] daemon not found!");
        let size = app_inode.get_file_size() as usize;
        let mut app_data = vec![0u8; size];
        app_inode.read_at(0, &mut app_data);

        let pcb = Rc::new(ProcessControlBlock::new(&app_data));

        let mut mlfq = MultilevelFeedbackQueue::new();
        mlfq.enqueue(pcb);
        let pcb = mlfq.get_task();
        assert!(pcb.is_some());
        let pcb = pcb.unwrap();
        assert!(pcb.as_ref().inner.borrow().task_pos == TaskPos::Fcfs1);
        assert!(mlfq.get_task().is_none());

        mlfq.requeue(pcb);
        let pcb = mlfq.get_task();
        assert!(pcb.is_some());
        let pcb = pcb.unwrap();
        assert!(pcb.as_ref().inner.borrow().task_pos == TaskPos::Fcfs2);
        assert!(mlfq.get_task().is_none());

        let pcb1 = Rc::new(ProcessControlBlock::new(&app_data));
        let pcb2 = Rc::new(ProcessControlBlock::new(&app_data));
        let pcb3 = Rc::new(ProcessControlBlock::new(&app_data));
        let pcb4 = Rc::new(ProcessControlBlock::new(&app_data));

        let pid1 = pcb1.as_ref().pid.0;
        let pid2 = pcb2.as_ref().pid.0;
        let pid3 = pcb3.as_ref().pid.0;
        let pid4 = pcb4.as_ref().pid.0;

        mlfq.enqueue(pcb1);
        mlfq.enqueue(pcb2);
        mlfq.enqueue(pcb3);

        let pcb1 = mlfq.get_task().unwrap();
        let pcb2 = mlfq.get_task().unwrap();
        let pcb3 = mlfq.get_task().unwrap();

        mlfq.requeue(pcb1);
        mlfq.requeue(pcb2);
        mlfq.requeue(pcb3);

        let pcb1 = mlfq.get_task().unwrap();
        mlfq.requeue(pcb1);

        mlfq.enqueue(pcb4);

        let pcb4 = mlfq.get_task().unwrap();
        let pcb2 = mlfq.get_task().unwrap();
        let pcb3 = mlfq.get_task().unwrap();
        let pcb1 = mlfq.get_task().unwrap();

        assert!(pid1 == pcb1.as_ref().pid.0);
        assert!(pid2 == pcb2.as_ref().pid.0);
        assert!(pid3 == pcb3.as_ref().pid.0);
        assert!(pid4 == pcb4.as_ref().pid.0);

        Ok("passed")
    });

    test!(test_schd, {
        let app_inode = find_inode("/bin/daemon").expect("[kernel] daemon not found!");
        let size = app_inode.get_file_size() as usize;
        let mut app_data = vec![0u8; size];
        app_inode.read_at(0, &mut app_data);

        let pcb1 = Rc::new(ProcessControlBlock::new(&app_data));
        let pcb2 = Rc::new(ProcessControlBlock::new(&app_data));
        let pcb3 = Rc::new(ProcessControlBlock::new(&app_data));
        let pcb4 = Rc::new(ProcessControlBlock::new(&app_data));

        let pid1 = pcb1.as_ref().pid.0;
        let pid2 = pcb2.as_ref().pid.0;
        let pid3 = pcb3.as_ref().pid.0;
        let pid4 = pcb4.as_ref().pid.0;

        let mut master = SchdMaster::new();
        
        master.add_new_task(pcb1);
        master.add_new_task(pcb2);
        master.add_new_task(pcb3);
        master.add_new_task(pcb4);

        let pcb1 = master.get_next().unwrap();
        let pcb2 = master.get_next().unwrap();
        let pcb3 = master.get_next().unwrap();

        assert!(pid1 == pcb1.as_ref().pid.0);
        assert!(pid2 == pcb2.as_ref().pid.0);
        assert!(pid3 == pcb3.as_ref().pid.0);

        master.requeue_current(pcb1);
        let pcb4 = master.get_next().unwrap();
        assert!(pid4 == pcb4.as_ref().pid.0);

        master.requeue_current(pcb2);
        master.requeue_current(pcb4);
        master.requeue_current(pcb3);

        let pcb1 = master.get_next().unwrap();
        let pcb2 = master.get_next().unwrap();
        let pcb4 = master.get_next().unwrap();
        let pcb3 = master.get_next().unwrap();

        assert!(pid1 == pcb1.as_ref().pid.0);
        assert!(pid2 == pcb2.as_ref().pid.0);
        assert!(pid3 == pcb3.as_ref().pid.0);
        assert!(pid4 == pcb4.as_ref().pid.0);

        Ok("passed")
    });
}
