//! 错误格式化器
//!
//! 实现 Rust 编译器风格的错误输出格式，支持：
//! - 彩色错误标题
//! - 错误详情展示
//! - 智能提示
//! - Verbose 模式详细堆栈

use super::{ErrorHint, XoreError, XoreErrorExt};

/// 错误格式化配置
#[derive(Debug, Clone)]
pub struct ErrorFormatterConfig {
    /// 是否显示详细堆栈
    pub verbose: bool,
    /// 是否使用彩色输出
    pub use_color: bool,
    /// 是否显示智能提示
    pub show_hints: bool,
}

impl Default for ErrorFormatterConfig {
    fn default() -> Self {
        Self { verbose: false, use_color: true, show_hints: true }
    }
}

/// 错误格式化器
///
/// 实现友好的错误输出格式，参考 Rust 编译器风格：
///
/// ```text
/// 错误: 文件不存在: /tmp/test.csv
///
///   --> 文件路径: /tmp/test.csv
///
/// 💡 提示: 文件 '/tmp/test.csv' 不存在，请检查路径是否正确
///    尝试运行: ls -la 或 find 命令确认文件是否存在
/// ```
pub struct ErrorFormatter {
    config: ErrorFormatterConfig,
}

impl ErrorFormatter {
    /// 创建新的格式化器
    pub fn new(config: ErrorFormatterConfig) -> Self {
        Self { config }
    }

    /// 使用默认配置创建格式化器
    pub fn default_format() -> Self {
        Self { config: ErrorFormatterConfig::default() }
    }

    /// 创建 verbose 模式的格式化器
    pub fn verbose() -> Self {
        Self { config: ErrorFormatterConfig { verbose: true, ..Default::default() } }
    }

    /// 格式化 XoreError
    pub fn format(&self, err: &XoreError) -> String {
        let mut output = String::new();

        // 1. 错误标题
        output.push_str(&self.format_error_title(err));

        // 2. 错误详情
        let details = self.format_error_details(err);
        if !details.is_empty() {
            output.push_str(&details);
        }

        // 3. 智能提示
        if self.config.show_hints {
            if let Some(hint) = err.hint() {
                output.push_str("\n\n");
                output.push_str(&self.format_hint(&hint));
            }
        }

        // 4. Verbose 模式：显示详细堆栈
        if self.config.verbose {
            output.push_str("\n\n");
            output.push_str(&self.format_verbose(err));
        }

        output
    }

    /// 格式化 anyhow::Error
    pub fn format_anyhow(&self, err: &anyhow::Error) -> String {
        let mut output = String::new();

        // 错误标题
        if self.config.use_color {
            output.push_str(&format!("\x1b[31m错误:\x1b[0m {}", err));
        } else {
            output.push_str(&format!("错误: {}", err));
        }

        // 尝试转换为 XoreError 以获取智能提示
        if let Some(xore_err) = err.downcast_ref::<XoreError>() {
            if self.config.show_hints {
                if let Some(hint) = xore_err.hint() {
                    output.push_str("\n\n");
                    output.push_str(&self.format_hint(&hint));
                }
            }

            if self.config.verbose {
                output.push_str("\n\n");
                output.push_str(&self.format_verbose(xore_err));
            }
        }

        // Verbose 模式：显示完整错误链
        if self.config.verbose {
            let chain: Vec<String> = err.chain().map(|e| e.to_string()).collect();
            if chain.len() > 1 {
                output.push_str("\n\n错误链:\n");
                for (i, cause) in chain.iter().enumerate() {
                    if self.config.use_color {
                        output.push_str(&format!("  \x1b[90m{}: {}\x1b[0m\n", i, cause));
                    } else {
                        output.push_str(&format!("  {}: {}\n", i, cause));
                    }
                }
            }
        }

        output
    }

    /// 格式化错误标题
    fn format_error_title(&self, err: &XoreError) -> String {
        if self.config.use_color {
            format!("\x1b[31m错误:\x1b[0m {}", err)
        } else {
            format!("错误: {}", err)
        }
    }

