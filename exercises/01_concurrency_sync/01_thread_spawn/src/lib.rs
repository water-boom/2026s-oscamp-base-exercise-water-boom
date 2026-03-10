//! # Thread Creation
//!
//! In this exercise, you will learn how to create threads and pass data between threads.
//！
//! ## 概念  
//! - `std::thread::spawn` 创建一个新线程  
//! - `move` 闭包捕获变量所有权  
//! - `JoinHandle::join()` 等待线程完成并获取返回值  
//！
//! ## 高级线程操作  
//! - **线程休眠**: `thread::sleep` 暂停当前线程。  
//! - **线程本地存储**: `thread_local!` 宏定义每个线程唯一的静态变量。  
//! - **线程命名**: `Builder::name` 为调试分配名称。  
//! - **线程优先级**: 通过 `thread::Builder` 设置（依赖于平台）。  
//! - **线程池**: 使用类似 `rayon` 的库管理线程重用。  
//! - **线程通信**: 使用 `std::sync::mpsc`（多生产者单消费者）或第三方库（如 `crossbeam`）。  
//! - **共享状态**: 使用 `Arc<Mutex<T>>` 或 `Arc<RwLock<T>>` 安全地在线程间共享可变数据。  
//! - **同步原语**: `Barrier` 同步多个线程，`Condvar` 实现条件变量。  
//! - **线程挂起/唤醒**: `thread::park` 阻塞线程，`unpark` 唤醒线程，用于自定义调度。  
//! - **获取当前线程句柄**: `thread::current()`。  
//! - **作用域线程**：`crossbeam::scope` 或标准库 `thread::scope`（Rust 1.63+）允许线程在不使用 `move` 的情况下借用栈数据。
//！
//! Rust 通过所有权系统以及 `Send` 和 `Sync` 特性静态地防止数据竞争。
//! 实现 `Send` 的类型可以在线程边界之间传递。
//! 实现 `Sync` 的类型可以被多个线程同时引用。
//! 大多数 Rust 标准类型都是 `Send   Sync`；例外包括 `Rc<T>`（非原子引用计数）和裸指针。
//！
//! ## 练习结构
//! 1. **基础练习** (`double_in_thread`, `parallel_sum`) – 介绍基本的线程创建。
//! 2. **高级练习** (`named_sleeper`, `increment_thread_local`, `scoped_slice_sum`, `handle_panic`) – 探索更多线程操作。
//! 每个函数都包含一个 `TODO` 注释，指出你需要编写代码的位置。
//! 运行 `cargo test` 来检查你的实现。

#[allow(unused_imports)]
use std::cell::RefCell;
use std::ops::ShrAssign;
#[allow(unused_imports)]
use std::thread;
#[allow(unused_imports)]
use std::time::Duration;

// ============================================================================
// Example Code: Advanced Thread Patterns
// ============================================================================
// The following examples illustrate additional thread‑related concepts that are
// useful in real‑world Rust concurrent programming.

/// 示例：处理线程崩溃。  
///
/// `join()` 返回一个 `Result`。如果线程崩溃，`Result` 是一个 `Err`。  
/// 这展示了如何捕获并处理来自已启动线程的崩溃。  

/// ```rust
/// use std::thread;
///
/// fn panic_handling_example() {
///     let handle = thread::spawn(|| {
///         // Simulate a panic
///         panic!("Thread panicked!");
///     });
///
///     match handle.join() {
///         Ok(_) => println!("Thread completed successfully."),
///         Err(e) => println!("Thread panicked: {:?}", e),
///     }
/// }
/// ```

/// 相反，下面的练习为了简单起见使用 `unwrap()`，假设线程永远不会恐慌。

/// 示例：命名线程和自定义堆栈大小。
///
/// 使用 `thread::Builder` 可以为线程分配一个名称（有助于调试）并设置其堆栈大小。
/// ```rust
/// use std::thread;
///
/// fn named_thread_example() {
///     let builder = thread::Builder::new()
///         .name("my-worker".into())
///         .stack_size(32 * 1024); // 32 KiB
///
///     let handle = builder.spawn(|| {
///         println!("Hello from thread: {:?}", thread::current().name());
///         42
///     }).unwrap();
///
///     let result = handle.join().unwrap();
///     println!("Thread returned: {}", result);
/// }
/// ```

///示例：Scoped 线程（Rust 1.63）。
///Scoped 线程允许借用栈数据而不转移所有权。
///这些线程保证在作用域结束之前完成，因此引用保持有效。

