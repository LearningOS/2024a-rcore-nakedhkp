use crate::sync::{Condvar, Mutex, MutexBlocking, MutexSpin, Semaphore};
use crate::task::{block_current_and_run_next, current_process, current_task};
use crate::timer::{add_timer, get_time_ms};
use alloc::sync::Arc;
use alloc::vec;
/// sleep syscall
pub fn sys_sleep(ms: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_sleep",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let expire_ms = get_time_ms() + ms;
    let task = current_task().unwrap();
    add_timer(expire_ms, task);
    block_current_and_run_next();
    0
}
/// mutex create syscall
pub fn sys_mutex_create(blocking: bool) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mutex: Option<Arc<dyn Mutex>> = if !blocking {
        Some(Arc::new(MutexSpin::new()))
    } else {
        Some(Arc::new(MutexBlocking::new()))
    };
    let mut process_inner = process.inner_exclusive_access();
    if let Some(id) = process_inner
        .mutex_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.mutex_list[id] = mutex;
        id as isize
    } else {
        process_inner.mutex_list.push(mutex);
        process_inner.mutex_list.len() as isize - 1
    }
}
/// mutex lock syscall
pub fn sys_mutex_lock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_lock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    let deadlock_detection_enabled = process_inner.deadlock_detection_enabled;

    drop(process_inner);




    if deadlock_detection_enabled {
        let task = current_task().unwrap();
        {
            let mut task_inner = task.inner_exclusive_access();
            if mutex_id >= task_inner.requested_mutexes.len() {
                task_inner.requested_mutexes.resize(mutex_id + 1, 0);
            }
            task_inner.requested_mutexes[mutex_id] += 1;
            drop(task_inner); 
        }

        if detect_deadlock_mutex() {
            let mut task_inner = task.inner_exclusive_access();
            task_inner.requested_mutexes[mutex_id] -= 1;
            return -0xDEAD;
        }

        mutex.lock();
        let mut task_inner = task.inner_exclusive_access();
        task_inner.requested_mutexes[mutex_id] -= 1;
        if mutex_id >= task_inner.held_mutexes.len() {
            task_inner.held_mutexes.resize(mutex_id + 1, 0);
        }
        task_inner.held_mutexes[mutex_id] += 1;
    } else {
        mutex.lock();
    }


    0
}
/// mutex unlock syscall
pub fn sys_mutex_unlock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_unlock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());

    let task = current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();

    if process_inner.deadlock_detection_enabled {
        task_inner.held_mutexes[mutex_id] -= 1;
    }

    drop(process_inner);
    drop(process);
    mutex.unlock();
    0
}
/// semaphore create syscall
pub fn sys_semaphore_create(res_count: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .semaphore_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.semaphore_list[id] = Some(Arc::new(Semaphore::new(res_count)));
        id
    } else {
        process_inner
            .semaphore_list
            .push(Some(Arc::new(Semaphore::new(res_count))));
        process_inner.semaphore_list.len() - 1
    };
    id as isize
}
/// semaphore up syscall
pub fn sys_semaphore_up(sem_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_up",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());

    let task = current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();

    if process_inner.deadlock_detection_enabled {
        task_inner.held_semaphore[sem_id] -= 1;
    }
    
    drop(process_inner);
    sem.up();
    0
}
/// semaphore down syscall
pub fn sys_semaphore_down(sem_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_down",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    let deadlock_detection_enabled = process_inner.deadlock_detection_enabled;

    drop(process_inner);



    if deadlock_detection_enabled {
        let task = current_task().unwrap();
        {
       
            let mut task_inner = task.inner_exclusive_access();
            if sem_id >= task_inner.requested_semaphore.len() {
                task_inner.requested_semaphore.resize(sem_id + 1, 0);
            }
            task_inner.requested_semaphore[sem_id] += 1;
            drop(task_inner); 
        }

        if detect_deadlock_semaphore() {
            let mut task_inner = task.inner_exclusive_access();
            task_inner.requested_semaphore[sem_id] -= 1;
            return  -0xDEAD;
        }

        sem.down();

        let mut task_inner = task.inner_exclusive_access();
        task_inner.requested_semaphore[sem_id] -= 1;
        if sem_id >= task_inner.held_semaphore.len() {
            task_inner.held_semaphore.resize(sem_id + 1, 0);
        }
        task_inner.held_semaphore[sem_id] += 1;

    } else {
        sem.down();
    }

    0
}
/// condvar create syscall
pub fn sys_condvar_create() -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .condvar_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.condvar_list[id] = Some(Arc::new(Condvar::new()));
        id
    } else {
        process_inner
            .condvar_list
            .push(Some(Arc::new(Condvar::new())));
        process_inner.condvar_list.len() - 1
    };
    id as isize
}
/// condvar signal syscall
pub fn sys_condvar_signal(condvar_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_signal",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    drop(process_inner);
    condvar.signal();
    0
}
/// condvar wait syscall
pub fn sys_condvar_wait(condvar_id: usize, mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_wait",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    condvar.wait(mutex);
    0
}
/// enable deadlock detection syscall
///
/// YOUR JOB: Implement deadlock detection, but might not all in this syscall
pub fn sys_enable_deadlock_detect(_enabled: usize) -> isize {
    trace!("kernel: sys_enable_deadlock_detect");
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();

    match _enabled {
        0 => {
            process_inner.deadlock_detection_enabled = false;
            0
        }
        1 => {
            process_inner.deadlock_detection_enabled = true;
            0
        }
        _ => {
            -1
        }
    }
}

