use super::TaskContext;

#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    pub task_sys: TaskSyscallTimes,
}

#[derive(Copy, Clone)]
pub struct TaskSyscallTimes{
    pub get_time_of_day:usize,
    pub task_info:usize,
    pub write:usize,
    pub yld:usize,
    pub exit:usize,
}

impl TaskSyscallTimes {
    pub fn zero_init() -> Self{
        Self {
            get_time_of_day:0,
            task_info:0,
            write:0,
            yld:0,
            exit:0,
        }
    }
    
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}
