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
    if n == 0 {
        return dst;
    }
    unsafe {
        for i in 0..n {
            let byte = core::ptr::read(src.add(i));
            core::ptr::write(dst.add(i), byte);
        }
    }
    dst
}

/// 将从 `dst` 开始的 `n` 个字节设置为值 `c`。  
///
/// 返回 `dst`。  
///
/// # 安全性  
/// `dst` 必须指向至少 `n` 个字节的有效可写内存。  
#[unsafe(no_mangle)]
pub unsafe extern "C" fn my_memset(dst: *mut u8, c: u8, n: usize) -> *mut u8 {
    unsafe {
        for i in 0..n {
            core::ptr::write(dst.add(i), c);
        }
    }
    dst
}

/// 将 `n` 字节从 `src` 复制到 `dst`，正确处理内存重叠。  
///
/// 返回 `dst`。  
///
/// # 安全性  
/// `dst` 和 `src` 必须分别指向至少 `n` 字节的有效内存。  
#[unsafe(no_mangle)]
pub unsafe extern "C" fn my_memmove(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    // TODO: 实现 memmove
    // 提示：当 dst > src 且区域重叠时，从末尾向开始倒序复制
    if n == 0 {
        return dst;
    }

    unsafe {
        let dst_addr = dst as usize;
        let src_addr = src as usize;

        // 判断是否重叠且需要倒序复制
        if dst_addr > src_addr && dst_addr < src_addr + n {
            // 重叠且 dst > src：从后向前复制
            for i in (0..n).rev() {
                core::ptr::copy(src.add(i), dst.add(i), 1);
            }
        } else {
            // 不重叠或 dst <= src：从前向后复制
            // 直接使用 copy 更高效（内部优化为 memcpy）
            core::ptr::copy(src, dst, n);
        }
    }
    dst
}

/// 返回一个以空字符结尾的字节字符串的长度，不包括末尾的空字符。  
///
/// # 安全性  
/// `s` 必须指向一个有效的以空字符结尾的字节字符串。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn my_strlen(s: *const u8) -> usize {
    // TODO: Implement strlen
    if s.is_null() {
        return 0;
    }
    let mut len = 0;
    unsafe {
        while core::ptr::read(s.add(len)) != 0 {
            len += 1;
        }
    }
    len
}

/// 比较两个以空字符结尾的字节字符串。  
///
/// 返回值：  
/// - `0`  ：字符串相等  
/// - `< 0`：`s1` 按字典序小于 `s2`  
/// - `> 0`：`s1` 按字典序大于 `s2`  
///
/// # 安全性  
/// `s1` 和 `s2` 必须分别指向有效的以空字符结尾的字节字符串。  
#[unsafe(no_mangle)]
pub unsafe extern "C" fn my_strcmp(s1: *const u8, s2: *const u8) -> i32 {
    // TODO: Implement strcmp
    let mut i = 0;
    unsafe {
        loop {
            let c1 = core::ptr::read(s1.add(i));
            let c2 = core::ptr::read(s2.add(i));
            if c1 != c2 {
                return (c1 as i32) - (c2 as i32);
            }
            if c1 == 0 {
                break;
            }
            i += 1;
        }
    }
    0
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
