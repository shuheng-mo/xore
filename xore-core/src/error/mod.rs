//! 错误处理模块
//!
//! XORE 统一错误处理系统，包含：
//! - 核心错误类型 `XoreError`
//! - 错误格式化器 `ErrorFormatter`
//! - 智能错误提示 `ErrorHint`
//! - 错误上下文 `ErrorContext`

use std::fmt;
use thiserror::Error;

pub mod format;

pub use format::{print_anyhow_error, print_error, ErrorFormatter, ErrorFormatterConfig};

/// XORE 统一错误类型
#[derive(Debug, Error)]
pub enum XoreError {
    #[error("索引错误: {0}")]
    IndexError(String),

    #[error("搜索错误: {0}")]
    SearchError(String),

    #[error("数据处理错误: {0}")]
    ProcessError(String),

    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),

    #[error("SQL语法错误: {0}")]
    SqlError(String),

    #[error("解析错误: {0}")]
    ParseError(String),

    #[error("验证错误: {0}")]
    ValidationError(String),

    #[error("文件不存在: {path}")]
    FileNotFound { path: String },

    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error("历史记录错误: {0}")]
    HistoryError(String),

    #[error("超时: {0}")]
    Timeout(String),

    #[error("权限不足: {0}")]
    PermissionDenied(String),

    #[error("超出资源限制: {resource} (当前: {current}, 最大: {max})")]
    ResourceLimit { resource: String, current: usize, max: usize },

    #[error("AI模型错误: {0}")]
    AiError(String),

    #[error("{0}")]
    Other(String),
}

/// XORE 统一 Result 类型
pub type Result<T> = std::result::Result<T, XoreError>;

/// 错误上下文信息
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// 上下文消息列表
    messages: Vec<String>,
    /// 错误发生的位置（文件）
    location: Option<String>,
    /// 错误发生的位置（行号）
    line: Option<u32>,
}

impl ErrorContext {
    /// 创建新的上下文
    pub fn new() -> Self {
        Self { messages: Vec::new(), location: None, line: None }
    }

    /// 添加上下文消息
    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.messages.push(msg.into());
        self
    }

    /// 添加位置信息
    pub fn with_location(mut self, file: impl Into<String>, line: u32) -> Self {
        self.location = Some(file.into());
        self.line = Some(line);
        self
    }

    /// 获取所有上下文消息
    pub fn messages(&self) -> &[String] {
        &self.messages
    }

    /// 获取位置信息
    pub fn location(&self) -> Option<(&str, u32)> {
        self.location.as_ref().zip(self.line).map(|(l, r)| (l.as_str(), r))
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for msg in &self.messages {
            writeln!(f, "{}", msg)?;
        }
        if let Some((loc, line)) = self.location() {
            writeln!(f, "    --> {}:{}", loc, line)?;
        }
        Ok(())
    }
}

/// 智能错误提示
#[derive(Debug, Clone)]
pub struct ErrorHint {
    /// 提示消息
    message: String,
    /// 建议的命令（可选）
    suggested_command: Option<String>,
    /// 文档链接（可选）
    doc_link: Option<String>,
}

impl ErrorHint {
    /// 创建新的提示
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into(), suggested_command: None, doc_link: None }
    }

    /// 添加建议命令
    pub fn with_command(mut self, cmd: impl Into<String>) -> Self {
        self.suggested_command = Some(cmd.into());
        self
    }

    /// 添加文档链接
    pub fn with_doc(mut self, link: impl Into<String>) -> Self {
        self.doc_link = Some(link.into());
        self
    }

    /// 格式化提示消息
    pub fn format(&self) -> String {
        let mut output = self.message.clone();
        if let Some(ref cmd) = self.suggested_command {
            output.push_str(&format!("\n   尝试运行: {}", cmd));
        }
        if let Some(ref link) = self.doc_link {
            output.push_str(&format!("\n   参考文档: {}", link));
        }
        output
    }
}

impl fmt::Display for ErrorHint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}

/// 为 XoreError 实现上下文添加功能
pub trait XoreErrorExt {
    /// 添加上下文信息
    fn context(self, msg: impl Into<String>) -> XoreError;

    /// 添加带位置的上下文信息
    fn with_location(self, file: impl Into<String>, line: u32, msg: impl Into<String>)
        -> XoreError;

    /// 获取错误提示
    fn hint(&self) -> Option<ErrorHint>;
}

