//! 日志系统配置模块
//!
//! 提供统一的日志配置和初始化功能，支持不同的日志级别和输出格式。

use anyhow::Context;
use tracing::Level;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Layer, Registry};

/// 日志配置
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// 日志级别
    pub level: LogLevel,
    /// 是否启用彩色输出
    pub color: bool,
    /// 是否显示时间戳
    pub with_timestamp: bool,
    /// 是否显示目标模块
    pub with_target: bool,
}

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// 静默模式（只输出错误）
    Quiet,
    /// 正常模式（info及以上）
    Normal,
    /// 详细模式（debug及以上）
    Verbose,
    /// 追踪模式（trace及所有）
    Trace,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self { level: LogLevel::Normal, color: true, with_timestamp: false, with_target: false }
    }
}

impl LogConfig {
    /// 创建新的日志配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置日志级别
    pub fn with_level(mut self, level: LogLevel) -> Self {
        self.level = level;
        self
    }

    /// 设置是否启用彩色输出
    pub fn with_color(mut self, color: bool) -> Self {
        self.color = color;
        self
    }

    /// 设置是否显示时间戳
    pub fn with_timestamp(mut self, enabled: bool) -> Self {
        self.with_timestamp = enabled;
        self
    }

    /// 设置是否显示目标模块
    pub fn with_target(mut self, enabled: bool) -> Self {
        self.with_target = enabled;
        self
    }

    /// 从命令行参数创建配置
    pub fn from_args(verbose: bool, quiet: bool, no_color: bool) -> Self {
        let level = if quiet {
            LogLevel::Quiet
        } else if verbose {
            LogLevel::Verbose
        } else {
            LogLevel::Normal
        };

        Self { level, color: !no_color, with_timestamp: verbose, with_target: verbose }
    }

    /// 初始化日志系统
    pub fn init(self) -> anyhow::Result<()> {
        let filter = self.create_env_filter();

        let fmt_layer = fmt::layer().with_ansi(self.color).with_target(self.with_target);

        let fmt_layer =
            if self.with_timestamp { fmt_layer.boxed() } else { fmt_layer.without_time().boxed() };

        let subscriber = Registry::default().with(filter).with(fmt_layer);

        tracing::subscriber::set_global_default(subscriber)
            .context("Failed to set global default subscriber")?;

        tracing::debug!("日志系统初始化完成: {:?}", self);

        Ok(())
    }

    /// 创建环境过滤器
    fn create_env_filter(&self) -> EnvFilter {
        // 优先使用环境变量 RUST_LOG
        if let Ok(env_filter) = EnvFilter::try_from_default_env() {
            return env_filter;
        }

        // 否则根据配置创建过滤器
        let level_str = match self.level {
            LogLevel::Quiet => "error",
            LogLevel::Normal => "info",
            LogLevel::Verbose => "debug",
            LogLevel::Trace => "trace",
        };

        // 为 XORE 模块设置日志级别
        EnvFilter::new(format!(
            "xore_cli={level},xore_core={level},xore_search={level},xore_process={level},xore_ai={level}",
            level = level_str
        ))
    }
}

impl From<LogLevel> for Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Quiet => Level::ERROR,
            LogLevel::Normal => Level::INFO,
            LogLevel::Verbose => Level::DEBUG,
            LogLevel::Trace => Level::TRACE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LogConfig::default();
        assert_eq!(config.level, LogLevel::Normal);
        assert!(config.color);
        assert!(!config.with_timestamp);
        assert!(!config.with_target);
    }

    #[test]
    fn test_from_args_normal() {
        let config = LogConfig::from_args(false, false, false);
        assert_eq!(config.level, LogLevel::Normal);
        assert!(config.color);
        assert!(!config.with_timestamp);
    }

    #[test]
    fn test_from_args_verbose() {
        let config = LogConfig::from_args(true, false, false);
        assert_eq!(config.level, LogLevel::Verbose);
        assert!(config.with_timestamp);
        assert!(config.with_target);
    }

    #[test]
    fn test_from_args_quiet() {
        let config = LogConfig::from_args(false, true, false);
        assert_eq!(config.level, LogLevel::Quiet);
    }

    #[test]
    fn test_from_args_no_color() {
        let config = LogConfig::from_args(false, false, true);
        assert!(!config.color);
    }

    #[test]
    fn test_level_conversion() {
        assert_eq!(Level::from(LogLevel::Quiet), Level::ERROR);
        assert_eq!(Level::from(LogLevel::Normal), Level::INFO);
        assert_eq!(Level::from(LogLevel::Verbose), Level::DEBUG);
        assert_eq!(Level::from(LogLevel::Trace), Level::TRACE);
    }
}
