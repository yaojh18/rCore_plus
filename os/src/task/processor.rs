//! Implementation of [`Processor`] and Intersection of control flow
//!
//! Here, the continuous operation of user apps in CPU is maintained,
//! the current running state of CPU is recorded,
//! and the replacement and transfer of control flow of different applications are executed.


use super::__switch;
use super::{fetch_task, TaskStatus};
use super::{TaskContext, TaskControlBlock, TaskSyscallTimes};
use crate::sync::UPSafeCell;
use crate::trap::TrapContext;
use crate::mm::{VirtAddr,MapPermission};
use alloc::sync::Arc;
use lazy_static::*;

const SYSCALL_UNLINKAT: usize = 35;
const SYSCALL_LINKAT: usize = 37;
const SYSCALL_OPEN: usize = 56;
const SYSCALL_CLOSE: usize = 57;
const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_FSTAT: usize = 80;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_GETPID: usize = 172;
const SYSCALL_FORK: usize = 220;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_WAITPID: usize = 260;
const SYSCALL_SPAWN: usize = 400;
const SYSCALL_MUNMAP: usize = 215;
const SYSCALL_MMAP: usize = 222;
const SYSCALL_SET_PRIORITY: usize = 140;
const SYSCALL_TASK_INFO: usize = 410;

/// Processor management structure
pub struct Processor {
    /// The task currently executing on the current processor
    current: Option<Arc<TaskControlBlock>>,
    /// The basic control flow of each core, helping to select and switch process
    idle_task_cx: TaskContext,
}

impl Processor {
    pub fn new() -> Self {
        Self {
            current: None,
            idle_task_cx: TaskContext::zero_init(),
        }
    }
    fn get_idle_task_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_cx as *mut _
    }
    pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current.take()
    }
    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.current.as_ref().map(|task| Arc::clone(task))
    }
}

lazy_static! {
    /// PROCESSOR instance through lazy_static!
    pub static ref PROCESSOR: UPSafeCell<Processor> = unsafe { UPSafeCell::new(Processor::new()) };
}

/// The main part of process execution and scheduling
///
/// Loop fetch_task to get the process that needs to run,
/// and switch the process through __switch
pub fn run_tasks() {
    loop {
        let mut processor = PROCESSOR.exclusive_access();
        if let Some(task) = fetch_task() {
            let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
            // access coming task TCB exclusively
            let mut task_inner = task.inner_exclusive_access();
            let next_task_cx_ptr = &task_inner.task_cx as *const TaskContext;
            task_inner.task_status = TaskStatus::Running;
            drop(task_inner);
            // release coming task TCB manually
            processor.current = Some(task);
            // release processor manually
            drop(processor);
            unsafe {
                __switch(idle_task_cx_ptr, next_task_cx_ptr);
            }
        }
    }
}

/// Get current task through take, leaving a None in its place
pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().take_current()
}

/// Get a copy of the current task
pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().current()
}

/// Get token of the address space of current task
pub fn current_user_token() -> usize {
    let task = current_task().unwrap();
    let token = task.inner_exclusive_access().get_user_token();
    token
}

/// Get the mutable reference to trap context of current task
pub fn current_trap_cx() -> &'static mut TrapContext {
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .get_trap_cx()
}

/// Return to idle control flow for new scheduling
pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    let mut processor = PROCESSOR.exclusive_access();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    drop(processor);
    unsafe {
        __switch(switched_task_cx_ptr, idle_task_cx_ptr);
    }
}

pub fn get_task_status() -> TaskStatus {
    let task = current_task().unwrap();
    let status=task.inner_exclusive_access().task_status;
    status
}

pub fn get_task_syscall_times() -> TaskSyscallTimes{
    let task = current_task().unwrap();
    let times=task.inner_exclusive_access().task_sys;
    times
}

pub fn get_task_begin_time() -> usize{
    let task=current_task().unwrap();
    let ti=task.inner_exclusive_access().begin_time;
    ti
}

pub fn change_task_syscall_times(id:usize){
    let task = current_task().unwrap();
    match id {
        SYSCALL_UNLINKAT => task.inner_exclusive_access().task_sys.unlink+=1,
        SYSCALL_LINKAT => task.inner_exclusive_access().task_sys.link+=1,
        SYSCALL_OPEN => task.inner_exclusive_access().task_sys.open+=1,
        SYSCALL_CLOSE => task.inner_exclusive_access().task_sys.close+=1,
        SYSCALL_READ => task.inner_exclusive_access().task_sys.read+=1,
        SYSCALL_WRITE => task.inner_exclusive_access().task_sys.write+=1,
        SYSCALL_FSTAT=> task.inner_exclusive_access().task_sys.fstat+=1,
        SYSCALL_EXIT => task.inner_exclusive_access().task_sys.exit+=1,
        SYSCALL_YIELD => task.inner_exclusive_access().task_sys.yld+=1,
        SYSCALL_GET_TIME => task.inner_exclusive_access().task_sys.get_time_of_day+=1,
        SYSCALL_GETPID => task.inner_exclusive_access().task_sys.get_pid+=1,
        SYSCALL_FORK => task.inner_exclusive_access().task_sys.fork+=1,
        SYSCALL_EXEC => task.inner_exclusive_access().task_sys.exec+=1,
        SYSCALL_WAITPID => task.inner_exclusive_access().task_sys.wait_pid+=1,
        SYSCALL_SPAWN => task.inner_exclusive_access().task_sys.spawn+=1,
        SYSCALL_MUNMAP=>task.inner_exclusive_access().task_sys.munmap+=1,
        SYSCALL_MMAP=>task.inner_exclusive_access().task_sys.mmap+=1,
        SYSCALL_SET_PRIORITY=>task.inner_exclusive_access().task_sys.priority+=1,
        SYSCALL_TASK_INFO => task.inner_exclusive_access().task_sys.task_info+=1,
        _ => panic!("Unsupported syscall_id: {}", id),
    }
}

pub fn in_task_page_table(start_va:usize)->bool{
    let task=current_task().unwrap();
    let va=VirtAddr::from(start_va);
    let vpn=va.floor();
    let ppn_op=task.inner_exclusive_access().memory_set.translate(vpn);
    if ppn_op.is_none(){
        return false;
    }
    let ppn=ppn_op.unwrap().ppn();
    if ppn.0==0{
        return false;
    }
    true
}

pub fn insert_task_area(start_va: VirtAddr,end_va: VirtAddr,permission: MapPermission){
    let task=current_task().unwrap();
    task.inner_exclusive_access().memory_set.insert_framed_area(start_va, end_va, permission);
}

pub fn change_task_priority(_prio:isize)->isize{
    if _prio<2{
        return -1;
    }
    let task=current_task().unwrap();
    task.inner_exclusive_access().priority=_prio;
    let prio=task.inner_exclusive_access().priority;
    prio
}