impl XoreErrorExt for XoreError {
    fn context(self, msg: impl Into<String>) -> XoreError {
        let current_msg = self.to_string();
        let new_msg = format!("{}\n    --> {}", current_msg, msg.into());
        XoreError::Other(new_msg)
    }

    fn with_location(
        self,
        file: impl Into<String>,
        line: u32,
        msg: impl Into<String>,
    ) -> XoreError {
        let file_str = file.into();
        let current_msg = self.to_string();
        let new_msg =
            format!("{}\n    --> {}:{}\n    |    {}", current_msg, file_str, line, msg.into());
        XoreError::Other(new_msg)
    }

    fn hint(&self) -> Option<ErrorHint> {
        match self {
            XoreError::FileNotFound { path } => Some(
                ErrorHint::new(format!("文件 '{}' 不存在，请检查路径是否正确", path))
                    .with_command("ls -la 或 find 命令确认文件是否存在"),
            ),
            XoreError::SqlError(_) => Some(
                ErrorHint::new("SQL 语法错误，请检查 SQL 语句是否正确")
                    .with_command("xore agent explain \"<你的SQL>\" 获取详细错误分析"),
            ),
            XoreError::ConfigError(_) => Some(
                ErrorHint::new("配置文件格式错误，请检查配置文件")
                    .with_doc("docs/reference/configuration.md"),
            ),
            XoreError::IndexError(_) => Some(
                ErrorHint::new("索引错误，可能需要重建索引")
                    .with_command("xore f --rebuild 重建索引"),
            ),
            XoreError::SearchError(_) => Some(
                ErrorHint::new("搜索错误，尝试重建索引后重试")
                    .with_command("xore f --rebuild 重建索引后重试"),
            ),
            XoreError::PermissionDenied(_) => Some(
                ErrorHint::new("权限不足，请检查文件权限").with_command("chmod 或 sudo 调整权限"),
            ),
            XoreError::ParseError(_) => Some(
                ErrorHint::new("文件解析错误，请检查文件格式是否正确")
                    .with_command("file <文件名> 查看文件类型"),
            ),
            XoreError::ValidationError(_) => {
                Some(ErrorHint::new("输入验证失败，请检查输入数据是否符合要求"))
            }
            XoreError::Timeout(_) => Some(
                ErrorHint::new("操作超时，请检查网络连接或增加超时时间")
                    .with_doc("docs/reference/configuration.md"),
            ),
            XoreError::AiError(_) => Some(
                ErrorHint::new("AI 模型错误，请检查模型文件是否存在")
                    .with_command("ls assets/models/ 查看模型文件"),
            ),
            _ => None,
        }
    }
}

impl XoreError {
    /// 创建带上下文的错误（兼容 anyhow 风格）
    pub fn with_context<C: Into<String>>(self, context: C) -> Self {
        XoreErrorExt::context(self, context)
    }

    /// 获取错误代码（用于程序化错误处理）
    pub fn error_code(&self) -> &'static str {
        match self {
            XoreError::IndexError(_) => "INDEX_ERROR",
            XoreError::SearchError(_) => "SEARCH_ERROR",
            XoreError::ProcessError(_) => "PROCESS_ERROR",
            XoreError::IoError(_) => "IO_ERROR",
            XoreError::SqlError(_) => "SQL_ERROR",
            XoreError::ParseError(_) => "PARSE_ERROR",
            XoreError::ValidationError(_) => "VALIDATION_ERROR",
            XoreError::FileNotFound { .. } => "FILE_NOT_FOUND",
            XoreError::ConfigError(_) => "CONFIG_ERROR",
            XoreError::HistoryError(_) => "HISTORY_ERROR",
            XoreError::Timeout(_) => "TIMEOUT",
            XoreError::PermissionDenied(_) => "PERMISSION_DENIED",
            XoreError::ResourceLimit { .. } => "RESOURCE_LIMIT",
            XoreError::AiError(_) => "AI_ERROR",
            XoreError::Other(_) => "OTHER_ERROR",
        }
    }
}

impl From<String> for XoreError {
    fn from(s: String) -> Self {
        XoreError::Other(s)
    }
}

impl From<&str> for XoreError {
    fn from(s: &str) -> Self {
        XoreError::Other(s.to_string())
    }
}