    /// 格式化错误详情
    fn format_error_details(&self, err: &XoreError) -> String {
        let mut details = String::new();

        match err {
            XoreError::FileNotFound { path } => {
                details.push_str("\n\n  --> 文件路径: ");
                if self.config.use_color {
                    details.push_str(&format!("\x1b[36m{}\x1b[0m", path));
                } else {
                    details.push_str(path);
                }
            }
            XoreError::ResourceLimit { resource, current, max } => {
                details.push_str("\n\n  --> 资源: ");
                if self.config.use_color {
                    details.push_str(&format!("\x1b[33m{}\x1b[0m", resource));
                } else {
                    details.push_str(resource);
                }
                details.push_str(&format!("\n  --> 当前值: {}, 最大值: {}", current, max));
            }
            XoreError::SqlError(msg) => {
                details.push_str("\n\n  --> ");
                if self.config.use_color {
                    details.push_str("\x1b[33mSQL 错误详情\x1b[0m");
                } else {
                    details.push_str("SQL 错误详情");
                }
                details.push_str(&format!(": {}", msg));
            }
            XoreError::IoError(io_err) => {
                details.push_str("\n\n  --> ");
                if self.config.use_color {
                    details.push_str("\x1b[33m系统错误\x1b[0m");
                } else {
                    details.push_str("系统错误");
                }
                details.push_str(&format!(": {} (错误类型: {:?})", io_err, io_err.kind()));
            }
            XoreError::PermissionDenied(path) => {
                details.push_str("\n\n  --> 路径: ");
                if self.config.use_color {
                    details.push_str(&format!("\x1b[36m{}\x1b[0m", path));
                } else {
                    details.push_str(path);
                }
            }
            _ => {}
        }

        details
    }

    /// 格式化智能提示
    fn format_hint(&self, hint: &ErrorHint) -> String {
        if self.config.use_color {
            format!("\x1b[36m💡 提示:\x1b[0m {}", hint.format())
        } else {
            format!("💡 提示: {}", hint.format())
        }
    }

    /// Verbose 模式：格式化详细堆栈信息
    fn format_verbose(&self, err: &XoreError) -> String {
        let mut output = String::new();

        if self.config.use_color {
            output.push_str("\x1b[90m--- 详细信息 ---\x1b[0m\n");
        } else {
            output.push_str("--- 详细信息 ---\n");
        }

        // 错误类型和代码
        output.push_str(&format!("  错误代码: {}\n", err.error_code()));
        output.push_str(&format!("  错误消息: {}\n", err));

        // 针对特定错误类型提供更多信息
        match err {
            XoreError::IoError(io_err) => {
                output.push_str(&format!("  IO 错误类型: {:?}\n", io_err.kind()));
                if let Some(os_err) = io_err.raw_os_error() {
                    output.push_str(&format!("  OS 错误码: {}\n", os_err));
                }
            }
            XoreError::FileNotFound { path } => {
                output.push_str(&format!("  文件路径: {}\n", path));
                output.push_str("  可能的解决方案:\n");
                output.push_str("    1. 检查文件路径是否正确（注意大小写）\n");
                output.push_str("    2. 使用 'ls -la' 或 'find' 命令确认文件存在\n");
                output.push_str("    3. 检查文件读取权限\n");
            }
            XoreError::SqlError(sql) => {
                output.push_str(&format!("  SQL 语句: {}\n", sql));
                output.push_str("  可能的解决方案:\n");
                output.push_str("    1. 检查 SQL 关键字拼写（如 FROM 不是 FORM）\n");
                output.push_str("    2. 确认表名和列名正确\n");
                output.push_str("    3. 运行 'xore agent explain' 获取详细分析\n");
            }
            XoreError::IndexError(_) => {
                output.push_str("  可能的解决方案:\n");
                output.push_str("    1. 运行 'xore f --rebuild' 重建索引\n");
                output.push_str("    2. 检查索引目录权限\n");
                output.push_str("    3. 确认磁盘空间充足\n");
            }
            _ => {}
        }

        output
    }
}

