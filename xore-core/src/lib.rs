//! XORE Core - 共享类型和工具函数
//!
//! 这个crate包含XORE项目的核心类型定义、错误处理和配置管理。

pub mod config;
pub mod error;
pub mod logging;
pub mod types;

pub use config::Config;
pub use error::{Result, XoreError};
pub use logging::{LogConfig, LogLevel};