/// ```rust
/// use std::thread;
///
/// fn scoped_thread_example() {
///     let a = vec![1, 2, 3];
///     let b = vec![4, 5, 6];
///
///     let (sum_a, sum_b) = thread::scope(|s| {
///         let h1 = s.spawn(|| a.iter().sum::<i32>());
///         let h2 = s.spawn(|| b.iter().sum::<i32>());
///         (h1.join().unwrap(), h2.join().unwrap())
///     });
///
///     // `a` and `b` are still accessible here.
///     println!("sum_a = {}, sum_b = {}", sum_a, sum_b);
/// }
/// ```

/// Example: Thread‑local storage.
///
/// Each thread gets its own independent copy of a `thread_local!` variable.
///
/// ```rust
/// use std::cell::RefCell;
/// use std::thread;
///
/// thread_local! {
///     static THREAD_ID: RefCell<usize> = RefCell::new(0);
/// }
///
/// fn thread_local_example() {
///     THREAD_ID.with(|id| {
///         *id.borrow_mut() = 1;
///     });
///
///     let handle = thread::spawn(|| {
///         THREAD_ID.with(|id| {
///             *id.borrow_mut() = 2;
///         });
///         THREAD_ID.with(|id| println!("Thread local value: {}", *id.borrow()));
///     });
///
///     handle.join().unwrap();
///
///     THREAD_ID.with(|id| println!("Main thread value: {}", *id.borrow()));
/// }
/// ```

// ============================================================================
// Exercise Functions
// ============================================================================

/// Multiply each element of a vector by 2 in a new thread, returning the result vector.
///
/// Hint: Use `thread::spawn` and `move` closure.
#[allow(unused_variables)]
pub fn double_in_thread(numbers: Vec<i32>) -> Vec<i32> {
    // TODO: Create a new thread to multiply each element of numbers by 2
    // Use thread::spawn and move closure
    // Use join().unwrap() to get result
    std::thread::spawn(move || numbers.into_iter().map(|x| x * 2).collect())
        .join()
        .unwrap()
}

/// Sum two vectors in parallel, returning a tuple of two sums.
///
/// Hint: Create two threads for each vector.
#[allow(unused_variables)]
pub fn parallel_sum(a: Vec<i32>, b: Vec<i32>) -> (i32, i32) {
    // TODO: Create two threads to sum a and b respectively
    // Join both threads to get results
    let handle_a = std::thread::spawn(move || a.into_iter().sum());
    let handle_b = std::thread::spawn(move || b.into_iter().sum());

    (handle_a.join().unwrap(), handle_b.join().unwrap())
}

// ============================================================================
// Advanced Exercise Functions
// ============================================================================

/// Create a named thread that sleeps for the given milliseconds and then returns the input value.
///
/// The thread should be named `"sleeper"`. Use `thread::Builder` to set the name.
/// Inside the thread, call `thread::sleep(Duration::from_millis(ms))` before returning `value`.
///
/// Hint: `thread::sleep` causes the current thread to block; it does not affect other threads.
#[allow(unused_variables)]
pub fn named_sleeper(value: i32, ms: u64) -> i32 {
    // TODO: Create a thread builder with name "sleeper"
    // TODO: Spawn a thread that sleeps for `ms` milliseconds and returns `value`
    // TODO: Join the thread and return the value
    let builder = std::thread::Builder::new().name("sleeper".into());
    let handle = builder
        .spawn(move || {
            std::thread::sleep(Duration::from_millis(ms));
            value
        })
        .unwrap();
    handle.join().unwrap()
}

thread_local! {
    static THREAD_COUNT: RefCell<usize> = RefCell::new(0);
}

/// Use thread‑local storage to count how many times each thread calls `increment`.
///
/// Define a `thread_local!` static `THREAD_COUNT` of type `RefCell<usize>` initialized to 0.
/// Each call to `increment` should increase the thread‑local count by 1 and return the new value.
///
/// Hint: Use `THREAD_COUNT.with(|cell| { ... })` to access the thread‑local variable.
pub fn increment_thread_local() -> usize {
    // TODO: Use THREAD_COUNT.with to increment and return the new count
    THREAD_COUNT.with(|cell| {
        let mut count = cell.borrow_mut();
        *count += 1;
        *count
    })
}

