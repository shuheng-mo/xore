//! 搜索历史记录模块
//!
//! 提供搜索历史存储和查询功能，使用 JSON 文件存储。

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use tracing::{debug, error, info};

/// 搜索类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SearchType {
    /// 全文搜索
    #[default]
    FullText,
    /// 语义搜索
    Semantic,
    /// 文件类型过滤
    FileType,
    /// 语义搜索 + 过滤
    SemanticWithFilter,
}

impl std::fmt::Display for SearchType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchType::FullText => write!(f, "全文搜索"),
            SearchType::Semantic => write!(f, "语义搜索"),
            SearchType::FileType => write!(f, "文件类型"),
            SearchType::SemanticWithFilter => write!(f, "语义+过滤"),
        }
    }
}

/// 搜索历史记录条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHistoryEntry {
    /// 唯一ID
    pub id: u64,
    /// 搜索查询
    pub query: String,
    /// 搜索类型
    pub search_type: SearchType,
    /// 搜索路径
    pub path: String,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 结果数量
    pub result_count: usize,
    /// 执行时间（毫秒）
    pub execution_time_ms: u64,
    /// 文件类型过滤（可选）
    pub file_type: Option<String>,
}

impl SearchHistoryEntry {
    /// 创建新的历史记录条目
    pub fn new(
        query: String,
        search_type: SearchType,
        path: String,
        result_count: usize,
        execution_time_ms: u64,
        file_type: Option<String>,
    ) -> Self {
        Self {
            id: 0, // 由数据库分配
            query,
            search_type,
            path,
            timestamp: Utc::now(),
            result_count,
            execution_time_ms,
            file_type,
        }
    }
}

/// 搜索统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchStats {
    /// 查询字符串
    pub query: String,
    /// 搜索次数
    pub count: usize,
    /// 平均结果数
    pub avg_result_count: f64,
    /// 平均执行时间（毫秒）
    pub avg_execution_time_ms: f64,
    /// 最常搜索的路径
    pub most_common_path: Option<String>,
    /// 最常使用的搜索类型
    pub most_common_type: Option<SearchType>,
}

/// 历史记录存储
pub struct HistoryStore {
    /// 数据库路径
    db_path: PathBuf,
    /// 内存缓存（用于快速查询）
    entries: Mutex<Vec<SearchHistoryEntry>>,
    /// 下一个ID
    next_id: Mutex<u64>,
    /// 查询统计缓存
    stats_cache: Mutex<HashMap<String, SearchStats>>,
}

impl HistoryStore {
    /// 创建新的历史记录存储
    pub fn new(db_path: PathBuf) -> Result<Self> {
        // 确保目录存在 - 直接创建完整路径
        std::fs::create_dir_all(&db_path)
            .with_context(|| format!("Failed to create history directory: {:?}", db_path))?;

        // 验证目录确实存在
        if !db_path.is_dir() {
            return Err(anyhow::anyhow!("Failed to create history directory: {:?}", db_path));
        }

        info!("Initializing history store at: {:?}", db_path);

        let store = Self {
            db_path,
            entries: Mutex::new(Vec::new()),
            next_id: Mutex::new(1),
            stats_cache: Mutex::new(HashMap::new()),
        };

        // 加载现有数据（如果存在）
        store.load_from_disk()?;

        Ok(store)
    }

    /// 从磁盘加载历史记录
    fn load_from_disk(&self) -> Result<()> {
        let history_file = self.db_path.join("history.json");

        if history_file.exists() {
            let content = std::fs::read_to_string(&history_file)
                .with_context(|| "Failed to read history file")?;

            let entries: Vec<SearchHistoryEntry> =
                serde_json::from_str(&content).with_context(|| "Failed to parse history file")?;

            let max_id = entries.iter().map(|e| e.id).max().unwrap_or(0);

            let mut entries_guard = self.entries.lock().unwrap();
            *entries_guard = entries;

            let mut next_id_guard = self.next_id.lock().unwrap();
            *next_id_guard = max_id + 1;

            info!("Loaded {} history entries", entries_guard.len());
        }

        Ok(())
    }

