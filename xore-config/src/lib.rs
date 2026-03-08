//! XORE 配置管理库
//!
//! 提供统一的配置加载、路径管理和环境变量解析功能。
//!
//! ## 核心功能
//!
//! - **路径管理**: 统一管理所有运行时数据的存储路径
//! - **配置加载**: 支持多位置配置加载和默认值
//! - **环境变量**: 支持环境变量覆盖配置文件
//!
//! ## 使用示例
//!
//! ```rust
//! use xore_config::{get_config, get_paths, init};
//!
//! // 初始化配置系统（确保目录存在）
//! let config = init();
//!
//! // 获取配置
//! let config = get_config();
//! println!("Log level: {}", config.env.log_level);
//!
//! // 获取路径管理器
//! let paths = get_paths();
//! println!("Index dir: {:?}", paths.index_dir());
//! ```

pub mod config;
pub mod env;
pub mod paths;

// 导出主要类型
pub use config::{
    Config, ConfigError, ContextConfig, EnvConfig, ExcludeConfig, OutputConfig, PathsConfig,
    SearchConfig, UiConfig,
};
pub use env::{get_config, get_paths, init, EnvOverride};
pub use paths::{PathError, XorePaths};
