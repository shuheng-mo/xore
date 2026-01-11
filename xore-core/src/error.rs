//! 错误类型定义

use thiserror::Error;

/// XORE统一错误类型
#[derive(Debug, Error)]
pub enum XoreError {
    #[error("索引错误: {0}")]
    IndexError(String),

    #[error("数据处理错误: {0}")]
    ProcessError(String),

    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),

    #[error("SQL语法错误: {0}")]
    SqlError(String),

    #[error("文件不存在: {path}")]
    FileNotFound { path: String },

    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error("超出资源限制: {resource} (当前: {current}, 最大: {max})")]
    ResourceLimit {
        resource: String,
        current: usize,
        max: usize,
    },

    #[error("AI模型错误: {0}")]
    AiError(String),

    #[error("{0}")]
    Other(String),
}

/// XORE统一Result类型
pub type Result<T> = std::result::Result<T, XoreError>;

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
