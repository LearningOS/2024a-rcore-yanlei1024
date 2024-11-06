//! Process management syscalls
use core::mem::size_of;

use crate::{
    config::MAX_SYSCALL_NUM, mm::{MapPermission, VirtAddr}, task::{
        change_program_brk, current_user_token, exit_current_and_run_next, get_current_task_first_run_time, get_current_task_memory_set, get_current_task_syscall_times, suspend_current_and_run_next, TaskStatus
    }, timer::{get_time_ms, get_time_us}
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let mut time_val = &TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    } as *const TimeVal as *const u8;
    let ts = crate::mm::translated_byte_buffer(current_user_token(), ts as *const u8, size_of::<TimeVal>());
    for dst in ts {
        unsafe {
            time_val.copy_to(dst.as_mut_ptr(), dst.len());
            time_val = time_val.add(dst.len());
        }
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");
    // let mut syscall_times = [0; MAX_SYSCALL_NUM];
    // for (i, &v) in get_current_task_syscall_times().iter().enumerate() {
    //     syscall_times[i] = v
    // }
    let mut task_info = &TaskInfo {
        status: TaskStatus::Running,
        syscall_times: get_current_task_syscall_times(),
        time: get_time_ms() - get_current_task_first_run_time(),
    } as *const _ as *const u8;
    let ti = crate::mm::translated_byte_buffer(current_user_token(), ti as *const u8, size_of::<TaskInfo>());
    for dst in ti {
        unsafe {
            task_info.copy_to(dst.as_mut_ptr(), dst.len());
            task_info = task_info.add(dst.len());
        }
    }
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    let start_va = VirtAddr::from(start);
    if start_va.floor().0 << 12 != start_va.0 || port & !7 != 0 || port & 7 == 0 {
        return -1; 
    }
    let end_va = VirtAddr::from(start + len);
    let permission = MapPermission::from_bits((port as u8) << 1).unwrap() | MapPermission::U;
    let memorey_set = get_current_task_memory_set();
    let mut memorey_set = memorey_set.exclusive_access();
    if memorey_set.conflicts_check(start_va, end_va) {
        return -1;
    }
    memorey_set.insert_framed_area(start_va, end_va, permission);
    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    let start_va = VirtAddr::from(start);
    if start_va.floor().0 << 12 != start_va.0 {
        return -1; 
    }
    let end_va = VirtAddr::from(start + len);
    let memorey_set = get_current_task_memory_set();
    let mut memorey_set = memorey_set.exclusive_access();
    memorey_set.remove_framed_area(start_va, end_va)
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
