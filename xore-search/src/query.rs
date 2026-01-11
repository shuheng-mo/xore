//! 查询引擎

use xore_core::{Result, types::SearchResult};

/// 搜索器
pub struct Searcher {
    // TODO: 添加字段
}

impl Searcher {
    /// 创建新的搜索器
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    /// 执行搜索
    pub fn search(&self, _query: &str) -> Result<Vec<SearchResult>> {
        // TODO: 实现搜索逻辑
        Ok(vec![])
    }
}

impl Default for Searcher {
    fn default() -> Self {
        Self::new().unwrap()
    }
}
