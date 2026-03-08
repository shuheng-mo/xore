//! XORE Core - 共享类型和工具函数
//!
//! 这个crate包含XORE项目的核心类型定义、错误处理和配置管理。

pub mod config;
pub mod context;
pub mod error;
pub mod history;
pub mod logging;
pub mod output;
pub mod recommendation;
pub mod types;

pub use config::Config;
pub use context::{get_default_sessions_dir, ContextData, ContextOperation, SessionContext};
pub use error::{
    print_anyhow_error, print_error, ErrorChain, ErrorContext, ErrorFormatter,
    ErrorFormatterConfig, ErrorHint, Result, XoreError, XoreErrorExt,
};
pub use history::{
    get_default_history_path, HistoryStore, SearchHistoryEntry, SearchStats, SearchType,
};
pub use logging::{LogConfig, LogLevel};
pub use output::{
    get_total_savings, reset_total_savings, OutputFormatter, OutputMode, TokenSavings,
};
pub use recommendation::{
    format_time_ago, Recommendation, RecommendationEngine, RecommendationType,
};