    /// 保存历史记录到磁盘
    #[allow(dead_code)]
    fn save_to_disk(&self) -> Result<()> {
        let history_file = self.db_path.join("history.json");

        // 验证目录存在
        if !self.db_path.is_dir() {
            error!("History directory does not exist: {:?}", self.db_path);
            return Err(anyhow::anyhow!("History directory does not exist: {:?}", self.db_path));
        }

        let entries = self.entries.lock().unwrap();
        let content = serde_json::to_string_pretty(&*entries)
            .with_context(|| "Failed to serialize history")?;

        // 添加更详细的错误处理
        if let Err(e) = std::fs::write(&history_file, &content) {
            error!("Failed to write history file: {:?}, error: {}", history_file, e);
            return Err(anyhow::anyhow!(
                "Failed to write history file: {}, error: {}",
                history_file.display(),
                e
            ))
            .context("Failed to write history file");
        }

        debug!("Saved {} history entries to disk", entries.len());

        Ok(())
    }

    /// 记录一次搜索
    pub fn record_search(&self, entry: SearchHistoryEntry) -> Result<()> {
        let mut entries = self.entries.lock().unwrap();
        let mut next_id = self.next_id.lock().unwrap();

        let mut new_entry = entry;
        new_entry.id = *next_id;
        *next_id += 1;

        entries.push(new_entry.clone());

        // 限制最大记录数（默认1000条）
        let max_entries = 1000;
        if entries.len() > max_entries {
            // 保留最新的记录
            let remove_count = entries.len() - max_entries;
            entries.drain(0..remove_count);
        }

        // 同步保存到磁盘
        // 先验证目录存在
        if !self.db_path.is_dir() {
            error!("History directory does not exist when saving: {:?}", self.db_path);
            return Err(anyhow::anyhow!("History directory does not exist: {:?}", self.db_path));
        }

        let history_file = self.db_path.join("history.json");
        let content = serde_json::to_string_pretty(&*entries)
            .with_context(|| "Failed to serialize history")?;

        if let Err(e) = std::fs::write(&history_file, &content) {
            error!("Failed to write history file: {:?}, error: {}", history_file, e);
            return Err(anyhow::anyhow!(
                "Failed to write history file: {}, error: {}",
                history_file.display(),
                e
            ))
            .context("Failed to write history file");
        }

        // 清除统计缓存
        drop(entries);
        let mut stats_cache = self.stats_cache.lock().unwrap();
        stats_cache.clear();

        info!(
            "Recorded search: '{}' (type: {}, path: {}, results: {})",
            new_entry.query, new_entry.search_type, new_entry.path, new_entry.result_count
        );

        Ok(())
    }

    /// 获取最近的搜索历史
    pub fn get_recent_searches(&self, limit: usize) -> Vec<SearchHistoryEntry> {
        let entries = self.entries.lock().unwrap();
        let limit = limit.min(entries.len());

        entries.iter().rev().take(limit).cloned().collect()
    }

    /// 获取搜索统计信息
    pub fn get_search_stats(&self, query: &str) -> Option<SearchStats> {
        // 先检查缓存
        {
            let stats_cache = self.stats_cache.lock().unwrap();
            if let Some(cached) = stats_cache.get(query) {
                return Some(cached.clone());
            }
        }

        let entries = self.entries.lock().unwrap();

        // 查找匹配的历史记录
        let matching: Vec<_> = entries
            .iter()
            .filter(|e| e.query.to_lowercase().contains(&query.to_lowercase()))
            .collect();

        if matching.is_empty() {
            return None;
        }

        let count = matching.len();
        let avg_result_count =
            matching.iter().map(|e| e.result_count).sum::<usize>() as f64 / count as f64;
        let avg_execution_time_ms =
            matching.iter().map(|e| e.execution_time_ms).sum::<u64>() as f64 / count as f64;

        // 统计最常见的路径
        let mut path_counts: HashMap<String, usize> = HashMap::new();
        for entry in &matching {
            *path_counts.entry(entry.path.clone()).or_insert(0) += 1;
        }
        let most_common_path =
            path_counts.into_iter().max_by_key(|(_, count)| *count).map(|(path, _)| path);

        // 统计最常见的搜索类型
        let mut type_counts: HashMap<SearchType, usize> = HashMap::new();
        for entry in &matching {
            *type_counts.entry(entry.search_type).or_insert(0) += 1;
        }
        let most_common_type =
            type_counts.into_iter().max_by_key(|(_, count)| *count).map(|(t, _)| t);

        let stats = SearchStats {
            query: query.to_string(),
            count,
            avg_result_count,
            avg_execution_time_ms,
            most_common_path,
            most_common_type,
        };

        // 缓存结果
        let mut stats_cache = self.stats_cache.lock().unwrap();
        stats_cache.insert(query.to_string(), stats.clone());

        Some(stats)
    }

