// ! # 文件描述符表
// !
// ! 实现一个简单的文件描述符（fd）表——用于管理操作系统内核中打开文件的核心数据结构。
// !
// ! ## 背景
// !
// ! 在 Linux 内核中，每个进程都有一个 fd 表，用于将整数 fd 映射到内核文件对象。
// ! 用户程序通过 fd 执行读/写/关闭操作，内核通过 fd 表查找对应的文件对象。
// !
// ! ```text
// ! fd 表：
// !   0 -> 标准输入
// !   1 -> 标准输出
// !   2 -> 标准错误
// !   3 -> 文件("/etc/passwd")
// !   4 -> （空）
// !   5 -> 套接字(...)
// ! ```
// !
// ! ## 任务
// !
// ! 在 `FdTable` 上实现以下方法：
// !
// ! - `new()` — 创建一个空的 fd 表
// ! - `alloc(file)` -> `usize` — 分配一个新的 fd，返回 fd 编号
// !   - 优先重用最小的已关闭 fd 编号
// !   - 如果没有空闲槽，则扩展表
// ! - `get(fd)` -> `Option<Arc<dyn File>>` — 获取 fd 对应的文件对象
// ! - `close(fd)` -> `bool` — 关闭一个 fd，返回是否成功（如果 fd 不存在则返回 false）
// ! - `count()` -> `usize` — 返回当前已分配的文件描述符数量（不包括已关闭的）
// !
// ! ## 关键概念
// !
// ! - 特性对象：`Arc<dyn File>`
// ! - 使用 `Vec<Option<T>>` 作为稀疏表
// ! - 文件描述符编号重用策略（寻找最小的空闲槽位）
// ! - `Arc` 引用计数和资源释放

use std::sync::Arc;

/// 文件抽象特性——内核中的所有“文件”（常规文件、管道、套接字）都实现了这一点
pub trait File: Send + Sync {
    fn read(&self, buf: &mut [u8]) -> isize;
    fn write(&self, buf: &[u8]) -> isize;
}

/// File descriptor table
pub struct FdTable {
    // TODO: 设计内部结构
    // 提示：使用 Vec<Option<Arc<dyn File>>>
    //       索引是文件描述符（fd）编号，None 表示 fd 已关闭或未分配
    table: Vec<Option<Arc<dyn File>>>,
}

impl FdTable {
    /// Create an empty fd table
    pub fn new() -> Self {
        // TODO
        Self { table: Vec::new() }
    }

    /// 分配一个新的文件描述符，返回文件描述符编号。  
    ///
    /// 优先复用最小的已关闭文件描述符编号；如果没有空闲槽，则追加到末尾。
    pub fn alloc(&mut self, file: Arc<dyn File>) -> usize {
        // TODO
        for (i, slot) in self.table.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(file);
                return i;
            }
        }
        self.table.push(Some(file));
        self.table.len() - 1
    }

    /// Get the file object for an fd. Returns None if the fd doesn't exist or is closed.
    pub fn get(&self, fd: usize) -> Option<Arc<dyn File>> {
        // TODO
        self.table.get(fd)?.clone()
    }

    /// Close an fd. Returns true on success, false if the fd doesn't exist or is already closed.
    pub fn close(&mut self, fd: usize) -> bool {
        // TODO
        for slot in self.table.get_mut(fd) {
            if slot.is_some() {
                *slot = None;
                return true;
            }
        }
        false
    }

    /// Return the number of currently allocated fds (excluding closed ones)
    pub fn count(&self) -> usize {
        // TODO
        self.table.iter().filter(|slot| slot.is_some()).count()
    }
}

impl Default for FdTable {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// Test File implementation
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct MockFile {
        id: usize,
        write_log: Mutex<Vec<Vec<u8>>>,
    }

    impl MockFile {
        fn new(id: usize) -> Arc<Self> {
            Arc::new(Self {
                id,
                write_log: Mutex::new(vec![]),
            })
        }
    }

    impl File for MockFile {
        fn read(&self, buf: &mut [u8]) -> isize {
            buf[0] = self.id as u8;
            1
        }
        fn write(&self, buf: &[u8]) -> isize {
            self.write_log.lock().unwrap().push(buf.to_vec());
            buf.len() as isize
        }
    }

    #[test]
    fn test_alloc_basic() {
        let mut table = FdTable::new();
        let fd = table.alloc(MockFile::new(0));
        assert_eq!(fd, 0, "first fd should be 0");
        let fd2 = table.alloc(MockFile::new(1));
        assert_eq!(fd2, 1, "second fd should be 1");
    }

    #[test]
    fn test_get() {
        let mut table = FdTable::new();
        let file = MockFile::new(42);
        let fd = table.alloc(file);
        let got = table.get(fd);
        assert!(got.is_some(), "get should return Some");
        let mut buf = [0u8; 1];
        got.unwrap().read(&mut buf);
        assert_eq!(buf[0], 42);
    }

    #[test]
    fn test_get_invalid() {
        let table = FdTable::new();
        assert!(table.get(0).is_none());
        assert!(table.get(999).is_none());
    }

    #[test]
    fn test_close_and_reuse() {
        let mut table = FdTable::new();
        let fd0 = table.alloc(MockFile::new(0)); // fd=0
        let fd1 = table.alloc(MockFile::new(1)); // fd=1
        let fd2 = table.alloc(MockFile::new(2)); // fd=2

        assert!(table.close(fd1), "closing fd=1 should succeed");
        assert!(
            table.get(fd1).is_none(),
            "get should return None after close"
        );

        // Next allocation should reuse fd=1 (smallest free)
        let fd_new = table.alloc(MockFile::new(99));
        assert_eq!(fd_new, fd1, "should reuse the smallest closed fd");

        let _ = (fd0, fd2);
    }

    #[test]
    fn test_close_invalid() {
        let mut table = FdTable::new();
        assert!(
            !table.close(0),
            "closing non-existent fd should return false"
        );
    }

    #[test]
    fn test_count() {
        let mut table = FdTable::new();
        assert_eq!(table.count(), 0);
        let fd0 = table.alloc(MockFile::new(0));
        let fd1 = table.alloc(MockFile::new(1));
        assert_eq!(table.count(), 2);
        table.close(fd0);
        assert_eq!(table.count(), 1);
        table.close(fd1);
        assert_eq!(table.count(), 0);
    }

    #[test]
    fn test_write_through_fd() {
        let mut table = FdTable::new();
        let file = MockFile::new(0);
        let fd = table.alloc(file);
        let f = table.get(fd).unwrap();
        let n = f.write(b"hello");
        assert_eq!(n, 5);
    }
}
