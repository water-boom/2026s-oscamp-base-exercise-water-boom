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
    todo!()
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
    // TODO: Create "cat" command, set stdin and stdout to piped
    // TODO: Spawn process
    // TODO: Write input to child process stdin
    // TODO: Drop stdin to close pipe (otherwise cat won't exit)
    // TODO: Read output from child process stdout
    todo!()
}

/// Get child process exit code.
/// Execute command `sh -c {command}` and return the exit code.
///
/// # Underlying System Calls
/// - `Command::new("sh")` → `fork()` + `execve("/bin/sh")`
/// - `.args(["-c", command])` passes the shell command line
/// - `.status()` → `waitpid()` (waits for child and retrieves exit status)
/// - `ExitStatus::code()` extracts the low‑byte exit code (0‑255)
///
/// # Implementation Steps
/// 1. Create a `Command` for `"sh"` with arguments `["-c", command]`.
/// 2. Call `.status()` to execute the shell and obtain an `ExitStatus`.
/// 3. Use `.code()` to get the exit code as `Option<i32>`.
/// 4. If the child terminated normally, return the exit code; otherwise return a default.
pub fn get_exit_code(command: &str) -> i32 {
    // TODO: Use Command::new("sh").args(["-c", command])
    // TODO: Execute and get status
    // TODO: Return exit code
    todo!()
}

/// Execute the given shell command and return its stdout output as a `Result`.
///
/// This version properly propagates errors that may occur during process creation,
/// execution, or I/O (e.g., command not found, permission denied, broken pipe).
///
/// # Underlying System Calls
/// Same as `run_command`, but errors are captured from the OS and returned as `Err`.
///
/// # Error Handling
/// - `Command::new()` only constructs the builder; errors occur at `.output()`.
/// - `.output()` returns `Result<Output, std::io::Error>`.
/// - `String::from_utf8()` may fail if the child's output is not valid UTF‑8.
///   In that case we return an `io::Error` with kind `InvalidData`.
///
/// # Implementation Steps
/// 1. Create a `Command` with the given program and arguments.
/// 2. Set `.stdout(Stdio::piped())`.
/// 3. Call `.output()` and propagate any `io::Error`.
/// 4. Convert `stdout` to `String` with `String::from_utf8`; if that fails, map to an `io::Error`.
pub fn run_command_with_result(program: &str, args: &[&str]) -> io::Result<String> {
    // TODO: Use Command::new to create process
    // TODO: Set stdout to Stdio::piped()
    // TODO: Execute with .output() and handle Result
    // TODO: Convert stdout to String with from_utf8, mapping errors to io::Error
    todo!()
}

/// Interact with `grep` via bidirectional pipes, filtering lines that contain a pattern.
///
/// This demonstrates complex parent‑child communication: the parent sends multiple
/// lines of input, the child (`grep`) filters them according to a pattern, and the
/// parent reads back only the matching lines.
///
/// # Underlying System Calls
/// - `Command::new("grep")` → `fork()` + `execve("grep")`
/// - Two pipes (stdin & stdout) as in `pipe_through_cat`
/// - Line‑by‑line writing and reading to simulate interactive filtering
///
/// # Implementation Steps
/// 1. Create a `Command` for `"grep"` with argument `pattern`, and both ends piped.
/// 2. `.spawn()` the command, obtaining `Child` with `stdin` and `stdout` handles.
/// 3. Write each line of `input` (separated by `'\n'`) to the child's stdin.
/// 4. Close the write end (drop stdin) to signal EOF.
/// 5. Read the child's stdout line by line, collecting matching lines.
/// 6. Wait for the child to exit (optional; `grep` exits after EOF).
/// 7. Return the concatenated matching lines as a single `String`.
///
pub fn pipe_through_grep(pattern: &str, input: &str) -> String {
    // TODO: Create "grep" command with pattern, set stdin and stdout to piped
    // TODO: Spawn process
    // TODO: Write input lines to child stdin
    // TODO: Drop stdin to close pipe
    // TODO: Read output from child stdout line by line
    // TODO: Collect and return matching lines
    todo!()
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
