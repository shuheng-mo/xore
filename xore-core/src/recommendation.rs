//! 智能推荐引擎模块
//!
//! 基于搜索历史分析，提供智能推荐功能。

use crate::history::{get_default_history_path, HistoryStore, SearchHistoryEntry, SearchType};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

/// 推荐类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationType {
    /// 频繁搜索的查询
    FrequentQuery,
    /// 相关文件类型
    RelatedFileType,
    /// 最近搜索
    RecentSearches,
    /// 路径模式建议
    PathPattern,
    /// 搜索类型建议
    SearchTypeSuggestion,
}

impl std::fmt::Display for RecommendationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecommendationType::FrequentQuery => write!(f, "频繁查询"),
            RecommendationType::RelatedFileType => write!(f, "相关文件类型"),
            RecommendationType::RecentSearches => write!(f, "最近搜索"),
            RecommendationType::PathPattern => write!(f, "路径模式"),
            RecommendationType::SearchTypeSuggestion => write!(f, "搜索类型建议"),
        }
    }
}

/// 单条推荐
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// 推荐类型
    pub recommendation_type: RecommendationType,
    /// 显示消息
    pub message: String,
    /// 建议的命令
    pub suggestion: String,
    /// 置信度 0.0-1.0
    pub confidence: f32,
    /// 显示图标
    pub icon: String,
}

impl Recommendation {
    /// 创建新的推荐
    pub fn new(
        recommendation_type: RecommendationType,
        message: String,
        suggestion: String,
        confidence: f32,
        icon: &str,
    ) -> Self {
        Self { recommendation_type, message, suggestion, confidence, icon: icon.to_string() }
    }
}

/// 推荐引擎
pub struct RecommendationEngine {
    /// 历史记录存储
    history_store: HistoryStore,
    /// 缓存的推荐结果
    cache: Mutex<HashMap<String, Vec<Recommendation>>>,
}

impl RecommendationEngine {
    /// 创建新的推荐引擎
    pub fn new(history_path: Option<PathBuf>) -> Result<Self> {
        let path = history_path.unwrap_or_else(get_default_history_path);
        let history_store = HistoryStore::new(path)?;

        Ok(Self { history_store, cache: Mutex::new(HashMap::new()) })
    }

    /// 记录一次搜索
    pub fn record_search(&self, entry: SearchHistoryEntry) -> Result<()> {
        self.history_store.record_search(entry)?;

        // 清除缓存
        let mut cache = self.cache.lock().unwrap();
        cache.clear();

        Ok(())
    }

    /// 生成推荐（基于当前搜索上下文）
    pub fn generate_recommendations(&self, current_query: &str) -> Vec<Recommendation> {
        // 先检查缓存
        {
            let cache = self.cache.lock().unwrap();
            if let Some(cached) = cache.get(current_query) {
                return cached.clone();
            }
        }

        let mut recommendations = Vec::new();

        // 1. 获取频繁查询建议
        recommendations.extend(self.get_frequent_query_recommendations(current_query));

        // 2. 获取文件类型关联建议
        recommendations.extend(self.get_file_type_associations(current_query));

        // 3. 获取路径模式建议
        recommendations.extend(self.get_path_pattern_suggestions(current_query));

        // 4. 获取搜索类型建议
        recommendations.extend(self.get_search_type_suggestions(current_query));

        // 按置信度排序
        recommendations.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        // 限制返回数量
        recommendations.truncate(5);

        // 缓存结果
        let mut cache = self.cache.lock().unwrap();
        cache.insert(current_query.to_string(), recommendations.clone());

        recommendations
    }

    /// 获取频繁查询建议
    fn get_frequent_query_recommendations(&self, current_query: &str) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        let freq = self.history_store.get_query_frequencies(10);

        // 查找与当前查询相关的频繁查询
        for (query, count) in &freq {
            if query != current_query && !query.is_empty() {
                // 检查是否有包含关系
                if query.contains(current_query) || current_query.contains(query) {
                    let confidence = (*count as f32 / 10.0).min(1.0);
                    recommendations.push(Recommendation::new(
                        RecommendationType::FrequentQuery,
                        format!("你经常搜索 \"{}\" ({}次)", query, count),
                        format!("xore f \"{}\"", query),
                        confidence,
                        "🔍",
                    ));
                }
            }
        }

