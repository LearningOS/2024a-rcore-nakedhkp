## 实验报告
- 一次性加载所有用户程序，减少任务切换开销
- 支持任务切换机制，保存切换前后程序的上下文
- 支持程序主动放弃处理器，实现yield系统调用
- 以时间片轮转算法调度用户程序，实现资源的时分复用

多道程序放置
`user/build/py`

多道程序加载
`os/src/loader.rs`

任务切换
`os/src/task/switch.S`
先把 `current_task_cx_ptr` 中包含的寄存器值逐个保存，再把 `next_task_cx_ptr` 中包含的寄存器值逐个恢复。

与trap切换不同，它不涉及特权级切换，部分由编译器完成

任务切换是来自两个不同应用在内核中的 Trap 控制流之间的切换

我们会调用该函数来完成切换功能，而不是直接跳转到符号 `__switch` 的地址。 因此在调用前后，编译器会帮我们保存和恢复调用者保存寄存器。

调用 `sys_yield` 可以避免等待过程造成的资源浪费。

内核需要一个全局的任务管理器来管理这些任务控制块：

`Cell<T>`, `RefCell<T>`, and `OnceCell<T>`. Each provides a different way of providing safe interior mutability.

**`struct UPSafeCell<T>`**:

- 这是一个泛型结构体，它的内部数据是 `RefCell<T>` 类型，即持有 `T` 类型的数据并提供运行时可变性检查。

A wrapper type for a mutably borrowed value from a `RefCell<T>`

**内部可变性 (Interior Mutability)**: `RefCell` 是 Rust 中提供的一种 "内部可变性" 机制，允许在不可变对象内部改变数据。它打破了 Rust 的编译时借用规则，通过在运行时检查借用。

默认情况下，当 Trap 进入某个特权级之后，在 Trap 处理的过程中同特权级的中断都会被屏蔽。所以不会出现嵌套中断

出发了S特权级时钟中断时，重新设置计时器，调用`suspend_current_and_next