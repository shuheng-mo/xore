//! XORE Search Engine - 全文搜索和语义搜索
//!
//! 这个crate提供基于Tantivy的全文搜索和基于ONNX的语义搜索功能。
//!
//! ## 模块
//!
//! - `scanner`: 高性能文件扫描器，支持并行遍历和多种过滤条件
//! - `indexer`: Tantivy 索引构建器
//! - `query`: 搜索查询引擎
//! - `tokenizer`: 中英文混合分词器
//! - `watcher`: 文件监控和增量索引

pub mod indexer;
pub mod query;
pub mod scanner;
pub mod tokenizer;
pub mod watcher;

// 索引相关导出
pub use indexer::{IndexBuilder, IndexConfig, IndexSchema, IndexStats, index_exists, open_index};

// 查询相关导出
pub use query::{SearchConfig, Searcher, SearchResultIter};

// 分词器导出
pub use tokenizer::{XoreTokenizer, register_xore_tokenizer};

// 文件扫描器导出
pub use scanner::{
    FileScanner, FileTypeFilter, MtimeFilter, ScanConfig, ScanStats, ScannedFile, SizeFilter,
};

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        assert_eq!(2 + 2, 4);
    }
}