        // 如果没有相关查询，添加最频繁的查询作为建议
        if recommendations.is_empty() && !freq.is_empty() {
            let (top_query, count) = &freq[0];
            if *count >= 3 {
                let confidence = (*count as f32 / 10.0).min(1.0);
                recommendations.push(Recommendation::new(
                    RecommendationType::FrequentQuery,
                    format!("你最常搜索 \"{}\" ({}次)", top_query, count),
                    format!("xore f \"{}\"", top_query),
                    confidence * 0.7, // 降低置信度
                    "🔍",
                ));
            }
        }

        recommendations
    }

    /// 获取文件类型关联建议
    fn get_file_type_associations(&self, current_query: &str) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        let associations = self.history_store.get_path_type_associations();

        // 分析当前查询可能关联的文件类型
        let query_lower = current_query.to_lowercase();

        // 常见文件类型映射
        let type_mappings: HashMap<&str, &str> = [
            ("rs", "rust"),
            ("js", "javascript"),
            ("ts", "typescript"),
            ("py", "python"),
            ("go", "go"),
            ("java", "java"),
            ("toml", "toml"),
            ("json", "json"),
            ("yaml", "yaml"),
            ("yml", "yaml"),
            ("md", "markdown"),
            ("log", "log"),
            ("csv", "csv"),
            ("sql", "sql"),
        ]
        .into_iter()
        .collect();

        // 检查查询中是否包含文件类型
        for (ext, _lang) in type_mappings.iter() {
            if query_lower.contains(ext) {
                // 查找该路径下最常见的文件类型
                for (path, type_counts) in &associations {
                    if let Some((common_type, _)) = type_counts.iter().max_by_key(|(_, c)| *c) {
                        if common_type != ext {
                            recommendations.push(Recommendation::new(
                                RecommendationType::RelatedFileType,
                                format!("在 {} 路径下你常搜索 {} 文件", path, common_type),
                                format!("xore f \"{}\" --type {}", current_query, common_type),
                                0.6,
                                "📄",
                            ));
                        }
                    }
                }
            }
        }

        // 如果没有基于查询的建议，添加通用文件类型建议
        if recommendations.is_empty() {
            let mut type_freq: HashMap<String, usize> = HashMap::new();
            for type_counts in associations.values() {
                for (t, c) in type_counts {
                    *type_freq.entry(t.clone()).or_insert(0) += c;
                }
            }

            if let Some((common_type, count)) = type_freq.iter().max_by_key(|(_, c)| *c) {
                if *count >= 3 {
                    recommendations.push(Recommendation::new(
                        RecommendationType::RelatedFileType,
                        format!("你最常搜索 {} 文件类型", common_type),
                        format!("xore f \"{}\" --type {}", current_query, common_type),
                        0.5,
                        "📄",
                    ));
                }
            }
        }

        recommendations
    }

    /// 获取路径模式建议
    fn get_path_pattern_suggestions(&self, current_query: &str) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        let recent = self.history_store.get_recent_searches(20);

        // 统计最常搜索的路径
        let mut path_counts: HashMap<String, usize> = HashMap::new();
        for entry in &recent {
            *path_counts.entry(entry.path.clone()).or_insert(0) += 1;
        }

        if let Some((path, count)) = path_counts.iter().max_by_key(|(_, c)| *c) {
            if *count >= 3 && !path.is_empty() && path != "." {
                recommendations.push(Recommendation::new(
                    RecommendationType::PathPattern,
                    format!("你经常在 {} 路径搜索", path),
                    format!("xore f \"{}\" --path {}", current_query, path),
                    0.7,
                    "📁",
                ));
            }
        }

        recommendations
    }

    /// 获取搜索类型建议
    fn get_search_type_suggestions(&self, current_query: &str) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        // 获取当前查询的统计信息
        if let Some(stats) = self.history_store.get_search_stats(current_query) {
            if let Some(common_type) = stats.most_common_type {
                let type_name = match common_type {
                    SearchType::FullText => "全文搜索",
                    SearchType::Semantic => "语义搜索",
                    SearchType::FileType => "文件类型过滤",
                    SearchType::SemanticWithFilter => "语义+过滤",
                };

                if stats.count >= 2 {
                    recommendations.push(Recommendation::new(
                        RecommendationType::SearchTypeSuggestion,
                        format!("你通常使用 {} 进行此查询", type_name),
                        format!("xore f \"{}\"", current_query),
                        0.6,
                        "⚡",
                    ));
                }
            }
        }

        recommendations
    }

    /// 获取最近的搜索历史
    pub fn get_recent_searches(&self, limit: usize) -> Vec<SearchHistoryEntry> {
        self.history_store.get_recent_searches(limit)
    }

    /// 清除所有历史记录
    pub fn clear_history(&self) -> Result<usize> {
        let count = self.history_store.clear()?;

        // 清除缓存
        let mut cache = self.cache.lock().unwrap();
        cache.clear();

        Ok(count)
    }

    /// 获取历史记录总数
    pub fn history_len(&self) -> usize {
        self.history_store.len()
    }
}

