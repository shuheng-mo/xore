//! XORE 路径管理模块
//!
//! 提供统一的路径管理功能，确保所有运行时数据存储在 ~/.xore/ 目录下。

use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PathError {
    #[error("无法获取用户主目录")]
    HomeNotFound,
    #[error("无法创建目录: {0}")]
    CannotCreateDir(PathBuf),
}

/// XORE 路径管理器
///
/// 管理所有 XORE 运行时数据的存储路径：
/// - 配置: ~/.xore/config.toml
/// - 索引: ~/.xore/index/
/// - 历史: ~/.xore/history/
/// - 日志: ~/.xore/logs/
/// - 模型: ~/.xore/models/
/// - 缓存: ~/.xore/cache/
#[derive(Debug, Clone)]
pub struct XorePaths {
    /// 用户主目录
    home: PathBuf,
    /// XORE 根目录 (~/.xore)
    xore_dir: PathBuf,
}

impl XorePaths {
    /// 创建新的路径管理器
    ///
    /// # Errors
    ///
    /// 如果无法获取用户主目录，返回错误
    pub fn new() -> Result<Self, PathError> {
        let home = dirs::home_dir().ok_or(PathError::HomeNotFound)?;
        let xore_dir = home.join(".xore");
        Ok(Self { home, xore_dir })
    }

    /// 从指定的主目录创建路径管理器（用于测试）
    #[cfg(test)]
    pub fn from_home(home: PathBuf) -> Self {
        Self { home: home.clone(), xore_dir: home.join(".xore") }
    }

    /// 获取 XORE 根目录 (~/.xore)
    pub fn xore_dir(&self) -> &PathBuf {
        &self.xore_dir
    }

    /// 获取配置目录 (~/.xore)
    pub fn config_dir(&self) -> PathBuf {
        self.xore_dir.clone()
    }

    /// 获取配置文件路径 (~/.xore/config.toml)
    pub fn config_file(&self) -> PathBuf {
        self.xore_dir.join("config.toml")
    }

    /// 获取索引目录 (~/.xore/index)
    pub fn index_dir(&self) -> PathBuf {
        self.xore_dir.join("index")
    }

    /// 获取默认索引路径 (~/.xore/index/default)
    pub fn default_index_dir(&self) -> PathBuf {
        self.index_dir().join("default")
    }

    /// 获取历史记录目录 (~/.xore/history)
    pub fn history_dir(&self) -> PathBuf {
        self.xore_dir.join("history")
    }

    /// 获取日志目录 (~/.xore/logs)
    pub fn logs_dir(&self) -> PathBuf {
        self.xore_dir.join("logs")
    }

    /// 获取模型存储目录 (~/.xore/models)
    pub fn models_dir(&self) -> PathBuf {
        self.xore_dir.join("models")
    }

    /// 获取缓存目录 (~/.xore/cache)
    pub fn cache_dir(&self) -> PathBuf {
        self.xore_dir.join("cache")
    }

    /// 确保所有必要的目录存在
    ///
    /// # Errors
    ///
    /// 如果无法创建任何目录，返回错误
    pub fn ensure_dirs(&self) -> Result<(), PathError> {
        let dirs = [
            self.xore_dir.clone(),
            self.index_dir(),
            self.history_dir(),
            self.logs_dir(),
            self.models_dir(),
            self.cache_dir(),
        ];

        for dir in dirs {
            if !dir.exists() {
                std::fs::create_dir_all(&dir).map_err(|_| PathError::CannotCreateDir(dir))?;
            }
        }

        tracing::debug!("All XORE directories ensured at {:?}", self.xore_dir);
        Ok(())
    }

    /// 展开路径中的 ~ 为实际主目录
    pub fn expand_path(path: &str) -> PathBuf {
        if path.starts_with("~") {
            let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            let rest = path.strip_prefix("~/").unwrap_or("");
            home.join(rest)
        } else {
            PathBuf::from(path)
        }
    }

    /// 将路径转换为相对路径表示（用于配置文件中存储）
    pub fn to_tilde_path(path: &PathBuf) -> String {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        if let Ok(rel) = path.strip_prefix(&home) {
            let mut result = String::from("~/");
            result.push_str(rel.to_string_lossy().as_ref());
            result
        } else {
            path.to_string_lossy().to_string()
        }
    }
}

impl Default for XorePaths {
    fn default() -> Self {
        Self::new().expect("Failed to get home directory")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_expand_path() {
        let temp_dir = TempDir::new().unwrap();
        let paths = XorePaths::from_home(temp_dir.path().to_path_buf());

        // 测试 ~ 展开
        let expanded = XorePaths::expand_path("~/test");
        assert!(expanded.to_string_lossy().contains("test"));

        // 测试普通路径
        let normal = XorePaths::expand_path("/tmp/test");
        assert_eq!(normal, PathBuf::from("/tmp/test"));
    }

    #[test]
    fn test_to_tilde_path() {
        // 使用实际的主目录进行测试
        let home = dirs::home_dir().unwrap();
        let path = home.join("test");
        let tilde = XorePaths::to_tilde_path(&path);
        assert!(tilde.starts_with("~/"));
    }

    #[test]
    fn test_ensure_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let paths = XorePaths::from_home(temp_dir.path().to_path_buf());

        paths.ensure_dirs().unwrap();

        assert!(paths.xore_dir().exists());
        assert!(paths.index_dir().exists());
        assert!(paths.history_dir().exists());
        assert!(paths.logs_dir().exists());
        assert!(paths.models_dir().exists());
        assert!(paths.cache_dir().exists());
    }
}
