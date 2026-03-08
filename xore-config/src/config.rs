//! XORE 配置管理模块
//!
//! 提供统一的配置加载、保存和合并功能。

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

use crate::paths::XorePaths;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("配置文件解析失败: {0}")]
    ParseError(String),
    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),
    #[error("TOML 序列化错误: {0}")]
    TomlError(#[from] toml::ser::Error),
}

/// 运行时环境配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EnvConfig {
    /// 日志级别: error, warn, info, debug, trace
    #[serde(default = "default_log_level")]
    pub log_level: String,
    /// 工作线程数（0 = 自动检测 CPU 核心数）
    #[serde(default = "default_num_threads")]
    pub num_threads: usize,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_num_threads() -> usize {
    0
}

impl Default for EnvConfig {
    fn default() -> Self {
        Self { log_level: "info".to_string(), num_threads: 0 }
    }
}

/// 存储路径配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathsConfig {
    /// 索引存储路径
    #[serde(default = "default_index_path")]
    pub index: PathBuf,
    /// 历史记录存储路径
    #[serde(default = "default_history_path")]
    pub history: PathBuf,
    /// 日志存储路径
    #[serde(default = "default_logs_path")]
    pub logs: PathBuf,
    /// AI 模型存储路径
    #[serde(default = "default_models_path")]
    pub models: PathBuf,
}

fn default_index_path() -> PathBuf {
    XorePaths::expand_path("~/.xore/index")
}

fn default_history_path() -> PathBuf {
    XorePaths::expand_path("~/.xore/history")
}

fn default_logs_path() -> PathBuf {
    XorePaths::expand_path("~/.xore/logs")
}

fn default_models_path() -> PathBuf {
    XorePaths::expand_path("~/.xore/models")
}

impl Default for PathsConfig {
    fn default() -> Self {
        Self {
            index: default_index_path(),
            history: default_history_path(),
            logs: default_logs_path(),
            models: default_models_path(),
        }
    }
}

/// 搜索配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// 是否使用项目级索引（优先于全局索引）
    #[serde(default = "default_true")]
    pub use_project_index: bool,
    /// 项目级索引路径（相对于项目根目录）
    #[serde(default = "default_project_index_path")]
    pub project_index_path: String,
    /// 单文件最大大小（MB），超过不索引
    #[serde(default = "default_max_file_size_mb")]
    pub max_file_size_mb: usize,
    /// 索引 Writer 缓冲区大小（MB）
    #[serde(default = "default_writer_buffer_mb")]
    pub writer_buffer_mb: usize,
}

fn default_true() -> bool {
    true
}

fn default_project_index_path() -> String {
    ".xore/index".to_string()
}

fn default_max_file_size_mb() -> usize {
    100
}

fn default_writer_buffer_mb() -> usize {
    50
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            use_project_index: true,
            project_index_path: ".xore/index".to_string(),
            max_file_size_mb: 100,
            writer_buffer_mb: 50,
        }
    }
}

/// 排除模式配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcludeConfig {
    /// 全局排除模式
    #[serde(default = "default_exclude_patterns")]
    pub patterns: Vec<String>,
}

fn default_exclude_patterns() -> Vec<String> {
    vec![
        "**/node_modules/**".to_string(),
        "**/.git/**".to_string(),
        "**/target/**".to_string(),
        "**/__pycache__/**".to_string(),
        "**/.DS_Store/**".to_string(),
        "**/Thumbs.db/**".to_string(),
    ]
}

impl Default for ExcludeConfig {
    fn default() -> Self {
        Self { patterns: default_exclude_patterns() }
    }
}

/// 界面配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// 主题: light, dark, auto
    #[serde(default = "default_theme")]
    pub theme: String,
    /// 是否显示进度条
    #[serde(default = "default_true")]
    pub progress_bar: bool,
    /// 是否使用彩色输出
    #[serde(default = "default_true")]
    pub color: bool,
}

fn default_theme() -> String {
    "auto".to_string()
}

impl Default for UiConfig {
    fn default() -> Self {
        Self { theme: "auto".to_string(), progress_bar: true, color: true }
    }
}

/// 输出配置 - Token 节省可视化
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// 是否显示 Token 节省信息
    #[serde(default = "default_true")]
    pub show_savings: bool,
    /// 节省信息模式: minimal, detailed, cumulative
    #[serde(default = "default_savings_mode")]
    pub savings_mode: String,
    /// 货币单位: auto, usd, cny
    #[serde(default = "default_currency")]
    pub currency: String,
}