    /// 获取所有查询的频次统计
    pub fn get_query_frequencies(&self, limit: usize) -> Vec<(String, usize)> {
        let entries = self.entries.lock().unwrap();

        let mut freq: HashMap<String, usize> = HashMap::new();
        for entry in entries.iter() {
            // 标准化查询（转小写，去除多余空格）
            let normalized = entry.query.to_lowercase();
            if !normalized.is_empty() {
                *freq.entry(normalized).or_insert(0) += 1;
            }
        }

        let mut sorted: Vec<_> = freq.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1)); // 按频次降序

        sorted.into_iter().take(limit).collect()
    }

    /// 获取路径-文件类型关联统计
    pub fn get_path_type_associations(&self) -> HashMap<String, HashMap<String, usize>> {
        let entries = self.entries.lock().unwrap();

        let mut associations: HashMap<String, HashMap<String, usize>> = HashMap::new();

        for entry in entries.iter() {
            if let Some(ref file_type) = entry.file_type {
                let path_associations = associations.entry(entry.path.clone()).or_default();
                *path_associations.entry(file_type.clone()).or_insert(0) += 1;
            }
        }

        associations
    }

    /// 清除所有历史记录
    pub fn clear(&self) -> Result<usize> {
        let mut entries = self.entries.lock().unwrap();
        let count = entries.len();
        entries.clear();

        // 清除统计缓存
        let mut stats_cache = self.stats_cache.lock().unwrap();
        stats_cache.clear();

        // 删除磁盘文件
        let history_file = self.db_path.join("history.json");
        if history_file.exists() {
            std::fs::remove_file(&history_file)?;
        }

        info!("Cleared {} history entries", count);

        Ok(count)
    }

    /// 获取历史记录总数
    pub fn len(&self) -> usize {
        self.entries.lock().unwrap().len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.entries.lock().unwrap().is_empty()
    }
}

/// 获取默认的历史记录存储路径
pub fn get_default_history_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".xore").join("history")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_store() -> (HistoryStore, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let store = HistoryStore::new(temp_dir.path().to_path_buf()).unwrap();
        (store, temp_dir)
    }

    #[test]
    fn test_record_search() {
        let (store, _temp) = create_test_store();

        let entry = SearchHistoryEntry::new(
            "error".to_string(),
            SearchType::FullText,
            "./src".to_string(),
            15,
            23,
            None,
        );

        store.record_search(entry).unwrap();

        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_get_recent_searches() {
        let (store, _temp) = create_test_store();

        for i in 0..15 {
            let entry = SearchHistoryEntry::new(
                format!("query{}", i),
                SearchType::FullText,
                "./src".to_string(),
                i,
                10,
                None,
            );
            store.record_search(entry).unwrap();
        }

        let recent = store.get_recent_searches(5);
        assert_eq!(recent.len(), 5);
        // 最新的应该在最前面
        assert!(recent[0].query.contains("14"));
    }

    #[test]
    fn test_get_query_frequencies() {
        let (store, _temp) = create_test_store();

        // 记录相同查询多次
        for _ in 0..3 {
            let entry = SearchHistoryEntry::new(
                "error".to_string(),
                SearchType::FullText,
                "./src".to_string(),
                10,
                10,
                None,
            );
            store.record_search(entry).unwrap();
        }

        // 记录不同查询
        let entry = SearchHistoryEntry::new(
            "warning".to_string(),
            SearchType::FullText,
            "./src".to_string(),
            5,
            10,
            None,
        );
        store.record_search(entry).unwrap();

        let freq = store.get_query_frequencies(10);
        assert_eq!(freq.len(), 2);
        assert_eq!(freq[0].1, 3); // "error" 出现3次
        assert_eq!(freq[1].1, 1); // "warning" 出现1次
    }

    #[test]
    fn test_clear() {
        let (store, _temp) = create_test_store();

        let entry = SearchHistoryEntry::new(
            "error".to_string(),
            SearchType::FullText,
            "./src".to_string(),
            10,
            10,
            None,
        );
        store.record_search(entry).unwrap();

        assert!(!store.is_empty());

        let count = store.clear().unwrap();
        assert_eq!(count, 1);
        assert!(store.is_empty());
    }

    #[test]
    fn test_get_search_stats() {
        let (store, _temp) = create_test_store();

        // 记录相同查询多次
        for i in 0..3u64 {
            let entry = SearchHistoryEntry::new(
                "error".to_string(),
                SearchType::FullText,
                "./src".to_string(),
                10 + i as usize,
                20 + i,
                None,
            );
            store.record_search(entry).unwrap();
        }

        let stats = store.get_search_stats("error").unwrap();
        assert_eq!(stats.count, 3);
        assert!((stats.avg_result_count - 11.0).abs() < 0.1);
    }
}
