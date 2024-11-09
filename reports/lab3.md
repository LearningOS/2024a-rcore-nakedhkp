# 我的实现
1. 实现spawn系统调用
- 获取父进程内部数据的独占访问权
- 从ELF文件创建新的地址空间(memory_set)，获得用户栈指针(user_sp)和入口点(entry_point)
- 为新进程分配必要资源:PID、内核栈、陷阱上下文页号
- 创建新的任务控制块(TaskControlBlock),继承父进程的部分属性(如heap信息)并初始化新进程特有的属性(如stride调度相关参数)
- 初始化新进程的陷阱上下文(trap context)
- 将新进程加入父进程的子进程列表并返回
1. 实现stride调度算法
为TCB添加相应的`pass`,`priority`属性，在`config`中添加`BIG_STRIDE`。另外更改fetch函数，for循环查找stride最小的task



# 问答题

1. stride 算法原理非常简单，但是有一个比较大的问题。例如两个 pass = 10 的进程，使用 8bit 无符号整形储存 stride， p1.stride = 255, p2.stride = 250，在 p2 执行一个时间片后，理论上下一次应该 p1 执行。实际情况是轮到 p1 执行吗？为什么？
当 p2 执行一个时间片后，p2.stride = 250 + 10 = 260。但由于使用 8bit 无符号整形存储，260 在 8bit 中会溢出为 4 ，此时会认为 4 < 255，从而选择 p2 执行

2.  我们之前要求进程优先级 >= 2 其实就是为了解决这个问题。可以证明， **在不考虑溢出的情况下** , 在进程优先级全部 >= 2 的情况下，如果严格按照算法执行，那么 STRIDE_MAX – STRIDE_MIN <= BigStride / 2。为什么？尝试简单说明（不要求严格证明）
当进程优先级 >= 2 时，每个进程的 pass 值 <= BigStride/2。因为每次调度选择stride最小的进程执行，考虑极限情况最大的stride进程为$BIG\_STRIDE / 2$，最小的stride进程为$BIG\_STRIDE / {\infty}$所以STRIDE_MAX – STRIDE_MIN <= BigStride / 2

3. 已知以上结论，**考虑溢出的情况下**，可以为 Stride 设计特别的比较器，让 `BinaryHeap<Stride>` 的 `pop` 方法能返回真正最小的 `Stride`。补全下列代码中的 `partial_cmp` 函数，假设两个 Stride 永远不会相等。

```rust
use core::cmp::Ordering;

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    
        const STRIDE_MAX: u64 = u64::MAX;
        let diff = self.0.wrapping_sub(other.0);
        Some(if diff == 0 {
            Ordering::Equal
        } else if diff < STRIDE_MAX / 2 {
            Ordering::Less
        } else {
            Ordering::Greater
        })
    }
}

impl PartialEq for Stride {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}
```

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 **以下各位** 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：
    
    > _无_
    
2. 此外，我也参考了 **以下资料** ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：
    
    > _无_
    

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。