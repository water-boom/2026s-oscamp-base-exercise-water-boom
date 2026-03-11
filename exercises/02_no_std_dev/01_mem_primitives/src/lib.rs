//! # no_std Memory Primitives

//! 在 `#![no_std]` 环境中，你没有标准库——只有 `core`。
//! 这些内存操作函数是操作系统内核中最基本的构建块。
//! 在裸机环境中，像 libc 中的 memcpy/memset 这样的函数必须由我们自己实现。

//! ## 任务
//!
//! 实现以下五个函数：
//! - 仅使用 `core` crate，不使用 `std`
//! - 不调用 `core::ptr::copy`、`core::ptr::copy_nonoverlapping` 等函数（自己编写循环）
//! - 正确处理边界情况（n=0、内存区域重叠等）
//! - 通过所有测试

// Force no_std in production; allow std in tests (cargo test framework requires it)
#![cfg_attr(not(test), no_std)]
#![allow(unused_variables)]

/// 将 `n` 字节从 `src` 复制到 `dst`。
///
/// - `dst` 和 `src` 不得重叠（对于重叠区域，请使用 `my_memmove`）
/// - 返回 `dst`
///
/// # 安全性
/// `dst` 和 `src` 必须分别指向至少 `n` 字节的有效内存。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn my_memcpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    // TODO: 实现 memcpy
    // 提示：逐个从 src 读取字节并写入 dst
    todo!()
}

/// Set `n` bytes starting at `dst` to the value `c`.
///
/// Returns `dst`.
///
/// # Safety
/// `dst` must point to at least `n` bytes of valid writable memory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn my_memset(dst: *mut u8, c: u8, n: usize) -> *mut u8 {
    // TODO: Implement memset
    todo!()
}

/// Copy `n` bytes from `src` to `dst`, correctly handling overlapping memory.
///
/// Returns `dst`.
///
/// # Safety
/// `dst` and `src` must each point to at least `n` bytes of valid memory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn my_memmove(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    // TODO: Implement memmove
    // Hint: when dst > src and regions overlap, copy backwards (from end to start)
    todo!()
}

/// Return the length of a null-terminated byte string, excluding the trailing null.
///
/// # Safety
/// `s` must point to a valid null-terminated byte string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn my_strlen(s: *const u8) -> usize {
    // TODO: Implement strlen
    todo!()
}

/// Compare two null-terminated byte strings.
///
/// Returns:
/// - `0`  : strings are equal
/// - `< 0`: `s1` is lexicographically less than `s2`
/// - `> 0`: `s1` is lexicographically greater than `s2`
///
/// # Safety
/// `s1` and `s2` must each point to a valid null-terminated byte string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn my_strcmp(s1: *const u8, s2: *const u8) -> i32 {
    // TODO: Implement strcmp
    todo!()
}

// ============================================================
// Tests (std is available under #[cfg(test)])
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memcpy_basic() {
        let src = [1u8, 2, 3, 4, 5];
        let mut dst = [0u8; 5];
        unsafe { my_memcpy(dst.as_mut_ptr(), src.as_ptr(), 5) };
        assert_eq!(dst, src);
    }

    #[test]
    fn test_memcpy_zero_len() {
        let src = [0xFFu8; 4];
        let mut dst = [0u8; 4];
        unsafe { my_memcpy(dst.as_mut_ptr(), src.as_ptr(), 0) };
        assert_eq!(dst, [0u8; 4]);
    }

    #[test]
    fn test_memset_basic() {
        let mut buf = [0u8; 8];
        unsafe { my_memset(buf.as_mut_ptr(), 0xAB, 8) };
        assert!(buf.iter().all(|&b| b == 0xAB));
    }

    #[test]
    fn test_memset_partial() {
        let mut buf = [0u8; 8];
        unsafe { my_memset(buf.as_mut_ptr(), 0xFF, 4) };
        assert_eq!(&buf[..4], &[0xFF; 4]);
        assert_eq!(&buf[4..], &[0x00; 4]);
    }

    #[test]
    fn test_memmove_no_overlap() {
        let src = [1u8, 2, 3, 4];
        let mut dst = [0u8; 4];
        unsafe { my_memmove(dst.as_mut_ptr(), src.as_ptr(), 4) };
        assert_eq!(dst, src);
    }

    #[test]
    fn test_memmove_overlap_forward() {
        // Copy buf[0..4] to buf[1..5], shifting right by 1
        let mut buf = [1u8, 2, 3, 4, 5];
        unsafe { my_memmove(buf.as_mut_ptr().add(1), buf.as_ptr(), 4) };
        assert_eq!(buf, [1, 1, 2, 3, 4]);
    }

    #[test]
    fn test_strlen_basic() {
        let s = b"hello\0";
        assert_eq!(unsafe { my_strlen(s.as_ptr()) }, 5);
    }

    #[test]
    fn test_strlen_empty() {
        let s = b"\0";
        assert_eq!(unsafe { my_strlen(s.as_ptr()) }, 0);
    }

    #[test]
    fn test_strcmp_equal() {
        let a = b"hello\0";
        let b = b"hello\0";
        assert_eq!(unsafe { my_strcmp(a.as_ptr(), b.as_ptr()) }, 0);
    }

    #[test]
    fn test_strcmp_less() {
        let a = b"abc\0";
        let b = b"abd\0";
        assert!(unsafe { my_strcmp(a.as_ptr(), b.as_ptr()) } < 0);
    }

    #[test]
    fn test_strcmp_greater() {
        let a = b"abd\0";
        let b = b"abc\0";
        assert!(unsafe { my_strcmp(a.as_ptr(), b.as_ptr()) } > 0);
    }
}
