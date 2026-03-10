//! # 互斥共享状态
//!
//! 在本练习中，你将使用 `Arc<Mutex<T>>` 在多个线程之间安全地共享和修改数据。
//!
//! ## 概念
//! - `Mutex<T>` 互斥锁保护共享数据
//! - `Arc<T>` 原子引用计数允许跨线程共享
//! - `lock()` 获取锁并访问数据

use std::sync::{Arc, Mutex};
use std::thread;

/// 使用 `n_threads` 线程并发增加计数器。
/// 每个线程将计数器增加 `count_per_thread` 次。
/// 返回最终的计数器值。
///
/// 提示：使用 `Arc<Mutex<usize>>` 作为共享计数器。
pub fn concurrent_counter(n_threads: usize, count_per_thread: usize) -> usize {
    // TODO：创建初始值为 0 的 Arc<Mutex<usize>>
    // TODO：生成 n_threads 个线程
    // TODO：在每个线程中，lock() 并增加 count_per_thread 次
    // TODO：等待所有线程完成，并返回最终值
    let counter = Arc::new(Mutex::new(0));
    let mut handles = Vec::new();
    for _ in 0..n_threads {
        let counter_clone = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            for _ in 0..count_per_thread {
                let mut num = counter_clone.lock().unwrap();
                *num += 1;
            }
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
    let final_count = counter.lock().unwrap();
    *final_count
}
/// 使用多个线程并发地向共享向量添加元素。
/// 每个线程将自己的 id（0..n_threads）推入向量。
/// 返回排序后的向量。
///
/// 提示：使用 `Arc<Mutex<Vec<usize>>>`。
pub fn concurrent_collect(n_threads: usize) -> Vec<usize> {
    // 待办：创建 Arc<Mutex<Vec<usize>>>
    // 待办：每个线程推送它自己的 id
    // 待办：在所有线程加入后，排序结果并返回
    let shared_vec = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();

    for i in 0..n_threads {
        let vec_clone = Arc::clone(&shared_vec);
        let handle = thread::spawn(move || {
            let mut vec = vec_clone.lock().unwrap();
            vec.push(i);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
    let mut result = shared_vec.lock().unwrap();
    result.sort();
    result.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_single_thread() {
        assert_eq!(concurrent_counter(1, 100), 100);
    }

    #[test]
    fn test_counter_multi_thread() {
        assert_eq!(concurrent_counter(10, 100), 1000);
    }

    #[test]
    fn test_counter_zero() {
        assert_eq!(concurrent_counter(5, 0), 0);
    }

    #[test]
    fn test_collect() {
        let result = concurrent_collect(5);
        assert_eq!(result, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_collect_single() {
        assert_eq!(concurrent_collect(1), vec![0]);
    }
}
