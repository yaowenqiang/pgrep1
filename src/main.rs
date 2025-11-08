// pgrep - 一个简单的 Rust 实现的 grep 工具
//
// 这个程序演示了以下 Rust 概念：
// 1. 命令行参数解析（使用 clap 库）
// 2. 错误处理（使用 failure 库）
// 3. 正则表达式匹配（使用 regex 库）
// 4. 文件系统操作和目录遍历
// 5. 泛型和闭包的使用

// 引入外部库
//
// clap: 命令行参数解析库
// 文档: <https://docs.rs/clap/>
// GitHub: <https://github.com/clap-rs/clap>
use clap;
use clap::Parser;

// failure: 错误处理库，提供结构化错误处理
// 文档: <https://docs.rs/failure/>
// 注意：failure 库已不再维护，新项目推荐使用 anyhow 或 thiserror
// anyhow 文档: <https://docs.rs/anyhow/>
// thiserror 文档: <https://docs.rs/thiserror/>
use failure::{Error, Fail};

// regex: 正则表达式库
// 文档: <https://docs.rs/regex/>
// GitHub: <https://github.com/rust-lang/regex>
use regex::Regex;

// 标准库引入
use std::fmt;
use std::path::Path;

// Failure 库的教程链接
// <https://boats.gitlab.io/failure/>
// 这个教程详细介绍了如何使用 failure 库进行错误处理

/// 记录结构体
///
/// 用于存储在文件中找到的匹配结果
///
/// # 字段
/// * `line` - 匹配行号（从0开始计数）
/// * `tx` - 匹配行的文本内容
#[derive(Debug)]
struct Record {
    line: usize,
    tx: String,
}

/// 参数错误结构体
///
/// 使用 failure 库的 Fail derive 宏来实现自定义错误类型
/// 这个结构体演示了如何创建结构化的错误信息
///
/// # 使用示例
/// ```
/// let error = ArgErr { arg: "file" };
/// println!("{}", error); // 输出: Argument not provided file
/// ```
///
/// # 相关文档
/// * failure 库文档: <https://docs.rs/failure/>
/// * Fail trait 文档: <https://docs.rs/failure/latest/failure/trait.Fail.html>
#[derive(Debug, Fail)]
#[fail(display = "Argument not provided {}", arg)]
struct ArgErr {
    arg: &'static str,
}

// 注意：下面的代码被注释掉了，因为使用了 Fail derive 宏后，
// Rust 会自动为我们实现 Fail trait 和 Display trait
//
// 如果不使用 derive 宏，我们需要手动实现这些 trait：

// impl Fail for ArgErr {}

/*
// 手动实现 Display trait 以支持错误信息的格式化
impl std::fmt::Display for ArgErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Argument Not provided: {}", self.arg)
    }
}
*/

/// 命令行参数结构体
///
/// 使用 clap 库的 Parser derive 宏来自动解析命令行参数
/// clap 会根据结构体定义自动生成帮助信息和错误处理
///
/// # 使用示例
/// ```bash
/// cargo run -- -f test.txt -p "pattern"
/// ```
///
/// # 相关文档
/// * clap 文档: <https://docs.rs/clap/>
/// * Parser derive 宏: <https://docs.rs/clap/latest/clap/trait.Parser.html>
#[derive(Parser, Debug)]
#[command(version = "0.1.0", about = "一个简单的 grep 工具")]
struct Args {
    /// 要搜索的文件路径
    ///
    /// 可以是文件名或目录路径。如果是目录，程序会递归搜索其中的所有文件。
    ///
    /// # 示例
    /// * `-f test.txt` - 搜索单个文件
    /// * `-f ./testdir` - 搜索整个目录
    #[arg(short = 'f', long)]
    file: String,

    /// 要搜索的正则表达式模式
    ///
    /// 支持完整的正则表达式语法，包括：
    /// - 字符类: [a-z], [0-9]
    /// - 量词: *, +, ?, {n,m}
    /// - 分组: (...)
    /// - 断言: ^, $, \b
    ///
    /// # 示例
    /// * `-p "abc"` - 搜索字符串 "abc"
    /// * `-p "a.*b"` - 搜索以 a 开头、b 结尾的行
    /// * `-p "[0-9]+"` - 搜索数字
    #[arg(short = 'p', long)]
    pattern: String,
}