/// Spawn two threads using a **scoped thread** to compute the sum of two slices without moving ownership.
///
/// Use `thread::scope` to allow threads to borrow the slices `&[i32]`.
/// Each thread should compute the sum of its slice, and the function returns `(sum_a, sum_b)`.
///
/// Hint: The slices are references, so you cannot move them into the closure.
/// `thread::scope` guarantees that all spawned threads finish before the scope ends,
/// making the borrow safe.
#[allow(unused_variables)]
pub fn scoped_slice_sum(a: &[i32], b: &[i32]) -> (i32, i32) {
    // TODO: Use thread::scope to spawn two threads
    // TODO: Each thread sums its slice
    // TODO: Wait for both threads and return the results
    let (sum_a, sum_b) = thread::scope(|s| {
        let h1 = s.spawn(|| a.iter().sum::<i32>());
        let h2 = s.spawn(|| b.iter().sum::<i32>());
        (h1.join().unwrap(), h2.join().unwrap())
    });
    (sum_a, sum_b)
}

/// Handle a possible panic in a spawned thread.
///
/// Spawn a thread that may panic: if `should_panic` is `true`, the thread calls `panic!("oops")`;
/// otherwise it returns `value`.
/// The function should return `Ok(value)` if the thread completed successfully,
/// or `Err(())` if the thread panicked.
///
/// Hint: `join()` returns `Result<Result<i32, Box<dyn Any + Send>>, _>`.
/// You'll need to match the outer `Result` (thread panic) and the inner `Result` (if the thread returns a `Result`).
/// In this exercise, the inner type is just `i32`, not a `Result`.
#[allow(unused_variables)]
pub fn handle_panic(value: i32, should_panic: bool) -> Result<i32, ()> {
    // TODO: Spawn a thread that either panics or returns value
    // TODO: Join and map the result appropriately
    let handle = thread::spawn(move || {
        if should_panic {
            panic!("oops");
        } else {
            value
        }
    });
    match handle.join() {
        Ok(result) => Ok(result),
        Err(_) => Err(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_double_basic() {
        let nums = vec![1, 2, 3, 4, 5];
        assert_eq!(double_in_thread(nums), vec![2, 4, 6, 8, 10]);
    }

    #[test]
    fn test_double_empty() {
        assert_eq!(double_in_thread(vec![]), vec![]);
    }

    #[test]
    fn test_double_negative() {
        assert_eq!(double_in_thread(vec![-1, 0, 1]), vec![-2, 0, 2]);
    }

    #[test]
    fn test_parallel_sum() {
        let a = vec![1, 2, 3];
        let b = vec![10, 20, 30];
        assert_eq!(parallel_sum(a, b), (6, 60));
    }

    #[test]
    fn test_parallel_sum_empty() {
        assert_eq!(parallel_sum(vec![], vec![]), (0, 0));
    }

    // Advanced exercise tests
    #[test]
    fn test_named_sleeper() {
        // The thread should sleep a short time; we just verify it returns the correct value.
        let result = named_sleeper(42, 10); // sleep 10 ms
        assert_eq!(result, 42);
    }

    #[test]
    fn test_thread_local() {
        // Each thread has its own counter, so spawning two threads and calling increment
        // in each should give each thread its own sequence.
        use std::sync::Arc;
        use std::sync::Mutex;

        let counters = Arc::new(Mutex::new(Vec::new()));
        let mut handles = Vec::new();
        for _ in 0..2 {
            let counters = Arc::clone(&counters);
            handles.push(thread::spawn(move || {
                let v1 = increment_thread_local();
                let v2 = increment_thread_local();
                counters.lock().unwrap().push((v1, v2));
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        let results = counters.lock().unwrap();
        // Each thread should have counted (1, 2) independently.
        assert_eq!(results.len(), 2);
        assert!(results.contains(&(1, 2)));
    }

    #[test]
    fn test_scoped_slice_sum() {
        let a = [1, 2, 3];
        let b = [10, 20, 30];
        let (sum_a, sum_b) = scoped_slice_sum(&a, &b);
        assert_eq!(sum_a, 6);
        assert_eq!(sum_b, 60);
        // Ensure slices are still accessible (they are borrowed, not moved).
        assert_eq!(a.len(), 3);
        assert_eq!(b.len(), 3);
    }

    #[test]
    fn test_handle_panic_ok() {
        let result = handle_panic(100, false);
        assert_eq!(result, Ok(100));
    }

    #[test]
    fn test_handle_panic_error() {
        let result = handle_panic(100, true);
        assert_eq!(result, Err(()));
    }
}
