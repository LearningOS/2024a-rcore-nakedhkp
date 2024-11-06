# 我实现的功能

本节中主要实现了死锁的检测算法，我将其独立地分为了互斥锁和信号量，分别对其进行检测。首先为了执行银行家算法，要实时地维护一些数据结构，所以我在`TaskControlBlock`上添加了`held_mutexes`,`requested_mutexes`和`held_semaphore`,`requested_semaphore`进行实时更新，在银行家算法中将其转换为`allocation`和`need`数组，并进行安全检测。我本次实验用了1天

# 问答题

1. 在我们的多线程实现中，当主线程 (即 0 号线程) 退出时，视为整个进程退出， 此时需要结束该进程管理的所有线程并回收其资源。

A. 需要回收的资源有哪些？
- 线程的 TCB (任务控制块)
- 栈内存
- 同步原语 (互斥锁、信号量等)

B. 其他线程的 TaskControlBlock 可能在哪些位置被引用，分别是否需要回收，为什么？
```
1. 任务管理相关：
- TaskManager.ready_queue (就绪队列)
- TaskManager.stop_task (停止任务队列) 
- ProcessControlBlock.tasks (进程任务列表)
- Processor.current (当前执行线程)

2. 同步原语相关：
- CondvarInner.wait_queue (条件变量等待队列)
- MutexBlockingInner.wait_queue (互斥锁等待队列)
- SemaphoreInner.wait_queue (信号量等待队列)
- TimerCondVar.task (定时器任务)
```
以上位置的 TaskControlBlock 都需要回收，以防止内存泄漏。

2. 对比以下两种 `Mutex.unlock` 的实现，二者有什么区别？这些区别可能会导致什么问题？

A. 实现差异：
```rust
// Mutex1: 先解锁后唤醒
mutex_inner.locked = false;
if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
    add_task(waking_task);
}

// Mutex2: 仅在无等待任务时解锁
if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
    add_task(waking_task);
} else {
    mutex_inner.locked = false;
}
```

B. 问题分析：
- Mutex1 主要问题：
  - 存在竞态条件：提前解锁可能导致多个线程同时获得锁
  - 违反互斥性：可能造成数据竞争和不一致

- Mutex2 的优势：
  - 保证互斥性：锁的所有权直接从释放者转移到等待者
  - 避免竞态：不会出现多个线程同时持有锁的情况

- Mutex2 的潜在问题：
  - 可能出现线程饥饿
  - 需要确保等待队列的公平性


1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 **以下各位** 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：
    
    > _无_
    
2. 此外，我也参考了 **以下资料** ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：
    
    > _无_
    

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。