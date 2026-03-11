//! # 进程与管道
//!
//! 在本练习中，你将学习如何创建子进程并通过管道进行通信。
//!
//! ## 概念
//! - `std::process::Command` 用于创建子进程（对应 `fork()` 和 `execve()` 系统调用）
//! - `Stdio::piped()` 用于设置管道（对应 `pipe()` 和 `dup2()` 系统调用）
//! - 通过 stdin/stdout 与子进程通信
//! - 获取子进程退出状态（对应 `waitpid()` 系统调用）

//! ## 操作系统概念映射
//! 本练习展示了用户空间对底层操作系统原语的抽象：
//! - **进程创建**：Rust 的 `Command::new()` 在内部调用 `fork()` 创建子进程，
//!   然后调用 `execve()`（或等价方法）将子进程的内存镜像替换为目标程序。
//! - **进程间通信（IPC）**：管道是由内核管理的缓冲区，允许相关进程之间进行单向数据流。`pipe()` 系统调用创建一个管道，返回两个文件描述符（读端、写端）。`dup2()` 复制一个文件描述符，从而实现标准输入/输出的重定向。
//! - **资源管理**：文件描述符（包括管道端）在其 Rust `Stdio` 对象被丢弃时会自动关闭，防止资源泄漏。
//!
//! ## 练习结构
//! 1. **基本命令执行**（`run_command`）——启动子进程并捕获其 stdout。
//! 2. **双向管道通信**（`pipe_through_cat`）——向子进程（`cat`）发送数据并读取其输出。
//! 3. **获取退出码**（`get_exit_code`）——获取子进程的终止状态。
//! 4. **高级：错误处理版本**（`run_command_with_result`）——学习正确的错误传播方法。
//! 5. **高级：复杂双向通信**（`pipe_through_grep`）——与读取多行并生成过滤输出的过滤程序进行交互。
//!
//! 每个函数都包含一个 `TODO` 注释，指示你需要编写代码的位置。
//! 运行 `cargo test` 来检查你的实现。

use std::io::{self, Read, Write};
use std::process::{Command, Stdio};

/// 执行给定的 shell 命令并返回其标准输出。
///
/// 例如：`run_command("echo", &["hello"])` 应该返回 `"hello\n"`
///
/// # 底层系统调用
/// - `Command::new(program)` → `fork()` `execve()` 系列
/// - `Stdio::piped()` → `pipe()` `dup2()`（为 stdout 设置管道）
/// - `.output()` → `waitpid()`（等待子进程结束）
///
/// # 实现步骤
/// 1. 使用给定的程序和参数创建一个 `Command`。
/// 2. 设置 `.stdout(Stdio::piped())` 以捕获子进程的 stdout。
/// 3. 调用 `.output()` 来执行子进程并获取其 `Output`。
/// 4. 将 `stdout` 字段（一个 `Vec<u8>`）转换为 `String`。

pub fn run_command(program: &str, args: &[&str]) -> String {
    // TODO：使用 Command::new 创建进程
    // TODO：将 stdout 设置为 Stdio::piped()
    // TODO：使用 .output() 执行并获取输出
    // TODO：将 stdout 转换为字符串并返回
    let output = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .output()
        .expect("Failed to execute command");
    //TODO：待查：from和from_utf8的区别
    String::from_utf8(output.stdout).expect("Failed to convert output to String")
}

/// 通过管道向子进程（cat）的标准输入写入数据，并读取其标准输出输出。
///
/// 这演示了父子进程之间的双向管道通信。
///
/// # 底层系统调用
/// - `Command::new("cat")` → `fork()`   `execve("cat")`
/// - `Stdio::piped()`（两次）→ `pipe()` 创建两个管道（stdin 和 stdout）   `dup2()` 重定向它们
/// - `ChildStdin::write_all()` → 向管道的写端 `write()`
/// - `drop(stdin)` → 在写端 `close()`，向子进程发送 EOF
/// - `ChildStdout::read_to_string()` → 从管道的读端 `read()`

