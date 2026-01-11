//! 共享类型定义

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub path: PathBuf,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub score: f32,
    pub snippet: Option<String>,
}

/// 文件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileType {
    Text,
    Csv,
    Json,
    Log,
    Code,
    Binary,
    Unknown,
}

/// 数据质量报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQualityReport {
    pub row_count: usize,
    pub column_count: usize,
    pub has_nulls: bool,
    pub has_duplicates: bool,
    pub has_outliers: bool,
    pub suggestions: Vec<String>,
}
