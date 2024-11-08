//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::sync::UPSafeCell;
use alloc::collections::{BinaryHeap, VecDeque};
use alloc::sync::Arc;
use lazy_static::*;

trait TaskManagerInterface {
    fn new() -> Self;
    fn add(&mut self, task: Arc<TaskControlBlock>);
    fn fetch(&mut self) -> Option<Arc<TaskControlBlock>>;
}

///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManagerFIFO {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManagerInterface for TaskManagerFIFO {
    ///Creat an empty TaskManager
    fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    /// Add process back to ready queue
    fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    /// Take a process out of the ready queue
    fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.ready_queue.pop_front()
    }
}

///
pub struct TaskManagerStride {
    ready_queue: BinaryHeap<Arc<TaskControlBlock>>,
}

/// A simple Stride scheduler.
impl TaskManagerInterface for TaskManagerStride {
    ///Creat an empty TaskManager
    fn new() -> Self {
        Self {
            ready_queue: BinaryHeap::new(),
        }
    }
    /// Add process back to ready queue
    fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push(task);
    }
    /// Take a process out of the ready queue
    fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        match self.ready_queue.pop() {
            Some(tcb) => {
                tcb.inner_exclusive_access().update_stride();
                Some(tcb)
            },
            None => None,
        }
    }
}

type TaskManager = TaskManagerStride;

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    //trace!("kernel: TaskManager::add_task");
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    //trace!("kernel: TaskManager::fetch_task");
    TASK_MANAGER.exclusive_access().fetch()
}
