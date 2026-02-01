//! 配置管理

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// XORE全局配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub search: SearchConfig,
    pub process: ProcessConfig,
    pub ai: AiConfig,
    pub limits: LimitsConfig,
    pub ui: UiConfig,
    pub exclude: ExcludeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// 全局索引路径 (默认 ~/.xore/index)
    pub global_index_path: PathBuf,
    /// 是否使用项目级索引 (.xore/index)
    pub use_project_index: bool,
    /// 项目级索引路径 (相对于项目根目录)
    pub project_index_path: PathBuf,
    /// 并行线程数
    pub num_threads: usize,
    /// 自动重建索引的天数
    pub auto_rebuild_days: u32,
    /// 最大索引大小 (GB)
    pub max_index_size_gb: usize,
    /// 单文件最大大小 (MB)，超过此大小的文件不会被索引
    pub max_file_size_mb: usize,
    /// Writer 缓冲区大小 (MB)
    pub writer_buffer_mb: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessConfig {
    pub lazy_execution: bool,
    pub chunk_size_mb: usize,
    pub cache_size_mb: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub model_path: PathBuf,
    pub enable_semantic: bool,
    pub embedding_dim: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitsConfig {
    pub max_file_size_mb: usize,
    pub max_memory_mb: usize,
    pub max_query_time_ms: u64,
    pub max_results: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: String,
    pub progress_bar: bool,
    pub color_output: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcludeConfig {
    pub patterns: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            search: SearchConfig {
                global_index_path: PathBuf::from("~/.xore/index"),
                use_project_index: true,
                project_index_path: PathBuf::from(".xore/index"),
                num_threads: num_cpus::get(),
                auto_rebuild_days: 30,
                max_index_size_gb: 10,
                max_file_size_mb: 100,
                writer_buffer_mb: 50,
            },
            process: ProcessConfig { lazy_execution: true, chunk_size_mb: 64, cache_size_mb: 512 },
            ai: AiConfig {
                model_path: PathBuf::from("~/.xore/models/minilm-l6-v2.onnx"),
                enable_semantic: true,
                embedding_dim: 384,
            },
            limits: LimitsConfig {
                max_file_size_mb: 100,
                max_memory_mb: 2048,
                max_query_time_ms: 5000,
                max_results: 1000,
            },
            ui: UiConfig { theme: "dark".to_string(), progress_bar: true, color_output: true },
            exclude: ExcludeConfig {
                patterns: vec![
                    ".git".to_string(),
                    "node_modules".to_string(),
                    "target".to_string(),
                    "*.lock".to_string(),
                ],
            },
        }
    }
}

impl Config {
    /// 从文件加载配置
    pub fn load(path: &std::path::Path) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config =
            toml::from_str(&content).map_err(|e| crate::XoreError::ConfigError(e.to_string()))?;
        Ok(config)
    }

    /// 保存配置到文件
    pub fn save(&self, path: &std::path::Path) -> crate::Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| crate::XoreError::ConfigError(e.to_string()))?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