/// 处理单个文件的函数
///
/// 读取指定文件的内容，逐行检查是否匹配给定的正则表达式，
/// 并返回所有匹配的记录。
///
/// # 参数
/// * `p` - 文件路径，实现了 AsRef<Path> trait，可以接受 &str, &Path, String 等类型
/// * `re` - 编译好的正则表达式对象
///
/// # 返回值
/// * `Ok(Vec<Record>)` - 包含所有匹配记录的向量
/// * `Err(Error)` - 文件读取或处理过程中的错误
///
/// # 泛型约束
/// `P: AsRef<Path>` - 允许函数接受多种路径类型作为参数
///
/// # 错误处理
/// 使用 `?` 操作符自动处理 I/O 错误，将其转换为 failure::Error
///
/// # 相关文档
/// * std::fs::read: <https://doc.rust-lang.org/std/fs/fn.read.html>
/// * String::from_utf8: <https://doc.rust-lang.org/std/string/struct.String.html#method.from_utf8>
/// * AsRef trait: <https://doc.rust-lang.org/std/convert/trait.AsRef.html>
fn process_file<P: AsRef<Path>>(p: P, re: &Regex) -> Result<Vec<Record>, Error> {
    // 用于存储匹配结果的向量
    let mut res = Vec::new();

    // 读取文件的二进制内容
    // `std::fs::read` 会将整个文件内容读入内存
    let bts = std::fs::read(p)?;

    // 尝试将字节数组转换为 UTF-8 字符串
    // 使用 if let 来处理可能的编码错误
    if let Ok(ss) = String::from_utf8(bts) {
        // 逐行处理文件内容
        // enumerate() 为每一行提供行号（从0开始）
        for (i, l) in ss.lines().enumerate() {
            // 检查当前行是否匹配正则表达式
            if re.is_match(l) {
                // 如果匹配，创建一个新的 Record 并添加到结果中
                res.push(Record {
                    line: i,
                    tx: l.to_string(),
                })
            }
        }
    }

    // 返回匹配结果
    Ok(res)
}

