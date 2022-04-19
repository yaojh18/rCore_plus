//! Process management syscalls

use core::ptr::null_mut;

use crate::config::{MAX_SYSCALL_NUM,PAGE_SIZE};
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next,current_user_token,insert_task_area,in_task_page_table, TaskStatus,TaskSyscallTimes,get_task_status,get_task_syscall_times};
use crate::timer::get_time_us;
use crate::mm::{translated_any, PhysPageNum, VirtAddr, VirtPageNum, PhysAddr, MapArea,MapType,MapPermission,PageTable};

const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_MUNMAP: usize = 215;
const SYSCALL_MMAP: usize = 222;
const SYSCALL_SET_PRIORITY: usize = 140;
const SYSCALL_TASK_INFO: usize = 410;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

#[derive(Clone, Copy)]
pub struct TaskInfo {
    pub status: TaskStatus,
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    pub time: usize,
}

pub fn sys_exit(exit_code: i32) -> ! {
    info!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

// YOUR JOB: 引入虚地址后重写 sys_get_time
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    let _us = get_time_us();
    // unsafe {
    //     *ts = TimeVal {
    //         sec: us / 1_000_000,
    //         usec: us % 1_000_000,
    //     };
    // }
    /*
    let start=_ts as *const u8;
    let len=8 as usize;
    //let buffers = translated_byte_buffer(current_user_token(), start, len);
    let tv=TimeVal{
        sec:_us/1_000_000,
        usec:_us%1_000_000,
    };
    let n_ptr=&tv as *const TimeVal;
    let mut ptr=n_ptr as *mut usize;
    unsafe{
    info!("{}",*ptr);
    }
    unsafe{
        info!("{}",*((ptr as usize - 4) as *mut usize));
    }
    info!("sec{}",tv.sec);
    info!("usec{}",tv.usec);
    
    for buffer in buffers{
        info!("begin{}",n_ptr as u8);
        for j in 0..buffer.len(){
            info!("mid{}",ptr as u8);
            unsafe{
                buffer[j]=*ptr;
            }
            ptr=(ptr as u8 - 1) as *mut u8;
            info!("end{}",ptr as u8);
        }
    }
    */
    
    let n_ptr=translated_any::<TimeVal>(current_user_token(),_ts);
    unsafe {
        *n_ptr=TimeVal{
            sec:_us/1_000_000,
            usec:_us%1_000_000,
        };
    }
    0
}

// CLUE: 从 ch4 开始不再对调度算法进行测试~
pub fn sys_set_priority(_prio: isize) -> isize {
    -1
}

// YOUR JOB: 扩展内核以实现 sys_mmap 和 sys_munmap
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    if _port & !0x7 != 0{
        return -1;
    }
    if _port & 0x7 == 0{
        return -1;
    }
    let _end=_start+_len;
    let start_va=VirtAddr::from(_start);
    let end_va=VirtAddr::from(_end);
    if start_va.page_offset()!=0{
        return -1;
    }
    let mut now=_start;
    //info!("id={},start={},end={}",current_user_token(),_start,_end);
    while now<_end{
        if in_task_page_table(now){
            return -1;
        }
        now+=PAGE_SIZE;
    }
    if in_task_page_table(_end){
        info!("id={},start={}",current_user_token(),_start);
    }

    let mut map_perm=MapPermission::U;
    if _port&1==1{
        map_perm|=MapPermission::R;
    }
    if (_port>>1)&1==1{
        map_perm|=MapPermission::W;
    }
    if (_port>>2)&1==1{
        map_perm|=MapPermission::X;
    }
    insert_task_area(start_va, end_va,map_perm);
    0
}

pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    let _end=_start+_len;
    let start_va=VirtAddr::from(_start);
    let end_va=VirtAddr::from(_end);
    if start_va.page_offset()!=0{
        return -1;
    }
    let mut now=_start;
    while now<_end{
        if !in_task_page_table(now){
            return -1;
        }
        now+=PAGE_SIZE;
    }
    let mut page_table=PageTable::from_token(current_user_token());
    let mut map_area=MapArea::new(start_va,end_va,MapType::Framed,MapPermission::U);
    map_area.unmap(&mut page_table);
    0
}

// YOUR JOB: 引入虚地址后重写 sys_task_info
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    let ptr=translated_any(current_user_token(), ti);
    let sys = get_task_syscall_times();
    unsafe {
        (*ptr).syscall_times[SYSCALL_GET_TIME]=sys.get_time_of_day as u32; 
        (*ptr).syscall_times[SYSCALL_TASK_INFO]=sys.task_info as u32;
        (*ptr).syscall_times[SYSCALL_WRITE]=sys.write as u32;
        (*ptr).syscall_times[SYSCALL_YIELD]=sys.yld as u32;
        (*ptr).syscall_times[SYSCALL_EXIT]=sys.exit as u32;
        (*ptr).syscall_times[SYSCALL_MMAP]=sys.mmap as u32;
        (*ptr).syscall_times[SYSCALL_MUNMAP]=sys.munmap as u32;
        (*ptr).syscall_times[SYSCALL_SET_PRIORITY]=sys.priority as u32;
        (*ptr).time=get_time_us()/1000;
        (*ptr).status=get_task_status();
    }
    0
}