/// 错误链，用于跟踪错误来源
#[derive(Debug)]
pub struct ErrorChain {
    /// 顶层错误
    pub error: XoreError,
    /// 错误来源（可能是其他错误）
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl ErrorChain {
    /// 创建新的错误链
    pub fn new(error: XoreError) -> Self {
        Self { error, source: None }
    }

    /// 添加错误来源
    pub fn with_source<E: std::error::Error + Send + Sync + 'static>(mut self, source: E) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    /// 获取完整的错误消息
    pub fn full_message(&self) -> String {
        let mut msg = self.error.to_string();
        if let Some(ref source) = self.source {
            msg.push_str(&format!("\n\n根本原因: {}", source));
        }
        msg
    }
}

impl fmt::Display for ErrorChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.full_message())
    }
}

impl std::error::Error for ErrorChain {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref() as _)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code() {
        assert_eq!(XoreError::IndexError("test".to_string()).error_code(), "INDEX_ERROR");
        assert_eq!(
            XoreError::FileNotFound { path: "test".to_string() }.error_code(),
            "FILE_NOT_FOUND"
        );
        assert_eq!(XoreError::SqlError("test".to_string()).error_code(), "SQL_ERROR");
        assert_eq!(XoreError::SearchError("test".to_string()).error_code(), "SEARCH_ERROR");
        assert_eq!(XoreError::ParseError("test".to_string()).error_code(), "PARSE_ERROR");
        assert_eq!(XoreError::ValidationError("test".to_string()).error_code(), "VALIDATION_ERROR");
        assert_eq!(XoreError::Timeout("test".to_string()).error_code(), "TIMEOUT");
        assert_eq!(
            XoreError::PermissionDenied("test".to_string()).error_code(),
            "PERMISSION_DENIED"
        );
    }

    #[test]
    fn test_error_hint_file_not_found() {
        let err = XoreError::FileNotFound { path: "/tmp/test.csv".to_string() };
        let hint = err.hint();
        assert!(hint.is_some());
        let hint = hint.unwrap();
        assert!(hint.format().contains("/tmp/test.csv"));
    }

    #[test]
    fn test_error_hint_sql_error() {
        let err = XoreError::SqlError("语法错误".to_string());
        let hint = err.hint();
        assert!(hint.is_some());
        let hint = hint.unwrap();
        assert!(hint.format().contains("xore agent explain"));
    }

    #[test]
    fn test_error_hint_none_for_other() {
        let err = XoreError::Other("test".to_string());
        let hint = err.hint();
        assert!(hint.is_none());
    }

    #[test]
    fn test_error_context() {
        let ctx = ErrorContext::new().with_message("测试上下文").with_message("第二条消息");

        assert_eq!(ctx.messages().len(), 2);
        assert_eq!(ctx.messages()[0], "测试上下文");
    }

    #[test]
    fn test_error_context_with_location() {
        let ctx = ErrorContext::new().with_message("测试").with_location("src/main.rs", 42);

        let (file, line) = ctx.location().unwrap();
        assert_eq!(file, "src/main.rs");
        assert_eq!(line, 42);
    }

    #[test]
    fn test_error_chain() {
        let err = XoreError::IndexError("索引构建失败".to_string());
        let chain = ErrorChain::new(err)
            .with_source(std::io::Error::new(std::io::ErrorKind::NotFound, "文件不存在"));

        let msg = chain.full_message();
        assert!(msg.contains("索引构建失败"));
        assert!(msg.contains("根本原因"));
    }

    #[test]
    fn test_error_with_context() {
        let err = XoreError::FileNotFound { path: "test.csv".to_string() };
        let err_with_ctx = XoreErrorExt::with_location(err, "main.rs", 100, "加载配置文件时");

        let msg = err_with_ctx.to_string();
        assert!(msg.contains("test.csv"));
        assert!(msg.contains("main.rs"));
        assert!(msg.contains("100"));
    }

    #[test]
    fn test_error_hint_format() {
        let hint =
            ErrorHint::new("测试提示").with_command("xore --help").with_doc("docs/README.md");

        let formatted = hint.format();
        assert!(formatted.contains("测试提示"));
        assert!(formatted.contains("xore --help"));
        assert!(formatted.contains("docs/README.md"));
    }

    #[test]
    fn test_from_string() {
        let err: XoreError = "测试错误".to_string().into();
        assert_eq!(err.error_code(), "OTHER_ERROR");
    }

    #[test]
    fn test_from_str() {
        let err: XoreError = "测试错误".into();
        assert_eq!(err.error_code(), "OTHER_ERROR");
    }
}