/// 递归处理路径的函数
///
/// 这个函数可以处理文件和目录。对于文件，直接调用 process_file 进行搜索；
/// 对于目录，递归遍历其中的所有文件并进行搜索。
///
/// # 参数
/// * `p` - 要处理的路径（文件或目录）
/// * `re` - 编译好的正则表达式对象
/// * `ff` - 文件处理完成时的回调函数，接收路径和匹配结果
/// * `ef` - 错误处理回调函数，接收发生的错误
///
/// # 泛型参数和约束
/// * `P: AsRef<Path>` - 路径类型，支持多种路径输入
/// * `FF: Fn(&Path, Vec<Record>)` - 文件处理回调函数类型
/// * `EF: Fn(Error)` - 错误处理回调函数类型
///
/// # 函数式编程特性
/// 这个函数展示了 Rust 中函数式编程的特性：
/// - 使用闭包作为回调函数
/// - 泛型约束确保类型安全
/// - 函数式风格的错误处理
///
/// # 递归处理
/// 目录处理是递归的，会遍历所有子目录和文件
///
/// # 相关文档
/// * std::fs::metadata: <https://doc.rust-lang.org/std/fs/fn.metadata.html>
/// * std::fs::read_dir: <https://doc.rust-lang.org/std/fs/fn.read_dir.html>
/// * 闭包文档: <https://doc.rust-lang.org/rust-by-example/fn/closures.html>
fn process_path<P, FF, EF>(p: P, re: &Regex, ff:&FF, ef: &EF) -> Result<(), Error>
where
    P: AsRef<Path>,
    FF: Fn(&Path, Vec<Record>),
    EF: Fn(Error),
{
    // 将输入路径转换为 Path 引用
    let p = p.as_ref();

    // 获取路径的元数据信息（文件类型、大小、权限等）
    let md = p.metadata()?;

    // 获取文件类型信息
    let ft = md.file_type();

    // 处理文件：如果是文件，直接搜索其内容
    if ft.is_file() {
        // 调用 process_file 处理文件内容
        let dt = process_file(p, re)?;

        // 调用文件处理回调函数，传递路径和匹配结果
        ff(p, dt);
    }

    // 处理目录：如果是目录，递归遍历其中的所有条目
    if ft.is_dir() {
        // 读取目录内容，返回一个迭代器
        let dd = std::fs::read_dir(p)?;

        // 遍历目录中的每个条目
        for d in dd {
            // 获取目录条目（可能失败，使用 ? 操作符处理）
            let entry = d?;

            // 递归调用 process_path 处理子路径
            // 如果递归调用失败，调用错误处理回调函数而不是直接返回错误
            if let Err(e) = process_path(entry.path(), re, ff, ef) {
                ef(e);
            }
        }
    }

    // 返回成功
    Ok(())
}
/// 主运行函数
///
/// 这个函数是程序的主要逻辑入口点，负责：
/// 1. 解析命令行参数
/// 2. 编译正则表达式
/// 3. 调用路径处理函数
/// 4. 处理结果和错误
///
/// # 返回值
/// * `Ok(())` - 程序成功执行
/// * `Err(Error)` - 执行过程中发生错误
///
/// # 错误处理
/// 使用 failure 库的 Result 类型进行错误处理，
/// 所有的 I/O 错误、正则表达式编译错误等都会被自动捕获
///
/// # 相关文档
/// * Regex::new: <https://docs.rs/regex/latest/regex/struct.Regex.html#method.new>
/// * Args::parse: <https://docs.rs/clap/latest/clap/trait.Parser.html#tymethod.parse>
fn run() -> Result<(), Error> {
    // 使用 clap 自动解析命令行参数
    // 如果参数格式不正确，clap 会自动显示帮助信息并退出
    let args = Args::parse();

    // 编译用户提供的正则表达式模式
    // 如果正则表达式语法错误，这里会返回编译错误
    let re = Regex::new(&args.pattern)?;

    // 调用递归路径处理函数
    // 使用闭包作为回调函数来处理文件处理结果和错误

    // 注释掉的代码：处理单个文件的方式
    //let p = process_file(args.file, &re);

    // 实际使用的代码：处理路径（文件或目录）的方式
    let p = process_path(
        // 要处理的路径
        args.file,
        // 编译好的正则表达式
        &re,

        // 文件处理完成回调函数
        // 这个闭包会在每个文件处理完成后被调用
        &|pt, v| {
            println!("文件路径: {:?}", pt);
            println!("匹配结果: {:?}", v);
        },

        // 错误处理回调函数
        // 这个闭包会在处理过程中发生错误时被调用
        &|e| {
            println!("处理错误: {}", e);
        }
    );

    // 输出整体处理结果
    // 这里的 Result 表示整个处理过程是否成功
    println!("整体处理结果: {:?}", p);

    // 返回成功
    Ok(())
}

/// 程序主入口函数
///
/// 这是程序的入口点，负责：
/// 1. 调用主运行函数
/// 2. 处理顶层错误并优雅退出
///
/// # 错误处理模式
/// 使用 Rust 推荐的错误处理模式：
/// - 使用 if let Err(e) 来检查 Result
/// - 打印友好的错误信息给用户
/// - 程序以非零状态码退出（通过 panic! 或 std::process::exit）
///
/// # 设计原则
/// 这种设计遵循了 Rust 的最佳实践：
/// 1. 将核心逻辑与错误处理分离
/// 2. main 函数保持简洁
/// 3. 提供清晰的错误信息
///
/// # 相关文档
/// * main 函数文档: <https://doc.rust-lang.org/std/fn.main.html>
/// * 错误处理指南: <https://doc.rust-lang.org/book/ch09.html>
fn main() {
    // 调用主运行函数并处理可能发生的错误
    // 这种模式确保程序在遇到错误时能够优雅地退出
    if let Err(e) = run() {
        // 打印用户友好的错误信息
        println!("程序执行时发生错误: {}", e);

        // 在实际的应用程序中，这里可能需要：
        // 1. 记录错误日志
        // 2. 返回适当的退出码
        // 3. 提供更详细的错误恢复建议
        // 例如：std::process::exit(1);
    }
}