fn detect_deadlock_mutex() -> bool {

    let process = current_process();
    let process_inner = process.inner_exclusive_access();

    let tasks = &process_inner.tasks;
    let mutexes = &process_inner.mutex_list;


    let n = tasks.len(); 
    let m = mutexes.len(); 

    let mut available = vec![1; m];
    let mut allocation = vec![vec![0; m]; n];
    let mut need = vec![vec![0; m]; n];


    for (i, task_option) in tasks.iter().enumerate() {
        if let Some(task) = task_option {
            let task_inner = task.inner_exclusive_access();

            // 更新 Allocation 矩阵
            for (j, &count) in task_inner.held_mutexes.iter().enumerate() {
                if j < m {
                    allocation[i][j] = count;
                }
            }

            // 更新 Need 矩阵
            for (j, &count) in task_inner.requested_mutexes.iter().enumerate() {
                if j < m {
                    need[i][j] = count;
                }
            }
        }
    }

    drop(process_inner);


    for j in 0..m {
        let mut allocated = 0;
        for i in 0..n {
            allocated += allocation[i][j];
        }
        // 对于互斥锁，总量为 1
        available[j] = 1 - allocated;
    }


    let mut finish = vec![false; n];


    loop {
        let mut found = false;
        for i in 0..n {
            if !finish[i] {
                let mut can_finish = true;
                for j in 0..m {
                    if need[i][j] > available[j] {
                        can_finish = false;
                        break;
                    }
                }
                if can_finish {
                    for j in 0..m {
                        available[j] += allocation[i][j];
                    }
                    finish[i] = true;
                    found = true;
                }
            }
        }
        if !found {
            break;
        }
    }


    for i in 0..n {
        if !finish[i] {
            return true; 
        }
    }

    false 
}

fn detect_deadlock_semaphore() -> bool {
    let process = current_process();
    let process_inner = process.inner_exclusive_access();

    let tasks = &process_inner.tasks;
    let semaphores = &process_inner.semaphore_list;

    let n = tasks.len(); // 线程数
    let m = semaphores.len(); // 信号量数

    let mut available = vec![0; m];
    let mut allocation = vec![vec![0; m]; n];
    let mut need = vec![vec![0; m]; n];
    let mut total = vec![0; m];


    for (j, sem_option) in semaphores.iter().enumerate() {
        if let Some(sem) = sem_option {
            total[j] = sem.initial_count;
        }
    }

    for (i, task_option) in tasks.iter().enumerate() {
        if let Some(task) = task_option {
            let task_inner = task.inner_exclusive_access();

      
            for (j, &count) in task_inner.held_semaphore.iter().enumerate() {
                if j < m {
                    allocation[i][j] = count;
                }
            }

        
            for (j, &count) in task_inner.requested_semaphore.iter().enumerate() {
                if j < m {
                    need[i][j] = count;
                }
            }
        }
    }

    drop(process_inner);


    for j in 0..m {
        let mut allocated = 0;
        for i in 0..n {
            allocated += allocation[i][j];
        }
        
        available[j] = total[j] - allocated;
    }

  
    let mut finish = vec![false; n];

    
    loop {
        let mut found = false;
        for i in 0..n {
            if !finish[i] {
                let mut can_finish = true;
                for j in 0..m {
                    if need[i][j] > available[j] {
                        can_finish = false;
                        break;
                    }
                }
                if can_finish {
            
                    for j in 0..m {
                        available[j] += allocation[i][j];
                    }
                    finish[i] = true;
                    found = true;
                }
            }
        }
        if !found {
            break;
        }
    }

    for i in 0..n {
        if !finish[i] {
            return true;
        }
    }

    false 
}