fn default_savings_mode() -> String {
    "minimal".to_string()
}

fn default_currency() -> String {
    "auto".to_string()
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            show_savings: true,
            savings_mode: "minimal".to_string(),
            currency: "auto".to_string(),
        }
    }
}

/// 会话上下文配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    /// 上下文过期时间（小时）
    #[serde(default = "default_session_ttl_hours")]
    pub session_ttl_hours: u64,
    /// 最大历史记录数
    #[serde(default = "default_max_context_history")]
    pub max_history: usize,
}

fn default_session_ttl_hours() -> u64 {
    24
}

fn default_max_context_history() -> usize {
    1000
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self { session_ttl_hours: 24, max_history: 1000 }
    }
}

/// XORE 全局配置（极简设计）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// 运行时环境配置
    #[serde(default)]
    pub env: EnvConfig,
    /// 存储路径配置
    #[serde(default)]
    pub paths: PathsConfig,
    /// 搜索配置
    #[serde(default)]
    pub search: SearchConfig,
    /// 排除模式配置
    #[serde(default)]
    pub exclude: ExcludeConfig,
    /// 界面配置
    #[serde(default)]
    pub ui: UiConfig,
    /// 输出配置
    #[serde(default)]
    pub output: OutputConfig,
    /// 会话上下文配置
    #[serde(default)]
    pub context: ContextConfig,
}

impl Config {
    /// 从文件加载配置
    ///
    /// # Errors
    ///
    /// 如果文件不存在或解析失败，返回错误
    pub fn load(path: &std::path::Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let config: Config =
            toml::from_str(&content).map_err(|e| ConfigError::ParseError(e.to_string()))?;
        Ok(config)
    }

    /// 保存配置到文件
    ///
    /// # Errors
    ///
    /// 如果序列化或写入失败，返回错误
    pub fn save(&self, path: &std::path::Path) -> Result<(), ConfigError> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// 加载配置（支持多位置）
    ///
    /// 优先级：
    /// 1. 命令行指定的配置文件
    /// 2. 环境变量 XORE_CONFIG_PATH 指定的文件
    /// 3. ~/.xore/config.toml
    /// 4. 默认配置
    pub fn load_with_defaults() -> Self {
        // 1. 首先尝试环境变量
        if let Ok(path) = std::env::var("XORE_CONFIG_PATH") {
            let path = PathBuf::from(path);
            if path.exists() {
                if let Ok(config) = Self::load(&path) {
                    tracing::debug!("Loaded config from XORE_CONFIG_PATH: {:?}", path);
                    return config;
                }
            }
        }

        // 2. 尝试默认配置文件
        if let Ok(paths) = XorePaths::new() {
            let config_file = paths.config_file();
            if config_file.exists() {
                if let Ok(config) = Self::load(&config_file) {
                    tracing::debug!("Loaded config from {:?}", config_file);
                    return config;
                }
            }
        }

        // 3. 返回默认配置
        tracing::debug!("Using default config");
        Self::default()
    }

    /// 创建默认配置文件
    pub fn create_default_config() -> Result<Self, ConfigError> {
        let config = Self::default();

        // 确保目录存在
        if let Ok(paths) = XorePaths::new() {
            if let Err(e) = paths.ensure_dirs() {
                tracing::warn!("Failed to create XORE directories: {}", e);
            }

            // 如果配置文件不存在，创建默认配置
            let config_file = paths.config_file();
            if !config_file.exists() {
                config.save(&config_file)?;
                tracing::info!("Created default config at {:?}", config_file);
            }
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = Config::default();

        assert_eq!(config.env.log_level, "info");
        assert_eq!(config.env.num_threads, 0);
        assert!(config.search.use_project_index);
        assert!(config.ui.color);
    }

    #[test]
    fn test_config_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let config = Config::default();
        config.save(&config_path).unwrap();

        let loaded = Config::load(&config_path).unwrap();

        assert_eq!(config.env.log_level, loaded.env.log_level);
        assert_eq!(config.search.use_project_index, loaded.search.use_project_index);
    }

    #[test]
    fn test_exclude_patterns() {
        let config = Config::default();

        assert!(config.exclude.patterns.contains(&"**/node_modules/**".to_string()));
        assert!(config.exclude.patterns.contains(&"**/.git/**".to_string()));
    }
}
