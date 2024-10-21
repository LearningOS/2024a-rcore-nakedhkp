//! Process management syscalls
use crate::{
    config::{MAX_SYSCALL_NUM, PAGE_SIZE}, mm::{translated_byte_buffer, MapPermission, VirtAddr}, task::{
        change_program_brk, current_user_token, exit_current_and_run_next, get_current_memset, get_task_info_count, get_task_info_time, suspend_current_and_run_next, TaskStatus
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
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");

    let current_time = get_time_us();
    let sec = current_time / 1_000_000;
    let usec = current_time % 1_000_000;

    let time_val = TimeVal { sec, usec };

   
    let buffer = translated_byte_buffer(current_user_token(), _ts as *const u8, core::mem::size_of::<TimeVal>());
    
    let time_val_bytes = unsafe { core::slice::from_raw_parts(&time_val as *const _ as *const u8, core::mem::size_of::<TimeVal>()) };

    let mut offset = 0;
    for page in buffer {
        let len = page.len().min(time_val_bytes.len() - offset);
        page[..len].copy_from_slice(&time_val_bytes[offset..offset + len]);
        offset += len;

        if offset >= time_val_bytes.len() {
            break;
        }
    }

    0
}


/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");
    
    let v = translated_byte_buffer(current_user_token(), _ti as *const u8, core::mem::size_of::<TaskInfo>());

    let task_info = TaskInfo {
        status: TaskStatus::Running,
        syscall_times: get_task_info_count(),
        time: get_time_ms() - get_task_info_time(),
    };

    let task_info_bytes = unsafe {core::slice::from_raw_parts(
        &task_info as *const _ as *const u8,
        core::mem::size_of::<TaskInfo>()
    )};

    let mut offset = 0;
    for page in v {
        let len = page.len().min(task_info_bytes.len() - offset);
        page[..len].copy_from_slice(&task_info_bytes[offset..offset + len]);
        offset += len;

        if offset >= task_info_bytes.len() {
            break;
        }
    }

    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");

    if _start % PAGE_SIZE != 0 || _port & !0x7 != 0 || _port & 0x7 == 0 {
        return -1;
    }

    let start_vpn = VirtAddr::from(_start).floor();
    let end_vpn = VirtAddr::from(_start + _len).ceil();

   
    let memory_set = unsafe { get_current_memset().as_mut() }.unwrap();

  
    if !memory_set.all_unmapped(start_vpn, end_vpn) {
        return -1;
    }


    memory_set.insert_framed_area(
        _start.into(),
        (_start + _len).into(),
        MapPermission::from_bits_truncate((_port << 1) as u8) | MapPermission::U,
    );

    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");

    if _start % PAGE_SIZE != 0 {
        return -1;
    }

    let start_vpn = VirtAddr::from(_start).floor();
    let end_vpn = VirtAddr::from(_start + _len).ceil();

    let memory_set = unsafe { get_current_memset().as_mut() }.unwrap();

    if !memory_set.all_mapped(start_vpn, end_vpn) {
        return -1;
    }

 
    memory_set.delete_frame_area(_start.into(), (_start + _len).into(), MapPermission::U);

    0
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
