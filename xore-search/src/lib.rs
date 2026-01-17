//! XORE Search Engine - 全文搜索和语义搜索
//!
//! 这个crate提供基于Tantivy的全文搜索和基于ONNX的语义搜索功能。
//!
//! ## 模块
//!
//! - `scanner`: 高性能文件扫描器，支持并行遍历和多种过滤条件
//! - `indexer`: Tantivy 索引构建器
//! - `query`: 搜索查询引擎
//! - `watcher`: 文件监控和增量索引

pub mod indexer;
pub mod query;
pub mod scanner;
pub mod watcher;

pub use indexer::IndexBuilder;
pub use query::Searcher;
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
