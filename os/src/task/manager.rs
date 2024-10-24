//!Implementation of [`TaskManager`]
use core::usize::MAX;

use super::TaskControlBlock;
use crate::sync::UPSafeCell;

use alloc::{sync::Arc, vec::Vec};
use lazy_static::*;
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready_vec: Vec<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_vec: Vec::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_vec.push(task);
    }
    /// Take a process out of the ready queue
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        let mut min_stride: usize = MAX;
        let mut min_task_index: Option<usize> = None;

        for (index, task) in self.ready_vec.iter().enumerate() {
            let inner = task.inner_exclusive_access();
            if inner.stride < min_stride {
                min_stride = inner.stride;
                min_task_index = Some(index);
            }
        }

        if let Some(index) = min_task_index {
            let task = self.ready_vec.remove(index);
            let mut task_inner = task.inner_exclusive_access();
            task_inner.stride += task_inner.pass;
            drop(task_inner);
            return Some(task);
        }

        None
    }
}

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