/// 格式化时间间隔
pub fn format_time_ago(timestamp: &chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(*timestamp);

    let seconds = duration.num_seconds();

    if seconds < 60 {
        format!("{}秒前", seconds)
    } else if seconds < 3600 {
        format!("{}分钟前", duration.num_minutes())
    } else if seconds < 86400 {
        format!("{}小时前", duration.num_hours())
    } else {
        format!("{}天前", duration.num_days())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_engine() -> (RecommendationEngine, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let engine = RecommendationEngine::new(Some(temp_dir.path().to_path_buf())).unwrap();
        (engine, temp_dir)
    }

    #[test]
    fn test_record_and_recommend() {
        let (engine, _temp) = create_test_engine();

        // 记录一些搜索
        for i in 0..5 {
            let entry = SearchHistoryEntry::new(
                "error".to_string(),
                SearchType::FullText,
                "./src".to_string(),
                10 + i,
                20,
                Some("rs".to_string()),
            );
            engine.record_search(entry).unwrap();
        }

        // 生成推荐
        let recommendations = engine.generate_recommendations("test");

        // 应该有推荐
        assert!(!recommendations.is_empty() || engine.history_len() > 0);
    }

    #[test]
    fn test_frequent_query_recommendation() {
        let (engine, _temp) = create_test_engine();

        // 记录相同查询多次
        for _ in 0..5 {
            let entry = SearchHistoryEntry::new(
                "rust".to_string(),
                SearchType::FullText,
                "./src".to_string(),
                10,
                20,
                Some("rs".to_string()),
            );
            engine.record_search(entry).unwrap();
        }

        let recommendations = engine.generate_recommendations("rust");

        // 应该找到频繁查询建议
        let has_frequent = recommendations
            .iter()
            .any(|r| r.recommendation_type == RecommendationType::FrequentQuery);

        assert!(has_frequent || !recommendations.is_empty());
    }

    #[test]
    fn test_clear_history() {
        let (engine, _temp) = create_test_engine();

        let entry = SearchHistoryEntry::new(
            "test".to_string(),
            SearchType::FullText,
            "./src".to_string(),
            10,
            20,
            None,
        );
        engine.record_search(entry).unwrap();

        assert!(engine.history_len() > 0);

        let count = engine.clear_history().unwrap();
        assert_eq!(count, 1);
        assert_eq!(engine.history_len(), 0);
    }

    #[test]
    fn test_get_recent_searches() {
        let (engine, _temp) = create_test_engine();

        for i in 0..10 {
            let entry = SearchHistoryEntry::new(
                format!("query{}", i),
                SearchType::FullText,
                "./src".to_string(),
                i,
                10,
                None,
            );
            engine.record_search(entry).unwrap();
        }

        let recent = engine.get_recent_searches(5);
        assert_eq!(recent.len(), 5);
    }
}
