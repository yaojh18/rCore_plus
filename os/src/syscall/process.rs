//! Process management syscalls

use crate::mm::{translated_refmut, translated_ref, translated_str,translated_any, PhysPageNum, VirtAddr, VirtPageNum, PhysAddr, MapArea,MapType,MapPermission,PageTable};
use crate::task::{
    add_task, current_task, current_user_token, exit_current_and_run_next,
    suspend_current_and_run_next, TaskStatus,TaskSyscallTimes,get_task_status,get_task_syscall_times,get_task_begin_time,in_task_page_table,insert_task_area,change_task_priority,
};
use crate::fs::{open_file, OpenFlags};
use crate::timer::get_time_us;
use alloc::sync::Arc;
use alloc::vec::Vec;
use crate::config::{MAX_SYSCALL_NUM,PAGE_SIZE};
use alloc::string::String;

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
    debug!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_getpid() -> isize {
    current_task().unwrap().pid.0 as isize
}

/// Syscall Fork which returns 0 for child process and child_pid for parent process
pub fn sys_fork() -> isize {
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    // modify trap context of new_task, because it returns immediately after switching
    let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.x[10] = 0;
    // add new task to scheduler
    add_task(new_task);
    new_pid as isize
}

/// Syscall Exec which accepts the elf path
pub fn sys_exec(path: *const u8) -> isize {
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(app_inode) = open_file(path.as_str(), OpenFlags::RDONLY) {
        let all_data = app_inode.read_all();
        let task = current_task().unwrap();
        task.exec(all_data.as_slice());
        0
    } else {
        -1
    }
}


/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let task = current_task().unwrap();
    // find a child process

    // ---- access current TCB exclusively
    let mut inner = task.inner_exclusive_access();
    if !inner
        .children
        .iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        return -1;
        // ---- release current PCB
    }
    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        // ++++ temporarily access child PCB lock exclusively
        p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        // ++++ release child PCB
    });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        // confirm that child will be deallocated after removing from children list
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // ++++ temporarily access child TCB exclusively
        let exit_code = child.inner_exclusive_access().exit_code;
        // ++++ release child PCB
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
    // ---- release current PCB lock automatically
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
    let n_ptr=translated_any::<TimeVal>(current_user_token(),_ts);
    unsafe {
        *n_ptr=TimeVal{
            sec:_us/1_000_000,
            usec:_us%1_000_000,
        };
    }

    0
}

// YOUR JOB: 引入虚地址后重写 sys_task_info
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    let ptr=translated_any(current_user_token(), ti);
    let sys = get_task_syscall_times();
    unsafe {
        (*ptr).syscall_times[SYSCALL_UNLINKAT]=sys.read as u32;
        (*ptr).syscall_times[SYSCALL_LINKAT]=sys.read as u32;
        (*ptr).syscall_times[SYSCALL_OPEN]=sys.read as u32;
        (*ptr).syscall_times[SYSCALL_CLOSE]=sys.read as u32;
        (*ptr).syscall_times[SYSCALL_READ]=sys.read as u32;
        (*ptr).syscall_times[SYSCALL_WRITE]=sys.write as u32;
        (*ptr).syscall_times[SYSCALL_FSTAT]=sys.read as u32;
        (*ptr).syscall_times[SYSCALL_EXIT]=sys.exit as u32;
        (*ptr).syscall_times[SYSCALL_YIELD]=sys.yld as u32;
        (*ptr).syscall_times[SYSCALL_GET_TIME]=sys.get_time_of_day as u32; 
        (*ptr).syscall_times[SYSCALL_GETPID]=sys.get_pid as u32;
        (*ptr).syscall_times[SYSCALL_FORK]=sys.fork as u32;
        (*ptr).syscall_times[SYSCALL_EXEC]=sys.exec as u32;
        (*ptr).syscall_times[SYSCALL_WAITPID]=sys.wait_pid as u32;
        (*ptr).syscall_times[SYSCALL_SPAWN]=sys.spawn as u32;
        (*ptr).syscall_times[SYSCALL_MUNMAP]=sys.munmap as u32;
        (*ptr).syscall_times[SYSCALL_MMAP]=sys.mmap as u32;
        (*ptr).syscall_times[SYSCALL_SET_PRIORITY]=sys.priority as u32;
        (*ptr).syscall_times[SYSCALL_TASK_INFO]=sys.task_info as u32;
        (*ptr).time=(get_time_us()-get_task_begin_time())/1000;
        (*ptr).status=get_task_status();
    }
    0

}

// YOUR JOB: 实现sys_set_priority，为任务添加优先级
pub fn sys_set_priority(_prio: isize) -> isize {
    change_task_priority(_prio)
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
    while now<_end{
        if in_task_page_table(now){
            return -1;
        }
        now+=PAGE_SIZE;
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

//
// YOUR JOB: 实现 sys_spawn 系统调用
// ALERT: 注意在实现 SPAWN 时不需要复制父进程地址空间，SPAWN != FORK + EXEC 
pub fn sys_spawn(_path: *const u8) -> isize {
    let token = current_user_token();
    let path = translated_str(token, _path);
    if let Some(app_inode) = open_file(path.as_str(), OpenFlags::RDONLY) {
        let all_data = app_inode.read_all();
        let current_task = current_task().unwrap();
        let new_task = current_task.spawn(all_data.as_slice());
        let new_pid = new_task.pid.0;
        let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
        trap_cx.x[10] = 0;
        add_task(new_task);
        new_pid as isize
    } else {
        -1
    }
}