impl Default for ErrorFormatter {
    fn default() -> Self {
        Self::default_format()
    }
}

/// CLI 错误输出辅助函数
///
/// 将 XoreError 格式化后输出到 stderr
pub fn print_error(err: &XoreError, verbose: bool, no_color: bool) {
    let config = ErrorFormatterConfig { verbose, use_color: !no_color, show_hints: true };
    let formatter = ErrorFormatter::new(config);
    eprintln!("{}", formatter.format(err));
}

/// CLI anyhow 错误输出辅助函数
///
/// 将 anyhow::Error 格式化后输出到 stderr
pub fn print_anyhow_error(err: &anyhow::Error, verbose: bool, no_color: bool) {
    let config = ErrorFormatterConfig { verbose, use_color: !no_color, show_hints: true };
    let formatter = ErrorFormatter::new(config);
    eprintln!("{}", formatter.format_anyhow(err));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_file_not_found() {
        let err = XoreError::FileNotFound { path: "/tmp/test.csv".to_string() };

        let formatter = ErrorFormatter::default_format();
        let output = formatter.format(&err);

        assert!(output.contains("错误:"));
        assert!(output.contains("/tmp/test.csv"));
        assert!(output.contains("提示:"));
    }

    #[test]
    fn test_format_sql_error() {
        let err = XoreError::SqlError("语法错误 near 'FORM'".to_string());

        let formatter = ErrorFormatter::default_format();
        let output = formatter.format(&err);

        assert!(output.contains("错误:"));
        assert!(output.contains("SQL"));
        assert!(output.contains("提示:"));
    }

    #[test]
    fn test_format_verbose() {
        let err = XoreError::FileNotFound { path: "test.csv".to_string() };

        let formatter = ErrorFormatter::verbose();
        let output = formatter.format(&err);

        assert!(output.contains("详细信息"));
        assert!(output.contains("FILE_NOT_FOUND"));
    }

    #[test]
    fn test_format_no_color() {
        let err = XoreError::Other("测试错误".to_string());

        let config = ErrorFormatterConfig { verbose: false, use_color: false, show_hints: false };
        let formatter = ErrorFormatter::new(config);
        let output = formatter.format(&err);

        // 不应包含 ANSI 转义序列
        assert!(!output.contains("\x1b["));
    }

    #[test]
    fn test_format_resource_limit() {
        let err =
            XoreError::ResourceLimit { resource: "内存".to_string(), current: 2048, max: 1024 };

        let config = ErrorFormatterConfig { verbose: false, use_color: false, show_hints: false };
        let formatter = ErrorFormatter::new(config);
        let output = formatter.format(&err);

        assert!(output.contains("内存"));
        assert!(output.contains("2048"));
        assert!(output.contains("1024"));
    }

    #[test]
    fn test_format_index_error_verbose() {
        let err = XoreError::IndexError("索引损坏".to_string());

        let formatter = ErrorFormatter::verbose();
        let output = formatter.format(&err);

        assert!(output.contains("详细信息"));
        assert!(output.contains("xore f --rebuild"));
    }

    #[test]
    fn test_print_error_no_panic() {
        let err = XoreError::FileNotFound { path: "test.csv".to_string() };
        // 不应 panic
        print_error(&err, false, true);
        print_error(&err, true, true);
    }

    #[test]
    fn test_format_anyhow_with_xore_error() {
        let xore_err = XoreError::SqlError("测试 SQL 错误".to_string());
        let anyhow_err: anyhow::Error = xore_err.into();

        let formatter = ErrorFormatter::default_format();
        let output = formatter.format_anyhow(&anyhow_err);

        assert!(output.contains("错误:"));
    }
}
