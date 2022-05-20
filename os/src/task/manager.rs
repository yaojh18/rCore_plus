//! Implementation of [`TaskManager`]
//!
//! It is only used to manage processes and schedule process based on ready queue.
//! Other CPU process monitoring functions are in Processor.


use super::{TaskControlBlock, processor};
use crate::sync::UPSafeCell;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use alloc::vec::Vec;
use lazy_static::*;

const BIG_STRIDE:usize=252000;

pub struct TaskManager {
    //ready_queue: VecDeque<Arc<TaskControlBlock>>,
    ready_vec: Vec<Arc<TaskControlBlock>>,
}

// YOUR JOB: FIFO->Stride
/// A simple FIFO scheduler.
impl TaskManager {
    pub fn new() -> Self {
        Self {
            //ready_queue: VecDeque::new(),
            ready_vec:Vec::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        //self.ready_queue.push_back(task);
        self.ready_vec.push(task);

    }
    /// Take a process out of the ready queue
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        //self.ready_queue.pop_front()
        let mut minstride=usize::MAX;
        let mut minord=usize::MAX;
        for i in 0..self.ready_vec.len(){
            let tcb=self.ready_vec.get(i).unwrap();
            let stride=tcb.inner_exclusive_access().stride;
            if stride<minstride{
                minstride=stride;
                minord=i;
            }
        }
        if(minord==usize::MAX){
            return None;
        }
        let mut tcb=self.ready_vec.get(minord).unwrap();
        let prio=tcb.inner_exclusive_access().priority as usize;
        tcb.inner_exclusive_access().stride+=BIG_STRIDE/prio;
        Some(self.ready_vec.remove(minord))

    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

pub fn add_task(task: Arc<TaskControlBlock>) {
    TASK_MANAGER.exclusive_access().add(task);
}

pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.exclusive_access().fetch()
}