///
/// # 所有权与资源管理
/// Rust 的所有权系统确保管道在正确的时间关闭：
/// 1. “ChildStdin”账户由父方拥有;写入它会将数据传输给子节点。
/// 2. 写入后，我们明确“drop（stdin）”（或让它离开作用域）以关闭写入端。
/// 3. 闭合写端信号给 EOF 到“cat”，处理完所有输入后退出。
/// 4. 然后将“ChildStdout”句柄读取至完整;丢掉它会关闭读端。
///
/// 如果不去掉“stdin”，孩子会等很久才能有更多输入（管道永远不会关闭）。
///
/// # 实施步骤
/// 1. 为“cat”创建一个命令，包含“.stdin（Stdio：:p iped（）”）“和”.stdout（Stdio：:p iped（））“。
/// 2. '.spawn（）' 命令，用于获取带有“stdin”和“stdout”句柄的“Child”。
/// 3. 将“输入”字节写入子节点的 stdin （'child.stdin.take（）.unwrap（）.write_all（...）`).
/// 4. 放下标准段的柄（明确“丢”或让它脱离作用域）以关闭管道。
/// 5. 读取子进程的 stdout（`child.stdout.take().unwrap().read_to_string(...)`）。
/// 6. 使用 `.wait()` 等待子进程退出（或依赖 drop‑wait）。
pub fn pipe_through_cat(input: &str) -> String {
    // TODO：创建“cat”命令，将标准输入和标准输出设置为管道
    // TODO：生成进程
    // TODO：将输入写入子进程的标准输入
    // TODO：放下标准输入以关闭管道（否则cat不会退出）
    // TODO：从子进程的标准输出读取输出
    let Builder = Command::new("cat")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut child = Builder;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();

    drop(child.stdin.take());
    let mut output = String::new();
    child
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut output)
        .unwrap();
    output
}

/// 获取子进程退出代码。  
/// 执行命令 `sh -c {command}` 并返回退出代码。  
///
/// # 底层系统调用  
/// - `Command::new("sh")` → `fork()` + `execve("/bin/sh")`  
/// - `.args(["-c", command])` 传递 shell 命令行  
/// - `.status()` → `waitpid()`（等待子进程并获取退出状态）  
/// - `ExitStatus::code()` 提取低字节退出代码（0‑255）  
///
/// # 实现步骤  
/// 1. 为 `"sh"` 创建一个带参数 `["-c", command]` 的 `Command`。  
/// 2. 调用 `.status()` 执行 shell 并获取 `ExitStatus`。  
/// 3. 使用 `.code()` 获取退出代码作为 `Option<i32>`。  
/// 4. 如果子进程正常终止，返回退出代码；否则返回默认值。  
pub fn get_exit_code(command: &str) -> i32 {
    // TODO: 使用 Command::new("sh").args(["-c", command])
    // TODO: 执行并获取状态
    // TODO: 返回退出代码
    let output = Command::new("sh").args(["-c", command]).status().unwrap();
    output.code().unwrap_or(-1)
}

/// 执行给定的 shell 命令并将其标准输出返回为 `Result`。
/// 此版本正确传播可能在进程创建、执行或 I/O（例如，命令未找到、权限被拒绝、管道中断）期间发生的错误。
/// # 底层系统调用
/// 与 `run_command` 相同，但错误由操作系统捕获并作为 `Err` 返回。
/// # 错误处理
/// - `Command::new()` 仅构造生成器；错误发生在 `.output()`。
/// - `.output()` 返回 `Result<Output, std::io::Error>`。
/// - 如果子进程的输出不是有效的 UTF‑8，`String::from_utf8()` 可能会失败。
///   在这种情况下，我们返回一个 `io::Error`，其种类为 `InvalidData`。
/// # 实现步骤
/// 1. 使用给定的程序和参数创建一个 `Command`。
/// 2. 设置 `.stdout(Stdio::piped())`。
/// 3. 调用 `.output()` 并传播任何 `io::Error`。
/// 4. 使用 `String::from_utf8` 将 `stdout` 转换为 `String`；如果失败，则映射为 `io::Error`。
pub fn run_command_with_result(program: &str, args: &[&str]) -> io::Result<String> {
    // TODO: 使用 Command::new 创建进程
    // TODO: 将 stdout 设置为 Stdio::piped()
    // TODO: 使用 .output() 执行并处理 Result
    // TODO: 使用 from_utf8 将 stdout 转换为 String，并将错误映射为 io::Error
    let output = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .output()?;
    String::from_utf8(output.stdout)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "not UTF-8"))
}

