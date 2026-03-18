//! # 自由链表分配器
//!
//! 基于 bump 分配器，实施一个支持内存回收的自由链表分配器。
//!
//! ## 工作原理
//!
//! 自由链表分配器使用一个链表来跟踪所有已释放的内存块。
//! 在分配时，它首先在链表中搜索一个合适的块（首次适配策略）；
//! 如果未找到，则回退到从未使用的区域分配。
//! 在释放时，该块被插入到链表的头部。
//!
//! ```text
//! free_list -> [block A: 64B] -> [block B: 128B] -> [block C: 32B] -> null
//! ```
//!
//! 每个空闲块在其头部存储一个 `FreeBlock` 结构（包含块大小和下一个指针）。
//!
//! ## 任务
//!
//! 实现 `FreeListAllocator` 的 `alloc` 和 `dealloc` 方法：
//!
//! ### alloc
//! 1. 遍历 free_list，找到第一个满足 `size >= layout.size()` 且对齐要求的块（首次适配）
//! 2. 如果找到，从链表中移除并返回
//! 3. 如果未找到，从 `bump` 区域分配（与 bump 分配器相同）
//!
//! ### dealloc
//! 1. 将 `FreeBlock` 头信息写入已释放的块  
//! 2. 将其插入到 free_list 的头部  
//!  
//! ## 关键概念  
//!  
//! - 内嵌链表  
//! - `*mut T` 读/写：`ptr.write(val)` / `ptr.read()`  
//! - 内存对齐检查  

#![cfg_attr(not(test), no_std)]

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

/// 空闲块头，存储在每个空闲内存块的开头
struct FreeBlock {
    size: usize,
    next: *mut FreeBlock,
}

pub struct FreeListAllocator {
    heap_start: usize,
    heap_end: usize,
    /// Bump pointer: unallocated region starts here
    bump_next: core::sync::atomic::AtomicUsize,
    /// Free list head (protected by Mutex in test, UnsafeCell otherwise)
    #[cfg(test)]
    free_list: std::sync::Mutex<*mut FreeBlock>,
    #[cfg(not(test))]
    free_list: core::cell::UnsafeCell<*mut FreeBlock>,
}

#[cfg(test)]
unsafe impl Send for FreeListAllocator {}
#[cfg(test)]
unsafe impl Sync for FreeListAllocator {}
#[cfg(not(test))]
unsafe impl Send for FreeListAllocator {}
#[cfg(not(test))]
unsafe impl Sync for FreeListAllocator {}

impl FreeListAllocator {
    /// # 安全性  
    /// `heap_start..heap_end` 必须是一个有效的可读写内存区域。
    pub unsafe fn new(heap_start: usize, heap_end: usize) -> Self {
        Self {
            heap_start,
            heap_end,
            bump_next: core::sync::atomic::AtomicUsize::new(heap_start),
            #[cfg(test)]
            free_list: std::sync::Mutex::new(null_mut()),
            #[cfg(not(test))]
            free_list: core::cell::UnsafeCell::new(null_mut()),
        }
    }

    #[cfg(test)]
    fn free_list_head(&self) -> *mut FreeBlock {
        *self.free_list.lock().unwrap()
    }

    #[cfg(test)]
    fn set_free_list_head(&self, head: *mut FreeBlock) {
        *self.free_list.lock().unwrap() = head;
    }

    #[cfg(not(test))]
    fn free_list_head(&self) -> *mut FreeBlock {
        unsafe { *self.free_list.get() }
    }

    #[cfg(not(test))]
    fn set_free_list_head(&self, head: *mut FreeBlock) {
        unsafe { *self.free_list.get() = head }
    }
}

