//! XORE Search Engine - 全文搜索和语义搜索
//!
//! 这个crate提供基于Tantivy的全文搜索和基于ONNX的语义搜索功能。
//!
//! ## 模块
//!
//! - `scanner`: 高性能文件扫描器，支持并行遍历和多种过滤条件
//! - `indexer`: Tantivy 索引构建器
//! - `incremental`: 增量索引器，支持文件监控和自动更新
//! - `query`: 搜索查询引擎
//! - `tokenizer`: 中英文混合分词器
//! - `watcher`: 文件监控和事件处理

pub mod incremental;
pub mod indexer;
pub mod query;
pub mod scanner;
pub mod tokenizer;
pub mod watcher;

// 索引相关导出
pub use indexer::{index_exists, open_index, IndexBuilder, IndexConfig, IndexSchema, IndexStats};

// 增量索引导出
pub use incremental::{IncrementalConfig, IncrementalIndexer, IncrementalStats, WriteAheadLog};

// 查询相关导出
pub use query::{QueryAnalyzer, QueryType, SearchConfig, SearchResultIter, Searcher};

// 分词器导出
pub use tokenizer::{register_xore_tokenizer, XoreTokenizer};

// 文件扫描器导出
pub use scanner::{
    FileScanner, FileTypeFilter, MtimeFilter, ScanConfig, ScanStats, ScannedFile, SizeFilter,
};

// 文件监控导出
pub use watcher::{FileEvent, FileWatcher, WatcherConfig};

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        assert_eq!(2 + 2, 4);
    }
}