/// 通过双向管道与 `grep` 交互，过滤包含特定模式的行。  
///
/// 这展示了复杂的父子进程通信：父进程发送多行输入，子进程（`grep`）根据模式过滤它们，父进程仅读取匹配的行。
///
/// # 底层系统调用
///
/// - `Command::new("grep")` → `fork()` + `execve("grep")`
/// - 两个管道（stdin 和 stdout），类似于 `pipe_through_cat`
/// - 按行写入和读取以模拟交互式过滤
///
///
/// # 实现步骤
/// 1. 创建一个带有参数 `pattern` 且两端均为管道的 `"grep"` `Command`。
/// 2. 使用 `.spawn()` 启动命令，获取带有 `stdin` 和 `stdout` 句柄的 `Child`。
/// 3. 将每行 `input`（以 `'\n'` 分隔）写入子进程的 stdin。
/// 4. 关闭写入端（丢弃 stdin）以发送 EOF 信号。
/// 5. 按行读取子进程的 stdout，收集匹配的行。
/// 6. 等待子进程退出（可选；`grep` 在 EOF 后退出）。
/// 7. 返回连接的匹配行作为单个 `String`。
///
pub fn pipe_through_grep(pattern: &str, input: &str) -> String {
    // TODO: 创建带有模式的 "grep" 命令，设置 stdin 和 stdout 为管道
    // TODO: 启动进程
    let grep = Command::new("grep")
        .args(&[pattern])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut child = grep;
    // TODO: 将输入行写入子进程的 stdin
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    // TODO: 关闭 stdin 以关闭管道
    drop(child.stdin.take());
    let mut output = String::new();
    // TODO: 从子进程的 stdout 按行读取输出
    child
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut output)
        .unwrap();
    // TODO: 收集并返回匹配的行
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_echo() {
        let output = run_command("echo", &["hello"]);
        assert_eq!(output.trim(), "hello");
    }

    #[test]
    fn test_run_with_args() {
        let output = run_command("echo", &["-n", "no newline"]);
        assert_eq!(output, "no newline");
    }

    #[test]
    fn test_pipe_cat() {
        let output = pipe_through_cat("hello pipe!");
        assert_eq!(output, "hello pipe!");
    }

    #[test]
    fn test_pipe_multiline() {
        let input = "line1\nline2\nline3";
        assert_eq!(pipe_through_cat(input), input);
    }

    #[test]
    fn test_exit_code_success() {
        assert_eq!(get_exit_code("true"), 0);
    }

    #[test]
    fn test_exit_code_failure() {
        assert_eq!(get_exit_code("false"), 1);
    }

    #[test]
    fn test_run_command_with_result_success() {
        let result = run_command_with_result("echo", &["hello"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "hello");
    }

    #[test]
    fn test_run_command_with_result_nonexistent() {
        let result = run_command_with_result("nonexistent_command_xyz", &[]);
        // Should be an error because command not found
        assert!(result.is_err());
    }

    #[test]
    fn test_pipe_through_grep_basic() {
        let input = "apple\nbanana\ncherry\n";
        let output = pipe_through_grep("a", input);
        // grep outputs matching lines with newline
        assert_eq!(output, "apple\nbanana\n");
    }

    #[test]
    fn test_pipe_through_grep_no_match() {
        let input = "apple\nbanana\ncherry\n";
        let output = pipe_through_grep("z", input);
        // No lines match -> empty string
        assert_eq!(output, "");
    }

    #[test]
    fn test_pipe_through_grep_multiline() {
        let input = "first line\nsecond line\nthird line\n";
        let output = pipe_through_grep("second", input);
        assert_eq!(output, "second line\n");
    }
}