unsafe impl GlobalAlloc for FreeListAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Ensure block is at least large enough to hold a FreeBlock header (for future dealloc)
        let size = layout.size().max(core::mem::size_of::<FreeBlock>());
        let align = layout.align().max(core::mem::align_of::<FreeBlock>());

        // TODO: 第1步 — 遍历 free_list，找到合适的块（首次适配）
        //
        // 提示：
        // - 使用 prev_ptr 和 curr 遍历列表
        // - 检查 curr 地址是否满足对齐，并且 (*curr).size >= size
        // - 如果找到，从列表中移除它（更新 prev 的 next 或 free_list 的头部）
        // - 返回 curr 作为 *mut u8
        let mut curr = self.free_list_head();
        let mut prev_ptr: *mut FreeBlock = null_mut();
        while !curr.is_null() {
            let curr_addr = curr as usize;
            if curr_addr % align == 0 && (*curr).size >= size {
                if prev_ptr.is_null() {
                    self.set_free_list_head((*curr).next);
                } else {
                    (*prev_ptr).next = (*curr).next;
                }
                return curr as *mut u8;
            }
            prev_ptr = curr;
            curr = (*curr).next;
        }
        // TODO: 第2步 — free_list中没有合适的块，从bump区域分配
        //
        // 逻辑与02_bump_allocator的alloc相同
        let mut bump = self.bump_next.load(core::sync::atomic::Ordering::Relaxed);
        loop {
            let aligned_bump = (bump + align - 1) & !(align - 1);
            let next_bump = aligned_bump.checked_add(size).unwrap_or(self.heap_end);
            if next_bump > self.heap_end {
                return null_mut(); // Out of memory
            }
            match self.bump_next.compare_exchange_weak(
                bump,
                next_bump,
                core::sync::atomic::Ordering::SeqCst,
                core::sync::atomic::Ordering::Relaxed,
            ) {
                Ok(_) => return aligned_bump as *mut u8,
                Err(current) => bump = current,
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size().max(core::mem::size_of::<FreeBlock>());

        // TODO: 将释放的块插入到 free_list 的头部
        //
        // 步骤：
        // 1. 将 ptr 转换为 *mut FreeBlock
        // 2. 写入 FreeBlock { size, next: 当前列表头部 }
        // 3. 更新 free_list 的头部为 ptr
        let free_block = ptr as *mut FreeBlock;
        unsafe {
            free_block.write(FreeBlock {
                size,
                next: self.free_list_head(),
            });
            self.set_free_list_head(free_block);
        }
    }
}

// ============================================================
// Tests
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;

    const HEAP_SIZE: usize = 4096;

    fn make_allocator() -> (FreeListAllocator, Vec<u8>) {
        let mut heap = vec![0u8; HEAP_SIZE];
        let start = heap.as_mut_ptr() as usize;
        let alloc = unsafe { FreeListAllocator::new(start, start + HEAP_SIZE) };
        (alloc, heap)
    }

    #[test]
    fn test_alloc_basic() {
        let (alloc, _heap) = make_allocator();
        let layout = Layout::from_size_align(32, 8).unwrap();
        let ptr = unsafe { alloc.alloc(layout) };
        assert!(!ptr.is_null());
    }

    #[test]
    fn test_alloc_alignment() {
        let (alloc, _heap) = make_allocator();
        for align in [1, 2, 4, 8, 16] {
            let layout = Layout::from_size_align(8, align).unwrap();
            let ptr = unsafe { alloc.alloc(layout) };
            assert!(!ptr.is_null());
            assert_eq!(ptr as usize % align, 0, "align={align}");
        }
    }

    #[test]
    fn test_dealloc_and_reuse() {
        let (alloc, _heap) = make_allocator();
        let layout = Layout::from_size_align(64, 8).unwrap();

        let p1 = unsafe { alloc.alloc(layout) };
        assert!(!p1.is_null());

        // After freeing, the next allocation should reuse the same block
        unsafe { alloc.dealloc(p1, layout) };
        let p2 = unsafe { alloc.alloc(layout) };
        assert!(!p2.is_null());
        assert_eq!(p1, p2, "should reuse the freed block");
    }

    #[test]
    fn test_multiple_alloc_dealloc() {
        let (alloc, _heap) = make_allocator();
        let layout = Layout::from_size_align(128, 8).unwrap();

        let p1 = unsafe { alloc.alloc(layout) };
        let p2 = unsafe { alloc.alloc(layout) };
        let p3 = unsafe { alloc.alloc(layout) };
        assert!(!p1.is_null() && !p2.is_null() && !p3.is_null());

        unsafe { alloc.dealloc(p2, layout) };
        unsafe { alloc.dealloc(p1, layout) };

        let q1 = unsafe { alloc.alloc(layout) };
        let q2 = unsafe { alloc.alloc(layout) };
        assert!(!q1.is_null() && !q2.is_null());
    }

    #[test]
    fn test_oom() {
        let (alloc, _heap) = make_allocator();
        let layout = Layout::from_size_align(HEAP_SIZE + 1, 1).unwrap();
        let ptr = unsafe { alloc.alloc(layout) };
        assert!(ptr.is_null(), "should return null when exceeding heap");
    }
}
