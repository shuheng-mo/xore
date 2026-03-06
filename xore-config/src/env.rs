//! XORE 环境变量解析模块
//!
//! 提供从环境变量加载配置覆盖的功能，支持向后兼容。

use std::path::PathBuf;
use std::sync::OnceLock;

use crate::config::Config;

/// 环境变量配置覆盖
///
/// 用于从环境变量加载配置值，实现向后兼容。
/// 环境变量优先级高于配置文件。
#[derive(Debug, Default)]
pub struct EnvOverride {
    /// 日志级别覆盖
    pub log_level: Option<String>,
    /// 线程数覆盖
    pub num_threads: Option<usize>,
    /// 索引路径覆盖
    pub index_path: Option<PathBuf>,
    /// 历史记录路径覆盖
    pub history_path: Option<PathBuf>,
    /// 日志路径覆盖
    pub logs_path: Option<PathBuf>,
    /// 模型路径覆盖
    pub models_path: Option<PathBuf>,
    /// 配置文件路径覆盖
    pub config_path: Option<PathBuf>,
    /// 禁用颜色输出
    pub no_color: bool,
}

impl EnvOverride {
    /// 从环境变量加载配置覆盖
    pub fn from_env() -> Self {
        Self {
            log_level: std::env::var("XORE_LOG_LEVEL").ok(),
            num_threads: std::env::var("XORE_NUM_THREADS").ok().and_then(|v| v.parse().ok()),
            index_path: std::env::var("XORE_INDEX_PATH").ok().map(PathBuf::from),
            history_path: std::env::var("XORE_HISTORY_PATH").ok().map(PathBuf::from),
            logs_path: std::env::var("XORE_LOGS_PATH").ok().map(PathBuf::from),
            models_path: std::env::var("XORE_MODELS_PATH").ok().map(PathBuf::from),
            config_path: std::env::var("XORE_CONFIG_PATH").ok().map(PathBuf::from),
            no_color: std::env::var("NO_COLOR").is_ok(),
        }
    }

    /// 将环境变量覆盖应用到配置
    pub fn apply_to_config(&self, mut config: Config) -> Config {
        // 应用环境变量覆盖
        if let Some(ref log_level) = self.log_level {
            config.env.log_level = log_level.clone();
        }

        if let Some(num_threads) = self.num_threads {
            config.env.num_threads = num_threads;
        }

        if let Some(ref index_path) = self.index_path {
            config.paths.index = index_path.clone();
        }

        if let Some(ref history_path) = self.history_path {
            config.paths.history = history_path.clone();
        }

        if let Some(ref logs_path) = self.logs_path {
            config.paths.logs = logs_path.clone();
        }

        if let Some(ref models_path) = self.models_path {
            config.paths.models = models_path.clone();
        }

        // NO_COLOR 覆盖 UI 颜色设置
        if self.no_color {
            config.ui.color = false;
        }

        config
    }
}

/// 全局配置缓存
static CONFIG_CACHE: OnceLock<Config> = OnceLock::new();

/// 获取全局配置（带缓存）
///
/// 使用环境变量覆盖配置文件设置。
pub fn get_config() -> Config {
    CONFIG_CACHE
        .get_or_init(|| {
            let config = Config::load_with_defaults();
            let env_override = EnvOverride::from_env();
            env_override.apply_to_config(config)
        })
        .clone()
}

/// 获取全局路径管理器
pub fn get_paths() -> crate::paths::XorePaths {
    crate::paths::XorePaths::new().expect("Failed to get home directory")
}

/// 初始化配置系统
///
/// 确保所有必要的目录存在，并返回配置。
pub fn init() -> Config {
    let config = get_config();

    // 确保目录存在
    if let Ok(paths) = crate::paths::XorePaths::new() {
        if let Err(e) = paths.ensure_dirs() {
            tracing::warn!("Failed to create XORE directories: {}", e);
        }
    }

    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_override_from_env() {
        // 设置环境变量
        std::env::set_var("XORE_LOG_LEVEL", "debug");
        std::env::set_var("XORE_NUM_THREADS", "8");

        let env_override = EnvOverride::from_env();

        assert_eq!(env_override.log_level, Some("debug".to_string()));
        assert_eq!(env_override.num_threads, Some(8));

        // 清理
        std::env::remove_var("XORE_LOG_LEVEL");
        std::env::remove_var("XORE_NUM_THREADS");
    }

    #[test]
    fn test_apply_override() {
        let env_override = EnvOverride {
            log_level: Some("debug".to_string()),
            num_threads: Some(4),
            ..Default::default()
        };

        let config = Config::default();
        let result = env_override.apply_to_config(config);

        assert_eq!(result.env.log_level, "debug");
        assert_eq!(result.env.num_threads, 4);
    }

    #[test]
    fn test_no_color_override() {
        std::env::set_var("NO_COLOR", "1");

        let env_override = EnvOverride::from_env();
        assert!(env_override.no_color);

        let config = Config::default();
        let result = env_override.apply_to_config(config);

        assert!(!result.ui.color);

        std::env::remove_var("NO_COLOR");
    }
}
